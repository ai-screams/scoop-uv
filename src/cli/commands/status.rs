//! Handler for the `scoop status` command.
//!
//! Summarises the current environment in one shot: which env is active, where
//! it came from (shell-activated vs version file), and a few metadata fields.
//! Designed to be fast — no package listing or directory size walk.

use rust_i18n::t;

use chrono::Utc;

use crate::core::{VersionService, VirtualenvService, get_active_env, list_installed_packages};
use crate::error::Result;
use crate::output::{Output, StatusData, format_last_used_value};
use crate::paths::abbreviate_home;

/// Sentinel state strings — kept as constants so the JSON discriminator and
/// the source-comparison in `emit_env` can't drift apart.
const SOURCE_ACTIVE: &str = "scoop_active_env";
const SOURCE_VERSION_FILE: &str = "version_file";

/// What the active-env resolver returned, split into the cases that matter
/// for `status` output.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum State {
    /// Shell-activated via `SCOOP_ACTIVE`.
    Active(String),
    /// Resolved from a version file (local or global; `.scuv-version`, with
    /// legacy `.scoop-version` fallback).
    Configured(String),
    /// `system` sentinel (system Python is in use, no virtualenv active).
    System,
    /// Neither `SCOOP_ACTIVE` nor any version file resolved.
    None,
}

/// Resolve the current state by combining `SCOOP_ACTIVE` with version-file
/// resolution. `SCOOP_ACTIVE` wins when set because it represents what the
/// shell actually activated, which can differ from what version files say.
pub(crate) fn resolve_state() -> State {
    // Empty `SCOOP_ACTIVE` (e.g. `SCOOP_ACTIVE=` in the parent shell) is broken
    // state — treat it as unset so we fall through to version-file resolution.
    if let Some(name) = get_active_env().filter(|n| !n.is_empty()) {
        return if name == "system" {
            State::System
        } else {
            State::Active(name)
        };
    }
    match VersionService::resolve_current() {
        Some(name) if name == "system" => State::System,
        Some(name) => State::Configured(name),
        None => State::None,
    }
}

/// Execute the `status` command.
pub fn execute(output: &Output) -> Result<()> {
    let state = resolve_state();
    let json = output.is_json();

    match state {
        State::None => emit_none(output, json),
        State::System => emit_system(output, json),
        State::Active(name) => emit_env(output, json, &name, SOURCE_ACTIVE),
        State::Configured(name) => emit_env(output, json, &name, SOURCE_VERSION_FILE),
    }
    Ok(())
}

fn emit_none(output: &Output, json: bool) {
    if json {
        output.json_success(
            "status",
            StatusData {
                state: "none",
                name: None,
                source: None,
                path: None,
                python: None,
                created_at: None,
                last_used: None,
                packages: None,
            },
        );
        return;
    }
    output.info(&t!("status.no_env"));
    output.info(&t!("status.hint_use"));
}

fn emit_system(output: &Output, json: bool) {
    if json {
        output.json_success(
            "status",
            StatusData {
                state: "system",
                name: None,
                source: None,
                path: None,
                python: None,
                created_at: None,
                last_used: None,
                packages: None,
            },
        );
        return;
    }
    output.info(&t!("status.system_python"));
}

