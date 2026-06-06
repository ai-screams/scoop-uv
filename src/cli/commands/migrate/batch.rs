//! Batch environment migration
//!
//! Handles migration of multiple environments with progress tracking.

use std::sync::Mutex;

use dialoguer::Confirm;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use rust_i18n::t;
use serde::Serialize;

use crate::core::migrate::{
    EnvironmentStatus, MigrateOptions, MigrationResult, Migrator, SourceEnvironment,
};
use crate::error::{Result, ScoopError};
use crate::output::Output;

use super::scan::{any_source_tool_available, scan_all_environments};
use super::types::{
    MigrateAllData, MigrateAllSummary, MigrateExecuteOptions, MigrateFailure, MigrateSkipped,
    MigrationConflictDetail,
};

/// Migrate all environments at once.
///
/// Scans all sources, filters migratable environments, and performs batch migration
/// with progress tracking.
///
/// # Errors
///
/// - [`ScoopError::MigrationSourcesNotFound`] (exit 3) — no source tool
///   (pyenv / virtualenvwrapper / conda) installed.
/// - [`ScoopError::MigrationBatchFailed`] (exit 2, Quiet render) — at
///   least one per-env failure or unresolved name conflict
///   (without `--force`). The full summary is already rendered to
///   stderr / stdout-as-JSON before this Err returns; `main.rs` MUST
///   NOT print the global `error:` prefix again.
pub fn migrate_all_environments(output: &Output, opts: &MigrateExecuteOptions) -> Result<()> {
    if !opts.json {
        let source_name = opts
            .source_filter
            .map(|s| s.to_string())
            .unwrap_or_else(|| "all sources".to_string());
        output.info(&t!("migrate.scanning", source = source_name));
    }

    let environments = scan_all_environments(opts.source_filter);

    // Empty scan branches:
    //   - No tool installed     → exit 3 via MigrationSourcesNotFound.
    //   - Tool present, no envs → existing Ok(()) info / JSON branch.
    if environments.is_empty() {
        if !any_source_tool_available(opts.source_filter) {
            return Err(ScoopError::MigrationSourcesNotFound {
                requested: opts.source_filter.map(|s| s.to_string()),
            });
        }
        emit_empty_envs(output, opts);
        return Ok(());
    }

    // Partition into migratable / conflicts (preflight) / skipped buckets.
    // Conflicts are tracked both as structured `MigrationConflictDetail`
    // entries (new in 0.14, see types.rs) AND, for backward compat, as
    // `MigrateSkipped` entries with the historical "name conflict
    // (use --force)" reason. Existing scripts that read `skipped[]`
    // continue to work.
    let partition = partition_envs(&environments, opts.force);
    let PartitionedEnvs {
        migratable,
        conflicts,
        skipped,
    } = partition;

    let skipped_count = skipped.len();
    let conflict_count = conflicts.len();

    // No migratable envs:
    //   - If preflight conflicts exist (without --force) → render +
    //     return MigrationBatchFailed (exit 2). Previously this path
    //     silently returned Ok(()), masking the conflict in CI.
    //   - Otherwise (only EOL / corrupted skipped) → info + Ok(()).
    if migratable.is_empty() {
        if conflict_count > 0 {
            // Render summary first (Quiet contract: no global error:
            // prefix; main.rs trusts batch.rs already wrote everything).
            if opts.json {
                emit_migrate_all_json_outcome(
                    &[],
                    &[],
                    &conflicts,
                    &skipped,
                    environments.len(),
                    /* is_failure = */ true,
                );
            } else {
                render_no_migratable_with_conflicts(output, &conflicts, &skipped);
            }
            return Err(ScoopError::MigrationBatchFailed {
                failed_count: 0,
                conflict_count,
            });
        }
        if opts.json {
            output.json_success(
                "migrate all",
                MigrateAllData {
                    migrated: Vec::new(),
                    failed: Vec::new(),
                    skipped,
                    conflicts,
                    summary: MigrateAllSummary {
                        total: environments.len(),
                        success: 0,
                        failed: 0,
                        skipped: skipped_count,
                    },
                },
            );
        } else {
            output.info(&t!("migrate.no_eligible"));
            if skipped_count > 0 {
                output.info(&t!("migrate.skipped_count", count = skipped_count));
            }
        }
        return Ok(());
    }

    // Show what will be migrated (only in non-JSON mode)
    if !opts.json {
        output.info(&format!(
            "Found {} environment(s) to migrate:",
            migratable.len()
        ));
        for env in &migratable {
            let status_hint = match &env.status {
                EnvironmentStatus::Ready => "".to_string(),
                EnvironmentStatus::NameConflict { .. } => " (will overwrite)".to_string(),
                EnvironmentStatus::PythonEol { version } => {
                    format!(" (Python {} EOL)", version)
                }
                EnvironmentStatus::Corrupted { .. } => " (corrupted)".to_string(),
            };
            output.info(&format!(
                "  - {} (Python {}){}",
                env.name, env.python_version, status_hint
            ));
        }

        if skipped_count > 0 {
            output.warn(&format!(
                "{} environment(s) will be skipped (see summary)",
                skipped_count
            ));
        }

        // Confirmation prompt if not --yes and not dry-run and not JSON
        if !opts.yes && !opts.dry_run {
            println!();
            let confirmed = Confirm::new()
                .with_prompt(format!("Migrate {} environment(s)?", migratable.len()))
                .default(false)
                .interact()
                .map_err(|e| ScoopError::Io(std::io::Error::other(e)))?;

            if !confirmed {
                output.info(&t!("migrate.batch_cancelled"));
                return Ok(());
            }
        }
    }

    // Perform migrations
    let migrator = Migrator::new()?;
    let options = MigrateOptions {
        dry_run: opts.dry_run,
        force: opts.force,
        skip_packages: false,
        rename_to: None,
        strict: opts.strict,
        delete_source: opts.delete_source,
        auto_install_python: false,
    };

    // Wrapped in Mutex so the parallel branch below can collect results from
    // multiple worker threads. The sequential branch (dry-run / single env)
    // also goes through the locks for code-path uniformity — contention is
    // zero there, so the cost is a few ns per env.
    let migrated_lock: Mutex<Vec<MigrationResult>> = Mutex::new(Vec::new());
    let failed_lock: Mutex<Vec<MigrateFailure>> = Mutex::new(Vec::new());

    if !opts.json {
        if opts.dry_run {
            output.info("");
            output.info(&t!("migrate.batch_dry_run"));
        } else {
            output.info("");
            output.info(&t!("migrate.batch_start"));
        }
    }

    // Create progress bar for batch migration
    let progress = if !opts.json && !opts.dry_run && !output.is_quiet() {
        let pb = ProgressBar::new(migratable.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:30.cyan/blue}] {pos}/{len} {msg}")
                .expect("valid template")
                .progress_chars("=>-"),
        );
        Some(pb)
    } else {
        None
    };

    // Per-env work. Shared by the sequential and parallel branches; closes
    // over `migrator`, `options`, `output`, `opts`, `progress`, and the two
    // result locks. Each branch in `progress` is thread-safe (`indicatif`
    // serialises println/inc internally) and `output.{success,info,warn,error}`
    // emit one `eprintln!` per call, which is atomic at the line level.
    let run_one = |env: &SourceEnvironment| {
        if let Some(ref pb) = progress {
            pb.set_message(t!("migrate.batch_item", name = &env.name).to_string());
        } else if !opts.json {
            output.info("");
            output.info(&t!("migrate.batch_item", name = &env.name));
        }

        match migrator.migrate(env, &options) {
            Ok(result) => {
                if let Some(ref pb) = progress {
                    pb.println(format!(
                        "✓ '{}' migrated ({} packages)",
                        result.name, result.packages_migrated
                    ));
                    if !result.packages_failed.is_empty() {
                        pb.println(format!(
                            "  ⚠ {} package(s) failed",
                            result.packages_failed.len()
                        ));
                    }
                } else if !opts.json {
                    if opts.dry_run {
                        output.info(&format!(
                            "  [DRY-RUN] Would create: {}",
                            result.path.display()
                        ));
                        output.info(&t!("migrate.packages", count = result.packages_migrated));
                    } else {
                        output.success(&t!("migrate.batch_item_success", name = &result.name));
                        output.info(&t!("migrate.packages", count = result.packages_migrated));
                        if !result.packages_failed.is_empty() {
                            output.warn(&t!(
                                "migrate.failed_packages",
                                count = result.packages_failed.len()
                            ));
                        }
                    }
                }
                migrated_lock
                    .lock()
                    .expect("results lock poisoned")
                    .push(result);
            }
            Err(e) => {
                let code = e.code();
                let msg = e.to_string();
                if let Some(ref pb) = progress {
                    pb.println(format!("✗ '{}' failed: {}", env.name, msg));
                } else if !opts.json {
                    output.error(&t!("migrate.batch_item_failed", error = msg.clone()));
                }
                failed_lock
                    .lock()
                    .expect("failures lock poisoned")
                    .push(MigrateFailure {
                        name: env.name.clone(),
                        source_type: env.source_type,
                        error_code: code,
                        error: msg,
                    });
            }
        }

        if let Some(ref pb) = progress {
            pb.inc(1);
        }
    };

    // Parallelise only when there's a real win: dry-run does no I/O work
    // (sequential gives cleaner, deterministic preview output) and a single
    // env has nothing to parallelise.
    if opts.dry_run || migratable.len() <= 1 {
        for env in migratable.iter() {
            run_one(env);
        }
    } else {
        // par_iter().for_each passes `&&T`; wrap so `run_one` keeps its
        // `&T` signature (which the sequential branch also uses).
        migratable.par_iter().for_each(|env| run_one(env));
    }

    let mut migrated = migrated_lock.into_inner().expect("results lock poisoned");
    let mut failed = failed_lock.into_inner().expect("failures lock poisoned");
    // Sort by env name so the summary and JSON output are deterministic
    // regardless of which worker thread finished first. This is NOT the
    // original scan order (scan sorts by source type then name) — across
    // source types or with duplicate names the two orders diverge, and
    // alphabetic-by-name is the cheaper, more useful default for a
    // user-facing summary.
    migrated.sort_by(|a, b| a.name.cmp(&b.name));
    failed.sort_by(|a, b| a.name.cmp(&b.name));

    // Finish progress bar
    if let Some(pb) = progress {
        pb.finish_with_message("Done");
    }

    let failed_count = failed.len();
    let is_failure = failed_count + conflict_count > 0;

    // Render BEFORE returning Err so the Quiet render policy on
    // MigrationBatchFailed is satisfied (main.rs writes nothing extra).
    if opts.json {
        emit_migrate_all_json_outcome(
            &migrated,
            &failed,
            &conflicts,
            &skipped,
            environments.len(),
            is_failure,
        );
    } else {
        render_human_summary(
            output,
            opts,
            &migrated,
            &failed,
            &conflicts,
            migratable.len(),
        );
    }

    if is_failure {
        return Err(ScoopError::MigrationBatchFailed {
            failed_count,
            conflict_count,
        });
    }

    Ok(())
}

