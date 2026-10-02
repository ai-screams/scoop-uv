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

/// Remove `link` if it is a symlink pointing at `target`.
///
/// Returns whether the link was removed. A missing path, a real directory, or
/// a symlink to anything else is left alone: only the link `scuv use --link`
/// made for this env is ours to clean up.
pub fn remove_venv_symlink_to(link: &Path, target: &Path) -> Result<bool> {
    match fs::read_link(link) {
        Ok(dest) if dest == target => {
            fs::remove_file(link)?;
            Ok(true)
        }
        _ => Ok(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Fails if the helper never removes anything.
    #[test]
    fn remove_venv_symlink_to_removes_link_to_target() {
        let tmp = tempfile::tempdir().unwrap();
        let target = tmp.path().join("env");
        let link = tmp.path().join(".venv");
        symlink(&target, &link).unwrap();

        assert!(remove_venv_symlink_to(&link, &target).unwrap());
        assert!(!link.is_symlink());
    }

    /// Fails if the `dest == target` guard is dropped.
    #[test]
    fn remove_venv_symlink_to_keeps_link_to_other_target() {
        let tmp = tempfile::tempdir().unwrap();
        let link = tmp.path().join(".venv");
        symlink(tmp.path().join("other"), &link).unwrap();

        assert!(!remove_venv_symlink_to(&link, &tmp.path().join("env")).unwrap());
        assert!(link.is_symlink());
    }

    /// Fails if the decision uses `exists()` instead of `read_link`.
    #[test]
    fn remove_venv_symlink_to_keeps_real_directory() {
        let tmp = tempfile::tempdir().unwrap();
        let link = tmp.path().join(".venv");
        fs::create_dir(&link).unwrap();

        assert!(!remove_venv_symlink_to(&link, &link).unwrap());
        assert!(link.is_dir());
    }

    /// Fails if a `read_link` error is propagated instead of meaning "not ours".
    #[test]
    fn remove_venv_symlink_to_ignores_missing_path() {
        let tmp = tempfile::tempdir().unwrap();
        let link = tmp.path().join(".venv");

        assert!(!remove_venv_symlink_to(&link, &tmp.path().join("env")).unwrap());
    }
}
