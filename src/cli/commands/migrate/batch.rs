//! Batch environment migration
//!
//! Handles migration of multiple environments with progress tracking.

use dialoguer::Confirm;
use indicatif::{ProgressBar, ProgressStyle};
use rust_i18n::t;

use crate::core::migrate::{EnvironmentStatus, MigrateOptions, MigrationResult, Migrator};
use crate::error::{Result, ScoopError};
use crate::output::Output;

use super::scan::scan_all_environments;
use super::types::{
    MigrateAllData, MigrateAllSummary, MigrateExecuteOptions, MigrateFailure, MigrateSkipped,
};

/// Migrate all environments at once.
///
/// Scans all sources, filters migratable environments, and performs batch migration
/// with progress tracking.
pub fn migrate_all_environments(output: &Output, opts: &MigrateExecuteOptions) -> Result<()> {
    if !opts.json {
        let source_name = opts
            .source_filter
            .map(|s| s.to_string())
            .unwrap_or_else(|| "all sources".to_string());
        output.info(&t!("migrate.scanning", source = source_name));
    }

    let environments = scan_all_environments(opts.source_filter);

    if environments.is_empty() {
        if opts.json {
            output.json_success(
                "migrate all",
                MigrateAllData {
                    migrated: Vec::new(),
                    failed: Vec::new(),
                    skipped: Vec::new(),
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
        return Ok(());
    }

    // Filter to migratable environments
    let migratable: Vec<_> = environments
        .iter()
        .filter(|e| {
            matches!(e.status, EnvironmentStatus::Ready)
                || (opts.force
                    && matches!(
                        e.status,
                        EnvironmentStatus::PythonEol { .. }
                            | EnvironmentStatus::NameConflict { .. }
                    ))
        })
        .collect();

    // Collect skipped environments
    let skipped: Vec<MigrateSkipped> = environments
        .iter()
        .filter(|e| !migratable.iter().any(|m| m.name == e.name))
        .map(|e| MigrateSkipped {
            name: e.name.clone(),
            reason: match &e.status {
                EnvironmentStatus::Corrupted { reason } => format!("corrupted: {}", reason),
                EnvironmentStatus::PythonEol { version } => {
                    format!("Python {} is EOL (use --force)", version)
                }
                EnvironmentStatus::NameConflict { .. } => "name conflict (use --force)".to_string(),
                EnvironmentStatus::Ready => "unknown".to_string(),
            },
        })
        .collect();

    let skipped_count = skipped.len();

    if migratable.is_empty() {
        if opts.json {
            output.json_success(
                "migrate all",
                MigrateAllData {
                    migrated: Vec::new(),
                    failed: Vec::new(),
                    skipped,
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
                "{} environment(s) will be skipped (corrupted)",
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

    let mut migrated: Vec<MigrationResult> = Vec::new();
    let mut failed: Vec<MigrateFailure> = Vec::new();

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

    for (idx, env) in migratable.iter().enumerate() {
        if let Some(ref pb) = progress {
            pb.set_message(t!("migrate.batch_item", name = &env.name).to_string());
            pb.set_position(idx as u64);
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
                migrated.push(result);
            }
            Err(e) => {
                failed.push(MigrateFailure {
                    name: env.name.clone(),
                    error: e.to_string(),
                });
                if let Some(ref pb) = progress {
                    pb.println(format!("✗ '{}' failed: {}", env.name, e));
                } else if !opts.json {
                    output.error(&t!("migrate.batch_item_failed", error = e.to_string()));
                }
            }
        }
    }

    // Finish progress bar
    if let Some(pb) = progress {
        pb.finish_with_message("Done");
    }

    // JSON output
    if opts.json {
        output.json_success(
            "migrate all",
            MigrateAllData {
                summary: MigrateAllSummary {
                    total: environments.len(),
                    success: migrated.len(),
                    failed: failed.len(),
                    skipped: skipped_count,
                },
                migrated,
                failed,
                skipped,
            },
        );
        return Ok(());
    }

    // Summary (non-JSON)
    output.info("");
    output.info("─".repeat(40).as_str());
    if opts.dry_run {
        output.info(&t!("migrate.batch_summary_dry", count = migrated.len()));
        output.info(&t!("migrate.batch_no_changes"));
    } else {
        output.success(&t!(
            "migrate.batch_summary",
            success = migrated.len(),
            total = migratable.len()
        ));
    }

    if !failed.is_empty() {
        let failed_names: Vec<_> = failed.iter().map(|f| f.name.as_str()).collect();
        output.warn(&t!(
            "migrate.batch_failed_list",
            names = failed_names.join(", ")
        ));
    }

    Ok(())
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
    // Test Helpers (filter_migratable & collect_skipped)
    // =========================================================================

    /// Filter environments that are eligible for migration.
    fn filter_migratable(
        environments: &[SourceEnvironment],
        force: bool,
    ) -> Vec<&SourceEnvironment> {
        environments
            .iter()
            .filter(|e| {
                matches!(e.status, EnvironmentStatus::Ready)
                    || (force
                        && matches!(
                            e.status,
                            EnvironmentStatus::PythonEol { .. }
                                | EnvironmentStatus::NameConflict { .. }
                        ))
            })
            .collect()
    }

    /// Collect skipped environments with reasons.
    fn collect_skipped(
        environments: &[SourceEnvironment],
        migratable: &[&SourceEnvironment],
    ) -> Vec<MigrateSkipped> {
        environments
            .iter()
            .filter(|e| !migratable.iter().any(|m| m.name == e.name))
            .map(|e| MigrateSkipped {
                name: e.name.clone(),
                reason: match &e.status {
                    EnvironmentStatus::Corrupted { reason } => format!("corrupted: {}", reason),
                    EnvironmentStatus::PythonEol { version } => {
                        format!("Python {} is EOL (use --force)", version)
                    }
                    EnvironmentStatus::NameConflict { .. } => {
                        "name conflict (use --force)".to_string()
                    }
                    EnvironmentStatus::Ready => "unknown".to_string(),
                },
            })
            .collect()
    }

    // =========================================================================
    // filter_migratable Tests
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

    #[test]
    fn filter_migratable_ready_always_included() {
        let envs = vec![
            create_test_env("ready1", EnvironmentStatus::Ready),
            create_test_env("ready2", EnvironmentStatus::Ready),
        ];

        let migratable = filter_migratable(&envs, false);
        assert_eq!(migratable.len(), 2);
    }

    #[test]
    fn filter_migratable_corrupted_never_included() {
        let envs = vec![create_test_env(
            "corrupted",
            EnvironmentStatus::Corrupted {
                reason: "No python".to_string(),
            },
        )];

        let migratable = filter_migratable(&envs, false);
        assert!(migratable.is_empty());

        // Even with force
        let migratable_force = filter_migratable(&envs, true);
        assert!(migratable_force.is_empty());
    }

    #[test]
    fn filter_migratable_eol_excluded_without_force() {
        let envs = vec![create_test_env(
            "eol",
            EnvironmentStatus::PythonEol {
                version: "2.7.18".to_string(),
            },
        )];

        let migratable = filter_migratable(&envs, false);
        assert!(migratable.is_empty());
    }

    #[test]
    fn filter_migratable_eol_included_with_force() {
        let envs = vec![create_test_env(
            "eol",
            EnvironmentStatus::PythonEol {
                version: "2.7.18".to_string(),
            },
        )];

        let migratable = filter_migratable(&envs, true);
        assert_eq!(migratable.len(), 1);
        assert_eq!(migratable[0].name, "eol");
    }

    #[test]
    fn filter_migratable_conflict_excluded_without_force() {
        let envs = vec![create_test_env(
            "conflict",
            EnvironmentStatus::NameConflict {
                existing: PathBuf::from("/existing"),
            },
        )];

        let migratable = filter_migratable(&envs, false);
        assert!(migratable.is_empty());
    }

    #[test]
    fn filter_migratable_conflict_included_with_force() {
        let envs = vec![create_test_env(
            "conflict",
            EnvironmentStatus::NameConflict {
                existing: PathBuf::from("/existing"),
            },
        )];

        let migratable = filter_migratable(&envs, true);
        assert_eq!(migratable.len(), 1);
    }

    #[test]
    fn filter_migratable_mixed_statuses() {
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

        // Without force: only ready
        let migratable = filter_migratable(&envs, false);
        assert_eq!(migratable.len(), 1);
        assert_eq!(migratable[0].name, "ready");

        // With force: ready + eol + conflict (not corrupted)
        let migratable_force = filter_migratable(&envs, true);
        assert_eq!(migratable_force.len(), 3);
    }

    // =========================================================================
    // collect_skipped Tests
    // =========================================================================

    #[test]
    fn collect_skipped_empty_when_all_migratable() {
        let envs = vec![
            create_test_env("ready1", EnvironmentStatus::Ready),
            create_test_env("ready2", EnvironmentStatus::Ready),
        ];
        let migratable: Vec<&SourceEnvironment> = envs.iter().collect();

        let skipped = collect_skipped(&envs, &migratable);
        assert!(skipped.is_empty());
    }

    #[test]
    fn collect_skipped_corrupted_with_reason() {
        let envs = vec![create_test_env(
            "broken",
            EnvironmentStatus::Corrupted {
                reason: "Python binary not found".to_string(),
            },
        )];
        let migratable: Vec<&SourceEnvironment> = vec![];

        let skipped = collect_skipped(&envs, &migratable);
        assert_eq!(skipped.len(), 1);
        assert_eq!(skipped[0].name, "broken");
        assert!(skipped[0].reason.contains("corrupted"));
        assert!(skipped[0].reason.contains("Python binary not found"));
    }

    #[test]
    fn collect_skipped_eol_with_version() {
        let envs = vec![create_test_env(
            "oldpy",
            EnvironmentStatus::PythonEol {
                version: "2.7.18".to_string(),
            },
        )];
        let migratable: Vec<&SourceEnvironment> = vec![];

        let skipped = collect_skipped(&envs, &migratable);
        assert_eq!(skipped.len(), 1);
        assert!(skipped[0].reason.contains("2.7.18"));
        assert!(skipped[0].reason.contains("EOL"));
        assert!(skipped[0].reason.contains("--force"));
    }

    #[test]
    fn collect_skipped_conflict_suggests_force() {
        let envs = vec![create_test_env(
            "dup",
            EnvironmentStatus::NameConflict {
                existing: PathBuf::from("/home/user/.scoop/virtualenvs/dup"),
            },
        )];
        let migratable: Vec<&SourceEnvironment> = vec![];

        let skipped = collect_skipped(&envs, &migratable);
        assert_eq!(skipped.len(), 1);
        assert!(skipped[0].reason.contains("conflict"));
        assert!(skipped[0].reason.contains("--force"));
    }

    // =========================================================================
    // MigrateAllSummary Tests
    // =========================================================================

    #[test]
    fn migrate_all_summary_default_values() {
        let summary = MigrateAllSummary {
            total: 0,
            success: 0,
            failed: 0,
            skipped: 0,
        };

        assert_eq!(summary.total, 0);
        assert_eq!(summary.success, 0);
        assert_eq!(summary.failed, 0);
        assert_eq!(summary.skipped, 0);
    }

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

    // =========================================================================
    // MigrateSkipped Tests
    // =========================================================================

    #[test]
    fn migrate_skipped_serializable() {
        let skipped = MigrateSkipped {
            name: "testenv".to_string(),
            reason: "Test reason".to_string(),
        };

        // Should be serializable
        let json = serde_json::to_string(&skipped).unwrap();
        assert!(json.contains("testenv"));
        assert!(json.contains("Test reason"));
    }

    // =========================================================================
    // MigrateFailure Tests
    // =========================================================================

    #[test]
    fn migrate_failure_serializable() {
        let failure = MigrateFailure {
            name: "failedenv".to_string(),
            error: "Something went wrong".to_string(),
        };

        let json = serde_json::to_string(&failure).unwrap();
        assert!(json.contains("failedenv"));
        assert!(json.contains("Something went wrong"));
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
                yes: true, // Skip confirmation
                ..Default::default()
            };

            let result = migrate_all_environments(&output, &opts);
            assert!(result.is_ok());
        });
    }

    #[test]
    #[serial]
    fn migrate_all_environments_json_empty() {
        with_isolated_migrate_env(|| {
            let output = Output::new(0, true, true, true);
            let opts = MigrateExecuteOptions {
                json: true,
                ..Default::default()
            };

            let result = migrate_all_environments(&output, &opts);
            assert!(result.is_ok());
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

            let result = migrate_all_environments(&output, &opts);
            assert!(result.is_ok());
        });
    }

    #[test]
    #[serial]
    fn migrate_all_environments_with_corrupted_envs() {
        with_full_migrate_env(|_scoop, pyenv| {
            // Create a corrupted environment
            create_corrupted_pyenv_env(pyenv.path(), "corrupted_batch", "3.12.0");

            let output = Output::new(0, false, true, false);
            let opts = MigrateExecuteOptions {
                source_filter: Some(MigrateSource::Pyenv),
                yes: true,
                ..Default::default()
            };

            // Should succeed (skip corrupted, report in summary)
            let result = migrate_all_environments(&output, &opts);
            assert!(result.is_ok());
        });
    }

    #[test]
    #[serial]
    fn migrate_all_environments_dry_run() {
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
            assert!(result.is_ok());
        });
    }

    #[test]
    #[serial]
    fn migrate_all_environments_json_with_envs() {
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
            assert!(result.is_ok());
        });
    }
}
