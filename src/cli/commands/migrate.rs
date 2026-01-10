//! Migration command implementation

use crate::cli::MigrateCommand;
use crate::core::migrate::{
    EnvironmentSource, EnvironmentStatus, MigrateOptions, Migrator, PyenvDiscovery,
};
use crate::error::{Result, ScoopError};
use crate::output::Output;

/// Execute migrate command
pub fn execute(output: &Output, command: Option<MigrateCommand>) -> Result<()> {
    match command {
        Some(MigrateCommand::List { json: _ }) => list_environments(output),
        Some(MigrateCommand::Env {
            name,
            dry_run,
            force,
            yes,
            json: _,
        }) => migrate_environment(output, &name, dry_run, force, yes),
        None => {
            // No subcommand - show help or list
            list_environments(output)
        }
    }
}

/// List environments available for migration
fn list_environments(output: &Output) -> Result<()> {
    output.info("Scanning for pyenv environments...");

    let discovery = PyenvDiscovery::default_root().ok_or(ScoopError::PyenvNotFound)?;

    let environments = discovery.scan_environments()?;

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
) -> Result<()> {
    let discovery = PyenvDiscovery::default_root().ok_or(ScoopError::PyenvNotFound)?;

    let source = discovery.find_environment(name)?;

    // Show environment info
    output.info(&format!(
        "Source: {} (Python {})",
        name, source.python_version
    ));
    output.info(&format!("  Path: {}", source.path.display()));

    let size_mb = source.size_bytes as f64 / 1_048_576.0;
    output.info(&format!("  Size: {:.1} MB", size_mb));

    // Check status
    match &source.status {
        EnvironmentStatus::Ready => {}
        EnvironmentStatus::NameConflict { existing } => {
            if !force {
                output.warn(&format!(
                    "Name conflict: '{}' already exists at {}",
                    name,
                    existing.display()
                ));
                output.info("Use --force to overwrite.");
                return Err(ScoopError::MigrationNameConflict {
                    name: name.to_string(),
                    existing: existing.clone(),
                });
            }
            output.warn("Forcing migration over existing environment.");
        }
        EnvironmentStatus::PythonEol { version } => {
            if !force {
                output.warn(&format!("Python {} is end-of-life.", version));
                output.info("Use --force to migrate anyway.");
                return Err(ScoopError::MigrationFailed {
                    reason: format!("Python {} is EOL", version),
                });
            }
            output.warn("Migrating with EOL Python version.");
        }
        EnvironmentStatus::Corrupted { reason } => {
            output.error(&format!("Environment is corrupted: {}", reason));
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

    if dry_run {
        output.info("[DRY-RUN] Simulating migration...");
    } else {
        output.info("Starting migration...");
    }

    // Perform migration
    let result = migrator.migrate(&source, &options)?;

    // Report results
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

    Ok(())
}
