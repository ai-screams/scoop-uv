//! The checks themselves: which environments to look at, and the per-env
//! diagnosis (`verify_one`).

use std::path::Path;
use std::process::Command;

use rust_i18n::t;

use crate::core::manifest::{ScoopManifest, find_manifest_from_cwd};
use crate::core::{MetadataState, VirtualenvInfo, VirtualenvService};
use crate::error::Result;
use crate::python_version::PythonVersion;
use crate::validate;

use super::types::{CheckKind, CheckResult, CheckStatus, EnvReport};

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
            validate::validate_existing_env_name(name)?;
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

pub(super) fn verify_one(env: &VirtualenvInfo, manifest: Option<&ScoopManifest>) -> EnvReport {
    let path = env.path.as_path();
    let mut checks = Vec::with_capacity(6);

    let layout = crate::core::inspect_env(path);

    // Check 1: metadata file present + parseable.
    let (metadata_check, recorded_python) = match layout.metadata {
        MetadataState::Present(m) => (
            CheckResult::pass(CheckKind::Metadata),
            Some(m.python_version),
        ),
        // A file that is there but does not parse is still a failure, just
        // a different one: surface "unreadable" so the user can fix it.
        MetadataState::Unreadable => (
            CheckResult::fail(CheckKind::Metadata, t!("verify.metadata_unreadable")),
            None,
        ),
        MetadataState::Missing => (
            CheckResult::fail(CheckKind::Metadata, t!("verify.metadata_missing")),
            None,
        ),
    };
    checks.push(metadata_check);

    // Check 2: interpreter binary on disk.
    let python_bin = crate::paths::virtualenv_python_exe(path);
    let python_present = layout.python;
    checks.push(if python_present {
        CheckResult::pass(CheckKind::PythonBinary)
    } else {
        CheckResult::fail(
            CheckKind::PythonBinary,
            t!("verify.file_missing", path = python_bin.display()),
        )
    });

    // Check 3: pyvenv.cfg marker. Without this, virtualenv-aware tools
    // (including `python -m venv`) won't recognise the directory.
    checks.push(if layout.pyvenv_cfg {
        CheckResult::pass(CheckKind::PyvenvCfg)
    } else {
        CheckResult::fail(
            CheckKind::PyvenvCfg,
            t!(
                "verify.file_missing",
                path = path.join("pyvenv.cfg").display()
            ),
        )
    });

    // Check 4: activate script.
    checks.push(if layout.activate {
        CheckResult::pass(CheckKind::ActivateScript)
    } else {
        CheckResult::fail(
            CheckKind::ActivateScript,
            t!(
                "verify.file_missing",
                path = crate::paths::virtualenv_activate_script(path).display()
            ),
        )
    });

    // Check 5: python actually runs. Skip the exec if the binary already
    // failed check 2 — re-reporting the same problem just adds noise.
    checks.push(if python_present {
        match run_python_version(&python_bin) {
            Ok(_) => CheckResult::pass(CheckKind::PythonExecutes),
            Err(msg) => CheckResult::fail(CheckKind::PythonExecutes, msg),
        }
    } else {
        CheckResult::skip(CheckKind::PythonExecutes)
    });

    // Check 6: manifest drift. Only meaningful when both a manifest and a
    // recorded env Python are available — otherwise skip silently.
    checks.push(match (manifest, recorded_python.as_deref()) {
        (Some(m), Some(env_python)) => check_manifest_drift(&m.environment.python, env_python),
        _ => CheckResult::skip(CheckKind::ManifestMatch),
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
        (Some(want), Some(have)) if want.matches(&have) => {
            CheckResult::pass(CheckKind::ManifestMatch)
        }
        (Some(_), Some(_)) => CheckResult::warn(
            CheckKind::ManifestMatch,
            t!(
                "verify.manifest_drift",
                env = env_python,
                manifest = manifest_python
            ),
        ),
        // If either side is unparseable we don't have enough information to
        // call drift — treat as Skip rather than a false positive.
        _ => CheckResult::skip(CheckKind::ManifestMatch),
    }
}
