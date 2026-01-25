//! Handle --unset flag for use command

use std::path::Path;

use rust_i18n::t;

use crate::core::VersionService;
use crate::error::Result;
use crate::output::{Output, UseData};

use super::output::output_result;

/// Handle `scoop use --unset` (remove version file)
pub fn handle(output: &Output, cwd: &Path, global: bool) -> Result<()> {
    if global {
        VersionService::unset_global()?;
        output_result(
            output,
            UseData {
                name: String::new(),
                mode: "global_unset",
                version_file: None,
                symlink: None,
            },
            &t!("use.global_unset"),
        )
    } else {
        VersionService::unset_local(cwd)?;
        output_result(
            output,
            UseData {
                name: String::new(),
                mode: "local_unset",
                version_file: None,
                symlink: None,
            },
            &t!("use.local_unset"),
        )
    }
}
