//! Environment listing display
//!
//! Displays discovered environments in human-readable or JSON format.

use crate::cli::MigrateSource;
use crate::core::migrate::{EnvironmentStatus, SourceType};
use crate::error::Result;
use crate::output::Output;

use super::scan::scan_all_environments;
use super::types::{MigrateListData, MigrateListSummary};

/// List environments available for migration.
///
/// Displays environments grouped by source type with status indicators:
/// - ✓ Ready to migrate
/// - ⚠ Has warnings (conflict, EOL)
/// - ✗ Corrupted
pub fn list_environments(
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
