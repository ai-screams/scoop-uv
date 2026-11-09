//! The `scuv doctor` report: header, one line per check, summary, and the
//! `--json` document. Kept with the command rather than in `output`, so
//! `output` does not depend on `core::doctor`.
//!
//! Each part is built as text first (`*_lines`, `json_document`) and then
//! printed, so the tests can read what would be printed.

use owo_colors::OwoColorize;
use rust_i18n::t;

use crate::core::doctor::{CheckResult, CheckStatus};
use crate::output::Output;

/// Print doctor report header.
pub(super) fn header(output: &Output) {
    if output.is_quiet() || output.is_json() {
        return;
    }
    eprintln!();
    eprintln!("{}", t!("doctor.report.header"));
    eprintln!();
}

/// Print a single check result.
pub(super) fn check(output: &Output, result: &CheckResult) {
    for line in check_lines(output, result) {
        eprintln!("{line}");
    }
}

/// The stderr lines for one check: none under `--json`, none for a passing
/// check under `--quiet`; otherwise the status line, the details when
/// verbose, and the suggestion.
fn check_lines(output: &Output, result: &CheckResult) -> Vec<String> {
    if output.is_json() || (output.is_quiet() && result.is_ok()) {
        return Vec::new();
    }
    let color = output.use_color_stderr();

    let (icon, color_fn): (&str, fn(&str) -> String) = match &result.status {
        CheckStatus::Ok => ("✓", |s| s.green().to_string()),
        CheckStatus::Warning(_) => ("⚠", |s| s.yellow().to_string()),
        CheckStatus::Error(_) => ("✗", |s| s.red().to_string()),
    };

    let message = match &result.status {
        CheckStatus::Ok => result.name.to_string(),
        CheckStatus::Warning(msg) => format!("{}: {}", result.name, msg),
        CheckStatus::Error(msg) => format!("{}: {}", result.name, msg),
    };

    let mut lines = Vec::new();
    if !color {
        lines.push(format!("{} {}", icon, message));
    } else {
        lines.push(format!("{} {}", color_fn(icon), message));
    }

    if output.verbosity() > 0
        && let Some(details) = &result.details
    {
        if !color {
            lines.push(format!("  {}", details));
        } else {
            lines.push(format!("  {}", details.dimmed()));
        }
    }

    if let Some(suggestion) = &result.suggestion {
        if !color {
            lines.push(format!("  → {}", suggestion));
        } else {
            lines.push(format!("  {} {}", "→".cyan(), suggestion));
        }
    }
    lines
}

/// Print doctor report summary.
pub(super) fn summary(output: &Output, results: &[CheckResult]) {
    if output.is_json() {
        return;
    }
    eprintln!();
    eprintln!("──────────────────────────────────");
    eprintln!("{}", summary_line(results, output.use_color_stderr()));
}

/// "All checks passed!" or "Found N error(s) and M warning(s).".
fn summary_line(results: &[CheckResult], color: bool) -> String {
    let errors = results.iter().filter(|r| r.is_error()).count();
    let warnings = results.iter().filter(|r| r.is_warning()).count();

    if errors == 0 && warnings == 0 {
        let line = t!("doctor.report.all_passed");
        return if color {
            line.green().to_string()
        } else {
            line.to_string()
        };
    }

    // One sentence per case, so each language keeps its own word order.
    let line = match (errors, warnings) {
        (_, 0) => t!("doctor.report.found_errors", errors = errors),
        (0, _) => t!("doctor.report.found_warnings", warnings = warnings),
        _ => t!(
            "doctor.report.found_both",
            errors = errors,
            warnings = warnings
        ),
    }
    .to_string();
    if color {
        line.yellow().to_string()
    } else {
        line
    }
}

/// Print doctor report as JSON.
pub(super) fn json(output: &Output, results: &[CheckResult]) {
    if !output.is_json() {
        return;
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&json_document(results)).unwrap_or_default()
    );
}

