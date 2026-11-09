//! Check for virtualenv integrity.

use std::borrow::Cow;

use rust_i18n::t;

use crate::core::MetadataState;
use crate::paths;

use super::super::types::{Check, CheckResult};

/// Check for virtualenv integrity.
pub(super) struct VirtualenvCheck;

impl Check for VirtualenvCheck {
    fn id(&self) -> &'static str {
        "venv"
    }

    fn name(&self) -> Cow<'static, str> {
        t!("doctor.name.venv")
    }

    fn run(&self) -> Vec<CheckResult> {
        let venvs_dir = match paths::virtualenvs_dir() {
            Ok(dir) => dir,
            Err(_) => {
                return vec![CheckResult::error(
                    self.id(),
                    self.name(),
                    t!("doctor.venv.dir_not_found"),
                )];
            }
        };

        if !venvs_dir.exists() {
            return vec![
                CheckResult::ok(self.id(), self.name()).with_details(t!("doctor.venv.none_yet")),
            ];
        }

        // The same walk as `scuv list` and `gc`: real directories only, so a
        // symlink planted under virtualenvs/ is not diagnosed as an env.
        let envs = match crate::core::env_dirs() {
            Ok(envs) => envs,
            Err(e) => {
                return vec![CheckResult::error(
                    self.id(),
                    self.name(),
                    t!("doctor.venv.unreadable_dir", error = e.to_string()),
                )];
            }
        };

        let mut broken = Vec::new();
        let mut warnings = Vec::new();
        let mut healthy = 0;
        for (name, path) in envs {
            let layout = crate::core::inspect_env(&path);
            // A missing interpreter or pyvenv.cfg is the problem to report;
            // its metadata adds nothing until the env is recreated.
            if !layout.python || !layout.pyvenv_cfg {
                broken.push(
                    CheckResult::error(
                        "venv",
                        t!("doctor.name.broken_venv"),
                        t!("doctor.venv.corrupted", name = name),
                    )
                    .with_suggestion(format!(
                        "scuv remove {} && scuv create {} <python-version>",
                        name, name
                    )),
                );
                continue;
            }
            // `scuv verify` fails an env without readable metadata; doctor
            // only warns, as the env itself still works.
            let problem = match layout.metadata {
                MetadataState::Present(_) => {
                    healthy += 1;
                    continue;
                }
                MetadataState::Missing => t!("doctor.venv.no_metadata", name = name),
                MetadataState::Unreadable => t!("doctor.venv.unreadable_metadata", name = name),
            };
            warnings.push(
                CheckResult::warn("venv", t!("doctor.name.venv_metadata"), problem)
                    .with_suggestion(format!("scuv verify {}", name)),
            );
        }

        // Broken environments first, then metadata warnings.
        let all_healthy = broken.is_empty() && warnings.is_empty();
        let mut results = broken;
        results.append(&mut warnings);

        // Summary only when every env is healthy.
        if all_healthy {
            if healthy > 0 {
                results.push(
                    CheckResult::ok(self.id(), self.name())
                        .with_details(t!("doctor.venv.all_healthy", count = healthy)),
                );
            } else {
                results.push(
                    CheckResult::ok(self.id(), self.name())
                        .with_details(t!("doctor.venv.none_yet")),
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
            write_metadata(&env);

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
    // exist and its metadata reads. Fake venvs with empty files suffice (no
    // real python needed).
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
        write_metadata(&dir);
    }

    fn write_metadata(dir: &std::path::Path) {
        let meta = crate::core::Metadata::new("e".to_string(), "3.12".to_string(), None);
        std::fs::write(
            dir.join(crate::core::Metadata::FILE_NAME),
            serde_json::to_string(&meta).unwrap(),
        )
        .unwrap();
    }

    fn in_temp_home<T>(f: impl FnOnce(&std::path::Path) -> T) -> T {
        let tmp = tempfile::tempdir().unwrap();
        let _g = crate::test_utils::env_guard(&[(
            paths::SCUV_HOME_ENV,
            Some(tmp.path().to_str().unwrap()),
        )]);
        let venvs = crate::paths::virtualenvs_dir().unwrap();
        std::fs::create_dir_all(&venvs).unwrap();
        f(&venvs)
    }

    /// An env that works but has no metadata, or metadata that does not
    /// parse, is a warning naming `scuv verify`, not an error, and there is
    /// no all-healthy summary. What fails this: treating metadata as part of
    /// "broken", or not looking at it.
    #[test]
    #[serial]
    fn venv_run_warns_about_missing_or_unreadable_metadata() {
        in_temp_home(|venvs| {
            make_venv(venvs, "fine", true, true);
            make_venv(venvs, "nometa", true, true);
            std::fs::remove_file(venvs.join("nometa").join(crate::core::Metadata::FILE_NAME))
                .unwrap();
            make_venv(venvs, "badmeta", true, true);
            std::fs::write(
                venvs.join("badmeta").join(crate::core::Metadata::FILE_NAME),
                "{x",
            )
            .unwrap();

            let results = VirtualenvCheck.run();
            assert_eq!(results.len(), 2, "{results:#?}");
            assert!(results.iter().all(|r| r.is_warning()), "{results:#?}");
            let messages: Vec<String> = results
                .iter()
                .map(|r| match &r.status {
                    CheckStatus::Warning(m) => m.clone(),
                    _ => unreachable!(),
                })
                .collect();
            assert!(messages.contains(&"'nometa' has no scuv metadata".to_string()));
            assert!(messages.contains(&"'badmeta' has unreadable scuv metadata".to_string()));
            let nometa = results
                .iter()
                .find(|r| matches!(&r.status, CheckStatus::Warning(m) if m.contains("nometa")))
                .unwrap();
            assert_eq!(nometa.suggestion.as_deref(), Some("scuv verify nometa"));
        });
    }

    /// A broken env is reported once, as broken: its missing metadata adds
    /// nothing until it is recreated.
    #[test]
    #[serial]
    fn venv_run_reports_a_broken_env_without_a_metadata_warning() {
        in_temp_home(|venvs| {
            std::fs::create_dir_all(venvs.join("empty")).unwrap();
            let results = VirtualenvCheck.run();
            assert_eq!(results.len(), 1, "{results:#?}");
            assert!(results[0].is_error());
        });
    }

    /// A symlink under virtualenvs/ is not an env, as for `scuv list`:
    /// neither this check nor the symlink check looks at it. What fails this:
    /// walking with `path.is_dir()`, which follows the link.
    #[cfg(unix)]
    #[test]
    #[serial]
    fn venv_run_ignores_a_symlinked_entry() {
        in_temp_home(|venvs| {
            make_venv(venvs, "fine", true, true);
            let elsewhere = tempfile::tempdir().unwrap();
            std::os::unix::fs::symlink(elsewhere.path(), venvs.join("linked")).unwrap();

            let results = VirtualenvCheck.run();
            assert_eq!(results.len(), 1, "{results:#?}");
            assert!(results[0].is_ok());
            assert!(
                results[0]
                    .details
                    .as_deref()
                    .unwrap()
                    .contains("1 environments")
            );
        });
    }

    /// Stands in for the symlink fix: reports an error and, when fixed,
    /// gives the env the interpreter and pyvenv.cfg it was missing.
    struct RepairsEnv(std::path::PathBuf);
    impl super::super::super::types::Check for RepairsEnv {
        fn id(&self) -> &'static str {
            "symlink"
        }
        fn name(&self) -> std::borrow::Cow<'static, str> {
            "symbolic links".into()
        }
        fn run(&self) -> Vec<CheckResult> {
            vec![CheckResult::error("symlink", "broken symlink", "broken")]
        }
        fn fix(&self, _: &CheckResult, _: &crate::output::Output) -> Option<CheckResult> {
            let py = crate::paths::virtualenv_python_exe(&self.0);
            std::fs::create_dir_all(py.parent().unwrap()).unwrap();
            std::fs::write(&py, b"").unwrap();
            std::fs::write(self.0.join("pyvenv.cfg"), b"").unwrap();
            Some(CheckResult::ok("symlink", "broken symlink"))
        }
    }

    /// `doctor --fix` mends an env that has no metadata: its "broken
    /// virtualenv" error is replaced by the metadata warning when the
    /// virtualenvs check runs again, so the run ends with a warning (exit
    /// 1), not an error. What fails this: the engine keeping the first
    /// result of the virtualenvs check after another check's fix.
    #[test]
    #[serial]
    fn a_repaired_env_without_metadata_ends_as_a_warning() {
        in_temp_home(|venvs| {
            let env = venvs.join("mended");
            std::fs::create_dir_all(&env).unwrap();

            let doctor = crate::core::doctor::Doctor::with_checks(vec![
                Box::new(VirtualenvCheck),
                Box::new(RepairsEnv(env)),
            ]);
            let output = crate::output::Output::new(0, true, crate::output::Colors::NONE, false);
            let results = doctor.collect_and_fix(&output);

            assert!(!results.iter().any(CheckResult::is_error), "{results:#?}");
            let warnings: Vec<_> = results.iter().filter(|r| r.is_warning()).collect();
            assert_eq!(warnings.len(), 1, "{results:#?}");
            assert!(
                matches!(&warnings[0].status, CheckStatus::Warning(m) if m == "'mended' has no scuv metadata")
            );
        });
    }

    /// doctor does not check the activate script; `scuv verify` does.
    #[test]
    #[serial]
    fn venv_run_does_not_require_an_activate_script() {
        in_temp_home(|venvs| {
            make_venv(venvs, "noact", true, true);
            let results = VirtualenvCheck.run();
            assert!(results.iter().all(|r| r.is_ok()), "{results:#?}");
        });
    }

    /// A virtualenvs/ that cannot be listed is an error, not "no environments
    /// yet". A file in its place makes the listing fail for any user.
    #[test]
    #[serial]
    fn venv_run_reports_an_unreadable_virtualenvs_dir() {
        in_temp_home(|venvs| {
            std::fs::remove_dir_all(venvs).unwrap();
            std::fs::write(venvs, "").unwrap();
            let results = VirtualenvCheck.run();
            assert_eq!(results.len(), 1, "{results:#?}");
            assert!(
                matches!(&results[0].status, CheckStatus::Error(m) if m.starts_with("cannot read virtualenvs directory")),
                "{results:#?}"
            );
        });
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
