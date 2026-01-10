//! Migration command implementation

use dialoguer::{Confirm, Input, Select};
use indicatif::{ProgressBar, ProgressStyle};
use serde::Serialize;

use crate::cli::{MigrateCommand, MigrateSource};
use crate::core::migrate::{
    CondaDiscovery, EnvironmentSource, EnvironmentStatus, MigrateOptions, MigrationResult,
    Migrator, PyenvDiscovery, SourceEnvironment, SourceType, VenvWrapperDiscovery,
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

/// Options for CLI migrate execution.
///
/// This struct consolidates the many boolean flags used by migrate commands
/// to improve readability and maintainability.
#[derive(Debug, Clone, Default)]
struct MigrateExecuteOptions {
    /// Preview migration without making changes
    dry_run: bool,
    /// Force overwrite existing environments
    force: bool,
    /// Skip confirmation prompts
    yes: bool,
    /// Output as JSON
    json: bool,
    /// Fail on first package error
    strict: bool,
    /// Delete original environment after successful migration
    delete_source: bool,
    /// Migrate with a different name
    rename: Option<String>,
    /// Auto-rename on conflict
    auto_rename: bool,
    /// Filter by source tool
    source_filter: Option<MigrateSource>,
}

/// User choice for handling name conflicts
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConflictResolution {
    Overwrite,
    Rename,
    Skip,
}

/// Prompts user for conflict resolution choice
fn prompt_conflict_resolution(
    output: &Output,
    name: &str,
    existing: &std::path::Path,
) -> Result<ConflictResolution> {
    output.warn(&format!(
        "Name conflict: '{}' already exists at {}",
        name,
        existing.display()
    ));

    let options = &[
        "Overwrite - Delete existing and migrate fresh",
        "Rename - Migrate with a different name",
        "Skip - Don't migrate this environment",
    ];

    let selection = Select::new()
        .with_prompt("How would you like to resolve this conflict?")
        .items(options)
        .default(2) // Default to Skip (safest)
        .interact()
        .map_err(|e| ScoopError::Io(std::io::Error::other(format!("Dialog error: {}", e))))?;

    Ok(match selection {
        0 => ConflictResolution::Overwrite,
        1 => ConflictResolution::Rename,
        _ => ConflictResolution::Skip,
    })
}

/// Prompts for new environment name when renaming
fn prompt_rename(name: &str) -> Result<String> {
    let suggested = format!("{}-pyenv", name);

    let new_name: String = Input::new()
        .with_prompt("Enter new name for the environment")
        .default(suggested)
        .validate_with(|input: &String| {
            crate::validate::validate_env_name(input)
                .map(|_| ())
                .map_err(|e| e.to_string())
        })
        .interact_text()
        .map_err(|e| ScoopError::Io(std::io::Error::other(format!("Dialog error: {}", e))))?;

    Ok(new_name)
}

/// Generates a unique name by appending suffixes
fn generate_unique_name(base_name: &str) -> Result<String> {
    // Try {name}-pyenv first
    let first_try = format!("{}-pyenv", base_name);
    if !crate::paths::virtualenv_path(&first_try)?.exists() {
        return Ok(first_try);
    }

    // Try numbered suffixes
    for i in 1..100 {
        let candidate = format!("{}-{}", base_name, i);
        if !crate::paths::virtualenv_path(&candidate)?.exists() {
            return Ok(candidate);
        }
    }

    Err(ScoopError::MigrationFailed {
        reason: format!("Could not find unique name for '{}'", base_name),
    })
}

/// Execute migrate command
pub fn execute(output: &Output, command: Option<MigrateCommand>) -> Result<()> {
    match command {
        Some(MigrateCommand::List { json, source }) => list_environments(output, json, source),
        Some(MigrateCommand::All {
            dry_run,
            force,
            yes,
            json,
            strict,
            delete_source,
            source,
        }) => {
            let opts = MigrateExecuteOptions {
                dry_run,
                force,
                yes,
                json,
                strict,
                delete_source,
                source_filter: source,
                ..Default::default()
            };
            migrate_all_environments(output, &opts)
        }
        Some(MigrateCommand::Env {
            name,
            dry_run,
            force,
            yes,
            json,
            strict,
            rename,
            auto_rename,
            delete_source,
            source,
        }) => {
            let opts = MigrateExecuteOptions {
                dry_run,
                force,
                yes,
                json,
                strict,
                rename,
                auto_rename,
                delete_source,
                source_filter: source,
            };
            migrate_environment(output, &name, &opts)
        }
        None => {
            // No subcommand - show help or list
            list_environments(output, output.is_json(), None)
        }
    }
}

/// Scan environments from all available sources or a specific source
fn scan_all_environments(source_filter: Option<MigrateSource>) -> Vec<SourceEnvironment> {
    let mut all_envs = Vec::new();

    // Scan sources based on filter
    let scan_pyenv = source_filter.is_none() || source_filter == Some(MigrateSource::Pyenv);
    let scan_venv =
        source_filter.is_none() || source_filter == Some(MigrateSource::Virtualenvwrapper);
    let scan_conda = source_filter.is_none() || source_filter == Some(MigrateSource::Conda);

    if scan_pyenv {
        if let Some(discovery) = PyenvDiscovery::default_root() {
            if let Ok(envs) = discovery.scan_environments() {
                all_envs.extend(envs);
            }
        }
    }

    if scan_venv {
        if let Some(discovery) = VenvWrapperDiscovery::default_root() {
            if let Ok(envs) = discovery.scan_environments() {
                all_envs.extend(envs);
            }
        }
    }

    if scan_conda {
        if let Some(discovery) = CondaDiscovery::default_roots() {
            if let Ok(envs) = discovery.scan_environments() {
                all_envs.extend(envs);
            }
        }
    }

    // Sort by source type, then by name
    all_envs.sort_by(|a, b| {
        let source_order = |s: &SourceType| match s {
            SourceType::Pyenv => 0,
            SourceType::VirtualenvWrapper => 1,
            SourceType::Conda => 2,
        };
        source_order(&a.source_type)
            .cmp(&source_order(&b.source_type))
            .then(a.name.cmp(&b.name))
    });

    all_envs
}

/// Find an environment by name, searching across sources
fn find_environment_by_name(
    name: &str,
    source_filter: Option<MigrateSource>,
) -> Result<SourceEnvironment> {
    // Try pyenv first
    if source_filter.is_none() || source_filter == Some(MigrateSource::Pyenv) {
        if let Some(discovery) = PyenvDiscovery::default_root() {
            if let Ok(env) = discovery.find_environment(name) {
                return Ok(env);
            }
        }
    }

    // Try virtualenvwrapper
    if source_filter.is_none() || source_filter == Some(MigrateSource::Virtualenvwrapper) {
        if let Some(discovery) = VenvWrapperDiscovery::default_root() {
            if let Ok(env) = discovery.find_environment(name) {
                return Ok(env);
            }
        }
    }

    // Try conda
    if source_filter.is_none() || source_filter == Some(MigrateSource::Conda) {
        if let Some(discovery) = CondaDiscovery::default_roots() {
            if let Ok(env) = discovery.find_environment(name) {
                return Ok(env);
            }
        }
    }

    // If a specific source was requested, return that error
    match source_filter {
        Some(MigrateSource::Pyenv) => Err(ScoopError::PyenvEnvNotFound {
            name: name.to_string(),
        }),
        Some(MigrateSource::Virtualenvwrapper) => Err(ScoopError::VenvWrapperEnvNotFound {
            name: name.to_string(),
        }),
        Some(MigrateSource::Conda) => Err(ScoopError::CondaEnvNotFound {
            name: name.to_string(),
        }),
        None => Err(ScoopError::PyenvEnvNotFound {
            name: name.to_string(),
        }),
    }
}

/// List environments available for migration
fn list_environments(
    output: &Output,
    json: bool,
    source_filter: Option<MigrateSource>,
) -> Result<()> {
    if !json {
        let source_name = source_filter
            .map(|s| s.to_string())
            .unwrap_or_else(|| "all sources".to_string());
        output.info(&format!("Scanning {} for environments...", source_name));
    }

    let environments = scan_all_environments(source_filter);

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

        let source_str = source_filter
            .map(|s| s.to_string())
            .unwrap_or_else(|| "all".to_string());

        output.json_success(
            "migrate list",
            MigrateListData {
                source: source_str,
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
        let source_name = source_filter.map(|s| format!("{} ", s)).unwrap_or_default();
        output.info(&format!("No {}environments found.", source_name));
        return Ok(());
    }

    output.success(&format!("Found {} environment(s):", environments.len()));
    println!();

    // Group by source type for display
    let mut current_source: Option<SourceType> = None;
    for env in &environments {
        // Print source header when it changes
        if current_source != Some(env.source_type) {
            if current_source.is_some() {
                println!();
            }
            println!("  [{}]", env.source_type);
            current_source = Some(env.source_type);
        }

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

        let size_mb = env.size_bytes.unwrap_or(0) as f64 / 1_048_576.0;
        let size_str = if env.size_bytes.is_some() {
            format!("{:>8.1} MB", size_mb)
        } else {
            "       - MB".to_string() // Not calculated
        };
        println!(
            "    {} {:<20} Python {:<10} {}{}",
            status_icon, env.name, env.python_version, size_str, status_hint
        );
    }

    println!();
    output.info("To migrate: scoop migrate @env <name>");
    output.info("To preview: scoop migrate @env <name> --dry-run");

    Ok(())
}

/// Migrate a single environment
fn migrate_environment(output: &Output, name: &str, opts: &MigrateExecuteOptions) -> Result<()> {
    let source = find_environment_by_name(name, opts.source_filter)?;

    if !opts.json {
        // Show environment info
        output.info(&format!(
            "Source: {} from {} (Python {})",
            name, source.source_type, source.python_version
        ));
        output.info(&format!("  Path: {}", source.path.display()));

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
                    output.info(&format!(
                        "Auto-renaming to '{}' to avoid conflict",
                        final_name
                    ));
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
                                output.warn("Will overwrite existing environment.");
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
                                output.info("Skipping migration.");
                            }
                            return Ok(());
                        }
                    }
                } else {
                    // Non-interactive mode: error out
                    if !opts.json {
                        output.warn(&format!(
                            "Name conflict: '{}' already exists at {}",
                            name,
                            existing.display()
                        ));
                        output.info("Use --force to overwrite, --rename to use different name, or --auto-rename.");
                    }
                    return Err(ScoopError::MigrationNameConflict {
                        name: name.to_string(),
                        existing: existing.clone(),
                    });
                }
            } else if !opts.json {
                output.warn("Forcing migration over existing environment.");
            }
        }
        EnvironmentStatus::PythonEol { version } => {
            if !opts.force {
                if !opts.json {
                    output.warn(&format!("Python {} is end-of-life.", version));
                    output.info("Use --force to migrate anyway.");
                }
                return Err(ScoopError::MigrationFailed {
                    reason: format!("Python {} is EOL", version),
                });
            }
            if !opts.json {
                output.warn("Migrating with EOL Python version.");
            }
        }
        EnvironmentStatus::Corrupted { reason } => {
            if !opts.json {
                output.error(&format!("Environment is corrupted: {}", reason));
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
            output.info("[DRY-RUN] Simulating migration...");
        } else {
            output.info("Starting migration...");
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

/// Migrate all environments at once
fn migrate_all_environments(output: &Output, opts: &MigrateExecuteOptions) -> Result<()> {
    if !opts.json {
        let source_name = opts
            .source_filter
            .map(|s| s.to_string())
            .unwrap_or_else(|| "all sources".to_string());
        output.info(&format!("Scanning {} for environments...", source_name));
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
            output.info(&format!("No {}environments found.", source_name));
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
                output.info("Migration cancelled.");
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
            output.info("[DRY-RUN] Simulating migration of all environments...");
        } else {
            output.info("");
            output.info("Starting batch migration...");
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
            pb.set_message(format!("Migrating '{}'...", env.name));
            pb.set_position(idx as u64);
        } else if !opts.json {
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
                } else if !opts.json {
                    if opts.dry_run {
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
                } else if !opts.json {
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
