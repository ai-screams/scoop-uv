//! Use command

use std::fs;
use std::os::unix::fs::symlink;
use std::path::Path;

use rust_i18n::t;

use crate::core::{VersionService, VirtualenvService};
use crate::error::Result;
use crate::output::{Output, UseData};

/// Execute the use command
pub fn execute(
    output: &Output,
    name: Option<&str>,
    unset: bool,
    global: bool,
    link: bool,
) -> Result<()> {
    let cwd = std::env::current_dir()?;

    // Handle --unset flag
    if unset {
        if global {
            VersionService::unset_global()?;

            if output.is_json() {
                output.json_success(
                    "use",
                    UseData {
                        name: String::new(),
                        mode: "global_unset",
                        version_file: None,
                        symlink: None,
                    },
                );
                return Ok(());
            }

            output.success(&t!("use.global_unset"));
        } else {
            VersionService::unset_local(&cwd)?;

            if output.is_json() {
                output.json_success(
                    "use",
                    UseData {
                        name: String::new(),
                        mode: "local_unset",
                        version_file: None,
                        symlink: None,
                    },
                );
                return Ok(());
            }

            output.success(&t!("use.local_unset"));
        }
        return Ok(());
    }

    // Name is required when not using --unset
    let name = name.ok_or_else(|| crate::error::ScoopError::InvalidArgument {
        message: t!("error.use_missing_name").to_string(),
    })?;

    // Handle "system" special value
    if name.to_lowercase() == "system" {
        if global {
            VersionService::set_global("system")?;

            if output.is_json() {
                output.json_success(
                    "use",
                    UseData {
                        name: "system".to_string(),
                        mode: "global",
                        version_file: None,
                        symlink: None,
                    },
                );
                return Ok(());
            }

            output.success(&t!("use.system_global"));
        } else {
            VersionService::set_local(&cwd, "system")?;

            if output.is_json() {
                output.json_success(
                    "use",
                    UseData {
                        name: "system".to_string(),
                        mode: "local",
                        version_file: Some(cwd.join(".scoop-version").display().to_string()),
                        symlink: None,
                    },
                );
                return Ok(());
            }

            output.success(&t!("use.system_local"));
        }
        return Ok(());
    }

    // Normal environment handling
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

        output.success(&t!("use.set_global", name = name));
    } else {
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

        output.success(&t!("use.set_local", name = name));
    }

    Ok(())
}

/// Create or update .venv symlink
fn create_venv_symlink(link: &Path, target: &Path, output: &Output) -> Result<()> {
    if link.exists() || link.is_symlink() {
        if link.is_symlink() {
            fs::remove_file(link)?;
        } else {
            output.warn(&t!("use.venv_not_symlink"));
            return Ok(());
        }
    }

    symlink(target, link)?;
    output.info(&t!(
        "use.linked",
        path = crate::paths::abbreviate_home(target)
    ));

    Ok(())
}
