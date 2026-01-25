//! Output helpers for use command

use crate::error::Result;
use crate::output::{Output, UseData};

/// Output result in JSON or text format
///
/// Unifies the repetitive pattern of checking is_json() and
/// calling either json_success or success.
pub fn output_result(output: &Output, data: UseData, message: &str) -> Result<()> {
    if output.is_json() {
        output.json_success("use", data);
    } else {
        output.success(message);
    }
    Ok(())
}
