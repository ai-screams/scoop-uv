//! Check for virtualenv integrity.

use crate::paths;

use super::super::types::{Check, CheckResult};

/// Check for virtualenv integrity.
pub(super) struct VirtualenvCheck;

impl Check for VirtualenvCheck {
    fn id(&self) -> &'static str {
        "venv"
    }

    fn name(&self) -> &'static str {
        "virtual environments"
    }

    fn run(&self) -> Vec<CheckResult> {
        let venvs_dir = match paths::virtualenvs_dir() {
            Ok(dir) => dir,
            Err(_) => {
                return vec![CheckResult::error(
                    self.id(),
                    self.name(),
                    "virtualenvs directory not found",
                )];
            }
        };

        if !venvs_dir.exists() {
            return vec![
                CheckResult::ok(self.id(), self.name()).with_details("no environments yet"),
            ];
        }

        let mut results = Vec::new();
        let mut healthy = 0;
        let mut broken_names = Vec::new();

        if let Ok(entries) = std::fs::read_dir(&venvs_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let name = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown")
                        .to_string();

                    let python_path = crate::paths::virtualenv_python_exe(&path);
                    let pyvenv_cfg = path.join("pyvenv.cfg");

                    if python_path.exists() && pyvenv_cfg.exists() {
                        healthy += 1;
                    } else {
                        broken_names.push(name);
                    }
                }
            }
        }

        // Report broken environments
        for name in &broken_names {
            results.push(
                CheckResult::error(
                    "venv",
                    "broken virtualenv",
                    format!("'{}' is corrupted", name),
                )
                .with_suggestion(format!(
                    "scuv remove {} && scuv create {} <python-version>",
                    name, name
                )),
            );
        }

        // Summary
        if broken_names.is_empty() {
            if healthy > 0 {
                results.push(
                    CheckResult::ok(self.id(), self.name())
                        .with_details(format!("{} environments, all healthy", healthy)),
                );
            } else {
                results.push(
                    CheckResult::ok(self.id(), self.name()).with_details("no environments yet"),
                );
            }
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==========================================================================
    // VirtualenvCheck coverage
    //
    // These tests pin the run() implementation against the cargo-mutants
    // `replace -> vec![]` mutations. Without them, deleting the entire
    // function body would still pass the suite, masking real regressions.
    // ==========================================================================

    use super::super::super::types::CheckStatus;
    use crate::test_utils::with_temp_scoop_home;
    use serial_test::serial;

    #[test]
    #[serial]
    fn virtualenv_check_emits_broken_for_corrupted_env() {
        with_temp_scoop_home(|temp| {
            // Build a deliberately broken venv: directory exists but
            // pyvenv.cfg is missing and there's no python binary. The
            // check must surface this as a broken-virtualenv error
            // (not silently return an empty vec or an Ok summary).
            let broken = temp.path().join("virtualenvs").join("brokenenv");
            std::fs::create_dir_all(&broken).unwrap();

            let results = VirtualenvCheck.run();
            assert!(!results.is_empty(), "expected at least one CheckResult");
            assert!(
                results.iter().any(|r| r.is_error()
                    && r.name.contains("broken")
                    && matches!(&r.status, CheckStatus::Error(msg) if msg.contains("brokenenv"))),
                "expected an error CheckResult naming the broken env, got {results:#?}"
            );
        });
    }

    #[test]
    #[serial]
    fn virtualenv_check_returns_ok_for_healthy_env() {
        with_temp_scoop_home(|temp| {
            let env = temp.path().join("virtualenvs").join("healthy");
            std::fs::create_dir_all(env.join("bin")).unwrap();
            std::fs::write(env.join("bin").join("python"), "").unwrap();
            std::fs::write(env.join("pyvenv.cfg"), "").unwrap();

            let results = VirtualenvCheck.run();
            assert!(!results.is_empty(), "expected a non-empty result set");
            assert!(
                results.iter().all(|r| r.is_ok()),
                "all results should be Ok, got {results:#?}"
            );
            assert_eq!(results[0].id, "venv");
            assert_eq!(results[0].name, "virtual environments");
        });
    }

    // ==========================================================================
    // VirtualenvCheck::run — healthy/broken counting and summary selection.
    // A venv is healthy iff `virtualenv_python_exe(dir)` and `pyvenv.cfg` both
    // exist. Fake venvs with empty files suffice (no real python needed).
    // `virtualenvs_dir()` derives from `SCUV_HOME`.
    // ==========================================================================

    fn make_venv(venvs_dir: &std::path::Path, name: &str, python: bool, cfg: bool) {
        let dir = venvs_dir.join(name);
        std::fs::create_dir_all(&dir).unwrap();
        if python {
            let py = crate::paths::virtualenv_python_exe(&dir);
            std::fs::create_dir_all(py.parent().unwrap()).unwrap();
            std::fs::write(&py, b"").unwrap();
        }
        if cfg {
            std::fs::write(dir.join("pyvenv.cfg"), b"home = /x\n").unwrap();
        }
    }

    #[test]
    #[serial]
    fn venv_run_reports_all_healthy_with_count() {
        // Kills `+= -> -=/*=` (count must equal 2) and `healthy > 0` -> `==`/`<`
        // (must take the all-healthy branch).
        let tmp = tempfile::tempdir().unwrap();
        let _g = crate::test_utils::env_guard(&[(
            paths::SCUV_HOME_ENV,
            Some(tmp.path().to_str().unwrap()),
        )]);
        let venvs = crate::paths::virtualenvs_dir().unwrap();
        make_venv(&venvs, "a", true, true);
        make_venv(&venvs, "b", true, true);
        let results = VirtualenvCheck.run();
        assert_eq!(results.len(), 1);
        assert!(results[0].is_ok());
        assert!(
            results[0]
                .details
                .as_deref()
                .unwrap_or("")
                .contains("2 environments, all healthy"),
            "got {:?}",
            results[0].details
        );
    }

    #[test]
    #[serial]
    fn venv_run_reports_broken_when_cfg_missing() {
        // Kills `&& -> ||`: with python present but pyvenv.cfg absent the env is
        // broken; an `||` mutant would count it healthy.
        let tmp = tempfile::tempdir().unwrap();
        let _g = crate::test_utils::env_guard(&[(
            paths::SCUV_HOME_ENV,
            Some(tmp.path().to_str().unwrap()),
        )]);
        let venvs = crate::paths::virtualenvs_dir().unwrap();
        make_venv(&venvs, "half", true, false); // python but no cfg -> broken
        let results = VirtualenvCheck.run();
        assert!(
            results.iter().any(|r| r.is_error() && r.id == "venv"),
            "half-built env must be reported broken: {results:#?}"
        );
        assert!(
            !results.iter().any(|r| r.is_ok()),
            "no all-healthy summary when a broken env exists: {results:#?}"
        );
    }

    #[test]
    #[serial]
    fn venv_run_says_no_environments_when_dir_empty() {
        // Kills `healthy > 0` -> `>=`: with zero healthy envs the summary must be
        // "no environments yet", not "0 environments, all healthy".
        let tmp = tempfile::tempdir().unwrap();
        let _g = crate::test_utils::env_guard(&[(
            paths::SCUV_HOME_ENV,
            Some(tmp.path().to_str().unwrap()),
        )]);
        let venvs = crate::paths::virtualenvs_dir().unwrap();
        std::fs::create_dir_all(&venvs).unwrap(); // exists but empty
        let results = VirtualenvCheck.run();
        assert_eq!(results.len(), 1);
        assert!(results[0].is_ok());
        assert!(
            results[0]
                .details
                .as_deref()
                .unwrap_or("")
                .contains("no environments yet"),
            "got {:?}",
            results[0].details
        );
    }
}
