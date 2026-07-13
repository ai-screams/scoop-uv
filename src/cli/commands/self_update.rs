//! `scuv self update` — reinstall scoop-uv from crates.io.
//!
//! Layered design:
//! * [`execute`] orchestrates; it owns user-facing output and never inspects
//!   exit codes or stdout itself.
//! * Helpers (`resolve_target_version`, `run_cargo_install`,
//!   `verify_with_new_binary`) each do one thing: resolve a version, shell
//!   out to cargo, or run doctor and report what happened.
//! * [`VerifyOutcome`] is the explicit domain type returned by the verify
//!   step. It carries everything a machine consumer needs (doctor exit-code
//!   semantics, launch failures) and serializes to the JSON envelope with a
//!   `status` tag so callers can branch precisely instead of guessing what
//!   `verified: bool` meant.

use std::path::PathBuf;
use std::process::{Command, Stdio};

use rust_i18n::t;
use serde::Serialize;

use crate::error::{Result, ScoopError};
use crate::output::Output;

const CRATE_NAME: &str = "scoop-uv";
const BINARY_NAME: &str = "scuv";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

// ============================================================================
// Domain types
// ============================================================================

/// Outcome of the post-install `scuv doctor` step.
///
/// Tagged JSON shape: `{ "status": "...", ... }`. Adding new variants is
/// backward-compatible for consumers that branch on `status`.
#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(tag = "status", rename_all = "snake_case")]
enum VerifyOutcome {
    /// `--no-verify` was passed.
    Skipped,
    /// doctor exited 0.
    Passed,
    /// doctor exited 1 (warnings only).
    Warned,
    /// doctor exited 2 or higher (one or more errors).
    Errored,
    /// doctor never ran (binary not found, IO error spawning, ...).
    LaunchFailed { error: String },
}

#[derive(Debug, Serialize)]
struct UpdateData {
    from: &'static str,
    to: String,
    skipped: bool,
    verify: VerifyOutcome,
}

// ============================================================================
// Entry point
// ============================================================================

/// Execute `scuv self update`.
pub fn execute(output: &Output, force: bool, version: Option<&str>, no_verify: bool) -> Result<()> {
    let requested_explicit = version.is_some();
    let target = resolve_target_version(output, version)?;

    if target == CURRENT_VERSION && !force {
        emit_skip(output, &target, requested_explicit);
        return Ok(());
    }

    output.info(&t!(
        "selfupdate.installing",
        from = CURRENT_VERSION,
        to = target.as_str()
    ));
    run_cargo_install(&target, output.is_json())?;
    output.success(&t!("selfupdate.installed", version = target.as_str()));

    let verify = if no_verify {
        VerifyOutcome::Skipped
    } else {
        output.info(&t!("selfupdate.verifying"));
        verify_with_new_binary(output.is_json())
    };
    emit_verify(output, &verify);

    if output.is_json() {
        output.json_success(
            "self update",
            UpdateData {
                from: CURRENT_VERSION,
                to: target,
                skipped: false,
                verify,
            },
        );
    }
    Ok(())
}

// ============================================================================
// Output helpers — keep text & JSON branches in lockstep
// ============================================================================

fn emit_skip(output: &Output, target: &str, requested_explicit: bool) {
    if output.is_json() {
        output.json_success(
            "self update",
            UpdateData {
                from: CURRENT_VERSION,
                to: target.to_string(),
                skipped: true,
                verify: VerifyOutcome::Skipped,
            },
        );
        return;
    }
    // Distinguish "you asked for this version" from "you're already current".
    let key = if requested_explicit {
        "selfupdate.already_on_requested"
    } else {
        "selfupdate.already_latest"
    };
    output.success(&t!(key, version = CURRENT_VERSION));
}

fn emit_verify(output: &Output, outcome: &VerifyOutcome) {
    match outcome {
        // Silent: nothing to add. Either we didn't run, or it passed cleanly.
        VerifyOutcome::Skipped | VerifyOutcome::Passed => {}
        VerifyOutcome::Warned => output.warn(&t!("selfupdate.verify_doctor_warnings")),
        VerifyOutcome::Errored => output.warn(&t!("selfupdate.verify_doctor_errors")),
        VerifyOutcome::LaunchFailed { error } => output.warn(&t!(
            "selfupdate.verify_launch_failed",
            error = error.as_str()
        )),
    }
}

