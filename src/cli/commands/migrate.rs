//! Migration command implementation

use dialoguer::Confirm;
use indicatif::{ProgressBar, ProgressStyle};
use serde::Serialize;

use crate::cli::MigrateCommand;
use crate::core::migrate::{
    EnvironmentSource, EnvironmentStatus, MigrateOptions, MigrationResult, Migrator,
    PyenvDiscovery, SourceEnvironment,
};
use crate::error::{Result, ScoopError};
use crate::output::Output;

/// JSON output for migrate list command
#[derive(Debug, Serialize)]
struct MigrateListData {
    source: String,
    environments: Vec<SourceEnvironment>,
    summary: MigrateListSummary,
}

/// Summary statistics for migrate list
#[derive(Debug, Serialize)]
struct MigrateListSummary {
    total: usize,
    ready: usize,
    conflict: usize,
    eol: usize,
    corrupted: usize,
}

/// JSON output for migrate all command
#[derive(Debug, Serialize)]
struct MigrateAllData {
    migrated: Vec<MigrationResult>,
    failed: Vec<MigrateFailure>,
    skipped: Vec<MigrateSkipped>,
    summary: MigrateAllSummary,
}

/// Failed migration info
#[derive(Debug, Serialize)]
struct MigrateFailure {
    name: String,
    error: String,
}

/// Skipped environment info
#[derive(Debug, Serialize)]
struct MigrateSkipped {
    name: String,
    reason: String,
}

/// Summary for migrate all
#[derive(Debug, Serialize)]
struct MigrateAllSummary {
    total: usize,
    success: usize,
    failed: usize,
    skipped: usize,
}

/// Execute migrate command
pub fn execute(output: &Output, command: Option<MigrateCommand>) -> Result<()> {
    match command {
        Some(MigrateCommand::List { json }) => list_environments(output, json),
        Some(MigrateCommand::All {
            dry_run,
            force,
            yes,
            json,
        }) => migrate_all_environments(output, dry_run, force, yes, json),
        Some(MigrateCommand::Env {
            name,
            dry_run,
            force,
            yes,
            json,
        }) => migrate_environment(output, &name, dry_run, force, yes, json),
        None => {
            // No subcommand - show help or list
            list_environments(output, output.is_json())
        }
    }
}

/// List environments available for migration
fn list_environments(output: &Output, json: bool) -> Result<()> {
    if !json {
        output.info("Scanning for pyenv environments...");
    }

    let discovery = PyenvDiscovery::default_root().ok_or(ScoopError::PyenvNotFound)?;

    let environments = discovery.scan_environments()?;

    // JSON output
    if json {
        let mut ready = 0;
        let mut conflict = 0;
        let mut eol = 0;
        let mut corrupted = 0;

        for env in &environments {
            match &env.status {
                EnvironmentStatus::Ready => ready += 1,
                EnvironmentStatus::NameConflict { .. } => conflict += 1,
                EnvironmentStatus::PythonEol { .. } => eol += 1,
                EnvironmentStatus::Corrupted { .. } => corrupted += 1,
            }
        }

        output.json_success(
            "migrate list",
            MigrateListData {
                source: "pyenv".to_string(),
                environments,
                summary: MigrateListSummary {
                    total: ready + conflict + eol + corrupted,
                    ready,
                    conflict,
                    eol,
                    corrupted,
                },
            },
        );
        return Ok(());
    }

    if environments.is_empty() {
        output.info("No pyenv environments found.");
        return Ok(());
    }

    output.success(&format!(
        "Found {} pyenv environment(s):",
        environments.len()
    ));
    println!();

    for env in &environments {
        let (status_icon, status_hint) = match &env.status {
            EnvironmentStatus::Ready => ("✓", "".to_string()),
            EnvironmentStatus::NameConflict { existing } => {
                ("⚠", format!(" (conflicts with {})", existing.display()))
            }
            EnvironmentStatus::PythonEol { version } => {
                ("⚠", format!(" (Python {} is EOL)", version))
            }
            EnvironmentStatus::Corrupted { reason } => ("✗", format!(" ({})", reason)),
        };

        let size_mb = env.size_bytes as f64 / 1_048_576.0;
        println!(
            "  {} {:<20} Python {:<10} {:>8.1} MB{}",
            status_icon, env.name, env.python_version, size_mb, status_hint
        );
    }

    println!();
    output.info("To migrate: scoop migrate <name>");
    output.info("To preview: scoop migrate <name> --dry-run");

    Ok(())
}

