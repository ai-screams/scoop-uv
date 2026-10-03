//! Everything `migrate all` prints: the plan preview, the human summary and
//! the JSON envelope. Nothing here decides or migrates.

use rust_i18n::t;
use serde::Serialize;

use crate::core::migrate::{EnvironmentStatus, MigrationResult, SourceEnvironment};
use crate::output::Output;

use super::super::types::{
    MigrateAllData, MigrateAllSummary, MigrateExecuteOptions, MigrateFailure, MigrateSkipped,
    MigrationConflictDetail,
};

/// Lists what is about to migrate (human mode only).
pub(super) fn print_plan(output: &Output, migratable: &[&SourceEnvironment], skipped_count: usize) {
    output.info(&format!(
        "Found {} environment(s) to migrate:",
        migratable.len()
    ));
    for env in migratable {
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
}

pub(super) fn emit_empty_envs(output: &Output, opts: &MigrateExecuteOptions) {
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

/// No environment is migratable and nothing conflicts (only EOL /
/// corrupted entries were skipped): say so, successfully.
pub(super) fn emit_nothing_to_migrate(
    output: &Output,
    opts: &MigrateExecuteOptions,
    skipped: Vec<MigrateSkipped>,
    conflicts: Vec<MigrationConflictDetail>,
    total: usize,
) {
    let skipped_count = skipped.len();
    if opts.json {
        output.json_success(
            "migrate all",
            MigrateAllData {
                migrated: Vec::new(),
                failed: Vec::new(),
                skipped,
                conflicts,
                summary: MigrateAllSummary {
                    total,
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
}

pub(super) fn render_no_migratable_with_conflicts(
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

pub(super) fn render_human_summary(
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
pub(super) fn emit_migrate_all_json_outcome(
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
