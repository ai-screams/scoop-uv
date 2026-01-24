//! Handle "system" special value for use command

use std::path::Path;

use rust_i18n::t;

use crate::core::VersionService;
use crate::error::Result;
use crate::output::{Output, UseData};

use super::output::output_result;

/// Handle `scoop use system` (use system Python)
pub fn handle(output: &Output, cwd: &Path, global: bool) -> Result<()> {
    if global {
        VersionService::set_global("system")?;
        output_result(
            output,
            UseData {
                name: "system".to_string(),
                mode: "global",
                version_file: None,
                symlink: None,
            },
            &t!("use.system_global"),
        )
    } else {
        VersionService::set_local(cwd, "system")?;
        output_result(
            output,
            UseData {
                name: "system".to_string(),
                mode: "local",
                version_file: Some(cwd.join(".scoop-version").display().to_string()),
                symlink: None,
            },
            &t!("use.system_local"),
        )
    }
}