/// Migrate a single environment
fn migrate_environment(
    output: &Output,
    name: &str,
    dry_run: bool,
    force: bool,
    _yes: bool,
    json: bool,
) -> Result<()> {
    let discovery = PyenvDiscovery::default_root().ok_or(ScoopError::PyenvNotFound)?;

    let source = discovery.find_environment(name)?;

    if !json {
        // Show environment info
        output.info(&format!(
            "Source: {} (Python {})",
            name, source.python_version
        ));
        output.info(&format!("  Path: {}", source.path.display()));

        let size_mb = source.size_bytes as f64 / 1_048_576.0;
        output.info(&format!("  Size: {:.1} MB", size_mb));
    }

    // Check status
    match &source.status {
        EnvironmentStatus::Ready => {}
        EnvironmentStatus::NameConflict { existing } => {
            if !force {
                if !json {
                    output.warn(&format!(
                        "Name conflict: '{}' already exists at {}",
                        name,
                        existing.display()
                    ));
                    output.info("Use --force to overwrite.");
                }
                return Err(ScoopError::MigrationNameConflict {
                    name: name.to_string(),
                    existing: existing.clone(),
                });
            }
            if !json {
                output.warn("Forcing migration over existing environment.");
            }
        }
        EnvironmentStatus::PythonEol { version } => {
            if !force {
                if !json {
                    output.warn(&format!("Python {} is end-of-life.", version));
                    output.info("Use --force to migrate anyway.");
                }
                return Err(ScoopError::MigrationFailed {
                    reason: format!("Python {} is EOL", version),
                });
            }
            if !json {
                output.warn("Migrating with EOL Python version.");
            }
        }
        EnvironmentStatus::Corrupted { reason } => {
            if !json {
                output.error(&format!("Environment is corrupted: {}", reason));
            }
            return Err(ScoopError::CorruptedEnvironment {
                name: name.to_string(),
                reason: reason.clone(),
            });
        }
    }

    // Create migrator and options
    let migrator = Migrator::new();
    let options = MigrateOptions {
        dry_run,
        force,
        skip_packages: false,
        rename_to: None,
    };

    if !json {
        if dry_run {
            output.info("[DRY-RUN] Simulating migration...");
        } else {
            output.info("Starting migration...");
        }
    }

    // Perform migration
    let result = migrator.migrate(&source, &options)?;

    // JSON output
    if json {
        output.json_success("migrate", &result);
        return Ok(());
    }

    // Report results
    print_migration_result(output, &result, dry_run);

    Ok(())
}

