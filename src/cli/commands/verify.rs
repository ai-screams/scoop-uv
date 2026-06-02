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
    /// All checks Pass or Skip (no Warn, no Fail). The "perfect" bucket.
    healthy: usize,
    /// At least one Warn check but no Fail. Informational drift (e.g.
    /// `.scoop.toml` python version mismatch) lives here — the env still
    /// works, the user should just know about it. Does NOT trigger
    /// `--strict` exit.
    warnings: usize,
    /// At least one Fail check. Real breakage. `--strict` exits non-zero
    /// when this is > 0.
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

/// Render the failed-strict JSON envelope in one consistent shape:
/// `{ status: "error", command: "verify", error: { ... }, data: { ... } }`.
///
/// Built manually because the shared [`crate::output::Output`] error
/// envelope doesn't carry domain data — and dropping the report would
/// strip the per-env breakdown that scripts came to `--json` for in the
/// first place. The error block mirrors [`ScoopError::VerifyFailed`]'s
/// code/message so consumers that only read `.error.code` still see
/// "VERIFY_FAILED".
fn emit_strict_json_failure(reports: &[EnvReport], summary: &Summary) {
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

/// Build the list of envs to verify. Single-target path validates the
/// name and confirms existence; all-targets path enumerates and sorts so
/// JSON output and `--strict` semantics are deterministic.
///
/// Returns `VirtualenvInfo` with `python_version: None` regardless of
/// source — the per-env check loop reads metadata directly so the two
/// paths converge on the same downstream shape.
fn collect_target_envs(
    service: &VirtualenvService,
    target: Option<&str>,
) -> Result<Vec<VirtualenvInfo>> {
    match target {
        Some(name) => {
            validate::validate_env_name(name)?;
            if !service.exists(name)? {
                return Err(ScoopError::VirtualenvNotFound {
                    name: name.to_string(),
                });
            }
            let path = service.get_path(name)?;
            Ok(vec![VirtualenvInfo {
                name: name.to_string(),
                path,
                python_version: None,
            }])
        }
        None => {
            let mut all = service.list()?;
            all.sort_by(|a, b| a.name.cmp(&b.name));
            Ok(all)
        }
    }
}

/// Bucket each report into exactly one of healthy / warnings / issues so
/// the three counts sum to `total`. `--strict` reads `summary.issues`
/// only — Warn-only envs (e.g. manifest drift) must NOT trip the exit.
fn compute_summary(reports: &[EnvReport]) -> Summary {
    let mut healthy = 0usize;
    let mut warnings = 0usize;
    let mut issues = 0usize;
    for r in reports {
        let has_fail = r.checks.iter().any(|c| c.status == CheckStatus::Fail);
        let has_warn = r.checks.iter().any(|c| c.status == CheckStatus::Warn);
        if has_fail {
            issues += 1;
        } else if has_warn {
            warnings += 1;
        } else {
            healthy += 1;
        }
    }
    Summary {
        total: reports.len(),
        healthy,
        warnings,
        issues,
    }
}

/// Render the "no envs to check" empty case in both JSON and human modes.
fn emit_empty(output: &crate::output::Output) {
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

    // `healthy` on the report means "no Warn and no Fail" — the perfect
    // case. Envs with Warn-only checks (e.g. manifest drift) are not
    // healthy by this definition, but they don't count as `issues`
    // either; they live in `summary.warnings`. This split keeps `--strict`
    // sane (only fails on Fail) while still letting the human report
    // surface drift to the user.
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
            let path = make_env("ok", "3.12.0");
            let output = Output::new(0, true, true, false);
            // No panic, no error.
            execute(&output, Some("ok"), false).unwrap();

            // Verify the per-env classification directly: every check must
            // be Pass (Unix runs the shell script we wrote) or Skip
            // (manifest_match skips with no .scoop.toml). Q10: previous
            // version of this test only asserted "no error", which would
            // have passed even if every check silently degraded to Warn.
            let service = VirtualenvService::auto().unwrap();
            let info = VirtualenvInfo {
                name: "ok".to_string(),
                path,
                python_version: None,
            };
            let report = verify_one(&service, &info, None);
            assert!(report.healthy, "report should be healthy: {:?}", report);
            assert!(
                report
                    .checks
                    .iter()
                    .all(|c| matches!(c.status, CheckStatus::Pass | CheckStatus::Skip)),
                "every check should be Pass or Skip: {:?}",
                report.checks
            );
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
