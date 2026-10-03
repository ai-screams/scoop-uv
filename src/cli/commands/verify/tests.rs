use super::checks::{check_manifest_drift, verify_one};
use super::types::CheckStatus;
use super::*;
use crate::core::Metadata;
use crate::core::VirtualenvInfo;
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
        last_used: None,
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
        let output = Output::new(0, true, crate::output::Colors::NONE, false);
        // No panic, no error.
        execute(&output, Some("ok"), false).unwrap();

        // Verify the per-env classification directly: every check must
        // be Pass (Unix runs the shell script we wrote) or Skip
        // (manifest_match skips with no .scuv.toml). Q10: previous
        // version of this test only asserted "no error", which would
        // have passed even if every check silently degraded to Warn.
        let service = VirtualenvService::auto().unwrap();
        let info = VirtualenvInfo {
            name: "ok".to_string(),
            path,
            python_version: None,
            created_at: None,
            last_used: None,
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
            created_at: None,
            last_used: None,
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
            created_at: None,
            last_used: None,
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
            created_at: None,
            last_used: None,
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
            created_at: None,
            last_used: None,
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
        let output = Output::new(0, true, crate::output::Colors::NONE, false);
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
            created_at: None,
            last_used: None,
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
