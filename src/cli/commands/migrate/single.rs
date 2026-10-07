//! Single environment migration
//!
//! Handles migration of individual environments with status validation.

use rust_i18n::t;

use std::io::IsTerminal;
use std::path::Path;

use crate::core::migrate::{
    EnvironmentStatus, MigrateOptions, MigrationResult, Migrator, SourceEnvironment,
};
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
        output.info(&t!("migrate.dry_run_header"));
        output.info(&t!(
            "migrate.to",
            path = crate::paths::abbreviate_home(&result.path)
        ));
        output.info(&t!("migrate.python", version = &result.python_version));
        output.info(&t!("migrate.packages", count = result.packages_migrated));

        if !result.packages_failed.is_empty() {
            output.warn(&t!(
                "migrate.failed_packages",
                count = result.packages_failed.len()
            ));
            for pkg in &result.packages_failed {
                output.info(&format!("    - {}", pkg));
            }
        }

        output.info("");
        output.info(&t!("migrate.dry_run_hint"));
    } else {
        output.success(&t!("migrate.success", name = &result.name));
        output.info(&t!(
            "create.path",
            path = crate::paths::abbreviate_home(&result.path)
        ));
        output.info(&t!("migrate.python", version = &result.python_version));
        output.info(&t!("migrate.packages", count = result.packages_migrated));

        if !result.packages_failed.is_empty() {
            output.warn(&t!(
                "migrate.failed_packages",
                count = result.packages_failed.len()
            ));
            for pkg in &result.packages_failed {
                output.info(&format!("    - {}", pkg));
            }
        }

        output.info("");
        output.info(&t!("migrate.activate_hint", name = &result.name));
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
        print_source_info(output, name, &source);
    }

    let Some(target) = resolve_target(output, name, &source.status, opts)? else {
        return Ok(()); // the user chose to skip
    };

    let migrator = Migrator::new()?;
    let options = MigrateOptions {
        dry_run: opts.dry_run,
        force: target.force,
        skip_packages: false,
        rename_to: (target.name != name).then(|| target.name.clone()),
        strict: opts.strict,
        delete_source: opts.delete_source,
        auto_install_python: false,
    };

    if !opts.json {
        if opts.dry_run {
            output.info(&t!("migrate.simulating"));
        } else {
            output.info(&t!("migrate.migrating"));
        }
    }

    let result = migrator.migrate(&source, &options)?;

    if opts.json {
        output.json_success("migrate", &result);
        return Ok(());
    }
    print_migration_result(output, &result, opts.dry_run);
    Ok(())
}

/// Where an environment will be migrated to, once its status is resolved.
struct Target {
    name: String,
    /// Overwrite an existing scuv env of that name.
    force: bool,
}

/// What is being migrated, before anything happens (human mode only).
fn print_source_info(output: &Output, name: &str, source: &SourceEnvironment) {
    output.info(&format!(
        "Source: {} ({}, Python {})",
        name, source.source_type, source.python_version
    ));
    output.info(&t!(
        "migrate.source_path",
        path = crate::paths::abbreviate_home(&source.path)
    ));

    if let Some(size_bytes) = source.size_bytes {
        let size_mb = size_bytes as f64 / 1_048_576.0;
        output.info(&t!("migrate.size", size = format!("{:.1}", size_mb)));
    }
}

/// Decides the target name and whether to overwrite, from the source's
/// status. `Ok(None)` means the user chose to skip; EOL (without --force)
/// and corrupted sources are errors.
fn resolve_target(
    output: &Output,
    name: &str,
    status: &EnvironmentStatus,
    opts: &MigrateExecuteOptions,
) -> Result<Option<Target>> {
    let target = Target {
        name: opts.rename.clone().unwrap_or_else(|| name.to_string()),
        force: opts.force,
    };

    match status {
        EnvironmentStatus::Ready => {}
        EnvironmentStatus::NameConflict { existing } => {
            return resolve_name_conflict(output, name, existing, opts, target, can_prompt());
        }
        EnvironmentStatus::PythonEol { version } => {
            if !opts.force {
                if !opts.json {
                    output.warn(&t!("migrate.eol_warning", version = version));
                    output.info(&t!("migrate.eol_force_hint"));
                }
                return Err(ScoopError::MigrationFailed {
                    reason: format!("Python {} is EOL", version),
                });
            }
            if !opts.json {
                output.warn(&t!("migrate.eol_proceeding", version = version));
            }
        }
        EnvironmentStatus::Corrupted { reason } => {
            if !opts.json {
                output.error(&t!("migrate.corrupted", reason = reason));
            }
            return Err(ScoopError::CorruptedEnvironment {
                name: name.to_string(),
                reason: reason.clone(),
            });
        }
    }
    Ok(Some(target))
}

/// Whether the conflict prompt can run: dialoguer draws it on stderr and
/// refuses ("not a terminal") when stderr is not one, so `2>log` fails it
/// too. A redirected stdin is fine — it reads keys from /dev/tty then.
fn can_prompt() -> bool {
    std::io::stderr().is_terminal()
}

/// A scuv env named `name` already exists. In order: `--auto-rename`
/// picks a free name; `--rename` must itself be free (unless `--force`);
/// `--force` overwrites; non-interactive runs (`--json`, `--yes`, or no
/// terminal to prompt on) fail with [`ScoopError::MigrationNameConflict`];
/// otherwise the user is asked.
fn resolve_name_conflict(
    output: &Output,
    name: &str,
    existing: &Path,
    opts: &MigrateExecuteOptions,
    mut target: Target,
    interactive: bool,
) -> Result<Option<Target>> {
    if opts.auto_rename {
        target.name = generate_unique_name(name)?;
        if !opts.json {
            output.info(&t!("migrate.auto_rename", name = &target.name));
        }
    } else if opts.rename.is_some() {
        let renamed_path = crate::paths::virtualenv_path(&target.name)?;
        if renamed_path.exists() && !opts.force {
            return Err(ScoopError::MigrationNameConflict {
                name: target.name,
                existing: renamed_path,
            });
        }
    } else if opts.force {
        if !opts.json {
            output.warn(&t!("migrate.overwriting"));
        }
    } else if opts.json || opts.yes || !interactive {
        if !opts.json {
            output.warn(&t!("migrate.name_exists", name = name));
            output.info(&t!("migrate.use_flags"));
        }
        return Err(ScoopError::MigrationNameConflict {
            name: name.to_string(),
            existing: existing.to_path_buf(),
        });
    } else {
        // Interactive (human mode, no --yes)
        match prompt_conflict_resolution(output, name, existing)? {
            ConflictResolution::Overwrite => {
                target.force = true;
                output.warn(&t!("migrate.will_overwrite"));
            }
            ConflictResolution::Rename => {
                target.name = prompt_rename(name)?;
                output.info(&t!("migrate.will_migrate_as", name = &target.name));
            }
            ConflictResolution::Skip => {
                output.info(&t!("migrate.skipped"));
                return Ok(None);
            }
        }
    }
    Ok(Some(target))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests;
