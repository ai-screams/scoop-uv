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
                .map(|s| format!("{} ", s))
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