fn emit_env(output: &Output, json: bool, name: &str, source: &'static str) {
    // Look up disk-side details, but tolerate a missing service/path: the env
    // name might be set even when the directory was removed by hand.
    let service = VirtualenvService::auto().ok();
    let path = service.as_ref().and_then(|s| s.get_path(name).ok());
    let metadata = path
        .as_deref()
        .and_then(|p| service.as_ref().and_then(|s| s.read_metadata(p)));

    let python = metadata.as_ref().map(|m| m.python_version.clone());
    let created_at = metadata.as_ref().map(|m| m.created_at.to_rfc3339());
    let last_used_ts = metadata.as_ref().and_then(|m| m.last_used);
    let last_used_rfc = last_used_ts.map(|t| t.to_rfc3339());
    // Best-effort package count from the venv's own pip. `None` distinguishes
    // "env has no pip" (broken / not yet bootstrapped) from "0 packages".
    let packages = path.as_deref().map(|p| list_installed_packages(p).len());

    if json {
        output.json_success(
            "status",
            StatusData {
                state: if source == SOURCE_ACTIVE {
                    "active"
                } else {
                    "configured"
                },
                name: Some(name.to_string()),
                source: Some(source),
                path: path.as_ref().map(|p| p.display().to_string()),
                python: python.clone(),
                created_at: created_at.clone(),
                last_used: last_used_rfc.clone(),
                packages,
            },
        );
        return;
    }

    let w = 10;
    // Direct stdout: `status` *is* its own output — `--quiet` users still want
    // the resolved state, matching how `resolve`/`info` print their result.
    println!("{:w$}{}", "Name:", name);
    println!("{:w$}{}", "Source:", source);
    println!("{:w$}{}", "Python:", python.as_deref().unwrap_or("-"));
    println!(
        "{:w$}{}",
        "Path:",
        path.as_ref()
            .map(|p| abbreviate_home(p))
            .unwrap_or_else(|| "-".to_string())
    );
    if let Some(m) = metadata.as_ref() {
        let date = m.created_at.format("%Y-%m-%d %H:%M:%S").to_string();
        println!("{:w$}{}", "Created:", date);
    }
    // Shared three-state contract — see [`format_last_used_value`] for
    // the "hide vs never vs N units ago" rules.
    if let Some(label) = format_last_used_value(metadata.is_some(), last_used_ts, Utc::now()) {
        println!("{:w$}{}", "Last used:", label);
    }
    if let Some(n) = packages {
        println!("{:w$}{}", "Packages:", n);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::with_temp_scoop_home;
    use serial_test::serial;
    use tempfile::TempDir;

    fn clear_env_vars() {
        // SAFETY: callers wrap this in `#[serial]`; no concurrent env access.
        unsafe {
            std::env::remove_var("SCOOP_ACTIVE");
        }
    }

    #[test]
    #[serial]
    fn resolve_state_returns_none_when_nothing_set() {
        with_temp_scoop_home(|_| {
            clear_env_vars();
            let workdir = TempDir::new().unwrap();
            let prev = std::env::current_dir().ok();
            std::env::set_current_dir(workdir.path()).unwrap();

            assert_eq!(resolve_state(), State::None);

            if let Some(p) = prev {
                std::env::set_current_dir(p).unwrap();
            }
        });
    }

    #[test]
    #[serial]
    fn resolve_state_prefers_scoop_active_over_version_file() {
        with_temp_scoop_home(|_| {
            // SAFETY: serial test.
            unsafe {
                std::env::set_var("SCOOP_ACTIVE", "shellenv");
            }

            let workdir = TempDir::new().unwrap();
            std::fs::write(workdir.path().join(".scuv-version"), "fileenv\n").unwrap();
            let prev = std::env::current_dir().ok();
            std::env::set_current_dir(workdir.path()).unwrap();

            assert_eq!(resolve_state(), State::Active("shellenv".to_string()));

            unsafe {
                std::env::remove_var("SCOOP_ACTIVE");
            }
            if let Some(p) = prev {
                std::env::set_current_dir(p).unwrap();
            }
        });
    }

    #[test]
    #[serial]
    fn resolve_state_falls_back_to_version_file() {
        with_temp_scoop_home(|_| {
            clear_env_vars();
            let workdir = TempDir::new().unwrap();
            std::fs::write(workdir.path().join(".scuv-version"), "fileenv\n").unwrap();
            let prev = std::env::current_dir().ok();
            std::env::set_current_dir(workdir.path()).unwrap();

            assert_eq!(resolve_state(), State::Configured("fileenv".to_string()));

            if let Some(p) = prev {
                std::env::set_current_dir(p).unwrap();
            }
        });
    }

    #[test]
    #[serial]
    fn resolve_state_detects_system_sentinel_in_active_env() {
        with_temp_scoop_home(|_| {
            // SAFETY: serial test.
            unsafe {
                std::env::set_var("SCOOP_ACTIVE", "system");
            }

            assert_eq!(resolve_state(), State::System);

            unsafe {
                std::env::remove_var("SCOOP_ACTIVE");
            }
        });
    }

    #[test]
    #[serial]
    fn resolve_state_detects_system_sentinel_in_version_file() {
        with_temp_scoop_home(|_| {
            clear_env_vars();
            let workdir = TempDir::new().unwrap();
            std::fs::write(workdir.path().join(".scuv-version"), "system\n").unwrap();
            let prev = std::env::current_dir().ok();
            std::env::set_current_dir(workdir.path()).unwrap();

            assert_eq!(resolve_state(), State::System);

            if let Some(p) = prev {
                std::env::set_current_dir(p).unwrap();
            }
        });
    }

    #[test]
    #[serial]
    fn execute_succeeds_for_each_state() {
        // Smoke: the handler should never error regardless of resolved state.
        with_temp_scoop_home(|_| {
            clear_env_vars();
            let workdir = TempDir::new().unwrap();
            let prev = std::env::current_dir().ok();
            std::env::set_current_dir(workdir.path()).unwrap();

            let output = Output::new(0, true, true, false);
            assert!(execute(&output).is_ok(), "None state");

            // Active state with no on-disk env: must still succeed.
            unsafe {
                std::env::set_var("SCOOP_ACTIVE", "ghost");
            }
            assert!(execute(&output).is_ok(), "Active state w/o disk env");
            unsafe {
                std::env::remove_var("SCOOP_ACTIVE");
            }

            // System sentinel.
            unsafe {
                std::env::set_var("SCOOP_ACTIVE", "system");
            }
            assert!(execute(&output).is_ok(), "System state");
            unsafe {
                std::env::remove_var("SCOOP_ACTIVE");
            }

            if let Some(p) = prev {
                std::env::set_current_dir(p).unwrap();
            }
        });
    }
}
