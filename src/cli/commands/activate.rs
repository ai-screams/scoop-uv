//! Activate command

use crate::core::VirtualenvService;
use crate::error::Result;
use crate::paths;

/// Execute the activate command
/// Outputs shell script to be eval'd
pub fn execute(name: &str) -> Result<()> {
    let service = VirtualenvService::auto()?;

    // Verify environment exists
    let venv_path = service.get_path(name)?;
    let bin_path = paths::virtualenv_bin(name)?;

    // Output activation script for eval
    println!("export VIRTUAL_ENV='{}'", venv_path.display());
    println!("export PATH='{}:$PATH'", bin_path.display());
    println!("export UVENV_ACTIVE='{name}'");
    println!("unset PYTHONHOME");

    Ok(())
}
