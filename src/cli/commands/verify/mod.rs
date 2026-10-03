//! Handler for the `scuv verify` command.
//!
//! Per-env health diagnosis. Where [`doctor`](super::doctor) reports on the
//! system (uv installed, shell wrapper wired up, etc.), `verify` looks at each
//! virtualenv directory and answers: "does this env look like something
//! Python can actually use?".
//!
//! Six checks, run in order, each independent:
//!
//! 1. **metadata** — `.scoop-metadata.json` exists and parses
//! 2. **python_binary** — interpreter file is present
//! 3. **pyvenv_cfg** — the standard venv marker exists
//! 4. **activate_script** — shell activate script exists
//! 5. **python_executes** — `python --version` actually runs (skipped if
//!    the binary is missing — would just duplicate check #2)
//! 6. **manifest_match** — if a `.scuv.toml` exists in the cwd hierarchy,
//!    the env's recorded Python matches the manifest. Skipped silently when
//!    there's no manifest — drift only matters in manifest-managed projects.
//!
//! By default the command always exits 0; pass `--strict` to opt into
//! `exit 1` when any check fails. This mirrors `doctor`'s default: surfacing
//! information shouldn't break CI just because someone wanted to look.

mod checks;
mod report;
mod types;

use crate::core::VirtualenvService;
use crate::error::{Result, ScoopError};

use checks::{collect_target_envs, load_manifest_for_drift_check, verify_one};
use report::{emit_empty, emit_strict_json_failure, render_human};
use types::{EnvReport, VerifyData, compute_summary};

/// Execute the `verify` command.
///
/// * `target` — `Some(name)` to check just that env; `None` checks every env
///   under `~/.scuv/virtualenvs/`.
/// * `strict` — when true, return [`ScoopError::VerifyFailed`] when any env
///   has at least one Fail check, so main.rs maps it to a non-zero exit.
pub fn execute(output: &crate::output::Output, target: Option<&str>, strict: bool) -> Result<()> {
    let service = VirtualenvService::auto()?;
    // Pre-load the manifest once — find_manifest_from_cwd walks the FS,
    // which is wasted work to repeat per env.
    let manifest = load_manifest_for_drift_check();

    let envs = collect_target_envs(&service, target)?;
    if envs.is_empty() {
        emit_empty(output);
        return Ok(());
    }

    let reports: Vec<EnvReport> = envs
        .iter()
        .map(|env| verify_one(&service, env, manifest.as_ref()))
        .collect();
    let summary = compute_summary(&reports);
    let strict_failure = strict && summary.issues > 0;

    if output.is_json() {
        if strict_failure {
            // C4: under `--strict --json` with failures, the prior
            // shape emitted `status: "success"` and then returned Err,
            // leaving the JSON envelope and the exit code disagreeing.
            // Emit a single error envelope that ALSO carries the full
            // report so consumers don't lose the per-env breakdown.
            // main.rs may also write a text line to stderr, but stdout
            // (which scripts actually parse) is internally consistent.
            emit_strict_json_failure(&reports, &summary);
        } else {
            output.json_success(
                "verify",
                VerifyData {
                    envs: reports,
                    summary,
                },
            );
        }
    } else {
        render_human(output, &reports, &summary);
    }

    // `--strict` opts into "fail loud" for CI gates. Drift (Warn) is
    // informational — only hard breakage (Fail) trips the exit. Returning
    // Err (instead of std::process::exit) lets destructors / stdout
    // buffers flush via main.rs's normal error path.
    if strict_failure {
        return Err(ScoopError::VerifyFailed {
            issues: summary.issues,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests;