/// Migrate all environments at once
fn migrate_all_environments(
    output: &Output,
    dry_run: bool,
    force: bool,
    yes: bool,
    json: bool,
) -> Result<()> {
    if !json {
        output.info("Scanning for pyenv environments...");
    }

    let discovery = PyenvDiscovery::default_root().ok_or(ScoopError::PyenvNotFound)?;
    let environments = discovery.scan_environments()?;

    if environments.is_empty() {
        if json {
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
            output.info("No pyenv environments found.");
        }
        return Ok(());
    }

    // Filter to migratable environments
    let migratable: Vec<_> = environments
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
        if json {
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
            output.info("No environments eligible for migration.");
            if skipped_count > 0 {
                output.info(&format!(
                    "  {} environment(s) skipped (corrupted or require --force)",
                    skipped_count
                ));
            }
        }
        return Ok(());
    }

    // Show what will be migrated (only in non-JSON mode)
    if !json {
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
        if !yes && !dry_run {
            println!();
            let confirmed = Confirm::new()
                .with_prompt(format!("Migrate {} environment(s)?", migratable.len()))
                .default(false)
                .interact()
                .map_err(|e| ScoopError::Io(std::io::Error::other(e)))?;

            if !confirmed {
                output.info("Migration cancelled.");
                return Ok(());
            }
        }
    }

    // Perform migrations
    let migrator = Migrator::new();
    let options = MigrateOptions {
        dry_run,
        force,
        skip_packages: false,
        rename_to: None,
    };

    let mut migrated: Vec<MigrationResult> = Vec::new();
    let mut failed: Vec<MigrateFailure> = Vec::new();

    if !json {
        if dry_run {
            output.info("");
            output.info("[DRY-RUN] Simulating migration of all environments...");
        } else {
            output.info("");
            output.info("Starting batch migration...");
        }
    }

    // Create progress bar for batch migration
    let progress = if !json && !dry_run && !output.is_quiet() {
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
            pb.set_message(format!("Migrating '{}'...", env.name));
            pb.set_position(idx as u64);
        } else if !json {
            output.info("");
            output.info(&format!("Migrating '{}'...", env.name));
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
                } else if !json {
                    if dry_run {
                        output.info(&format!(
                            "  [DRY-RUN] Would create: {}",
                            result.path.display()
                        ));
                        output.info(&format!(
                            "  Packages to install: {}",
                            result.packages_migrated
                        ));
                    } else {
                        output.success(&format!("  '{}' migrated successfully", result.name));
                        output.info(&format!(
                            "  Packages installed: {}",
                            result.packages_migrated
                        ));
                        if !result.packages_failed.is_empty() {
                            output.warn(&format!(
                                "  Packages failed: {}",
                                result.packages_failed.len()
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
                } else if !json {
                    output.error(&format!("  Failed: {}", e));
                }
            }
        }
    }

    // Finish progress bar
    if let Some(pb) = progress {
        pb.finish_with_message("Done");
    }

    // JSON output
    if json {
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
    if dry_run {
        output.info(&format!(
            "[DRY-RUN] Would migrate: {}/{}",
            migrated.len(),
            migratable.len()
        ));
        output.info("No changes made. Run without --dry-run to migrate.");
    } else {
        output.success(&format!(
            "Migration complete: {}/{} succeeded",
            migrated.len(),
            migratable.len()
        ));
    }

    if !failed.is_empty() {
        let failed_names: Vec<_> = failed.iter().map(|f| f.name.as_str()).collect();
        output.warn(&format!("Failed environments: {}", failed_names.join(", ")));
    }

    Ok(())
}

/// Print migration result in human-readable format
fn print_migration_result(output: &Output, result: &MigrationResult, dry_run: bool) {
    if dry_run {
        output.info("");
        output.info("[DRY-RUN] Migration preview:");
        output.info(&format!("  Would create: {}", result.path.display()));
        output.info(&format!("  Python version: {}", result.python_version));
        output.info(&format!(
            "  Packages to install: {}",
            result.packages_migrated
        ));

        if !result.packages_failed.is_empty() {
            output.warn(&format!(
                "  Packages that may fail: {}",
                result.packages_failed.len()
            ));
            for pkg in &result.packages_failed {
                output.info(&format!("    - {}", pkg));
            }
        }

        output.info("");
        output.info("No changes made. Run without --dry-run to migrate.");
    } else {
        output.success(&format!(
            "Environment '{}' migrated successfully!",
            result.name
        ));
        output.info(&format!("  Path: {}", result.path.display()));
        output.info(&format!("  Python: {}", result.python_version));
        output.info(&format!(
            "  Packages installed: {}",
            result.packages_migrated
        ));

        if !result.packages_failed.is_empty() {
            output.warn(&format!(
                "  Packages failed: {} (may require manual installation)",
                result.packages_failed.len()
            ));
            for pkg in &result.packages_failed {
                output.info(&format!("    - {}", pkg));
            }
        }

        output.info("");
        output.info(&format!("Activate with: scoop use {}", result.name));
    }
}
