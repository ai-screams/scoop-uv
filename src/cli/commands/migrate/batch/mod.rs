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

use rust_i18n::t;

use crate::cli::confirm::Confirmation;
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
    migrate_all_with(output, opts, confirm)
}

/// [`migrate_all_environments`] with the confirmation prompt passed in, so
/// tests can answer it.
fn migrate_all_with(
    output: &Output,
    opts: &MigrateExecuteOptions,
    confirm: impl FnOnce(usize) -> Result<bool>,
) -> Result<()> {
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
        invalid,
    } = partition_envs(
        &environments,
        opts.force,
        opts.allows_eol(),
        &paths::virtualenvs_dir()?,
    );
    let conflict_count = conflicts.len();

    // Nothing migratable: unresolved conflicts fail (exit 2, rendered here
    // first — the Quiet contract means main.rs adds nothing); otherwise only
    // EOL / corrupted entries were skipped and that is fine.
    if !opts.json {
        for f in &invalid {
            output.error(&t!("migrate.batch_item_failed", error = f.error.clone()));
        }
    }

    if migratable.is_empty() && invalid.is_empty() {
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

    let mut cancelled = false;
    if !migratable.is_empty() && !opts.json {
        report::print_plan(output, &migratable, skipped.len());
        if !opts.yes && !opts.dry_run && !confirm(migratable.len())? {
            output.info(&t!("migrate.batch_cancelled"));
            // Declining the plan does not undo the names already reported
            // as failed: they still fail the batch.
            if invalid.is_empty() {
                return Ok(());
            }
            cancelled = true;
        }
    }

    // Nothing left to run (only invalid names, or the plan was declined):
    // the invalid names still fail.
    let mut outcome = if migratable.is_empty() || cancelled {
        run::BatchOutcome {
            migrated: Vec::new(),
            failed: Vec::new(),
        }
    } else {
        run::run_batch(output, opts, &migratable)?
    };
    // The summary's total counts them, as it did when the migrator refused
    // them; a declined plan attempted none of the others.
    let attempted = if cancelled { 0 } else { migratable.len() } + invalid.len();
    merge_failures(&mut outcome.failed, invalid);
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
            attempted,
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
/// Adds the names that failed before the plan to the run's failures,
/// keeping the list sorted by env name as the JSON summary documents.
fn merge_failures(
    failed: &mut Vec<super::types::MigrateFailure>,
    invalid: Vec<super::types::MigrateFailure>,
) {
    failed.extend(invalid);
    failed.sort_by(|a, b| a.name.cmp(&b.name));
}

/// Asks whether to migrate `count` environments. No terminal to ask on is
/// an error naming `--yes`, not a "no".
fn confirm(count: usize) -> Result<bool> {
    println!();
    match crate::cli::confirm::confirm(&t!("migrate.batch_confirm", count = count)) {
        Confirmation::Yes => Ok(true),
        Confirmation::No => Ok(false),
        Confirmation::Unavailable => Err(ScoopError::ConfirmationUnavailable {
            flag: "--yes".to_string(),
        }),
    }
}
