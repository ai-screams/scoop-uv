//! Uninstall command

use crate::error::Result;
use crate::output::Output;
use crate::uv::UvClient;

/// Execute the uninstall command
pub fn execute(output: &Output, version: &str) -> Result<()> {
    let uv = UvClient::new()?;

    output.info(&format!("Uninstalling Python {version}..."));

    uv.uninstall_python(version)?;

    output.success(&format!("Uninstalled Python {version}"));

    Ok(())
}
