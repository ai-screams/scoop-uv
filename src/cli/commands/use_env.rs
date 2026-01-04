//! Use command

use std::fs;
use std::os::unix::fs::symlink;
use std::path::Path;

use crate::core::{VersionService, VirtualenvService};
use crate::error::Result;
use crate::output::Output;

/// Execute the use command
pub fn execute(output: &Output, name: &str, global: bool, link: bool) -> Result<()> {
    let service = VirtualenvService::auto()?;

    // Verify environment exists
    let venv_path = service.get_path(name)?;

    if global {
        VersionService::set_global(name)?;
        output.success(&format!("Set global environment to '{name}'"));
    } else {
        let cwd = std::env::current_dir()?;

        // Set local version
        VersionService::set_local(&cwd, name)?;
        output.success(&format!("Set local environment to '{name}'"));

        // Create .venv symlink only if --link flag is provided
        if link {
            let venv_link = cwd.join(".venv");
            create_venv_symlink(&venv_link, &venv_path, output)?;
        }
    }

    Ok(())
}

/// Create or update .venv symlink
fn create_venv_symlink(link: &Path, target: &Path, output: &Output) -> Result<()> {
    if link.exists() || link.is_symlink() {
        if link.is_symlink() {
            fs::remove_file(link)?;
        } else {
            output.warn(".venv exists and is not a symlink, skipping");
            return Ok(());
        }
    }

    symlink(target, link)?;
    output.info(&format!("Created symlink: .venv -> {}", target.display()));

    Ok(())
}
