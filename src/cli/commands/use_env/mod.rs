//! Use command - set local/global environment
//!
//! This module handles the `scoop use` command which:
//! - Sets environment version files (.scoop-version)
//! - Handles the "system" special value
//! - Supports --unset to remove version files
//! - Optionally creates .venv symlinks

mod normal;
mod output;
mod symlink;
mod system;
mod unset;

use rust_i18n::t;

use crate::error::{Result, ScoopError};
use crate::output::Output;

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
        return unset::handle(output, &cwd, global);
    }

    // Name is required when not using --unset
    let name = name.ok_or_else(|| ScoopError::InvalidArgument {
        message: t!("error.use_missing_name").to_string(),
    })?;

    // Handle "system" special value (case-insensitive)
    if name.eq_ignore_ascii_case("system") {
        return system::handle(output, &cwd, global);
    }

    // Handle normal environment
    normal::handle(output, &cwd, name, global, link)
}
