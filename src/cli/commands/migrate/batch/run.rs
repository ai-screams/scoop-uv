//! Running a planned batch: every environment prepared and checked first,
//! then migrated one at a time (a removal can then never interleave with
//! another env's creation), with source deletions made at the end.

use std::io::IsTerminal;

use indicatif::{ProgressBar, ProgressStyle};
use rust_i18n::t;

use crate::core::migrate::{MigrationResult, Migrator, Prepared, SourceEnvironment};
use crate::error::{Result, ScoopError};
use crate::output::Output;

use super::super::types::{MigrateExecuteOptions, MigrateFailure};

/// What a batch run produced, each list sorted by environment name.
pub(super) struct BatchOutcome {
    pub(super) migrated: Vec<MigrationResult>,
    pub(super) failed: Vec<MigrateFailure>,
}

/// Migrates every environment in `migratable` and collects the results.
/// `others` are the envs found but not migrated (skipped, conflicting, an
/// invalid name): their packages must survive what the batch deletes.
///
/// Per-environment failures land in [`BatchOutcome::failed`]; only a failure
/// to set up the migrator or the checks returns `Err`.
pub(super) fn run_batch(
    output: &Output,
    opts: &MigrateExecuteOptions,
    migratable: &[&SourceEnvironment],
    others: &[&SourceEnvironment],
) -> Result<BatchOutcome> {
    // The migrator is set up first: failing here aborts the batch before
    // anything is printed.
    let migrator = Migrator::new()?;

    if !opts.json {
        output.info("");
        if opts.dry_run {
            output.info(&t!("migrate.batch_dry_run"));
        } else {
            output.info(&t!("migrate.batch_start"));
        }
    }

    let options = opts.migrate_options(opts.force, None);
    let progress = progress_bar(
        output,
        opts,
        migratable.len(),
        std::io::stderr().is_terminal(),
    );
    let plan = migrator.plan_batch(migratable, others, &options)?;
    let mut failed = Vec::new();
    for (source, e) in &plan.failed {
        report_failure(output, opts, progress.as_ref(), &source.name, e);
        failed.push(failure(source, e));
        if let Some(pb) = &progress {
            pb.inc(1);
        }
    }

    let (mut results, run_failed) = migrator.execute_batch(
        &plan,
        &options,
        |env| announce(output, opts, progress.as_ref(), &env.source.name),
        |env, outcome| {
            match outcome {
                Ok(result) => report_success(output, opts, progress.as_ref(), result),
                Err(e) => report_failure(output, opts, progress.as_ref(), &env.source.name, e),
            }
            if let Some(pb) = &progress {
                pb.inc(1);
            }
        },
    )?;
    failed.extend(run_failed.iter().map(|(source, e)| failure(source, e)));

    if !opts.dry_run {
        let all: Vec<&Prepared> = plan.ready.iter().collect();
        let mut done: Vec<(usize, &mut MigrationResult)> =
            results.iter_mut().map(|(i, r)| (*i, r)).collect();
        migrator.finish_deletion(&all, &plan.extra, &mut done)?;
        for (_, result) in &results {
            report_kept(output, opts, progress.as_ref(), result);
        }
    }

    if let Some(pb) = &progress {
        pb.finish_with_message(t!("migrate.progress_done").to_string());
    }

    let mut migrated: Vec<MigrationResult> = results.into_iter().map(|(_, r)| r).collect();
    // Sort by env name so the summary and JSON output are deterministic.
    // This is NOT the original scan order (scan sorts by source type then
    // name) — across source types or with duplicate names the two orders
    // diverge, and alphabetic-by-name is the cheaper, more useful default
    // for a user-facing summary.
    migrated.sort_by(|a, b| a.name.cmp(&b.name));
    failed.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(BatchOutcome { migrated, failed })
}