// ============================================================================
// Helpers
// ============================================================================

struct PartitionedEnvs<'a> {
    migratable: Vec<&'a SourceEnvironment>,
    conflicts: Vec<MigrationConflictDetail>,
    skipped: Vec<MigrateSkipped>,
}

/// Split scanned envs into migratable / conflict / skipped buckets.
///
/// `conflicts` carries the structured detail (added in 0.14). `skipped`
/// still includes name-conflicts under the historical
/// "name conflict (use --force)" reason so consumers reading
/// `data.skipped[]` continue to work — this is intentional, see the
/// `MigrateAllData` doc comment.
fn partition_envs(environments: &[SourceEnvironment], force: bool) -> PartitionedEnvs<'_> {
    let mut migratable: Vec<&SourceEnvironment> = Vec::new();
    let mut conflicts: Vec<MigrationConflictDetail> = Vec::new();
    let mut skipped: Vec<MigrateSkipped> = Vec::new();

    for env in environments {
        let is_ready = matches!(env.status, EnvironmentStatus::Ready);
        let force_eligible = force
            && matches!(
                env.status,
                EnvironmentStatus::PythonEol { .. } | EnvironmentStatus::NameConflict { .. }
            );

        if is_ready || force_eligible {
            migratable.push(env);
            continue;
        }

        match &env.status {
            EnvironmentStatus::Corrupted { reason } => skipped.push(MigrateSkipped {
                name: env.name.clone(),
                reason: format!("corrupted: {}", reason),
            }),
            EnvironmentStatus::PythonEol { version } => skipped.push(MigrateSkipped {
                name: env.name.clone(),
                reason: format!("Python {} is EOL (use --force)", version),
            }),
            EnvironmentStatus::NameConflict { existing } => {
                conflicts.push(MigrationConflictDetail {
                    name: env.name.clone(),
                    source_type: env.source_type,
                    existing: existing.clone(),
                });
                skipped.push(MigrateSkipped {
                    name: env.name.clone(),
                    reason: "name conflict (use --force)".to_string(),
                });
            }
            EnvironmentStatus::Ready => {} // unreachable given the matched arms above
        }
    }

    PartitionedEnvs {
        migratable,
        conflicts,
        skipped,
    }
}

