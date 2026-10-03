//! Check for a dangling project `.venv` symlink in the current directory.
//!
//! `scuv use --link` makes `.venv` point at an env; removing that env from
//! another directory leaves the link dangling, and uv then fails in that
//! project with `File exists (os error 17)` (#202).

use std::path::{Path, PathBuf};

use crate::paths;

use super::super::types::{Check, CheckResult};

const ID: &str = "venv_link";
const NAME: &str = "project .venv link";

/// Check for a dangling `.venv` symlink in the current directory.
pub(super) struct VenvLinkCheck;

/// Resolves `link`'s target the way the kernel does: relative to the link's
/// directory. `None` when `link` is not a symlink.
fn link_target(link: &Path) -> Option<PathBuf> {
    Some(link.parent()?.join(std::fs::read_link(link).ok()?))
}

/// Whether `dest` names an entry directly inside `venvs_dir`, i.e. a link
/// `scuv use --link` made. The env may be gone but its parent still exists,
/// so the parent is compared canonically: that accepts an alias of
/// `venvs_dir` and rejects a target that only looks inside it lexically
/// (`virtualenvs/../../elsewhere/x`).
fn is_in_venvs_dir(dest: &Path, venvs_dir: &Path) -> bool {
    match (
        dest.parent().map(std::fs::canonicalize),
        std::fs::canonicalize(venvs_dir),
    ) {
        (Some(Ok(parent)), Ok(dir)) => parent == dir,
        _ => false,
    }
}

/// Classifies `link`; `None` when it is not a symlink (absent or a real venv).
///
/// A dangling link to a removed scuv env is an error that `--fix` clears. A
/// dangling link elsewhere is not scuv's to fix, so it is only a warning:
/// otherwise doctor would exit 2 in any project with a stale link of its own.
fn classify(link: &Path, venvs_dir: Option<&Path>) -> Option<CheckResult> {
    let dest = link_target(link)?;
    if link.exists() {
        return Some(CheckResult::ok(ID, NAME).with_details(format!("-> {}", dest.display())));
    }
    let message = format!(".venv points to missing '{}'", dest.display());
    Some(
        if venvs_dir.is_some_and(|dir| is_in_venvs_dir(&dest, dir)) {
            CheckResult::error(ID, NAME, message)
                .with_suggestion("Run: scuv doctor --fix, or scuv use <env> --link")
        } else {
            CheckResult::warn(ID, NAME, message).with_suggestion("Run: rm .venv")
        },
    )
}

