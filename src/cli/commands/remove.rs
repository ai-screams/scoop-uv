//! Remove command

use dialoguer::Confirm;

use crate::core::VirtualenvService;
use crate::error::Result;
use crate::output::{Output, RemoveData};

/// Execute the remove command
pub fn execute(output: &Output, name: &str, force: bool) -> Result<()> {
    let service = VirtualenvService::auto()?;

    // Verify environment exists
    let path = service.get_path(name)?;

    // JSON mode always implies force (no interactive confirmation)
    if !force && !output.is_json() {
        // Show what will be deleted
        output.info(&format!("Environment path: {}", path.display()));

        let confirmed = Confirm::new()
            .with_prompt(format!("Remove virtual environment '{name}'?"))
            .default(false)
            .interact()
            .unwrap_or(false);

        if !confirmed {
            output.info("Cancelled");
            return Ok(());
        }
    }

    output.info(&format!("Removing virtual environment '{name}'..."));
    service.delete(name)?;

    // JSON output
    if output.is_json() {
        output.json_success(
            "remove",
            RemoveData {
                name: name.to_string(),
                path: path.display().to_string(),
            },
        );
        return Ok(());
    }

    output.success(&format!("Removed virtual environment '{name}'"));

    Ok(())
}