fn emit_empty_envs(output: &Output, opts: &MigrateExecuteOptions) {
    if opts.json {
        output.json_success(
            "migrate all",
            MigrateAllData {
                migrated: Vec::new(),
                failed: Vec::new(),
                skipped: Vec::new(),
                conflicts: Vec::new(),
                summary: MigrateAllSummary {
                    total: 0,
                    success: 0,
                    failed: 0,
                    skipped: 0,
                },
            },
        );
    } else {
        let source_name = opts
            .source_filter
            .map(|s| format!("{}", s))
            .unwrap_or_default();
        output.info(&t!("migrate.no_envs", source = source_name));
    }
}

fn render_no_migratable_with_conflicts(
    output: &Output,
    conflicts: &[MigrationConflictDetail],
    skipped: &[MigrateSkipped],
) {
    output.info("");
    output.info("─".repeat(40).as_str());
    output.warn(&t!(
        "migrate.batch_no_migratable_conflicts",
        count = conflicts.len()
    ));
    for c in conflicts {
        output.warn(&t!(
            "migrate.batch_conflict_line",
            name = c.name,
            source = c.source_type.to_string(),
            existing = c.existing.display().to_string()
        ));
    }
    if skipped.len() > conflicts.len() {
        let other = skipped.len() - conflicts.len();
        output.info(&t!("migrate.batch_other_skipped", count = other));
    }
    output.info(&t!("migrate.batch_force_hint"));
}

