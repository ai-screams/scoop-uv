//! `scoop self update` — reinstall scoop-uv from crates.io.
//!
//! Active update flow:
//! 1. Resolve target version (explicit `--version` or latest via `cargo search`).
//! 2. Skip when already on latest (unless `--force`).
//! 3. Shell out to `cargo install --force --locked scoop-uv --version <V>`.
//! 4. Re-spawn the freshly installed binary as `scoop doctor` so users
//!    immediately see any environment drift (e.g. uv below the new minimum)
//!    without having to run a verify command themselves. Skip with `--no-verify`.

use std::process::Command;

use rust_i18n::t;
use serde::Serialize;

use crate::error::{Result, ScoopError};
use crate::output::Output;

const CRATE_NAME: &str = "scoop-uv";
const BINARY_NAME: &str = "scoop";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Serialize)]
struct UpdateData {
    from: &'static str,
    to: String,
    skipped: bool,
    verified: bool,
}

/// Execute `scoop self update`.
pub fn execute(output: &Output, force: bool, version: Option<&str>, no_verify: bool) -> Result<()> {
    let target = resolve_target_version(output, version)?;

    if target == CURRENT_VERSION && !force {
        if output.is_json() {
            output.json_success(
                "self update",
                UpdateData {
                    from: CURRENT_VERSION,
                    to: target,
                    skipped: true,
                    verified: false,
                },
            );
        } else {
            output.success(&t!("selfupdate.already_latest", version = CURRENT_VERSION));
        }
        return Ok(());
    }

    output.info(&t!(
        "selfupdate.installing",
        from = CURRENT_VERSION,
        to = target.as_str()
    ));

    run_cargo_install(&target)?;

    output.success(&t!("selfupdate.installed", version = target.as_str()));

    let verified = if no_verify {
        false
    } else {
        verify_with_new_binary(output);
        true
    };

    if output.is_json() {
        output.json_success(
            "self update",
            UpdateData {
                from: CURRENT_VERSION,
                to: target,
                skipped: false,
                verified,
            },
        );
    }

    Ok(())
}

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
    for line in stdout.lines() {
        if let Some(rest) = line.trim_start().strip_prefix(&prefix) {
            if let Some(end) = rest.find('"') {
                return Some(rest[..end].to_string());
            }
        }
    }
    None
}

fn run_cargo_install(version: &str) -> Result<()> {
    let status = Command::new("cargo")
        .args([
            "install",
            "--force",
            "--locked",
            CRATE_NAME,
            "--version",
            version,
        ])
        .status()
        .map_err(|e| ScoopError::SelfUpdateFailed {
            message: format!("could not invoke `cargo install`: {e}"),
        })?;

    if !status.success() {
        return Err(ScoopError::SelfUpdateFailed {
            message: format!("`cargo install` exited with {status}"),
        });
    }
    Ok(())
}

/// Spawn the freshly installed `scoop doctor` so the user immediately sees
/// any new minimum-version mismatches. Failures here are surfaced as warnings
/// but never promoted to errors — the install itself already succeeded.
fn verify_with_new_binary(output: &Output) {
    let new_bin = match which::which(BINARY_NAME) {
        Ok(p) => p,
        Err(_) => {
            output.warn(&t!("selfupdate.verify_skipped_no_binary"));
            return;
        }
    };

    output.info(&t!("selfupdate.verifying"));

    match Command::new(&new_bin).arg("doctor").status() {
        Ok(status) if status.success() => {}
        Ok(_) => output.warn(&t!("selfupdate.verify_doctor_warnings")),
        Err(e) => output.warn(&t!(
            "selfupdate.verify_skipped_error",
            error = e.to_string()
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
