//! Uninstall command

use crate::error::Result;
use crate::output::{Output, UninstallData};
use crate::uv::UvClient;

/// Execute the uninstall command
pub fn execute(output: &Output, version: &str) -> Result<()> {
    let uv = UvClient::new()?;

    output.info(&format!("Uninstalling Python {version}..."));

    uv.uninstall_python(version)?;

    // JSON output
    if output.is_json() {
        output.json_success(
            "uninstall",
            UninstallData {
                version: version.to_string(),
            },
        );
        return Ok(());
    }

    output.success(&format!("Python {version} uninstalled"));

    Ok(())
}
