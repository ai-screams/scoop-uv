//! Doctor command

use std::env;
use std::io::{self, IsTerminal};

use dialoguer::Confirm;
use rust_i18n::t;

use crate::core::doctor::{CheckResult, Doctor, apply_uv_install};
use crate::error::Result;
use crate::output::Output;
use crate::uv::install::{self, MANUAL_URL, Outcome, Plan, ProcessRunner, Skip};

/// Calculates exit code based on check results.
///
/// - 0: All checks passed
/// - 1: Warnings found (no errors)
/// - 2: Errors found (takes priority over warnings)
#[inline]
fn calculate_exit_code(results: &[CheckResult]) -> i32 {
    if results.iter().any(|r| r.is_error()) {
        2
    } else if results.iter().any(|r| r.is_warning()) {
        1
    } else {
        0
    }
}

/// Exit process with appropriate code based on check results.
///
/// - Exit 2: If any errors found
/// - Exit 1: If any warnings found (no errors)
/// - No exit: If all checks passed
fn exit_with_result_status(results: &[CheckResult]) {
    let code = calculate_exit_code(results);
    if code != 0 {
        std::process::exit(code);
    }
}

/// Offers to install uv if it is missing. `None` when nothing was offered.
///
/// The temporary directory holding the script ends with this function: the
/// caller may leave through `process::exit`, which runs no destructors.
fn offer_uv_install(output: &Output) -> Option<Outcome> {
    if which::which("uv").is_ok() {
        return None;
    }

    let context = install::Context {
        machine_output: output.is_json() || output.is_quiet(),
        unix: cfg!(unix),
        offline: install::offline_enabled(env::var_os("UV_OFFLINE").as_deref()),
        terminal: io::stdin().is_terminal() && io::stderr().is_terminal(),
    };
    if let Err(skip) = install::check(context) {
        match skip {
            // The uv finding in the report covers it, and these modes print
            // nothing else.
            Skip::MachineOutput => {}
            Skip::Unsupported => output.warn(&t!("doctor.uv_install.skip_windows")),
            Skip::Offline => output.warn(&t!("doctor.uv_install.skip_offline")),
            Skip::NoTerminal => output.warn(&t!("doctor.uv_install.skip_no_terminal")),
        }
        return None;
    }

    let tool = |name: &'static str| {
        let found = which::which(name);
        if found.is_err() {
            output.warn(&t!(
                "doctor.uv_install.skip_no_tool",
                tool = name,
                url = MANUAL_URL
            ));
        }
        found.ok()
    };
    let curl = tool("curl")?;
    let sh = tool("sh")?;
    let home = dirs::home_dir();
    let cargo_home = env::var_os("CARGO_HOME");
    let Some(dir) = install::install_dir(home.as_deref(), cargo_home.as_deref()) else {
        output.warn(&t!(
            "doctor.uv_install.skip_no_destination",
            url = MANUAL_URL
        ));
        return None;
    };
    let temp = match tempfile::tempdir() {
        Ok(temp) => temp,
        Err(e) => {
            return Some(Outcome::Failed {
                stage: install::Stage::Download,
                detail: Some(e.to_string()),
            });
        }
    };

    let plan = Plan::new(curl, sh, &temp.path().join("install.sh"), dir);
    Some(install::install(
        &plan,
        |plan| confirm(output, plan),
        &mut ProcessRunner,
    ))
}

/// Shows exactly what will run and asks. Anything but a yes is a no, including
/// an unusable terminal.
fn confirm(output: &Output, plan: &Plan) -> bool {
    output.info(&t!("doctor.uv_install.plan"));
    output.info(&plan.download.display());
    output.info(&plan.run.display());
    output.info(&t!(
        "doctor.uv_install.effects",
        dir = plan.dir.display().to_string()
    ));
    let inherited = install::inherited_settings(env::vars_os());
    if !inherited.is_empty() {
        output.warn(&t!(
            "doctor.uv_install.inherited",
            names = inherited.join(", ")
        ));
    }
    Confirm::new()
        .with_prompt(t!("doctor.uv_install.confirm").to_string())
        .default(false)
        .interact()
        .unwrap_or(false)
}

