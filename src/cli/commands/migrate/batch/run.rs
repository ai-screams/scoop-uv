//! Running a planned batch: one migration per environment, in parallel when
//! that helps, with a progress bar on a terminal.

use std::sync::Mutex;

use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use rust_i18n::t;

use crate::core::migrate::{MigrateOptions, MigrationResult, Migrator, SourceEnvironment};
use crate::error::Result;
use crate::output::Output;

use super::super::types::{MigrateExecuteOptions, MigrateFailure};

/// What a batch run produced, each list sorted by environment name.
pub(super) struct BatchOutcome {
    pub(super) migrated: Vec<MigrationResult>,
    pub(super) failed: Vec<MigrateFailure>,
}

/// Migrates every environment in `migratable` and collects the results.
///
/// Per-environment failures land in [`BatchOutcome::failed`]; only a failure
/// to set up the migrator itself returns `Err`.
pub(super) fn run_batch(
    output: &Output,
    opts: &MigrateExecuteOptions,
    migratable: &[&SourceEnvironment],
) -> Result<BatchOutcome> {
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

    // Wrapped in Mutex so the parallel branch below can collect results from
    // multiple worker threads. The sequential branch (dry-run / single env)
    // also goes through the locks for code-path uniformity — contention is
    // zero there, so the cost is a few ns per env.
    let migrated_lock: Mutex<Vec<MigrationResult>> = Mutex::new(Vec::new());
    let failed_lock: Mutex<Vec<MigrateFailure>> = Mutex::new(Vec::new());

    if !opts.json {
        output.info("");
        if opts.dry_run {
            output.info(&t!("migrate.batch_dry_run"));
        } else {
            output.info(&t!("migrate.batch_start"));
        }
    }

    let progress = progress_bar(output, opts, migratable.len());

    // Per-env work. Shared by the sequential and parallel branches. Each
    // branch in `progress` is thread-safe (`indicatif` serialises
    // println/inc internally) and `output.{success,info,warn,error}` emit one
    // `eprintln!` per call, which is atomic at the line level.
    let run_one = |env: &SourceEnvironment| {
        if let Some(ref pb) = progress {
            pb.set_message(t!("migrate.batch_item", name = &env.name).to_string());
        } else if !opts.json {
            output.info("");
            output.info(&t!("migrate.batch_item", name = &env.name));
        }

        match migrator.migrate(env, &options) {
            Ok(result) => {
                report_success(output, opts, progress.as_ref(), &result);
                migrated_lock
                    .lock()
                    .expect("results lock poisoned")
                    .push(result);
            }
            Err(e) => {
                let msg = e.to_string();
                if let Some(ref pb) = progress {
                    pb.println(format!("✗ '{}' failed: {}", env.name, msg));
                } else if !opts.json {
                    output.error(&t!("migrate.batch_item_failed", error = msg.clone()));
                }
                failed_lock
                    .lock()
                    .expect("failures lock poisoned")
                    .push(MigrateFailure {
                        name: env.name.clone(),
                        source_type: env.source_type,
                        error_code: e.code(),
                        error: msg,
                    });
            }
        }

        if let Some(ref pb) = progress {
            pb.inc(1);
        }
    };

    // Parallelise only when there's a real win: dry-run does no I/O work
    // (sequential gives cleaner, deterministic preview output) and a single
    // env has nothing to parallelise.
    if opts.dry_run || migratable.len() <= 1 {
        migratable.iter().for_each(|env| run_one(env));
    } else {
        migratable.par_iter().for_each(|env| run_one(env));
    }

    if let Some(pb) = progress {
        pb.finish_with_message("Done");
    }

    let mut migrated = migrated_lock.into_inner().expect("results lock poisoned");
    let mut failed = failed_lock.into_inner().expect("failures lock poisoned");
    // Sort by env name so the summary and JSON output are deterministic
    // regardless of which worker thread finished first. This is NOT the
    // original scan order (scan sorts by source type then name) — across
    // source types or with duplicate names the two orders diverge, and
    // alphabetic-by-name is the cheaper, more useful default for a
    // user-facing summary.
    migrated.sort_by(|a, b| a.name.cmp(&b.name));
    failed.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(BatchOutcome { migrated, failed })
}

/// The progress bar, only for a real (not dry-run) human-facing run.
fn progress_bar(output: &Output, opts: &MigrateExecuteOptions, len: usize) -> Option<ProgressBar> {
    if opts.json || opts.dry_run || output.is_quiet() {
        return None;
    }
    let pb = ProgressBar::new(len as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(progress_template(output))
            .expect("valid template")
            .progress_chars("=>-"),
    );
    Some(pb)
}

/// Prints one environment's success line: through the progress bar when
/// there is one, otherwise as regular output.
fn report_success(
    output: &Output,
    opts: &MigrateExecuteOptions,
    progress: Option<&ProgressBar>,
    result: &MigrationResult,
) {
    if let Some(pb) = progress {
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
        return;
    }
    if opts.json {
        return;
    }
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

/// Returns the progress bar template; the colorless one when color is off.
///
/// indicatif applies the colors named in the template. It already drops them
/// on its own for `NO_COLOR` or a non-terminal stderr, but it cannot see
/// `--color`/`--no-color`.
fn progress_template(output: &Output) -> &'static str {
    if output.use_color_stderr() {
        "{spinner:.green} [{bar:30.cyan/blue}] {pos}/{len} {msg}"
    } else {
        "{spinner} [{bar:30}] {pos}/{len} {msg}"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output::Colors;

    /// `--no-color` must reach the progress bar too. Fails if the template
    /// ignores the color setting, either way round.
    #[test]
    fn progress_template_has_no_style_without_color() {
        let plain = progress_template(&Output::new(0, false, Colors::NONE, false));
        let colored = progress_template(&Output::new(0, false, Colors::ALL, false));
        for style in [".green", ".cyan", "/blue"] {
            assert!(!plain.contains(style), "{plain} carries {style}");
        }
        assert!(colored.contains(".green") && colored.contains("cyan/blue"));
        // Both must stay valid indicatif templates.
        ProgressStyle::default_bar().template(plain).unwrap();
        ProgressStyle::default_bar().template(colored).unwrap();
    }
}
