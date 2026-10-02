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

/// Returns whether `link` is a symlink that resolves to `target`.
///
/// Both sides are canonicalized, so the same directory reached through an
/// alias (a symlinked `SCUV_HOME`, `.` or `..` in the stored path) still
/// matches. Call it while `target` exists: a dangling link resolves to
/// nothing. A missing path or a real directory is never a match, so only the
/// link `scuv use --link` made for this env is treated as ours.
pub fn is_venv_symlink_to(link: &Path, target: &Path) -> bool {
    link.is_symlink()
        && matches!(
            (fs::canonicalize(link), fs::canonicalize(target)),
            (Ok(a), Ok(b)) if a == b
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Fails if the helper never matches.
    #[test]
    fn is_venv_symlink_to_matches_link_to_target() {
        let tmp = tempfile::tempdir().unwrap();
        let target = tmp.path().join("env");
        fs::create_dir(&target).unwrap();
        let link = tmp.path().join(".venv");
        symlink(&target, &link).unwrap();

        assert!(is_venv_symlink_to(&link, &target));
    }

    /// The link and the env path can spell the same directory differently.
    /// Fails if the comparison is lexical (`read_link(link) == target`).
    #[test]
    fn is_venv_symlink_to_matches_through_alias_and_dotdot() {
        let tmp = tempfile::tempdir().unwrap();
        let target = tmp.path().join("home").join("env");
        fs::create_dir_all(&target).unwrap();
        symlink(tmp.path().join("home"), tmp.path().join("alias")).unwrap();

        let via_alias = tmp.path().join("a.venv");
        symlink(tmp.path().join("alias").join("env"), &via_alias).unwrap();
        assert!(is_venv_symlink_to(&via_alias, &target));

        let via_dotdot = tmp.path().join("b.venv");
        symlink(tmp.path().join("home/x/../env"), &via_dotdot).unwrap();
        fs::create_dir(tmp.path().join("home/x")).unwrap();
        assert!(is_venv_symlink_to(&via_dotdot, &target));
    }

    /// Fails if the target comparison is dropped.
    #[test]
    fn is_venv_symlink_to_rejects_link_to_other_target() {
        let tmp = tempfile::tempdir().unwrap();
        let (env, other) = (tmp.path().join("env"), tmp.path().join("other"));
        fs::create_dir(&env).unwrap();
        fs::create_dir(&other).unwrap();
        let link = tmp.path().join(".venv");
        symlink(&other, &link).unwrap();

        assert!(!is_venv_symlink_to(&link, &env));
    }

    /// A real `.venv` resolves to itself; it must not count as a link.
    /// Fails if the `is_symlink()` guard is dropped.
    #[test]
    fn is_venv_symlink_to_rejects_real_directory() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join(".venv");
        fs::create_dir(&dir).unwrap();

        assert!(!is_venv_symlink_to(&dir, &dir));
    }
}
