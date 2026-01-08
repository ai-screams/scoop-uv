//! Doctor command

use crate::core::doctor::Doctor;
use crate::error::Result;
use crate::output::Output;

/// Execute the doctor command.
///
/// Runs all health checks and reports any issues found.
///
/// # Exit codes
///
/// - 0: All checks passed
/// - 1: Some warnings found
/// - 2: Some errors found
pub fn execute(output: &Output) -> Result<()> {
    let doctor = Doctor::new();
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

    // Determine exit code based on results
    let has_errors = results.iter().any(|r| r.is_error());
    let has_warnings = results.iter().any(|r| r.is_warning());

    if has_errors {
        std::process::exit(2);
    } else if has_warnings {
        std::process::exit(1);
    }

    Ok(())
}
