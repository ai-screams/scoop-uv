//! The `scuv doctor` report: header, one line per check, summary, and the
//! `--json` document. Kept with the command rather than in `output`, so
//! `output` does not depend on `core::doctor`.

use owo_colors::OwoColorize;

use crate::core::doctor::{CheckResult, CheckStatus};
use crate::output::Output;

/// Print doctor report header.
pub(super) fn header(output: &Output) {
    if output.is_quiet() || output.is_json() {
        return;
    }
    eprintln!();
    eprintln!("Checking installation...");
    eprintln!();
}

/// Print a single check result.
pub(super) fn check(output: &Output, result: &CheckResult) {
    if output.is_json() {
        return;
    }

    // Skip OK results in quiet mode
    if output.is_quiet() && result.is_ok() {
        return;
    }

    let (icon, color_fn): (&str, fn(&str) -> String) = match &result.status {
        CheckStatus::Ok => ("✓", |s| s.green().to_string()),
        CheckStatus::Warning(_) => ("⚠", |s| s.yellow().to_string()),
        CheckStatus::Error(_) => ("✗", |s| s.red().to_string()),
    };

    // Build message
    let message = match &result.status {
        CheckStatus::Ok => result.name.to_string(),
        CheckStatus::Warning(msg) => format!("{}: {}", result.name, msg),
        CheckStatus::Error(msg) => format!("{}: {}", result.name, msg),
    };

    // Print with or without color
    if !output.use_color_stderr() {
        eprintln!("{} {}", icon, message);
    } else {
        eprintln!("{} {}", color_fn(icon), message);
    }

    // Print details in verbose mode
    if output.verbosity() > 0
        && let Some(details) = &result.details
    {
        if !output.use_color_stderr() {
            eprintln!("  {}", details);
        } else {
            eprintln!("  {}", details.dimmed());
        }
    }

    // Print suggestion for errors/warnings
    if let Some(suggestion) = &result.suggestion {
        if !output.use_color_stderr() {
            eprintln!("  → {}", suggestion);
        } else {
            eprintln!("  {} {}", "→".cyan(), suggestion);
        }
    }
}

/// Print doctor report summary.
pub(super) fn summary(output: &Output, results: &[CheckResult]) {
    if output.is_json() {
        return;
    }

    let errors = results.iter().filter(|r| r.is_error()).count();
    let warnings = results.iter().filter(|r| r.is_warning()).count();

    eprintln!();
    eprintln!("──────────────────────────────────");

    if errors == 0 && warnings == 0 {
        if !output.use_color_stderr() {
            eprintln!("All checks passed!");
        } else {
            eprintln!("{}", "All checks passed!".green());
        }
    } else {
        let mut parts = Vec::new();
        if errors > 0 {
            parts.push(format!("{} error(s)", errors));
        }
        if warnings > 0 {
            parts.push(format!("{} warning(s)", warnings));
        }

        let summary = format!("Found {}.", parts.join(" and "));
        if !output.use_color_stderr() {
            eprintln!("{}", summary);
        } else {
            eprintln!("{}", summary.yellow());
        }
    }
}

/// Print doctor report as JSON.
pub(super) fn json(output: &Output, results: &[CheckResult]) {
    if !output.is_json() {
        return;
    }

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

    let document = serde_json::json!({
        "version": env!("CARGO_PKG_VERSION"),
        "summary": {
            "total": results.len(),
            "ok": ok,
            "warnings": warnings,
            "errors": errors,
        },
        "checks": json_results,
    });

    println!(
        "{}",
        serde_json::to_string_pretty(&document).unwrap_or_default()
    );
}