/// The `--json` document: version, counts, and one object per check.
fn json_document(results: &[CheckResult]) -> serde_json::Value {
    let json_results: Vec<serde_json::Value> = results
        .iter()
        .map(|r| {
            let status = match &r.status {
                CheckStatus::Ok => "ok",
                CheckStatus::Warning(_) => "warning",
                CheckStatus::Error(_) => "error",
            };

            let message = match &r.status {
                CheckStatus::Ok => None,
                CheckStatus::Warning(msg) => Some(msg.clone()),
                CheckStatus::Error(msg) => Some(msg.clone()),
            };

            serde_json::json!({
                "id": r.id,
                "name": r.name,
                "status": status,
                "message": message,
                "suggestion": r.suggestion,
                "details": r.details,
            })
        })
        .collect();

    let errors = results.iter().filter(|r| r.is_error()).count();
    let warnings = results.iter().filter(|r| r.is_warning()).count();
    let ok = results.iter().filter(|r| r.is_ok()).count();

    serde_json::json!({
        "version": env!("CARGO_PKG_VERSION"),
        "summary": {
            "total": results.len(),
            "ok": ok,
            "warnings": warnings,
            "errors": errors,
        },
        "checks": json_results,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output::Colors;

    fn output(verbose: u8, quiet: bool, colors: Colors, json: bool) -> Output {
        Output::new(verbose, quiet, colors, json)
    }

    fn error() -> CheckResult {
        CheckResult::error("venv", "broken virtualenv", "'x' is corrupted")
            .with_suggestion("scuv remove x")
            .with_details("1 environment")
    }

    #[test]
    fn check_lines_show_status_details_when_verbose_and_suggestion() {
        let plain = output(1, false, Colors::NONE, false);
        assert_eq!(
            check_lines(&plain, &error()),
            [
                "✗ broken virtualenv: 'x' is corrupted",
                "  1 environment",
                "  → scuv remove x"
            ]
        );
        let not_verbose = output(0, false, Colors::NONE, false);
        assert_eq!(check_lines(&not_verbose, &error()).len(), 2);

        let warn = CheckResult::warn("shell", "shell configuration", "unsupported");
        assert_eq!(
            check_lines(&plain, &warn),
            ["⚠ shell configuration: unsupported"]
        );
        let ok = CheckResult::ok("uv", "uv installation");
        assert_eq!(check_lines(&plain, &ok), ["✓ uv installation"]);
    }

    /// `--quiet` keeps problems and drops passing checks; `--json` prints
    /// no check lines at all.
    #[test]
    fn check_lines_follow_quiet_and_json() {
        let quiet = output(0, true, Colors::NONE, false);
        assert!(check_lines(&quiet, &CheckResult::ok("uv", "uv installation")).is_empty());
        assert_eq!(check_lines(&quiet, &error()).len(), 2);
        let json = output(0, false, Colors::NONE, true);
        assert!(check_lines(&json, &error()).is_empty());
    }

    /// With color the icon, details and arrow are wrapped in escape codes
    /// and the text stays the same.
    #[test]
    fn check_lines_color_the_icon_details_and_arrow() {
        let colored = output(1, false, Colors::ALL, false);
        let lines = check_lines(&colored, &error());
        assert_eq!(lines.len(), 3);
        assert!(lines.iter().all(|l| l.contains('\u{1b}')), "{lines:?}");
        assert!(lines[0].ends_with(" broken virtualenv: 'x' is corrupted"));
        assert!(lines[2].ends_with(" scuv remove x"));
        for status in [
            CheckResult::ok("uv", "uv installation"),
            CheckResult::warn("shell", "shell configuration", "unsupported"),
        ] {
            assert!(check_lines(&colored, &status)[0].starts_with('\u{1b}'));
        }
    }

    #[test]
    fn summary_line_counts_errors_and_warnings() {
        let warn = || CheckResult::warn("shell", "shell configuration", "unsupported");
        let ok = || CheckResult::ok("uv", "uv installation");
        assert_eq!(summary_line(&[ok()], false), "All checks passed!");
        assert_eq!(
            summary_line(&[error(), warn(), error()], false),
            "Found 2 error(s) and 1 warning(s)."
        );
        assert_eq!(summary_line(&[error()], false), "Found 1 error(s).");
        assert_eq!(summary_line(&[warn()], false), "Found 1 warning(s).");
        let colored = summary_line(&[ok()], true);
        assert!(colored.contains('\u{1b}') && colored.contains("All checks passed!"));
        assert!(summary_line(&[error()], true).contains('\u{1b}'));
    }

    #[test]
    fn json_document_counts_and_lists_every_check() {
        let warn = CheckResult::warn("shell", "shell configuration", "unsupported");
        let ok = CheckResult::ok("uv", "uv installation");
        // Distinct counts, so swapping two of them is caught.
        let ok2 = CheckResult::ok("home", "SCUV_HOME directory");
        let ok3 = CheckResult::ok("legacy", "legacy scoop remnants");
        let doc = json_document(&[ok, ok2, ok3, warn, error(), error()]);
        assert_eq!(doc["version"], env!("CARGO_PKG_VERSION"));
        assert_eq!(
            doc["summary"],
            serde_json::json!({"total": 6, "ok": 3, "warnings": 1, "errors": 2})
        );
        let checks = doc["checks"].as_array().unwrap();
        assert_eq!(checks[0]["status"], "ok");
        assert!(checks[0]["message"].is_null());
        assert_eq!(checks[3]["status"], "warning");
        assert_eq!(checks[3]["message"], "unsupported");
        assert_eq!(checks[4]["status"], "error");
        assert_eq!(checks[4]["suggestion"], "scuv remove x");
        assert_eq!(checks[4]["details"], "1 environment");
        assert_eq!(checks[4]["id"], "venv");
    }
}
