//! Handler for the `scoop verify` command.
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
//! 6. **manifest_match** — if a `.scoop.toml` exists in the cwd hierarchy,
//!    the env's recorded Python matches the manifest. Skipped silently when
//!    there's no manifest — drift only matters in manifest-managed projects.
//!
//! By default the command always exits 0; pass `--strict` to opt into
//! `exit 1` when any check fails. This mirrors `doctor`'s default: surfacing
//! information shouldn't break CI just because someone wanted to look.

use std::path::Path;
use std::process::Command;

use rust_i18n::t;
use serde::Serialize;

use crate::core::manifest::{ScoopManifest, find_manifest_from_cwd};
use crate::core::{VirtualenvInfo, VirtualenvService};
use crate::error::{Result, ScoopError};
use crate::validate::{self, PythonVersion};

/// Outcome of a single check.
///
/// The strings stay stable — they're part of the JSON contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum CheckStatus {
    /// Everything as expected.
    Pass,
    /// Check is irrelevant for this env (e.g. manifest_match with no manifest).
    Skip,
    /// Something is off but the env might still work (e.g. version drift).
    Warn,
    /// Hard breakage — env almost certainly unusable.
    Fail,
}

#[derive(Debug, Serialize)]
struct CheckResult {
    /// Stable identifier — clients use this, the human-readable label is
    /// rendered via i18n at print time.
    name: &'static str,
    status: CheckStatus,
    /// Populated for Warn/Fail with a short hint. Pass/Skip leave this null.
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
}

impl CheckResult {
    fn pass(name: &'static str) -> Self {
        Self {
            name,
            status: CheckStatus::Pass,
            message: None,
        }
    }
    fn skip(name: &'static str) -> Self {
        Self {
            name,
            status: CheckStatus::Skip,
            message: None,
        }
    }
    fn warn(name: &'static str, message: impl Into<String>) -> Self {
        Self {
            name,
            status: CheckStatus::Warn,
            message: Some(message.into()),
        }
    }
    fn fail(name: &'static str, message: impl Into<String>) -> Self {
        Self {
            name,
            status: CheckStatus::Fail,
            message: Some(message.into()),
        }
    }
}

#[derive(Debug, Serialize)]
struct EnvReport {
    name: String,
    /// `true` when every check is Pass or Skip.
    healthy: bool,
    /// Python version pulled from `.scoop-metadata.json` (null if metadata is
    /// missing or invalid).
    #[serde(skip_serializing_if = "Option::is_none")]
    python: Option<String>,
    checks: Vec<CheckResult>,
}

#[derive(Debug, Clone, Copy, Serialize)]
struct Summary {
    total: usize,
    healthy: usize,
    issues: usize,
}

#[derive(Debug, Serialize)]
struct VerifyData {
    envs: Vec<EnvReport>,
    summary: Summary,
}

/// Execute the `verify` command.
///
/// * `target` — `Some(name)` to check just that env; `None` checks every env
///   under `~/.scoop/virtualenvs/`.
/// * `strict` — when true, return early with a non-zero process exit if any
///   env has at least one Warn or Fail check.
pub fn execute(output: &crate::output::Output, target: Option<&str>, strict: bool) -> Result<()> {
    let service = VirtualenvService::auto()?;

    // Pre-load manifest *once* outside the per-env loop. find_manifest_from_cwd
    // walks the filesystem, which is wasted work to repeat per env.
    let manifest = load_manifest_for_drift_check();

    let envs: Vec<VirtualenvInfo> = match target {
        Some(name) => {
            validate::validate_env_name(name)?;
            if !service.exists(name)? {
                return Err(ScoopError::VirtualenvNotFound {
                    name: name.to_string(),
                });
            }
            let path = service.get_path(name)?;
            // Re-use the VirtualenvInfo shape so the per-env code path doesn't
            // diverge between "single" and "all" modes. `python_version` is
            // populated from metadata downstream.
            vec![VirtualenvInfo {
                name: name.to_string(),
                path,
                python_version: None,
            }]
        }
        None => {
            let mut all = service.list()?;
            // Sort by name for deterministic output — important for JSON
            // consumers and for `--strict` semantics (same set of envs every
            // time).
            all.sort_by(|a, b| a.name.cmp(&b.name));
            all
        }
    };

    if envs.is_empty() {
        if output.is_json() {
            output.json_success(
                "verify",
                VerifyData {
                    envs: Vec::new(),
                    summary: Summary {
                        total: 0,
                        healthy: 0,
                        issues: 0,
                    },
                },
            );
        } else {
            output.info(&t!("verify.no_envs"));
        }
        return Ok(());
    }

    let reports: Vec<EnvReport> = envs
        .iter()
        .map(|env| verify_one(&service, env, manifest.as_ref()))
        .collect();

    let healthy_count = reports.iter().filter(|r| r.healthy).count();
    let summary = Summary {
        total: reports.len(),
        healthy: healthy_count,
        issues: reports.len() - healthy_count,
    };

    if output.is_json() {
        output.json_success(
            "verify",
            VerifyData {
                envs: reports,
                summary,
            },
        );
    } else {
        render_human(output, &reports, &summary);
    }

    // `--strict` opts into the "fail loud" mode. We exit *after* writing the
    // full report so users still get to see what failed even in strict CI.
    if strict && summary.issues > 0 {
        std::process::exit(1);
    }

    Ok(())
}

