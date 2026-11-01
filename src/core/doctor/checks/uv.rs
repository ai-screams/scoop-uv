//! Check for uv installation.

use std::io;
use std::process::{Command, Output};

use crate::uv::install::{MANUAL_URL, Outcome, Stage};
use crate::uv::version as uv_version;

use super::super::types::{Check, CheckResult};

/// Check for uv installation.
pub(super) struct UvCheck;

impl UvCheck {
    /// Platform-appropriate install or upgrade command for uv.
    fn install_hint() -> &'static str {
        if cfg!(target_os = "macos") {
            "brew install uv  OR  curl -LsSf https://astral.sh/uv/install.sh | sh"
        } else if cfg!(target_os = "windows") {
            "powershell -ExecutionPolicy ByPass -c \"irm https://astral.sh/uv/install.ps1 | iex\""
        } else {
            "curl -LsSf https://astral.sh/uv/install.sh | sh"
        }
    }
}

impl Check for UvCheck {
    fn id(&self) -> &'static str {
        "uv"
    }

    fn name(&self) -> &'static str {
        "uv installation"
    }

    fn run(&self) -> Vec<CheckResult> {
        self.report(
            Command::new("uv").arg("--version").output(),
            which::which("uv").is_ok(),
            cfg!(unix),
        )
    }
}

impl UvCheck {
    /// Turns the `uv --version` attempt into a finding. `located` is whether
    /// an executable uv is on `PATH`: running one whose interpreter is gone
    /// also fails with `NotFound`, and that uv is broken, not missing.
    /// `can_install` is whether `doctor --fix` can install uv on this platform.
    fn report(
        &self,
        probe: io::Result<Output>,
        located: bool,
        can_install: bool,
    ) -> Vec<CheckResult> {
        match probe {
            Ok(output) if output.status.success() => {
                let raw = String::from_utf8_lossy(&output.stdout);
                let raw = raw.trim();

                // Enforce the minimum supported uv version when parseable.
                // Unparseable output (custom build, unknown format) is treated
                // as a soft pass so we don't block users on a banner change.
                if let Some(version) = uv_version::parse(raw)
                    && !uv_version::meets_minimum(version)
                {
                    return vec![
                        CheckResult::error(
                            self.id(),
                            self.name(),
                            format!(
                                "uv {} is older than the supported minimum ({})",
                                uv_version::format_version(version),
                                uv_version::format_version(uv_version::MIN_VERSION),
                            ),
                        )
                        .with_details(raw.to_string())
                        .with_suggestion(format!("Upgrade uv: {}", Self::install_hint())),
                    ];
                }

                vec![CheckResult::ok(self.id(), self.name()).with_details(raw.to_string())]
            }
            Err(e) if !located && e.kind() == io::ErrorKind::NotFound => {
                let suggestion = if can_install {
                    rust_i18n::t!("doctor.uv_install.suggestion", url = MANUAL_URL).to_string()
                } else {
                    format!("Install uv: {}", Self::install_hint())
                };
                vec![
                    CheckResult::error(self.id(), self.name(), "uv not found in PATH")
                        .with_details("scuv requires uv to manage Python environments")
                        .with_suggestion(suggestion),
                ]
            }
            // Present but unusable. `doctor --fix` will not install over it,
            // so pointing there would send people round in a circle.
            Ok(output) => self.broken(String::from_utf8_lossy(&output.stderr).trim().to_string()),
            Err(e) => self.broken(e.to_string()),
        }
    }

    fn broken(&self, details: String) -> Vec<CheckResult> {
        vec![
            CheckResult::error(self.id(), self.name(), rust_i18n::t!("doctor.uv_broken"))
                .with_details(details)
                .with_suggestion(rust_i18n::t!(
                    "doctor.uv_broken_suggestion",
                    url = MANUAL_URL
                )),
        ]
    }
}

