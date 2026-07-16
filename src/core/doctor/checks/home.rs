//! Check for SCUV_HOME directory.

use crate::paths;

use super::super::types::{Check, CheckResult, CheckStatus};

/// Check for SCUV_HOME directory.
pub(super) struct HomeCheck;

impl Check for HomeCheck {
    fn id(&self) -> &'static str {
        "home"
    }

    fn name(&self) -> &'static str {
        "SCUV_HOME directory"
    }

    fn run(&self) -> Vec<CheckResult> {
        match paths::scoop_home() {
            Ok(path) if path.exists() => {
                // Check write permission
                match path.metadata() {
                    Ok(meta) if !meta.permissions().readonly() => {
                        vec![
                            CheckResult::ok(self.id(), self.name())
                                .with_details(format!("{}", path.display())),
                        ]
                    }
                    _ => {
                        vec![
                            CheckResult::error(self.id(), self.name(), "directory not writable")
                                .with_suggestion(format!("chmod 755 {}", path.display())),
                        ]
                    }
                }
            }
            Ok(path) => {
                vec![
                    CheckResult::error(self.id(), self.name(), "directory not found")
                        .with_suggestion(format!("mkdir -p {}", path.display())),
                ]
            }
            Err(_) => {
                vec![
                    CheckResult::error(
                        self.id(),
                        self.name(),
                        "could not determine home directory",
                    )
                    .with_suggestion("Set SCUV_HOME environment variable"),
                ]
            }
        }
    }

    fn fix(&self, result: &CheckResult, output: &crate::output::Output) -> Option<CheckResult> {
        // Only fix "directory not found" errors
        if let CheckStatus::Error(msg) = &result.status {
            if msg.contains("not found") {
                // Create the directory
                if let Ok(home) = paths::scoop_home() {
                    output.info(&format!("Creating {}...", home.display()));

                    match std::fs::create_dir_all(&home) {
                        Ok(_) => {
                            // Also create virtualenvs subdirectory
                            let _ = std::fs::create_dir_all(home.join("virtualenvs"));

                            return Some(
                                CheckResult::ok("home", "SCUV_HOME directory")
                                    .with_details(format!("created {}", home.display())),
                            );
                        }
                        Err(e) => {
                            return Some(
                                CheckResult::error(
                                    "home",
                                    "SCUV_HOME directory",
                                    format!("failed to create: {}", e),
                                )
                                .with_suggestion("Check permissions"),
                            );
                        }
                    }
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::test_utils::with_temp_scoop_home;
    use serial_test::serial;

    #[test]
    #[serial]
    fn home_check_trait_dispatch_ok_on_existing_writable_home() {
        with_temp_scoop_home(|temp| {
            let check: &dyn Check = &HomeCheck;
            assert_eq!(check.id(), "home");
            assert_eq!(check.name(), "SCUV_HOME directory");
            let results = check.run();
            assert_eq!(results.len(), 1, "got {results:#?}");
            assert!(results[0].is_ok(), "got {results:#?}");
            let details = results[0].details.as_deref().unwrap_or_default();
            assert!(
                details.contains(temp.path().to_str().unwrap()),
                "details should name the home path, got {details:?}"
            );
        });
    }

    // ==========================================================================
    // HomeCheck::run — permission/existence branches. `paths::scoop_home()`
    // reads `SCUV_HOME`, so point it at fixtures via env_guard + #[serial].
    // ==========================================================================

    #[test]
    #[serial]
    fn home_run_ok_when_dir_exists_and_writable() {
        let tmp = tempfile::tempdir().unwrap();
        let _g = crate::test_utils::env_guard(&[(
            paths::SCUV_HOME_ENV,
            Some(tmp.path().to_str().unwrap()),
        )]);
        let results = HomeCheck.run();
        assert_eq!(results.len(), 1);
        assert!(
            results[0].is_ok(),
            "existing writable home must be ok: {:#?}",
            results[0]
        );
        assert_eq!(results[0].id, "home");
    }

    #[test]
    #[serial]
    fn home_run_errors_directory_not_found_when_missing() {
        // Kills the `path.exists()` guard mutant: if the guard is forced true,
        // run() falls into the metadata branch and reports "directory not
        // writable" instead of "directory not found".
        let tmp = tempfile::tempdir().unwrap();
        let missing = tmp.path().join("does-not-exist");
        let _g = crate::test_utils::env_guard(&[(
            paths::SCUV_HOME_ENV,
            Some(missing.to_str().unwrap()),
        )]);
        let results = HomeCheck.run();
        assert_eq!(results.len(), 1);
        assert!(results[0].is_error());
        // The suggestion for a missing dir is `mkdir -p ...`; for unwritable it is
        // `chmod 755 ...`. Assert on the missing-dir suggestion to distinguish.
        assert!(
            results[0]
                .suggestion
                .as_deref()
                .unwrap_or("")
                .contains("mkdir"),
            "missing home must suggest mkdir, got {:?}",
            results[0].suggestion
        );
    }

    #[cfg(unix)]
    #[test]
    #[serial]
    fn home_run_errors_not_writable_when_readonly() {
        // Kills the `!meta.permissions().readonly()` guard mutant. chmod 0o555 is
        // ignored by root (Docker CI), so probe first and skip if perms aren't
        // enforced — same pattern as version.rs's unwritable-dir test.
        use std::os::unix::fs::PermissionsExt;
        let tmp = tempfile::tempdir().unwrap();
        let home = tmp.path().join("ro-home");
        std::fs::create_dir_all(&home).unwrap();
        std::fs::set_permissions(&home, std::fs::Permissions::from_mode(0o555)).unwrap();
        // root-skip probe: if we can still create inside, perms aren't enforced.
        let probe = home.join(".perm-probe");
        let perms_enforced = std::fs::write(&probe, b"x").is_err();
        let _ = std::fs::remove_file(&probe);
        if !perms_enforced {
            // restore + skip
            let _ = std::fs::set_permissions(&home, std::fs::Permissions::from_mode(0o755));
            return;
        }
        let _g =
            crate::test_utils::env_guard(&[(paths::SCUV_HOME_ENV, Some(home.to_str().unwrap()))]);
        let results = HomeCheck.run();
        // restore perms so tempdir cleanup works
        let _ = std::fs::set_permissions(&home, std::fs::Permissions::from_mode(0o755));
        assert_eq!(results.len(), 1);
        assert!(
            results[0].is_error(),
            "readonly home must error: {:#?}",
            results[0]
        );
        assert!(
            results[0]
                .suggestion
                .as_deref()
                .unwrap_or("")
                .contains("chmod"),
            "readonly home must suggest chmod, got {:?}",
            results[0].suggestion
        );
    }

    // ==========================================================================
    // HomeCheck::fix — creates the missing home (+ virtualenvs) only for
    // "not found" errors, and leaves anything else alone.
    // ==========================================================================

    #[test]
    #[serial]
    fn fix_home_creates_missing_home_and_virtualenvs_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let missing_home = tmp.path().join("scuvhome");
        let _g = crate::test_utils::env_guard(&[(
            paths::SCUV_HOME_ENV,
            Some(missing_home.to_str().unwrap()),
        )]);

        let output = crate::output::Output::new(0, true, true, false);
        let broken = CheckResult::error("home", "SCUV_HOME directory", "directory not found");

        let fixed = HomeCheck.fix(&broken, &output);
        let fixed = fixed.expect("a 'not found' home error must be fixable");
        assert!(fixed.is_ok(), "got {fixed:#?}");
        assert!(missing_home.is_dir(), "home dir must be created");
        assert!(
            missing_home.join("virtualenvs").is_dir(),
            "virtualenvs subdir must be created"
        );
    }

    #[test]
    #[serial]
    fn fix_home_ignores_non_not_found_results() {
        let tmp = tempfile::tempdir().unwrap();
        let _g = crate::test_utils::env_guard(&[(
            paths::SCUV_HOME_ENV,
            Some(tmp.path().to_str().unwrap()),
        )]);

        let output = crate::output::Output::new(0, true, true, false);
        let unrelated = CheckResult::error("home", "SCUV_HOME directory", "permission denied");
        assert!(HomeCheck.fix(&unrelated, &output).is_none());

        let warning = CheckResult::warn("home", "SCUV_HOME directory", "directory not found");
        assert!(
            HomeCheck.fix(&warning, &output).is_none(),
            "only Error status is fixable, not Warning"
        );
    }
}