/// Walk up from cwd looking for `.scoop.toml`. Returns the parsed manifest if
/// found *and* parseable. Parse errors are treated as "no manifest" — we don't
/// want a broken manifest to drown out the actual env checks.
fn load_manifest_for_drift_check() -> Option<ScoopManifest> {
    let path = find_manifest_from_cwd()?;
    ScoopManifest::load(&path).ok()
}

fn verify_one(
    service: &VirtualenvService,
    env: &VirtualenvInfo,
    manifest: Option<&ScoopManifest>,
) -> EnvReport {
    let path = env.path.as_path();
    let mut checks = Vec::with_capacity(6);

    // Check 1: metadata file present + parseable.
    let metadata = service.read_metadata(path);
    let recorded_python = metadata.as_ref().map(|m| m.python_version.clone());
    checks.push(if metadata.is_some() {
        CheckResult::pass("metadata")
    } else if path.join(".scoop-metadata.json").exists() {
        // File exists but failed to deserialize — read_metadata swallows the
        // error and returns None. From a user's view that's still a failure,
        // just a different reason. We surface "unreadable" so they can fix it.
        CheckResult::fail("metadata", t!("verify.metadata_unreadable"))
    } else {
        CheckResult::fail("metadata", t!("verify.metadata_missing"))
    });

    // Check 2: interpreter binary on disk.
    let python_bin = python_binary_path(path);
    let python_present = python_bin.exists();
    checks.push(if python_present {
        CheckResult::pass("python_binary")
    } else {
        CheckResult::fail(
            "python_binary",
            t!("verify.file_missing", path = python_bin.display()),
        )
    });

    // Check 3: pyvenv.cfg marker. Without this, virtualenv-aware tools
    // (including `python -m venv`) won't recognise the directory.
    let pyvenv = path.join("pyvenv.cfg");
    checks.push(if pyvenv.exists() {
        CheckResult::pass("pyvenv_cfg")
    } else {
        CheckResult::fail(
            "pyvenv_cfg",
            t!("verify.file_missing", path = pyvenv.display()),
        )
    });

    // Check 4: activate script.
    let activate = activate_script_path(path);
    checks.push(if activate.exists() {
        CheckResult::pass("activate_script")
    } else {
        CheckResult::fail(
            "activate_script",
            t!("verify.file_missing", path = activate.display()),
        )
    });

    // Check 5: python actually runs. Skip the exec if the binary already
    // failed check 2 — re-reporting the same problem just adds noise.
    checks.push(if python_present {
        match run_python_version(&python_bin) {
            Ok(_) => CheckResult::pass("python_executes"),
            Err(msg) => CheckResult::fail("python_executes", msg),
        }
    } else {
        CheckResult::skip("python_executes")
    });

    // Check 6: manifest drift. Only meaningful when both a manifest and a
    // recorded env Python are available — otherwise skip silently.
    checks.push(match (manifest, recorded_python.as_deref()) {
        (Some(m), Some(env_python)) => check_manifest_drift(&m.environment.python, env_python),
        _ => CheckResult::skip("manifest_match"),
    });

    // Healthy = no Warn / Fail. Skip is treated as "doesn't count against".
    let healthy = checks
        .iter()
        .all(|c| matches!(c.status, CheckStatus::Pass | CheckStatus::Skip));

    EnvReport {
        name: env.name.clone(),
        healthy,
        python: recorded_python,
        checks,
    }
}

fn python_binary_path(env_root: &Path) -> std::path::PathBuf {
    if cfg!(windows) {
        env_root.join("Scripts").join("python.exe")
    } else {
        env_root.join("bin").join("python")
    }
}

fn activate_script_path(env_root: &Path) -> std::path::PathBuf {
    if cfg!(windows) {
        env_root.join("Scripts").join("Activate.ps1")
    } else {
        env_root.join("bin").join("activate")
    }
}

