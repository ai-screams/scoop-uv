//! Single environment migration
//!
//! Handles migration of individual environments with status validation.

use crate::core::migrate::{EnvironmentStatus, MigrateOptions, MigrationResult, Migrator};
use crate::error::{Result, ScoopError};
use crate::output::Output;

use super::conflict::{
    ConflictResolution, generate_unique_name, prompt_conflict_resolution, prompt_rename,
};
use super::scan::find_environment_by_name;
use super::types::MigrateExecuteOptions;

/// Print migration result in human-readable format.
pub fn print_migration_result(output: &Output, result: &MigrationResult, dry_run: bool) {
    if dry_run {
        output.info("");
        output.info("[DRY-RUN] Migration preview:");
        output.info(&format!(
            "  Would create: {}",
            crate::paths::abbreviate_home(&result.path)
        ));
        output.info(&format!("  Python: {}", result.python_version));
        output.info(&format!("  Packages: {}", result.packages_migrated));

        if !result.packages_failed.is_empty() {
            output.warn(&format!(
                "  May fail: {} packages",
                result.packages_failed.len()
            ));
            for pkg in &result.packages_failed {
                output.info(&format!("    - {}", pkg));
            }
        }

        output.info("");
        output.info("→ Run without --dry-run to migrate");
    } else {
        output.success(&format!("Migrated '{}'", result.name));
        output.info(&format!(
            "  Path: {}",
            crate::paths::abbreviate_home(&result.path)
        ));
        output.info(&format!("  Python: {}", result.python_version));
        output.info(&format!(
            "  Packages: {} installed",
            result.packages_migrated
        ));

        if !result.packages_failed.is_empty() {
            output.warn(&format!(
                "  Failed: {} packages (may need manual install)",
                result.packages_failed.len()
            ));
            for pkg in &result.packages_failed {
                output.info(&format!("    - {}", pkg));
            }
        }

        output.info("");
        output.info(&format!("→ Activate: scoop use {}", result.name));
    }
}

/// Migrate a single environment.
///
/// Handles conflict resolution, status validation, and actual migration.
pub fn migrate_environment(
    output: &Output,
    name: &str,
    opts: &MigrateExecuteOptions,
) -> Result<()> {
    let source = find_environment_by_name(name, opts.source_filter)?;

    if !opts.json {
        // Show environment info
        output.info(&format!(
            "Source: {} ({}, Python {})",
            name, source.source_type, source.python_version
        ));
        output.info(&format!(
            "  Path: {}",
            crate::paths::abbreviate_home(&source.path)
        ));

        if let Some(size_bytes) = source.size_bytes {
            let size_mb = size_bytes as f64 / 1_048_576.0;
            output.info(&format!("  Size: {:.1} MB", size_mb));
        }
    }

    // Determine final name (may be renamed)
    let mut final_name = opts.rename.clone().unwrap_or_else(|| name.to_string());
    let mut effective_force = opts.force;

    // Check status
    match &source.status {
        EnvironmentStatus::Ready => {}
        EnvironmentStatus::NameConflict { existing } => {
            if opts.auto_rename {
                // Auto-rename: generate unique name
                final_name = generate_unique_name(name)?;
                if !opts.json {
                    output.info(&format!("Auto-renaming to '{}'", final_name));
                }
            } else if opts.rename.is_some() {
                // User provided explicit rename, check if that conflicts too
                let renamed_path = crate::paths::virtualenv_path(&final_name)?;
                if renamed_path.exists() && !opts.force {
                    return Err(ScoopError::MigrationNameConflict {
                        name: final_name.clone(),
                        existing: renamed_path,
                    });
                }
            } else if !opts.force {
                // Interactive conflict resolution (if not json and not yes)
                if !opts.json && !opts.yes {
                    let resolution = prompt_conflict_resolution(output, name, existing)?;
                    match resolution {
                        ConflictResolution::Overwrite => {
                            effective_force = true;
                            if !opts.json {
                                output.warn("Will overwrite existing environment");
                            }
                        }
                        ConflictResolution::Rename => {
                            final_name = prompt_rename(name)?;
                            if !opts.json {
                                output.info(&format!("Will migrate as '{}'", final_name));
                            }
                        }
                        ConflictResolution::Skip => {
                            if !opts.json {
                                output.info("Skipped");
                            }
                            return Ok(());
                        }
                    }
                } else {
                    // Non-interactive mode: error out
                    if !opts.json {
                        output.warn(&format!(
                            "'{}' already exists at {}",
                            name,
                            crate::paths::abbreviate_home(existing)
                        ));
                        output.info("→ Use --force, --rename, or --auto-rename");
                    }
                    return Err(ScoopError::MigrationNameConflict {
                        name: name.to_string(),
                        existing: existing.clone(),
                    });
                }
            } else if !opts.json {
                output.warn("Overwriting existing environment");
            }
        }
        EnvironmentStatus::PythonEol { version } => {
            if !opts.force {
                if !opts.json {
                    output.warn(&format!("Python {} reached end-of-life", version));
                    output.info("→ Use --force to proceed anyway");
                }
                return Err(ScoopError::MigrationFailed {
                    reason: format!("Python {} is EOL", version),
                });
            }
            if !opts.json {
                output.warn(&format!("Python {} is EOL, proceeding anyway", version));
            }
        }
        EnvironmentStatus::Corrupted { reason } => {
            if !opts.json {
                output.error(&format!("Corrupted: {}", reason));
            }
            return Err(ScoopError::CorruptedEnvironment {
                name: name.to_string(),
                reason: reason.clone(),
            });
        }
    }

    // Create migrator and options
    let migrator = Migrator::new()?;
    let options = MigrateOptions {
        dry_run: opts.dry_run,
        force: effective_force,
        skip_packages: false,
        rename_to: if final_name != name {
            Some(final_name.clone())
        } else {
            None
        },
        strict: opts.strict,
        delete_source: opts.delete_source,
        auto_install_python: false,
    };

    if !opts.json {
        if opts.dry_run {
            output.info("[DRY-RUN] Simulating...");
        } else {
            output.info("Migrating...");
        }
    }

    // Perform migration
    let result = migrator.migrate(&source, &options)?;

    // JSON output
    if opts.json {
        output.json_success("migrate", &result);
        return Ok(());
    }

    // Report results
    print_migration_result(output, &result, opts.dry_run);

    Ok(())
}