// ============================================================================
// Version resolution
// ============================================================================

fn resolve_target_version(output: &Output, requested: Option<&str>) -> Result<String> {
    if let Some(v) = requested {
        return Ok(v.to_string());
    }
    output.info(&t!("selfupdate.checking_latest"));
    latest_version_from_cargo_search()
}

fn latest_version_from_cargo_search() -> Result<String> {
    let out = Command::new("cargo")
        .args(["search", "--limit", "1", CRATE_NAME])
        .output()
        .map_err(|e| ScoopError::SelfUpdateFailed {
            message: format!("could not invoke `cargo search`: {e}"),
        })?;

    if !out.status.success() {
        return Err(ScoopError::SelfUpdateFailed {
            message: format!(
                "`cargo search` exited with {}: {}",
                out.status,
                String::from_utf8_lossy(&out.stderr).trim()
            ),
        });
    }

    let stdout = String::from_utf8_lossy(&out.stdout);
    parse_cargo_search_version(&stdout).ok_or_else(|| ScoopError::SelfUpdateFailed {
        message: format!("could not parse `cargo search` output for `{CRATE_NAME}`: {stdout}"),
    })
}

/// Parse the first matching `<crate> = "X.Y.Z"` line from `cargo search` stdout.
fn parse_cargo_search_version(stdout: &str) -> Option<String> {
    let prefix = format!("{CRATE_NAME} = \"");
    stdout
        .lines()
        .filter_map(|line| line.trim_start().strip_prefix(&prefix))
        .find_map(|rest| rest.find('"').map(|end| rest[..end].to_string()))
}

// ============================================================================
// Install
// ============================================================================

fn run_cargo_install(version: &str, json_mode: bool) -> Result<()> {
    let mut cmd = Command::new("cargo");
    cmd.args([
        "install",
        "--force",
        "--locked",
        CRATE_NAME,
        "--version",
        version,
    ]);

    if json_mode {
        // Keep stdout clean so the trailing JSON envelope is parseable.
        // Cargo's progress goes to stderr, which we leave inherited.
        cmd.arg("--quiet");
        cmd.stdout(Stdio::null());
    }

    let status = cmd.status().map_err(|e| ScoopError::SelfUpdateFailed {
        message: format!("could not invoke `cargo install`: {e}"),
    })?;

    if !status.success() {
        return Err(ScoopError::SelfUpdateFailed {
            message: format!("`cargo install` exited with {status}"),
        });
    }
    Ok(())
}

// ============================================================================
// Verify
// ============================================================================

/// Run `<new_binary> doctor` and translate its exit code into a [`VerifyOutcome`].
///
/// This function deliberately returns an outcome instead of printing — output
/// is the caller's responsibility (see [`emit_verify`]). That keeps the verify
/// path testable in isolation and keeps the text/JSON branches in one place.
fn verify_with_new_binary(json_mode: bool) -> VerifyOutcome {
    let Some(new_bin) = installed_binary_path() else {
        return VerifyOutcome::LaunchFailed {
            error: format!(
                "could not locate the freshly installed `{BINARY_NAME}` binary in CARGO_INSTALL_ROOT/CARGO_HOME/PATH"
            ),
        };
    };

    let mut cmd = Command::new(&new_bin);
    cmd.arg("doctor");
    if json_mode {
        // doctor prints free-form text; muffle both streams in JSON mode so the
        // envelope on stdout stays parseable. The VerifyOutcome carries
        // everything a JSON consumer needs.
        cmd.stdout(Stdio::null());
        cmd.stderr(Stdio::null());
    }

    match cmd.status() {
        Ok(s) if s.success() => VerifyOutcome::Passed,
        Ok(s) => match s.code() {
            Some(1) => VerifyOutcome::Warned,
            _ => VerifyOutcome::Errored,
        },
        Err(e) => VerifyOutcome::LaunchFailed {
            error: e.to_string(),
        },
    }
}

