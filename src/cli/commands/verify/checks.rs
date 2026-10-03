//! The checks themselves: which environments to look at, and the per-env
//! diagnosis (`verify_one`).

use std::path::Path;
use std::process::Command;

use rust_i18n::t;

use crate::core::manifest::{ScoopManifest, find_manifest_from_cwd};
use crate::core::{Metadata, VirtualenvInfo, VirtualenvService};
use crate::error::{Result, ScoopError};
use crate::validate::{self, PythonVersion};

use super::types::{CheckResult, CheckStatus, EnvReport};

/// Build the list of envs to verify. Single-target path validates the
/// name and confirms existence; all-targets path enumerates and sorts so
/// JSON output and `--strict` semantics are deterministic.
///
/// Returns `VirtualenvInfo` with `python_version: None` regardless of
/// source — the per-env check loop reads metadata directly so the two
/// paths converge on the same downstream shape.
pub(super) fn collect_target_envs(
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
                created_at: None,
                last_used: None,
            }])
        }
        None => {
            let mut all = service.list()?;
            all.sort_by(|a, b| a.name.cmp(&b.name));
            Ok(all)
        }
    }
}

/// Walk up from cwd looking for `.scuv.toml`. Returns the parsed manifest if
/// found *and* parseable. Parse errors are treated as "no manifest" — we don't
/// want a broken manifest to drown out the actual env checks.
pub(super) fn load_manifest_for_drift_check() -> Option<ScoopManifest> {
    let path = find_manifest_from_cwd()?;
    ScoopManifest::load(&path).ok()
}

pub(super) fn verify_one(
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
    } else if path.join(Metadata::FILE_NAME).exists() {
        // File exists but failed to deserialize — read_metadata swallows the
        // error and returns None. From a user's view that's still a failure,
        // just a different reason. We surface "unreadable" so they can fix it.
        CheckResult::fail("metadata", t!("verify.metadata_unreadable"))
    } else {
        CheckResult::fail("metadata", t!("verify.metadata_missing"))
    });

    // Check 2: interpreter binary on disk.
    let python_bin = crate::paths::virtualenv_python_exe(path);
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
    let activate = crate::paths::virtualenv_activate_script(path);
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

/// Run `<python> --version` and return Ok if it exits successfully.
///
/// Returns the failure reason as a String so callers can stash it in a
/// `CheckResult::fail` without wrapping.
pub(super) fn run_python_version(python: &Path) -> std::result::Result<(), String> {
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

pub(super) fn check_manifest_drift(manifest_python: &str, env_python: &str) -> CheckResult {
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
