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
