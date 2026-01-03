//! Remove command

use crate::core::VirtualenvService;
use crate::error::Result;
use crate::output::Output;

/// Execute the remove command
pub fn execute(output: &Output, name: &str, force: bool) -> Result<()> {
    let service = VirtualenvService::auto()?;

    // Verify environment exists
    service.get_path(name)?;

    if !force {
        // TODO: Add confirmation prompt with dialoguer
        output.warn(&format!(
            "Are you sure you want to remove '{name}'? Use --force to skip this prompt."
        ));
    }

    output.info(&format!("Removing virtual environment '{name}'..."));
    service.delete(name)?;
    output.success(&format!("Removed virtual environment '{name}'"));

    Ok(())
}
