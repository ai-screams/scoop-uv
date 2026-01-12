//! Doctor command

use crate::core::doctor::{CheckResult, Doctor};
use crate::error::Result;
use crate::output::Output;

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
        let results = doctor.run_and_fix(output);

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
