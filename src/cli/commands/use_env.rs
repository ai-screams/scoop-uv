//! Use command

use std::fs;
use std::os::unix::fs::symlink;
use std::path::Path;

use crate::core::{VersionService, VirtualenvService};
use crate::error::Result;
use crate::output::{Output, UseData};

/// Execute the use command
pub fn execute(output: &Output, name: &str, global: bool, link: bool) -> Result<()> {
    let service = VirtualenvService::auto()?;

    // Verify environment exists
    let venv_path = service.get_path(name)?;

    if global {
        VersionService::set_global(name)?;

        // JSON output
        if output.is_json() {
            output.json_success(
                "use",
                UseData {
                    name: name.to_string(),
                    mode: "global",
                    version_file: None,
                    symlink: None,
                },
            );
            return Ok(());
        }

        output.success(&format!("Set '{name}' as global environment"));
    } else {
        let cwd = std::env::current_dir()?;

        // Set local version
        VersionService::set_local(&cwd, name)?;

        let mut symlink_path = None;

        // Create .venv symlink only if --link flag is provided
        if link {
            let venv_link = cwd.join(".venv");
            create_venv_symlink(&venv_link, &venv_path, output)?;
            symlink_path = Some(venv_link.display().to_string());
        }

        // JSON output
        if output.is_json() {
            output.json_success(
                "use",
                UseData {
                    name: name.to_string(),
                    mode: "local",
                    version_file: Some(cwd.join(".scoop-version").display().to_string()),
                    symlink: symlink_path,
                },
            );
            return Ok(());
        }

        output.success(&format!("Set '{name}' as local environment"));
    }

    Ok(())
}

/// Create or update .venv symlink
fn create_venv_symlink(link: &Path, target: &Path, output: &Output) -> Result<()> {
    if link.exists() || link.is_symlink() {
        if link.is_symlink() {
            fs::remove_file(link)?;
        } else {
            output.warn(".venv exists but isn't a symlink, skipping");
            return Ok(());
        }
    }

    symlink(target, link)?;
    output.info(&format!(
        "Linked .venv -> {}",
        crate::paths::abbreviate_home(target)
    ));

    Ok(())
}