fn render_human_summary(
    output: &Output,
    opts: &MigrateExecuteOptions,
    migrated: &[MigrationResult],
    failed: &[MigrateFailure],
    conflicts: &[MigrationConflictDetail],
    migratable_count: usize,
) {
    output.info("");
    output.info("─".repeat(40).as_str());
    if opts.dry_run {
        output.info(&t!("migrate.batch_summary_dry", count = migrated.len()));
        output.info(&t!("migrate.batch_no_changes"));
    } else {
        output.success(&t!(
            "migrate.batch_summary",
            success = migrated.len(),
            total = migratable_count
        ));
    }

    if !failed.is_empty() {
        let failed_names: Vec<_> = failed.iter().map(|f| f.name.as_str()).collect();
        output.warn(&t!(
            "migrate.batch_failed_list",
            names = failed_names.join(", ")
        ));
    }

    // Mixed success + conflict: previous version omitted conflict
    // details from the summary, so the user got a Quiet exit 2 with
    // no explanation about the conflict (Codex post-impl SHOULD #1).
    // Show the same conflict block here that the all-conflict branch
    // shows, so every non-JSON failure path explains itself.
    if !conflicts.is_empty() {
        output.warn(&t!(
            "migrate.batch_conflicts_header",
            count = conflicts.len()
        ));
        for c in conflicts {
            output.warn(&t!(
                "migrate.batch_conflict_line",
                name = c.name,
                source = c.source_type.to_string(),
                existing = c.existing.display().to_string()
            ));
        }
        output.info(&t!("migrate.batch_force_hint"));
    }
}

