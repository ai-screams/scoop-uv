//! Doctor command

mod render;

use std::env;
use std::io::{self, IsTerminal};

use rust_i18n::t;

use crate::core::doctor::{CheckResult, Doctor, apply_uv_install};
use crate::error::{Result, ScoopError};
use crate::output::Output;
use crate::uv::install::{self, MANUAL_URL, Outcome, Plan, ProcessRunner, Skip};

/// The outcome of a run: `Ok` when every check passed, otherwise
/// [`ScoopError::DoctorFindings`] with the counts (exit 2 with any error, 1
/// with only warnings). The report is already printed, so the error is quiet.
fn findings(results: &[CheckResult]) -> Result<()> {
    let errors = results.iter().filter(|r| r.is_error()).count();
    let warnings = results.iter().filter(|r| r.is_warning()).count();
    if errors + warnings == 0 {
        Ok(())
    } else {
        Err(ScoopError::DoctorFindings { errors, warnings })
    }
}

fn install_context(output: &Output, terminal: bool) -> install::Context {
    install::Context {
        machine_output: output.is_json() || output.is_quiet(),
        unix: cfg!(unix),
        offline: install::offline_enabled(env::var_os("UV_OFFLINE").as_deref()),
        terminal,
    }
}

/// Offers to install uv if it is missing. `None` when nothing was offered.
///
/// The temporary directory holding the script ends with this function, so it
/// is gone before the report is printed.
fn offer_uv_install(output: &Output) -> Option<Outcome> {
    if which::which("uv").is_ok() {
        return None;
    }

    let terminal = io::stdin().is_terminal() && io::stderr().is_terminal();
    if let Err(skip) = install::check(install_context(output, terminal)) {
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
    crate::cli::confirm::confirm(&t!("doctor.uv_install.confirm"))
        == crate::cli::confirm::Confirmation::Yes
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
        render::header(output);
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
            render::check(output, result);
        }

        // Print summary or JSON
        if output.is_json() {
            render::json(output, &results);
        } else {
            render::summary(output, &results);
        }

        findings(&results)
    } else {
        // Normal run
        let results = doctor.run_all();

        // Print header
        render::header(output);

        // Print each result
        for result in &results {
            render::check(output, result);
        }

        // Print summary or JSON
        if output.is_json() {
            render::json(output, &results);
        } else {
            render::summary(output, &results);
        }

        findings(&results)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::doctor::CheckStatus;
    use crate::output::Colors;

    // A broken wiring here would still look safe from the CLI tests, which run
    // without a terminal and so never get past that check.
    #[rstest::rstest]
    #[case::json(false, true)]
    #[case::quiet(true, false)]
    fn machine_output_is_refused_even_at_a_terminal(#[case] quiet: bool, #[case] json: bool) {
        let output = Output::new(0, quiet, Colors::NONE, json);
        let context = install_context(&output, true);
        assert_eq!(install::check(context), Err(Skip::MachineOutput));
    }

    #[test]
    fn normal_output_is_not_machine_output() {
        let output = Output::new(0, false, Colors::NONE, false);
        assert!(!install_context(&output, true).machine_output);
    }

    /// Creates a test CheckResult with given status.
    fn exit_code(results: &[CheckResult]) -> u8 {
        findings(results).map_or_else(|e| e.exit_code(), |()| 0)
    }

    fn make_result(status: CheckStatus) -> CheckResult {
        CheckResult {
            id: "test",
            name: "Test Check".into(),
            status,
            suggestion: None,
            details: None,
            target: None,
        }
    }

    #[test]
    fn exit_code_all_passed_returns_zero() {
        let results = vec![make_result(CheckStatus::Ok), make_result(CheckStatus::Ok)];
        assert_eq!(exit_code(&results), 0);
    }

    #[test]
    fn exit_code_empty_results_returns_zero() {
        let results: Vec<CheckResult> = vec![];
        assert_eq!(exit_code(&results), 0);
    }

    #[test]
    fn exit_code_warnings_only_returns_one() {
        let results = vec![
            make_result(CheckStatus::Ok),
            make_result(CheckStatus::Warning("something".into())),
        ];
        assert_eq!(exit_code(&results), 1);
    }

    #[test]
    fn exit_code_errors_only_returns_two() {
        let results = vec![
            make_result(CheckStatus::Ok),
            make_result(CheckStatus::Error("critical".into())),
        ];
        assert_eq!(exit_code(&results), 2);
    }

    #[test]
    fn exit_code_mixed_errors_and_warnings_returns_two() {
        // Error takes priority over warning
        let results = vec![
            make_result(CheckStatus::Warning("minor".into())),
            make_result(CheckStatus::Error("major".into())),
            make_result(CheckStatus::Ok),
        ];
        assert_eq!(exit_code(&results), 2);
    }

    #[test]
    fn exit_code_multiple_warnings_returns_one() {
        let results = vec![
            make_result(CheckStatus::Warning("warn1".into())),
            make_result(CheckStatus::Warning("warn2".into())),
        ];
        assert_eq!(exit_code(&results), 1);
    }

    #[test]
    fn exit_code_multiple_errors_returns_two() {
        let results = vec![
            make_result(CheckStatus::Error("err1".into())),
            make_result(CheckStatus::Error("err2".into())),
        ];
        assert_eq!(exit_code(&results), 2);
    }
}