/// Run `<python> --version` and return Ok if it exits successfully.
///
/// Returns the failure reason as a String so callers can stash it in a
/// `CheckResult::fail` without wrapping.
fn run_python_version(python: &Path) -> std::result::Result<(), String> {
    let out = Command::new(python)
        .arg("--version")
        .output()
        .map_err(|e| format!("spawn failed: {e}"))?;
    if out.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&out.stderr);
        Err(format!(
            "exit {}: {}",
            out.status.code().unwrap_or(-1),
            stderr.trim()
        ))
    }
}

fn check_manifest_drift(manifest_python: &str, env_python: &str) -> CheckResult {
    let manifest_parsed = PythonVersion::parse(manifest_python);
    let env_parsed = PythonVersion::parse(env_python);
    match (manifest_parsed, env_parsed) {
        (Some(want), Some(have)) if want.matches(&have) => CheckResult::pass("manifest_match"),
        (Some(_), Some(_)) => CheckResult::warn(
            "manifest_match",
            t!(
                "verify.manifest_drift",
                env = env_python,
                manifest = manifest_python
            ),
        ),
        // If either side is unparseable we don't have enough information to
        // call drift — treat as Skip rather than a false positive.
        _ => CheckResult::skip("manifest_match"),
    }
}

fn render_human(output: &crate::output::Output, reports: &[EnvReport], summary: &Summary) {
    for report in reports {
        let symbol = if report.healthy { "✓" } else { "✗" };
        let py = report
            .python
            .as_deref()
            .map(|v| format!("Python {v}"))
            .unwrap_or_else(|| t!("verify.python_unknown").to_string());
        println!("{symbol} {} ({})", report.name, py);

        // Only show per-check detail when something failed — keep the happy
        // path output compact.
        if !report.healthy {
            for check in &report.checks {
                let detail = match check.status {
                    CheckStatus::Pass | CheckStatus::Skip => continue,
                    CheckStatus::Warn => "  ⚠",
                    CheckStatus::Fail => "  ✗",
                };
                let label = label_for_check(check.name);
                let msg = check.message.as_deref().unwrap_or("");
                if msg.is_empty() {
                    println!("{detail} {label}");
                } else {
                    println!("{detail} {label}: {msg}");
                }
            }
        }
    }

    output.info(&t!(
        "verify.summary",
        total = summary.total.to_string(),
        healthy = summary.healthy.to_string(),
        issues = summary.issues.to_string()
    ));
}