/// JSON envelope helper for `migrate all`.
///
/// Modelled on [`crate::cli::commands::verify::emit_strict_json_failure`]
/// — one helper that emits either a `success` or `error` envelope based
/// on `is_failure`. Both carry the full `MigrateAllData` so consumers
/// don't lose detail on either path; the failure side adds an
/// `error: { code, message, failed_count, conflict_count }` block.
///
/// Keeping the success shape unchanged from previous releases preserves
/// JSON backward compatibility. `conflicts` is the only additive top-
/// level data key.
fn emit_migrate_all_json_outcome(
    migrated: &[MigrationResult],
    failed: &[MigrateFailure],
    conflicts: &[MigrationConflictDetail],
    skipped: &[MigrateSkipped],
    total: usize,
    is_failure: bool,
) {
    #[derive(Serialize)]
    struct SuccessEnvelope<'a> {
        status: &'a str,
        command: &'a str,
        data: DataView<'a>,
    }
    #[derive(Serialize)]
    struct FailureEnvelope<'a> {
        status: &'a str,
        command: &'a str,
        error: ErrorBody,
        data: DataView<'a>,
    }
    #[derive(Serialize)]
    struct ErrorBody {
        code: &'static str,
        message: String,
        failed_count: usize,
        conflict_count: usize,
    }
    #[derive(Serialize)]
    struct DataView<'a> {
        migrated: &'a [MigrationResult],
        failed: &'a [MigrateFailure],
        skipped: &'a [MigrateSkipped],
        conflicts: &'a [MigrationConflictDetail],
        summary: MigrateAllSummary,
    }

    let summary = MigrateAllSummary {
        total,
        success: migrated.len(),
        failed: failed.len(),
        skipped: skipped.len(),
    };

    let data = DataView {
        migrated,
        failed,
        skipped,
        conflicts,
        summary,
    };

    if is_failure {
        let envelope = FailureEnvelope {
            status: "error",
            command: "migrate all",
            error: ErrorBody {
                code: "MIGRATE_BATCH_FAILED",
                message: t!(
                    "error.migration_batch_failed",
                    failed = failed.len().to_string(),
                    conflicts = conflicts.len().to_string()
                )
                .to_string(),
                failed_count: failed.len(),
                conflict_count: conflicts.len(),
            },
            data,
        };
        emit_envelope_or_fallback(&envelope);
    } else {
        let envelope = SuccessEnvelope {
            status: "success",
            command: "migrate all",
            data,
        };
        emit_envelope_or_fallback(&envelope);
    }
}

