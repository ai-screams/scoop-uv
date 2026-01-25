//! Symlink creation for use command

use std::fs;
use std::os::unix::fs::symlink;
use std::path::Path;

use rust_i18n::t;

use crate::error::Result;
use crate::output::Output;

/// Create or update .venv symlink
pub fn create_venv_symlink(link: &Path, target: &Path, output: &Output) -> Result<()> {
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
