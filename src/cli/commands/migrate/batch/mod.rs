//! `scuv migrate all`: migrate every environment the scan finds.
//!
//! The flow lives here; each step has its own module:
//! - [`plan`] decides which environments migrate, conflict or are skipped,
//! - [`run`] migrates them (in parallel when it helps),
//! - [`report`] prints the preview, the summary and the JSON envelope.

mod plan;
mod report;
mod run;

#[cfg(test)]
mod tests;

use dialoguer::Confirm;
use rust_i18n::t;

use crate::error::{Result, ScoopError};
use crate::output::Output;
use crate::paths;

use super::scan::{any_source_tool_available, scan_all_environments, scanning_message};
use super::types::MigrateExecuteOptions;
use plan::{PartitionedEnvs, partition_envs};

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
        output.info(&scanning_message(opts.source_filter, &rust_i18n::locale()));
    }

    let environments = scan_all_environments(opts.source_filter);
    let total = environments.len();

    // Empty scan: no tool installed is exit 3; a tool with no envs is Ok.
    if environments.is_empty() {
        if !any_source_tool_available(opts.source_filter) {
            return Err(ScoopError::MigrationSourcesNotFound {
                requested: opts.source_filter.map(|s| s.to_string()),
            });
        }
        report::emit_empty_envs(output, opts);
        return Ok(());
    }

    // Conflicts are tracked both as structured `MigrationConflictDetail`
    // entries (new in 0.14, see types.rs) AND, for backward compat, as
    // `MigrateSkipped` entries with the historical "name conflict
    // (use --force)" reason. Existing scripts that read `skipped[]`
    // continue to work.
    let PartitionedEnvs {
        migratable,
        conflicts,
        skipped,
    } = partition_envs(&environments, opts.force, &paths::virtualenvs_dir()?);
    let conflict_count = conflicts.len();

    // Nothing migratable: unresolved conflicts fail (exit 2, rendered here
    // first — the Quiet contract means main.rs adds nothing); otherwise only
    // EOL / corrupted entries were skipped and that is fine.
    if migratable.is_empty() {
        if conflict_count == 0 {
            report::emit_nothing_to_migrate(output, opts, skipped, conflicts, total);
            return Ok(());
        }
        if opts.json {
            report::emit_migrate_all_json_outcome(&[], &[], &conflicts, &skipped, total, true);
        } else {
            report::render_no_migratable_with_conflicts(output, &conflicts, &skipped);
        }
        return Err(ScoopError::MigrationBatchFailed {
            failed_count: 0,
            conflict_count,
        });
    }

    if !opts.json {
        report::print_plan(output, &migratable, skipped.len());
        if !opts.yes && !opts.dry_run && !confirm(migratable.len())? {
            output.info(&t!("migrate.batch_cancelled"));
            return Ok(());
        }
    }

    let outcome = run::run_batch(output, opts, &migratable)?;
    let failed_count = outcome.failed.len();
    let is_failure = failed_count + conflict_count > 0;

    // Render BEFORE returning Err so the Quiet render policy on
    // MigrationBatchFailed is satisfied (main.rs writes nothing extra).
    if opts.json {
        report::emit_migrate_all_json_outcome(
            &outcome.migrated,
            &outcome.failed,
            &conflicts,
            &skipped,
            total,
            is_failure,
        );
    } else {
        report::render_human_summary(
            output,
            opts,
            &outcome.migrated,
            &outcome.failed,
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

/// Asks before migrating `count` environments.
fn confirm(count: usize) -> Result<bool> {
    println!();
    Confirm::new()
        .with_prompt(format!("Migrate {} environment(s)?", count))
        .default(false)
        .interact()
        .map_err(|e| ScoopError::Io(std::io::Error::other(e)))
}
