//! Uninstall command

use rust_i18n::t;

use crate::error::Result;
use crate::output::{Output, UninstallData};
use crate::uv::UvClient;

/// Execute the uninstall command
pub fn execute(output: &Output, version: &str) -> Result<()> {
    let uv = UvClient::new()?;

    output.info(&t!("uninstall.uninstalling", version = version));

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

    output.success(&t!("uninstall.success", version = version));

    Ok(())
}