fn label_for_check(name: &str) -> String {
    let key = match name {
        "metadata" => "verify.check_metadata",
        "python_binary" => "verify.check_python_binary",
        "pyvenv_cfg" => "verify.check_pyvenv_cfg",
        "activate_script" => "verify.check_activate_script",
        "python_executes" => "verify.check_python_executes",
        "manifest_match" => "verify.check_manifest_match",
        // Unknown check id — fall back to the id itself so we don't lie via
        // t!()-default if a new check is added without a label.
        other => return other.to_string(),
    };
    t!(key).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Metadata;
    use crate::output::Output;
    use crate::test_utils::with_temp_scoop_home;
    use chrono::Utc;
    use serial_test::serial;
    use std::fs;

    /// Build a minimally-valid env at `<virtualenvs>/<name>`. Returns the
    /// path so callers can mess with it (delete files, etc.) to set up
    /// specific failure scenarios.
    fn make_env(name: &str, python_version: &str) -> std::path::PathBuf {
        let venvs = crate::paths::virtualenvs_dir().unwrap();
        let env_path = venvs.join(name);
        fs::create_dir_all(&env_path).unwrap();

        // metadata
        let meta = Metadata {
            name: name.to_string(),
            python_version: python_version.to_string(),
            created_at: Utc::now(),
            created_by: "scoop test".to_string(),
            uv_version: None,
            python_path: None,
        };
        fs::write(
            env_path.join(".scoop-metadata.json"),
            serde_json::to_string(&meta).unwrap(),
        )
        .unwrap();

        // bin/python — on Unix make it executable + capable of `--version`.
        // Tests gate the actual exec on cfg(unix) so Windows CI doesn't
        // misinterpret a shell script as python.exe.
        let bin = env_path.join("bin");
        fs::create_dir_all(&bin).unwrap();
        let py = bin.join("python");
        fs::write(&py, format!("#!/bin/sh\necho 'Python {python_version}'\n")).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&py, fs::Permissions::from_mode(0o755)).unwrap();
        }

        // pyvenv.cfg
        fs::write(
            env_path.join("pyvenv.cfg"),
            format!("version = {python_version}\n"),
        )
        .unwrap();

        // activate script
        fs::write(bin.join("activate"), "# fake activate\n").unwrap();

        env_path
    }

    #[test]
    #[serial]
    fn healthy_env_passes_all_checks() {
        with_temp_scoop_home(|_| {
            make_env("ok", "3.12.0");
            let output = Output::new(0, true, true, false);
            // No panic, no error — and on Unix, exec check should also pass.
            execute(&output, Some("ok"), false).unwrap();
        });
    }

    #[test]
    #[serial]
    fn missing_metadata_fails_metadata_check() {
        with_temp_scoop_home(|_| {
            let path = make_env("no-meta", "3.12.0");
            fs::remove_file(path.join(".scoop-metadata.json")).unwrap();

            let service = VirtualenvService::auto().unwrap();
            let info = VirtualenvInfo {
                name: "no-meta".to_string(),
                path: path.clone(),
                python_version: None,
            };
            let report = verify_one(&service, &info, None);
            assert!(!report.healthy);
            let meta_check = report
                .checks
                .iter()
                .find(|c| c.name == "metadata")
                .expect("metadata check present");
            assert_eq!(meta_check.status, CheckStatus::Fail);
            assert!(report.python.is_none());
        });
    }

    #[test]
    #[serial]
    fn missing_python_binary_skips_exec_check() {
        with_temp_scoop_home(|_| {
            let path = make_env("no-py", "3.12.0");
            fs::remove_file(path.join("bin").join("python")).unwrap();

            let service = VirtualenvService::auto().unwrap();
            let info = VirtualenvInfo {
                name: "no-py".to_string(),
                path: path.clone(),
                python_version: None,
            };
            let report = verify_one(&service, &info, None);
            let py_bin = report
                .checks
                .iter()
                .find(|c| c.name == "python_binary")
                .unwrap();
            let py_exec = report
                .checks
                .iter()
                .find(|c| c.name == "python_executes")
                .unwrap();
            assert_eq!(py_bin.status, CheckStatus::Fail);
            // Exec is *skipped* when the binary is gone — failing it would
            // just duplicate the python_binary failure.
            assert_eq!(py_exec.status, CheckStatus::Skip);
        });
    }

    #[test]
    #[serial]
    fn missing_pyvenv_cfg_fails() {
        with_temp_scoop_home(|_| {
            let path = make_env("no-cfg", "3.12.0");
            fs::remove_file(path.join("pyvenv.cfg")).unwrap();

            let service = VirtualenvService::auto().unwrap();
            let info = VirtualenvInfo {
                name: "no-cfg".to_string(),
                path,
                python_version: None,
            };
            let report = verify_one(&service, &info, None);
            let cfg = report
                .checks
                .iter()
                .find(|c| c.name == "pyvenv_cfg")
                .unwrap();
            assert_eq!(cfg.status, CheckStatus::Fail);
        });
    }

    #[test]
    #[serial]
    fn missing_activate_fails() {
        with_temp_scoop_home(|_| {
            let path = make_env("no-act", "3.12.0");
            fs::remove_file(path.join("bin").join("activate")).unwrap();

            let service = VirtualenvService::auto().unwrap();
            let info = VirtualenvInfo {
                name: "no-act".to_string(),
                path,
                python_version: None,
            };
            let report = verify_one(&service, &info, None);
            let act = report
                .checks
                .iter()
                .find(|c| c.name == "activate_script")
                .unwrap();
            assert_eq!(act.status, CheckStatus::Fail);
        });
    }

    #[test]
    fn manifest_drift_detected_for_minor_version_mismatch() {
        // No `with_temp_scoop_home` needed — pure function.
        let result = check_manifest_drift("3.12", "3.11.5");
        assert_eq!(result.status, CheckStatus::Warn);
    }

    #[test]
    fn manifest_match_pass_when_versions_compatible() {
        // `3.12` is a pattern, `3.12.4` matches it.
        let result = check_manifest_drift("3.12", "3.12.4");
        assert_eq!(result.status, CheckStatus::Pass);
    }

    #[test]
    fn manifest_match_skip_when_unparseable() {
        let result = check_manifest_drift("latest", "3.12.4");
        assert_eq!(result.status, CheckStatus::Skip);
    }

    #[test]
    #[serial]
    fn empty_scoop_home_emits_no_envs_message() {
        with_temp_scoop_home(|_| {
            let output = Output::new(0, true, true, false);
            execute(&output, None, false).unwrap();
        });
    }

    #[cfg(unix)]
    #[test]
    #[serial]
    fn python_executes_check_passes_on_unix() {
        with_temp_scoop_home(|_| {
            let path = make_env("runs", "3.12.0");

            let service = VirtualenvService::auto().unwrap();
            let info = VirtualenvInfo {
                name: "runs".to_string(),
                path,
                python_version: None,
            };
            let report = verify_one(&service, &info, None);
            let exec = report
                .checks
                .iter()
                .find(|c| c.name == "python_executes")
                .unwrap();
            // We installed a shell script that `echo`s the version — it
            // exits 0, so the check should pass.
            assert_eq!(exec.status, CheckStatus::Pass);
            assert!(report.healthy);
        });
    }
}