/// Removes `link` when it dangles to an entry directly inside `venvs_dir`; a
/// link scuv did not make is left for the user.
fn remove_if_dangling_into(link: &Path, venvs_dir: &Path) -> Option<CheckResult> {
    let dest = link_target(link)?;
    if link.exists() || !is_in_venvs_dir(&dest, venvs_dir) {
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
            .and_then(|cwd| {
                let venvs_dir = paths::virtualenvs_dir().ok();
                classify(&cwd.join(".venv"), venvs_dir.as_deref())
            })
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
        assert!(
            classify(&link, None).is_none(),
            "absent .venv is not a finding"
        );
        std::fs::create_dir(&link).unwrap();
        assert!(
            classify(&link, None).is_none(),
            "a real .venv is not a finding"
        );
    }

    /// Fails if `classify` returns `None` or a default result for a link.
    #[test]
    fn classify_ok_for_live_link_error_for_dangling() {
        let tmp = tempfile::tempdir().unwrap();
        let venvs = tmp.path().join("virtualenvs");
        let env = venvs.join("env");
        std::fs::create_dir_all(&env).unwrap();
        let link = tmp.path().join(".venv");
        symlink(&env, &link).unwrap();
        assert!(classify(&link, Some(&venvs)).unwrap().is_ok());

        std::fs::remove_dir(&env).unwrap();
        let result = classify(&link, Some(&venvs)).unwrap();
        assert!(
            result.is_error(),
            "dangling link to a scuv env must be an error: {result:#?}"
        );
    }

    /// A stale link of the user's own must not fail `doctor` (exit 2) with an
    /// error `--fix` cannot clear. Fails if the venvs-dir test is dropped.
    #[test]
    fn classify_warns_for_dangling_link_outside_venvs_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let venvs = tmp.path().join("virtualenvs");
        std::fs::create_dir_all(&venvs).unwrap();
        let link = tmp.path().join(".venv");
        symlink(tmp.path().join("poetry-env"), &link).unwrap();

        let result = classify(&link, Some(&venvs)).unwrap();
        assert!(
            result.is_warning(),
            "foreign link must only warn: {result:#?}"
        );
        let result = classify(&link, None).unwrap();
        assert!(result.is_warning(), "no venvs dir: nothing is ours");
    }

    /// Fails if the live-link guard or the parent comparison is dropped.
    #[test]
    fn remove_if_dangling_into_removes_only_scuv_links() {
        let tmp = tempfile::tempdir().unwrap();
        let venvs = tmp.path().join("virtualenvs");
        std::fs::create_dir_all(venvs.join("live")).unwrap();
        let link = tmp.path().join(".venv");

        // Dangling, but outside venvs_dir: not ours.
        symlink(tmp.path().join("elsewhere"), &link).unwrap();
        assert!(remove_if_dangling_into(&link, &venvs).is_none());
        assert!(link.is_symlink());
        std::fs::remove_file(&link).unwrap();

        // Inside venvs_dir but still alive: nothing to fix.
        symlink(venvs.join("live"), &link).unwrap();
        assert!(remove_if_dangling_into(&link, &venvs).is_none());
        assert!(link.is_symlink());
        std::fs::remove_file(&link).unwrap();

        // Dangling into venvs_dir: removed.
        symlink(venvs.join("gone"), &link).unwrap();
        assert!(remove_if_dangling_into(&link, &venvs).unwrap().is_ok());
        assert!(!link.is_symlink());
    }

    /// `starts_with` is lexical: `virtualenvs/../outside/x` passes it while
    /// resolving outside. Fails if the guard goes back to a prefix match.
    #[test]
    fn remove_if_dangling_into_rejects_dotdot_escape() {
        let tmp = tempfile::tempdir().unwrap();
        let venvs = tmp.path().join("virtualenvs");
        std::fs::create_dir_all(&venvs).unwrap();
        std::fs::create_dir_all(tmp.path().join("outside")).unwrap();
        let link = tmp.path().join(".venv");
        symlink(venvs.join("../outside/missing"), &link).unwrap();

        assert!(remove_if_dangling_into(&link, &venvs).is_none());
        assert!(link.is_symlink());
    }

    /// The link may name venvs_dir through an alias (a symlinked SCUV_HOME).
    /// Fails if the parent comparison is lexical.
    #[test]
    fn remove_if_dangling_into_accepts_alias_of_venvs_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let venvs = tmp.path().join("home").join("virtualenvs");
        std::fs::create_dir_all(&venvs).unwrap();
        symlink(tmp.path().join("home"), tmp.path().join("alias")).unwrap();
        let link = tmp.path().join(".venv");
        symlink(tmp.path().join("alias/virtualenvs/gone"), &link).unwrap();

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
            std::fs::create_dir_all(temp.path().join("virtualenvs")).unwrap();
            symlink(temp.path().join("virtualenvs").join("gone"), &link).unwrap();

            let results = VenvLinkCheck.run();
            assert_eq!(results.len(), 1);
            assert!(results[0].is_error());

            let fixed = VenvLinkCheck
                .fix(
                    &results[0],
                    &Output::new(0, true, crate::output::Colors::NONE, false),
                )
                .expect("a dangling scuv link is fixable");
            assert!(fixed.is_ok());
            assert!(!link.is_symlink());
            assert!(VenvLinkCheck.run().is_empty());
        });
    }
}