/// One environment's failure as the batch report records it.
fn failure(source: &SourceEnvironment, e: &ScoopError) -> MigrateFailure {
    MigrateFailure {
        name: source.name.clone(),
        source_type: source.source_type,
        error_code: e.code(),
        error: e.to_string(),
    }
}

/// The line that names the environment about to migrate.
fn announce(
    output: &Output,
    opts: &MigrateExecuteOptions,
    progress: Option<&ProgressBar>,
    name: &str,
) {
    if let Some(pb) = progress {
        pb.set_message(t!("migrate.batch_item", name = name).to_string());
    } else if !opts.json {
        output.info("");
        output.info(&t!("migrate.batch_item", name = name));
    }
}

/// Prints one environment's failure.
fn report_failure(
    output: &Output,
    opts: &MigrateExecuteOptions,
    progress: Option<&ProgressBar>,
    name: &str,
    e: &ScoopError,
) {
    let msg = e.to_string();
    if let Some(pb) = progress {
        pb.println(progress_failed_line(name, &msg));
    } else if !opts.json {
        output.error(&t!("migrate.batch_item_failed", error = msg));
    }
}

/// Prints that `--delete-source` kept a source, with why (known only
/// once every env ran).
fn report_kept(
    output: &Output,
    opts: &MigrateExecuteOptions,
    progress: Option<&ProgressBar>,
    result: &MigrationResult,
) {
    if !opts.kept_source(result) || opts.json {
        return;
    }
    let reason = result
        .source_kept_reason
        .clone()
        .unwrap_or_else(|| t!("migrate.kept_failed").to_string());
    let line = t!("migrate.source_kept", name = &result.name, reason = reason).to_string();
    match progress {
        Some(pb) => pb.println(format!("  ⚠ {line}")),
        None => output.warn(&line),
    }
}

/// The progress bar, only for a real (not dry-run) human-facing run on a
/// terminal. indicatif hides a bar on a stderr that is not one and drops
/// what `println` sends through it, so redirected stderr (`2>log`) would
/// lose the per-env lines and warnings; ordinary output prints them then.
fn progress_bar(
    output: &Output,
    opts: &MigrateExecuteOptions,
    len: usize,
    stderr_is_terminal: bool,
) -> Option<ProgressBar> {
    if opts.json || opts.dry_run || output.is_quiet() || !stderr_is_terminal {
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
        for line in progress_success_lines(result) {
            pb.println(line);
        }
        return;
    }
    if opts.json {
        return;
    }
    if opts.dry_run {
        output.info(&t!(
            "migrate.dry_run_would_create",
            path = result.path.display()
        ));
        output.info(&t!("migrate.packages", count = result.packages_migrated));
    } else {
        output.success(&t!("migrate.batch_item_success", name = &result.name));
        output.info(&t!("migrate.packages", count = result.packages_migrated));
    }
    super::super::single::print_package_notes(output, result);
}

/// The progress-bar line for one environment that failed.
fn progress_failed_line(name: &str, error: &str) -> String {
    t!("migrate.progress_failed", name = name, error = error).to_string()
}

