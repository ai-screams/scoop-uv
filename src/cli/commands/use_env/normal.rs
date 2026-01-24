//! Handle normal environment activation for use command

use std::path::Path;

use rust_i18n::t;

use crate::core::{VersionService, VirtualenvService};
use crate::error::Result;
use crate::output::{Output, UseData};

use super::output::output_result;
use super::symlink::create_venv_symlink;

/// Handle `scoop use <name>` (normal environment)
pub fn handle(output: &Output, cwd: &Path, name: &str, global: bool, link: bool) -> Result<()> {
    let service = VirtualenvService::auto()?;

    // Verify environment exists
    let venv_path = service.get_path(name)?;

    if global {
        VersionService::set_global(name)?;
        output_result(
            output,
            UseData {
                name: name.to_string(),
                mode: "global",
                version_file: None,
                symlink: None,
            },
            &t!("use.set_global", name = name),
        )
    } else {
        VersionService::set_local(cwd, name)?;

        let mut symlink_path = None;

        // Create .venv symlink only if --link flag is provided
        if link {
            let venv_link = cwd.join(".venv");
            create_venv_symlink(&venv_link, &venv_path, output)?;
            symlink_path = Some(venv_link.display().to_string());
        }

        output_result(
            output,
            UseData {
                name: name.to_string(),
                mode: "local",
                version_file: Some(cwd.join(".scoop-version").display().to_string()),
                symlink: symlink_path,
            },
            &t!("use.set_local", name = name),
        )
    }
}