/// Serialise `envelope` to stdout, falling back to a minimal hand-rolled
/// JSON error envelope on serde failure.
///
/// The migrate envelopes embed `PathBuf` (via `MigrationResult.path` and
/// `MigrationConflictDetail.existing`). serde's default `PathBuf` adapter
/// fails on non-UTF-8 paths, so the canonical `unwrap()` pattern from
/// `verify.rs::emit_strict_json_failure` (whose data is all UTF-8 owned
/// strings) is unsafe here.
///
/// On serde failure the fallback always says `status: "error"` even
/// when the caller was on the success path — because if PathBuf
/// serialisation failed we can no longer trust the data, and silently
/// emitting nothing would leave scripts parsing an empty stdout. The
/// process exit code is unchanged: the caller of this function decides
/// the return value independently.
fn emit_envelope_or_fallback<T: Serialize>(envelope: &T) {
    match serde_json::to_string(envelope) {
        Ok(json) => println!("{json}"),
        Err(err) => {
            // Hand-rolled JSON to avoid recursive serialisation failure.
            println!(
                "{{\"status\":\"error\",\"command\":\"migrate all\",\"error\":{{\"code\":\"INTERNAL_JSON_ERROR\",\"message\":\"failed to serialise migrate envelope: {}\"}}}}",
                err.to_string().replace('"', "\\\"")
            );
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::MigrateSource;
    use crate::core::migrate::{SourceEnvironment, SourceType};
    use crate::test_utils::{
        create_corrupted_pyenv_env, create_mock_pyenv_env, with_full_migrate_env,
        with_isolated_migrate_env,
    };
    use serial_test::serial;
    use std::path::PathBuf;

    // =========================================================================
    // Test Helpers
    // =========================================================================

    fn create_test_env(name: &str, status: EnvironmentStatus) -> SourceEnvironment {
        SourceEnvironment {
            name: name.to_string(),
            python_version: "3.12.0".to_string(),
            path: PathBuf::from(format!("/test/{}", name)),
            source_type: SourceType::Pyenv,
            size_bytes: None,
            status,
        }
    }

    // =========================================================================
    // partition_envs Tests (replaces filter_migratable + collect_skipped)
    // =========================================================================

    #[test]
    fn partition_ready_always_migratable() {
        let envs = vec![
            create_test_env("ready1", EnvironmentStatus::Ready),
            create_test_env("ready2", EnvironmentStatus::Ready),
        ];

        let p = partition_envs(&envs, false);
        assert_eq!(p.migratable.len(), 2);
        assert!(p.conflicts.is_empty());
        assert!(p.skipped.is_empty());
    }

    #[test]
    fn partition_corrupted_never_migratable() {
        let envs = vec![create_test_env(
            "corrupted",
            EnvironmentStatus::Corrupted {
                reason: "No python".to_string(),
            },
        )];

        let p = partition_envs(&envs, false);
        assert!(p.migratable.is_empty());
        assert_eq!(p.skipped.len(), 1);
        assert!(p.skipped[0].reason.contains("corrupted"));

        let p_force = partition_envs(&envs, true);
        assert!(p_force.migratable.is_empty());
    }

    #[test]
    fn partition_eol_excluded_without_force() {
        let envs = vec![create_test_env(
            "eol",
            EnvironmentStatus::PythonEol {
                version: "2.7.18".to_string(),
            },
        )];

        let p = partition_envs(&envs, false);
        assert!(p.migratable.is_empty());
        assert_eq!(p.skipped.len(), 1);
        assert!(p.skipped[0].reason.contains("2.7.18"));
        assert!(p.skipped[0].reason.contains("EOL"));
    }

    #[test]
    fn partition_eol_included_with_force() {
        let envs = vec![create_test_env(
            "eol",
            EnvironmentStatus::PythonEol {
                version: "2.7.18".to_string(),
            },
        )];

        let p = partition_envs(&envs, true);
        assert_eq!(p.migratable.len(), 1);
        assert!(p.skipped.is_empty());
    }

    #[test]
    fn partition_conflict_excluded_without_force_appears_in_both_buckets() {
        // Codex MUST FIX #2 contract: name-conflict appears in BOTH
        // `conflicts` (new structured surface) AND `skipped` (legacy
        // shape for backward compatibility with existing JSON consumers).
        let envs = vec![create_test_env(
            "dup",
            EnvironmentStatus::NameConflict {
                existing: PathBuf::from("/existing"),
            },
        )];

        let p = partition_envs(&envs, false);
        assert!(p.migratable.is_empty());
        assert_eq!(p.conflicts.len(), 1);
        assert_eq!(p.conflicts[0].name, "dup");
        assert_eq!(p.conflicts[0].existing, PathBuf::from("/existing"));
        assert_eq!(p.skipped.len(), 1);
        assert!(p.skipped[0].reason.contains("conflict"));
    }

    #[test]
    fn partition_conflict_included_with_force() {
        let envs = vec![create_test_env(
            "dup",
            EnvironmentStatus::NameConflict {
                existing: PathBuf::from("/existing"),
            },
        )];

        let p = partition_envs(&envs, true);
        assert_eq!(p.migratable.len(), 1);
        assert!(p.conflicts.is_empty());
        assert!(p.skipped.is_empty());
    }

    #[test]
    fn partition_mixed_statuses() {
        let envs = vec![
            create_test_env("ready", EnvironmentStatus::Ready),
            create_test_env(
                "eol",
                EnvironmentStatus::PythonEol {
                    version: "2.7".to_string(),
                },
            ),
            create_test_env(
                "corrupted",
                EnvironmentStatus::Corrupted {
                    reason: "broken".to_string(),
                },
            ),
            create_test_env(
                "conflict",
                EnvironmentStatus::NameConflict {
                    existing: PathBuf::from("/x"),
                },
            ),
        ];

        let p = partition_envs(&envs, false);
        assert_eq!(p.migratable.len(), 1);
        assert_eq!(p.migratable[0].name, "ready");
        assert_eq!(p.conflicts.len(), 1);
        // skipped has: eol + corrupted + conflict
        assert_eq!(p.skipped.len(), 3);

        let p_force = partition_envs(&envs, true);
        // ready + eol + conflict (not corrupted)
        assert_eq!(p_force.migratable.len(), 3);
        assert!(p_force.conflicts.is_empty());
        assert_eq!(p_force.skipped.len(), 1); // only corrupted
    }

    // =========================================================================
    // MigrateAllSummary / MigrateSkipped / MigrateFailure shape
    // =========================================================================

    #[test]
    fn migrate_all_summary_counts_match() {
        let summary = MigrateAllSummary {
            total: 10,
            success: 5,
            failed: 2,
            skipped: 3,
        };
        assert_eq!(summary.success + summary.failed + summary.skipped, 10);
    }

    #[test]
    fn migrate_skipped_serializable() {
        let skipped = MigrateSkipped {
            name: "testenv".to_string(),
            reason: "Test reason".to_string(),
        };

        let json = serde_json::to_string(&skipped).unwrap();
        assert!(json.contains("testenv"));
        assert!(json.contains("Test reason"));
    }

    #[test]
    fn migrate_failure_serializable_with_new_fields() {
        // Inc 4: source_type + error_code are additive (Codex MUST FIX #4).
        let failure = MigrateFailure {
            name: "failedenv".to_string(),
            source_type: SourceType::Pyenv,
            error_code: "MIGRATE_FAILED",
            error: "Something went wrong".to_string(),
        };

        let json = serde_json::to_string(&failure).unwrap();
        assert!(json.contains("failedenv"));
        assert!(json.contains("Something went wrong"));
        assert!(json.contains("\"source_type\":\"pyenv\""));
        assert!(json.contains("\"error_code\":\"MIGRATE_FAILED\""));
    }

    // =========================================================================
    // migrate_all_environments Integration Tests
    // =========================================================================

    #[test]
    #[serial]
    fn migrate_all_environments_empty_when_no_sources() {
        with_isolated_migrate_env(|| {
            let output = Output::new(0, false, true, false);
            let opts = MigrateExecuteOptions {
                yes: true,
                ..Default::default()
            };

            // No source tools detected → MigrationSourcesNotFound (exit 3).
            // Previously this returned Ok(()) silently.
            let result = migrate_all_environments(&output, &opts);
            assert!(matches!(
                result,
                Err(ScoopError::MigrationSourcesNotFound { .. })
            ));
        });
    }

    #[test]
    #[serial]
    fn migrate_all_environments_json_empty_no_sources() {
        with_isolated_migrate_env(|| {
            let output = Output::new(0, true, true, true);
            let opts = MigrateExecuteOptions {
                json: true,
                ..Default::default()
            };

            // Same exit-3 contract under --json (no tool present).
            let result = migrate_all_environments(&output, &opts);
            assert!(matches!(
                result,
                Err(ScoopError::MigrationSourcesNotFound { .. })
            ));
        });
    }

    #[test]
    #[serial]
    fn migrate_all_environments_empty_with_tools_present() {
        // pyenv root exists but has zero envs → Ok(()) (no exit 3).
        with_full_migrate_env(|_scoop, _pyenv| {
            let output = Output::new(0, false, true, false);
            let opts = MigrateExecuteOptions {
                source_filter: Some(MigrateSource::Pyenv),
                yes: true,
                ..Default::default()
            };

            let result = migrate_all_environments(&output, &opts);
            assert!(result.is_ok(), "tool present + no envs must be Ok(())");
        });
    }

    #[test]
    #[serial]
    fn migrate_all_environments_with_source_filter() {
        with_isolated_migrate_env(|| {
            let output = Output::new(0, false, true, false);
            let opts = MigrateExecuteOptions {
                source_filter: Some(MigrateSource::Pyenv),
                yes: true,
                ..Default::default()
            };

            // Filtering to a source tool that's also absent → exit 3 path.
            let result = migrate_all_environments(&output, &opts);
            assert!(matches!(
                result,
                Err(ScoopError::MigrationSourcesNotFound { requested: Some(_) })
            ));
        });
    }

    #[test]
    #[serial]
    fn migrate_all_environments_with_corrupted_envs() {
        with_full_migrate_env(|_scoop, pyenv| {
            create_corrupted_pyenv_env(pyenv.path(), "corrupted_batch", "3.12.0");

            let output = Output::new(0, false, true, false);
            let opts = MigrateExecuteOptions {
                source_filter: Some(MigrateSource::Pyenv),
                yes: true,
                ..Default::default()
            };

            // Corrupted is skipped, no migratable, no conflicts → Ok(()).
            let result = migrate_all_environments(&output, &opts);
            assert!(result.is_ok());
        });
    }

    #[test]
    #[serial]
    fn migrate_all_environments_dry_run_propagates_pip_failure() {
        // create_mock_pyenv_env builds a directory skeleton WITHOUT a real
        // pip binary, so the migrator's package-extraction step fails.
        // Pre-Inc4 this was silently swallowed (Ok regardless). Now it
        // correctly surfaces as MigrationBatchFailed so users (and CI)
        // learn the dry-run would fail in production.
        with_full_migrate_env(|_scoop, pyenv| {
            create_mock_pyenv_env(pyenv.path(), "dryrun_env", "3.12.0");

            let output = Output::new(0, false, true, false);
            let opts = MigrateExecuteOptions {
                source_filter: Some(MigrateSource::Pyenv),
                dry_run: true,
                yes: true,
                ..Default::default()
            };

            let result = migrate_all_environments(&output, &opts);
            assert!(matches!(
                result,
                Err(ScoopError::MigrationBatchFailed {
                    failed_count: 1,
                    conflict_count: 0,
                })
            ));
        });
    }

    #[test]
    #[serial]
    fn migrate_all_environments_json_emits_failure_envelope() {
        // Mock env has no pip → migrator returns extraction error → JSON
        // failure envelope is emitted on stdout BEFORE the Err returns.
        // The Quiet render policy on MigrationBatchFailed then prevents
        // main.rs from appending the generic `error:` prefix.
        with_full_migrate_env(|_scoop, pyenv| {
            create_mock_pyenv_env(pyenv.path(), "json_batch", "3.12.0");
            create_corrupted_pyenv_env(pyenv.path(), "json_corrupted", "3.11.0");

            let output = Output::new(0, true, true, true);
            let opts = MigrateExecuteOptions {
                source_filter: Some(MigrateSource::Pyenv),
                json: true,
                yes: true,
                ..Default::default()
            };

            let result = migrate_all_environments(&output, &opts);
            assert!(matches!(
                result,
                Err(ScoopError::MigrationBatchFailed { .. })
            ));
        });
    }
}