/// Folds what `doctor --fix` did about uv into the final report.
///
/// `on_path` is whether uv can be found on this process's `PATH` now. A
/// successful install into a directory that is not on it leaves the check
/// failing, and that is what the warning replaces: the check cannot see the
/// shell profile the installer edited. A failed install stays an error even
/// when a runnable uv was left behind, so doctor's exit code reports it.
pub(crate) fn apply_uv_install(results: &mut [CheckResult], outcome: &Outcome, on_path: bool) {
    let id = UvCheck.id();
    let name = UvCheck.name();
    let Some(result) = results.iter_mut().find(|r| r.id == id) else {
        return;
    };
    match outcome {
        Outcome::Declined => {}
        Outcome::Installed { .. } if on_path || !result.is_error() => {}
        Outcome::Installed { path, version } => {
            *result = CheckResult::warn(
                id,
                name,
                rust_i18n::t!(
                    "doctor.uv_install.not_on_path",
                    version = version,
                    path = path.display().to_string()
                ),
            )
            .with_suggestion(rust_i18n::t!("doctor.uv_install.new_shell"));
        }
        Outcome::Collision { path, dir } => {
            *result = CheckResult::error(
                id,
                name,
                rust_i18n::t!(
                    "doctor.uv_install.collision",
                    path = path.display().to_string()
                ),
            )
            .with_suggestion(rust_i18n::t!(
                "doctor.uv_install.collision_suggestion",
                dir = dir.display().to_string()
            ));
        }
        Outcome::Failed { stage, detail } => {
            let stage = match stage {
                Stage::Download => rust_i18n::t!("doctor.uv_install.stage_download"),
                Stage::Run => rust_i18n::t!("doctor.uv_install.stage_run"),
                Stage::Verify => rust_i18n::t!("doctor.uv_install.stage_verify"),
            };
            let mut failed = CheckResult::error(
                id,
                name,
                rust_i18n::t!("doctor.uv_install.failed", stage = stage),
            )
            .with_suggestion(rust_i18n::t!(
                "doctor.uv_install.failed_suggestion",
                url = MANUAL_URL
            ));
            failed.details.clone_from(detail);
            *result = failed;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[test]
    fn uv_check_trait_dispatch_always_reports_under_uv_id() {
        // Environment-tolerant: passes whether or not a usable `uv` is on
        // PATH — what it pins is that run() actually runs and every result
        // carries this check's identity.
        let check: &dyn Check = &UvCheck;
        assert_eq!(check.id(), "uv");
        assert_eq!(check.name(), "uv installation");
        let results = check.run();
        assert!(!results.is_empty(), "run() must produce results");
        assert!(
            results.iter().all(|r| r.id == "uv"),
            "all results must carry the uv id, got {results:#?}"
        );
    }

    #[test]
    fn install_hint_names_uv() {
        // A `-> ""` / `-> "xyzzy"` mutant would drop the real install command.
        let hint = UvCheck::install_hint();
        assert!(
            hint.contains("uv"),
            "install hint must mention uv, got {hint:?}"
        );
        assert!(
            hint.contains("astral") || hint.contains("brew"),
            "install hint must reference a real installer, got {hint:?}"
        );
    }

    #[cfg(unix)]
    fn exited(code: i32, stdout: &str, stderr: &str) -> io::Result<Output> {
        use std::os::unix::process::ExitStatusExt;

        Ok(Output {
            status: std::process::ExitStatus::from_raw(code << 8),
            stdout: stdout.into(),
            stderr: stderr.into(),
        })
    }

    fn not_found() -> io::Result<Output> {
        Err(io::ErrorKind::NotFound.into())
    }

    fn only(mut results: Vec<CheckResult>) -> CheckResult {
        assert_eq!(results.len(), 1, "{results:#?}");
        results.remove(0)
    }

    #[test]
    fn report_points_missing_uv_at_doctor_fix_where_it_can_install() {
        let result = only(UvCheck.report(not_found(), false, true));
        assert!(result.is_error());
        let suggestion = result.suggestion.unwrap();
        assert!(suggestion.contains("scuv doctor --fix"), "{suggestion}");
        assert!(suggestion.contains(MANUAL_URL), "{suggestion}");
    }

    #[test]
    fn report_gives_the_manual_command_where_it_cannot_install() {
        let result = only(UvCheck.report(not_found(), false, false));
        assert!(result.is_error());
        let suggestion = result.suggestion.unwrap();
        assert!(!suggestion.contains("doctor --fix"), "{suggestion}");
        assert!(suggestion.contains(UvCheck::install_hint()), "{suggestion}");
    }

    #[test]
    fn report_treats_a_located_uv_that_will_not_start_as_broken() {
        let result = only(UvCheck.report(not_found(), true, true));
        assert!(result.is_error());
        assert!(!result.suggestion.unwrap().contains("doctor --fix"));
    }

    #[test]
    fn report_does_not_send_a_spawn_failure_to_doctor_fix() {
        let denied = Err(io::ErrorKind::PermissionDenied.into());
        let result = only(UvCheck.report(denied, true, true));
        assert!(result.is_error());
        assert!(!result.suggestion.unwrap().contains("doctor --fix"));
    }

    #[cfg(unix)]
    #[test]
    fn report_does_not_send_a_failing_uv_to_doctor_fix() {
        let result = only(UvCheck.report(exited(1, "", "boom\n"), true, true));
        assert!(result.is_error());
        assert_eq!(result.details.as_deref(), Some("boom"));
        assert!(!result.suggestion.unwrap().contains("doctor --fix"));
    }

    #[cfg(unix)]
    #[test]
    fn report_keeps_upgrade_advice_for_an_old_uv() {
        let result = only(UvCheck.report(exited(0, "uv 0.1.0\n", ""), true, true));
        assert!(result.is_error());
        let suggestion = result.suggestion.unwrap();
        assert!(suggestion.starts_with("Upgrade uv"), "{suggestion}");
        assert!(!suggestion.contains("doctor --fix"), "{suggestion}");
    }

    #[cfg(unix)]
    #[rstest]
    #[case::supported("uv 0.13.6\n")]
    #[case::at_floor("uv 0.5.19\n")]
    #[case::unreadable_banner("mystery\n")]
    fn report_passes_a_working_uv(#[case] stdout: &str) {
        assert!(only(UvCheck.report(exited(0, stdout, ""), true, true)).is_ok());
    }

    fn installed() -> Outcome {
        Outcome::Installed {
            path: "/home/me/.local/bin/uv".into(),
            version: "0.13.6".to_string(),
        }
    }

    fn missing() -> CheckResult {
        CheckResult::error("uv", "uv installation", "uv not found in PATH")
    }

    fn other_error() -> CheckResult {
        CheckResult::error("home", "SCUV_HOME directory", "broken")
    }

    #[test]
    fn apply_turns_the_error_into_a_new_shell_warning_when_not_on_path() {
        let mut results = vec![other_error(), missing()];
        apply_uv_install(&mut results, &installed(), false);

        assert!(results[1].is_warning(), "{:#?}", results[1]);
        let suggestion = results[1].suggestion.as_deref().unwrap();
        assert!(suggestion.contains("scuv doctor --fix"), "{suggestion}");
        assert!(results[0].is_error(), "other findings are untouched");
    }

    #[test]
    fn apply_leaves_a_passing_check_alone_after_an_install() {
        let mut results = vec![CheckResult::ok("uv", "uv installation")];
        apply_uv_install(&mut results, &installed(), true);
        assert!(results[0].is_ok(), "{:#?}", results[0]);
    }

    #[test]
    fn apply_does_not_excuse_an_error_from_the_uv_on_path() {
        let mut results = vec![CheckResult::error("uv", "uv installation", "old")];
        apply_uv_install(&mut results, &installed(), true);
        assert!(results[0].is_error(), "{:#?}", results[0]);
    }

    #[test]
    fn apply_does_not_warn_over_a_passing_check_when_not_on_path() {
        let mut results = vec![CheckResult::ok("uv", "uv installation")];
        apply_uv_install(&mut results, &installed(), false);
        assert!(results[0].is_ok(), "{:#?}", results[0]);
    }

    #[test]
    fn apply_keeps_a_failed_install_an_error_even_if_uv_now_runs() {
        let mut results = vec![CheckResult::ok("uv", "uv installation")];
        let failed = Outcome::Failed {
            stage: Stage::Run,
            detail: Some("exit 1".to_string()),
        };
        apply_uv_install(&mut results, &failed, true);

        assert!(results[0].is_error(), "{:#?}", results[0]);
        assert_eq!(results[0].details.as_deref(), Some("exit 1"));
        assert!(
            results[0]
                .suggestion
                .as_deref()
                .unwrap()
                .contains(MANUAL_URL)
        );
    }

    #[rstest]
    #[case::download(Stage::Download, rust_i18n::t!("doctor.uv_install.stage_download"))]
    #[case::run(Stage::Run, rust_i18n::t!("doctor.uv_install.stage_run"))]
    #[case::verify(Stage::Verify, rust_i18n::t!("doctor.uv_install.stage_verify"))]
    fn apply_names_the_stage_that_failed(
        #[case] stage: Stage,
        #[case] expected: std::borrow::Cow<'static, str>,
    ) {
        let mut results = vec![missing()];
        let failed = Outcome::Failed {
            stage,
            detail: None,
        };
        apply_uv_install(&mut results, &failed, false);

        let crate::core::doctor::CheckStatus::Error(message) = &results[0].status else {
            panic!("{:#?}", results[0]);
        };
        assert!(message.contains(expected.as_ref()), "{message}");
    }

    #[test]
    fn apply_explains_a_collision_and_where_to_look() {
        let mut results = vec![missing()];
        let collision = Outcome::Collision {
            path: "/home/me/.local/bin/uv".into(),
            dir: "/home/me/.local/bin".into(),
        };
        apply_uv_install(&mut results, &collision, false);

        assert!(results[0].is_error());
        let suggestion = results[0].suggestion.as_deref().unwrap();
        assert!(suggestion.contains("/home/me/.local/bin"), "{suggestion}");
    }

    #[test]
    fn apply_leaves_the_finding_when_declined() {
        let mut results = vec![missing()];
        apply_uv_install(&mut results, &Outcome::Declined, false);
        assert_eq!(results[0].status, missing().status);
        assert!(results[0].suggestion.is_none());
    }

    #[test]
    fn apply_ignores_a_report_without_a_uv_check() {
        let mut results = vec![other_error()];
        apply_uv_install(&mut results, &installed(), false);
        assert!(results[0].is_error());
    }
}