/// The progress-bar lines for one migrated environment: the package count,
/// then the number of packages that did not install, if any.
fn progress_success_lines(result: &MigrationResult) -> Vec<String> {
    let mut lines = vec![
        t!(
            "migrate.progress_migrated",
            name = &result.name,
            count = result.packages_migrated
        )
        .to_string(),
    ];
    if !result.packages_failed.is_empty() {
        lines.push(
            t!(
                "migrate.progress_packages_failed",
                count = result.packages_failed.len()
            )
            .to_string(),
        );
    }
    // What changed on the way, as single-env output says it: pip left out,
    // conda's builds from the registry, dependencies added.
    for excluded in &result.packages_excluded {
        lines.push(format!(
            "  {}",
            t!(
                "migrate.excluded",
                name = &excluded.name,
                version = &excluded.version
            )
        ));
    }
    if !result.packages_from_conda.is_empty() {
        lines.push(format!(
            "  {}",
            t!(
                "migrate.from_conda",
                count = result.packages_from_conda.len()
            )
        ));
    }
    if !result.packages_added.is_empty() {
        lines.push(format!(
            "  {}",
            t!("migrate.added", count = result.packages_added.len())
        ));
    }
    lines
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

    /// The progress-bar lines are word for word the English they were and
    /// follow the language; the failed-packages line appears only when a
    /// package failed. Fails if a line is hard-coded again or the second
    /// line shows up for a clean migration.
    #[test]
    #[serial_test::serial]
    fn progress_lines_follow_the_language() {
        let _locale = crate::test_utils::LocaleGuard::capture();
        let mut result = MigrationResult {
            name: "web".to_string(),
            packages_migrated: 3,
            packages_failed: Vec::new(),
            packages_excluded: Vec::new(),
            packages_from_conda: Vec::new(),
            packages_added: Vec::new(),
            python_version: "3.12".to_string(),
            dry_run: false,
            path: std::path::PathBuf::from("/v/web"),
            source_deleted: false,
            source_kept_reason: None,
            actual_python_version: "3.12".to_string(),
        };
        rust_i18n::set_locale("en");
        assert_eq!(
            progress_success_lines(&result),
            ["✓ 'web' migrated (3 packages)"]
        );
        assert_eq!(progress_failed_line("web", "boom"), "✗ 'web' failed: boom");
        result.packages_failed = vec!["a".to_string(), "b".to_string()];
        assert_eq!(
            progress_success_lines(&result),
            ["✓ 'web' migrated (3 packages)", "  ⚠ 2 package(s) failed"]
        );
        rust_i18n::set_locale("ko");
        assert_eq!(
            progress_success_lines(&result),
            [
                "✓ 'web' 마이그레이션 완료 (패키지 3개)",
                "  ⚠ 패키지 2개 실패"
            ]
        );
        assert_eq!(progress_failed_line("web", "boom"), "✗ 'web' 실패: boom");
    }
    /// The bar also says what changed on the way: pip left out, conda's
    /// builds from the registry, dependencies added. Fails if a batch run
    /// hides a conda substitution behind a success line.
    #[test]
    #[serial_test::serial]
    fn progress_lines_report_what_changed() {
        let _locale = crate::test_utils::LocaleGuard::capture();
        rust_i18n::set_locale("en");
        let result = MigrationResult {
            name: "web".to_string(),
            packages_migrated: 2,
            packages_failed: Vec::new(),
            packages_excluded: vec![crate::core::migrate::ExcludedPackage {
                name: "pip".to_string(),
                version: "24.0".to_string(),
                reason: "installer".to_string(),
            }],
            packages_from_conda: vec!["numpy".to_string()],
            packages_added: vec!["idna==3.4".to_string()],
            python_version: "3.12".to_string(),
            dry_run: false,
            path: std::path::PathBuf::from("/v/web"),
            source_deleted: false,
            source_kept_reason: None,
            actual_python_version: "3.12".to_string(),
        };
        let lines = progress_success_lines(&result);
        assert_eq!(lines.len(), 4, "{lines:?}");
        assert!(lines[1].contains("pip 24.0"), "{lines:?}");
        assert!(lines[2].contains("instead of conda: 1"), "{lines:?}");
        assert!(lines[3].contains("dependencies"), "{lines:?}");
    }

    use crate::output::Colors;

    /// No bar without a terminal on stderr: a hidden bar would swallow the
    /// lines sent through it. Fails if the terminal check is dropped.
    #[test]
    fn progress_bar_needs_a_terminal() {
        let output = Output::new(0, false, Colors::NONE, false);
        let opts = MigrateExecuteOptions::default();
        assert!(progress_bar(&output, &opts, 2, false).is_none());
        assert!(progress_bar(&output, &opts, 2, true).is_some());
    }

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
