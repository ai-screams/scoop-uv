//! Create command

use crate::core::VirtualenvService;
use crate::error::Result;
use crate::output::Output;
use crate::paths;

/// Execute the create command
pub fn execute(output: &Output, name: &str, python: &str, force: bool) -> Result<()> {
    let service = VirtualenvService::auto()?;

    // Check if exists and handle force
    if service.exists(name)? {
        if force {
            output.info(&format!("Removing existing environment '{name}'..."));
            service.delete(name)?;
        } else {
            return Err(crate::error::UvenvError::VirtualenvExists {
                name: name.to_string(),
            });
        }
    }

    output.info(&format!(
        "Creating virtual environment '{name}' with Python {python}..."
    ));

    let path = service.create(name, python)?;

    output.success(&format!("Created virtual environment '{name}'"));
    output.info(&format!("Location: {}", path.display()));
    output.info(&format!(
        "Activate with: uvenv use {name}  # or: source {}/bin/activate",
        paths::virtualenv_path(name)?.display()
    ));

    Ok(())
}
