//! Activate command

use crate::core::VirtualenvService;
use crate::error::Result;
use crate::paths;
use crate::validate;

/// Execute the activate command
/// Outputs shell script to be eval'd
pub fn execute(name: &str) -> Result<()> {
    // Security: Validate input before any processing
    // This is defense-in-depth against command injection via malicious .scoop-version files
    validate::validate_env_name(name)?;

    let service = VirtualenvService::auto()?;

    // Verify environment exists
    let venv_path = service.get_path(name)?;
    let bin_path = paths::virtualenv_bin(name)?;

    // Output activation script for eval
    // Use double quotes so $PATH expands in shell
    println!("export VIRTUAL_ENV=\"{}\"", venv_path.display());
    println!("export PATH=\"{}:$PATH\"", bin_path.display());
    println!("export SCOOP_ACTIVE=\"{}\"", name);
    println!("unset PYTHONHOME");

    Ok(())
}
