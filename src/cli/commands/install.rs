//! Install command

use crate::error::Result;
use crate::output::Output;
use crate::uv::UvClient;

/// Execute the install command
pub fn execute(output: &Output, version: &str) -> Result<()> {
    let uv = UvClient::new()?;

    output.info(&format!("Installing Python {version}..."));

    uv.install_python(version)?;

    output.success(&format!("Installed Python {version}"));

    Ok(())
}
