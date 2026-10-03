//! Rendering `verify` results: the human report and the JSON envelopes.

use rust_i18n::t;
use serde::Serialize;

use super::types::{CheckStatus, EnvReport, Summary, VerifyData};

/// Render the failed-strict JSON envelope in one consistent shape:
/// `{ status: "error", command: "verify", error: { ... }, data: { ... } }`.
///
/// Built manually because the shared [`crate::output::Output`] error
/// envelope doesn't carry domain data — and dropping the report would
/// strip the per-env breakdown that scripts came to `--json` for in the
/// first place. The error block mirrors [`ScoopError::VerifyFailed`]'s
/// code/message so consumers that only read `.error.code` still see
/// "VERIFY_FAILED".
pub(super) fn emit_strict_json_failure(reports: &[EnvReport], summary: &Summary) {
    #[derive(Serialize)]
    struct Envelope<'a> {
        status: &'a str,
        command: &'a str,
        error: ErrorBody,
        data: DataView<'a>,
    }
    #[derive(Serialize)]
    struct ErrorBody {
        code: &'static str,
        message: String,
        issues: usize,
    }
    #[derive(Serialize)]
    struct DataView<'a> {
        envs: &'a [EnvReport],
        summary: &'a Summary,
    }

    let envelope = Envelope {
        status: "error",
        command: "verify",
        error: ErrorBody {
            code: "VERIFY_FAILED",
            message: t!("error.verify_failed", issues = summary.issues.to_string()).to_string(),
            issues: summary.issues,
        },
        data: DataView {
            envs: reports,
            summary,
        },
    };
    // serde_json::to_string never fails on these owned types — the only
    // failure mode is custom Serialize impls returning Err, which we don't
    // use. unwrap() here keeps the error path linear; a panic would be a
    // serde regression, not a verify bug.
    println!("{}", serde_json::to_string(&envelope).unwrap());
}

/// Render the "no envs to check" empty case in both JSON and human modes.
pub(super) fn emit_empty(output: &crate::output::Output) {
    if output.is_json() {
        output.json_success(
            "verify",
            VerifyData {
                envs: Vec::new(),
                summary: Summary {
                    total: 0,
                    healthy: 0,
                    warnings: 0,
                    issues: 0,
                },
            },
        );
    } else {
        output.info(&t!("verify.no_envs"));
    }
}

pub(super) fn render_human(
    output: &crate::output::Output,
    reports: &[EnvReport],
    summary: &Summary,
) {
    for report in reports {
        // Per-env symbol mirrors the summary buckets: ✓ healthy,
        // ⚠ warning-only, ✗ has at least one Fail. Picking by Fail-first
        // is important so a Warn+Fail env shows ✗, not ⚠.
        let has_fail = report.checks.iter().any(|c| c.status == CheckStatus::Fail);
        let has_warn = report.checks.iter().any(|c| c.status == CheckStatus::Warn);
        let symbol = if has_fail {
            "✗"
        } else if has_warn {
            "⚠"
        } else {
            "✓"
        };
        let py = report
            .python
            .as_deref()
            .map(|v| format!("Python {v}"))
            .unwrap_or_else(|| t!("verify.python_unknown").to_string());
        println!("{symbol} {} ({})", report.name, py);

        // Only show per-check detail when there's anything non-Pass —
        // keep the happy path output compact.
        if !report.healthy {
            for check in &report.checks {
                let detail = match check.status {
                    CheckStatus::Pass | CheckStatus::Skip => continue,
                    CheckStatus::Warn => "  ⚠",
                    CheckStatus::Fail => "  ✗",
                };
                let label = t!(check.name.label_key()).to_string();
                let msg = check.message.as_deref().unwrap_or("");
                if msg.is_empty() {
                    println!("{detail} {label}");
                } else {
                    println!("{detail} {label}: {msg}");
                }
            }
        }
    }

    // Use the three-bucket summary when there are any warnings so users
    // can see "drift but no breakage" without `--strict` mis-triggering.
    if summary.warnings > 0 {
        output.info(&t!(
            "verify.summary_with_warnings",
            total = summary.total.to_string(),
            healthy = summary.healthy.to_string(),
            warnings = summary.warnings.to_string(),
            issues = summary.issues.to_string()
        ));
    } else {
        output.info(&t!(
            "verify.summary",
            total = summary.total.to_string(),
            healthy = summary.healthy.to_string(),
            issues = summary.issues.to_string()
        ));
    }
}