/// Execute the doctor command.
///
/// Runs all health checks and reports any issues found.
///
/// # Exit codes
///
/// - 0: All checks passed
/// - 1: Some warnings found
/// - 2: Some errors found
pub fn execute(output: &Output, fix: bool) -> Result<()> {
    let doctor = Doctor::new();

    if fix {
        // Run with auto-fix
        output.doctor_header();
        // Before the fixes, so a repair that needs uv can find it if it landed
        // on PATH.
        let install = offer_uv_install(output);
        let mut results = doctor.collect_and_fix(output);
        match &install {
            Some(Outcome::Declined) => output.info(&t!("doctor.uv_install.cancelled")),
            Some(Outcome::Installed { path, version }) => output.success(&t!(
                "doctor.uv_install.success",
                version = version,
                path = path.display().to_string()
            )),
            _ => {}
        }
        if let Some(outcome) = &install {
            apply_uv_install(&mut results, outcome, which::which("uv").is_ok());
        }
        for result in &results {
            output.doctor_check(result);
        }

        // Print summary or JSON
        if output.is_json() {
            output.doctor_json(&results);
        } else {
            output.doctor_summary(&results);
        }

        exit_with_result_status(&results);
    } else {
        // Normal run
        let results = doctor.run_all();

        // Print header
        output.doctor_header();

        // Print each result
        for result in &results {
            output.doctor_check(result);
        }

        // Print summary or JSON
        if output.is_json() {
            output.doctor_json(&results);
        } else {
            output.doctor_summary(&results);
        }

        exit_with_result_status(&results);
    }

    Ok(())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::doctor::CheckStatus;

    /// Creates a test CheckResult with given status.
    fn make_result(status: CheckStatus) -> CheckResult {
        CheckResult {
            id: "test",
            name: "Test Check",
            status,
            suggestion: None,
            details: None,
        }
    }

    #[test]
    fn calculate_exit_code_all_passed_returns_zero() {
        let results = vec![make_result(CheckStatus::Ok), make_result(CheckStatus::Ok)];
        assert_eq!(calculate_exit_code(&results), 0);
    }

    #[test]
    fn calculate_exit_code_empty_results_returns_zero() {
        let results: Vec<CheckResult> = vec![];
        assert_eq!(calculate_exit_code(&results), 0);
    }

    #[test]
    fn calculate_exit_code_warnings_only_returns_one() {
        let results = vec![
            make_result(CheckStatus::Ok),
            make_result(CheckStatus::Warning("something".into())),
        ];
        assert_eq!(calculate_exit_code(&results), 1);
    }

    #[test]
    fn calculate_exit_code_errors_only_returns_two() {
        let results = vec![
            make_result(CheckStatus::Ok),
            make_result(CheckStatus::Error("critical".into())),
        ];
        assert_eq!(calculate_exit_code(&results), 2);
    }

    #[test]
    fn calculate_exit_code_mixed_errors_and_warnings_returns_two() {
        // Error takes priority over warning
        let results = vec![
            make_result(CheckStatus::Warning("minor".into())),
            make_result(CheckStatus::Error("major".into())),
            make_result(CheckStatus::Ok),
        ];
        assert_eq!(calculate_exit_code(&results), 2);
    }

    #[test]
    fn calculate_exit_code_multiple_warnings_returns_one() {
        let results = vec![
            make_result(CheckStatus::Warning("warn1".into())),
            make_result(CheckStatus::Warning("warn2".into())),
        ];
        assert_eq!(calculate_exit_code(&results), 1);
    }

    #[test]
    fn calculate_exit_code_multiple_errors_returns_two() {
        let results = vec![
            make_result(CheckStatus::Error("err1".into())),
            make_result(CheckStatus::Error("err2".into())),
        ];
        assert_eq!(calculate_exit_code(&results), 2);
    }
}
