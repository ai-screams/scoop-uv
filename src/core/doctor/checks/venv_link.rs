//! Check for a dangling project `.venv` symlink in the current directory.
//!
//! `scuv use --link` makes `.venv` point at an env; removing that env from
//! another directory leaves the link dangling, and uv then fails in that
//! project with `File exists (os error 17)` (#202).

use std::path::Path;

use crate::paths;

use super::super::types::{Check, CheckResult};

const ID: &str = "venv_link";
const NAME: &str = "project .venv link";

/// Check for a dangling `.venv` symlink in the current directory.
pub(super) struct VenvLinkCheck;

/// Classifies `link`; `None` when it is not a symlink (absent or a real venv).
fn classify(link: &Path) -> Option<CheckResult> {
    let dest = std::fs::read_link(link).ok()?;
    if link.exists() {
        return Some(CheckResult::ok(ID, NAME).with_details(format!("-> {}", dest.display())));
    }
    Some(
        CheckResult::error(
            ID,
            NAME,
            format!(".venv points to missing '{}'", dest.display()),
        )
        .with_suggestion("Run: rm .venv, or scuv use <env> --link"),
    )
}

/// Removes `link` when it dangles into `venvs_dir`; a link scuv did not make
/// (target outside `venvs_dir`) is left for the user.
fn remove_if_dangling_into(link: &Path, venvs_dir: &Path) -> Option<CheckResult> {
    let dest = std::fs::read_link(link).ok()?;
    if link.exists() || !dest.starts_with(venvs_dir) {
        return None;
    }
    Some(match std::fs::remove_file(link) {
        Ok(()) => CheckResult::ok(ID, NAME).with_details("removed dangling .venv link"),
        Err(e) => CheckResult::error(ID, NAME, format!("failed to remove .venv link: {e}"))
            .with_suggestion("Check file permissions"),
    })
}

impl Check for VenvLinkCheck {
    fn id(&self) -> &'static str {
        ID
    }

    fn name(&self) -> &'static str {
        NAME
    }

    fn run(&self) -> Vec<CheckResult> {
        std::env::current_dir()
            .ok()
            .and_then(|cwd| classify(&cwd.join(".venv")))
            .into_iter()
            .collect()
    }

    fn fix(&self, _result: &CheckResult, output: &crate::output::Output) -> Option<CheckResult> {
        let link = std::env::current_dir().ok()?.join(".venv");
        let venvs_dir = paths::virtualenvs_dir().ok()?;
        let fixed = remove_if_dangling_into(&link, &venvs_dir)?;
        if fixed.is_ok() {
            output.success("Removed dangling .venv link");
        }
        Some(fixed)
    }
}

#[cfg(all(test, unix))]
mod tests {
    use std::os::unix::fs::symlink;

    use super::*;
    use crate::core::doctor::checks::test_support::TempDirCwdGuard;
    use crate::output::Output;
    use crate::test_utils::with_temp_scoop_home;
    use serial_test::serial;

    /// Fails if `id`/`name` return anything else.
    #[test]
    fn venv_link_check_reports_identity() {
        assert_eq!(VenvLinkCheck.id(), "venv_link");
        assert_eq!(VenvLinkCheck.name(), "project .venv link");
    }

    /// Fails if `classify` reports a finding for an absent or real `.venv`.
    #[test]
    fn classify_skips_non_symlinks() {
        let tmp = tempfile::tempdir().unwrap();
        let link = tmp.path().join(".venv");
        assert!(classify(&link).is_none(), "absent .venv is not a finding");
        std::fs::create_dir(&link).unwrap();
        assert!(classify(&link).is_none(), "a real .venv is not a finding");
    }

    /// Fails if `classify` returns `None` or a default result for a link.
    #[test]
    fn classify_ok_for_live_link_error_for_dangling() {
        let tmp = tempfile::tempdir().unwrap();
        let env = tmp.path().join("env");
        std::fs::create_dir(&env).unwrap();
        let link = tmp.path().join(".venv");
        symlink(&env, &link).unwrap();
        assert!(classify(&link).unwrap().is_ok());

        std::fs::remove_dir(&env).unwrap();
        let result = classify(&link).unwrap();
        assert!(
            result.is_error(),
            "dangling link must be an error: {result:#?}"
        );
    }

    /// Fails if `||` becomes `&&` or the `!` before `starts_with` is deleted.
    #[test]
    fn remove_if_dangling_into_removes_only_scuv_links() {
        let tmp = tempfile::tempdir().unwrap();
        let venvs = tmp.path().join("virtualenvs");
        let link = tmp.path().join(".venv");

        // Dangling, but outside venvs_dir: not ours.
        symlink(tmp.path().join("elsewhere"), &link).unwrap();
        assert!(remove_if_dangling_into(&link, &venvs).is_none());
        assert!(link.is_symlink());
        std::fs::remove_file(&link).unwrap();

        // Inside venvs_dir but still alive: nothing to fix.
        std::fs::create_dir_all(venvs.join("live")).unwrap();
        symlink(venvs.join("live"), &link).unwrap();
        assert!(remove_if_dangling_into(&link, &venvs).is_none());
        assert!(link.is_symlink());
        std::fs::remove_file(&link).unwrap();

        // Dangling into venvs_dir: removed.
        symlink(venvs.join("gone"), &link).unwrap();
        assert!(remove_if_dangling_into(&link, &venvs).unwrap().is_ok());
        assert!(!link.is_symlink());
    }

    /// Dispatch through the trait: `run` sees the cwd link, `fix` removes it.
    /// Fails if `run` or `fix` is replaced by an empty or default result.
    #[test]
    #[serial]
    fn venv_link_check_run_and_fix_in_cwd() {
        with_temp_scoop_home(|temp| {
            let cwd = TempDirCwdGuard::new();
            let link = cwd.path().join(".venv");
            symlink(temp.path().join("virtualenvs").join("gone"), &link).unwrap();

            let results = VenvLinkCheck.run();
            assert_eq!(results.len(), 1);
            assert!(results[0].is_error());

            let fixed = VenvLinkCheck
                .fix(&results[0], &Output::new(0, true, true, false))
                .expect("a dangling scuv link is fixable");
            assert!(fixed.is_ok());
            assert!(!link.is_symlink());
            assert!(VenvLinkCheck.run().is_empty());
        });
    }
}