/// Locate the binary `cargo install` just wrote.
///
/// Prefer the deterministic cargo install target
/// (`$CARGO_INSTALL_ROOT/bin` → `$CARGO_HOME/bin` → `~/.cargo/bin`) over
/// `which::which`, because the user's `PATH` may have an older `scuv`
/// from a different install channel ranked higher. Falling back to
/// `which` only when the deterministic path doesn't exist preserves
/// compatibility with non-cargo install layouts (manual symlinks, etc.).
fn installed_binary_path() -> Option<PathBuf> {
    let filename = binary_filename();
    cargo_install_root()
        .map(|root| root.join("bin").join(&filename))
        .filter(|p| p.exists())
        .or_else(|| which::which(BINARY_NAME).ok())
}

/// Resolve cargo's install root from env first, then user home.
fn cargo_install_root() -> Option<PathBuf> {
    std::env::var_os("CARGO_INSTALL_ROOT")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("CARGO_HOME").map(PathBuf::from))
        .or_else(|| dirs::home_dir().map(|h| h.join(".cargo")))
}

fn binary_filename() -> String {
    if cfg!(windows) {
        format!("{BINARY_NAME}.exe")
    } else {
        BINARY_NAME.to_string()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ---- parse_cargo_search_version -----------------------------------------

    #[test]
    fn parse_cargo_search_extracts_first_match() {
        let stdout = "\
scoop-uv = \"0.9.0\"    # Scoop up your Python envs ...
scoop = \"5.0.0\"       # Decentralized command-line installer ...
";
        assert_eq!(
            parse_cargo_search_version(stdout),
            Some("0.9.0".to_string())
        );
    }

    #[test]
    fn parse_cargo_search_handles_indented_line() {
        let stdout = "    scoop-uv = \"1.2.3\"\n";
        assert_eq!(
            parse_cargo_search_version(stdout),
            Some("1.2.3".to_string())
        );
    }

    #[test]
    fn parse_cargo_search_returns_none_when_absent() {
        let stdout = "other-crate = \"0.1.0\"\n";
        assert_eq!(parse_cargo_search_version(stdout), None);
    }

    #[test]
    fn parse_cargo_search_returns_none_when_malformed() {
        let stdout = "scoop-uv = malformed\n";
        assert_eq!(parse_cargo_search_version(stdout), None);
    }

    #[test]
    fn parse_cargo_search_skips_unrelated_crate_with_same_prefix() {
        let stdout = "scoop-uv-something = \"9.9.9\"\nscoop-uv = \"0.1.0\"\n";
        assert_eq!(
            parse_cargo_search_version(stdout),
            Some("0.1.0".to_string())
        );
    }

    // ---- VerifyOutcome JSON shape -------------------------------------------

    #[test]
    fn verify_outcome_skipped_serializes_with_status_tag() {
        let v = serde_json::to_value(VerifyOutcome::Skipped).unwrap();
        assert_eq!(v["status"], "skipped");
    }

    #[test]
    fn verify_outcome_passed_warned_errored_have_distinct_tags() {
        assert_eq!(
            serde_json::to_value(VerifyOutcome::Passed).unwrap()["status"],
            "passed"
        );
        assert_eq!(
            serde_json::to_value(VerifyOutcome::Warned).unwrap()["status"],
            "warned"
        );
        assert_eq!(
            serde_json::to_value(VerifyOutcome::Errored).unwrap()["status"],
            "errored"
        );
    }

    #[test]
    fn verify_outcome_launch_failed_carries_error_message() {
        let v = serde_json::to_value(VerifyOutcome::LaunchFailed {
            error: "permission denied".into(),
        })
        .unwrap();
        assert_eq!(v["status"], "launch_failed");
        assert_eq!(v["error"], "permission denied");
    }

    // ---- Platform binary name ----------------------------------------------

    #[test]
    fn binary_filename_matches_platform() {
        let f = binary_filename();
        if cfg!(windows) {
            assert!(f.ends_with(".exe"));
        } else {
            assert_eq!(f, BINARY_NAME);
        }
    }
}
