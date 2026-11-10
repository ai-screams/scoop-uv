//! Remove command

use std::path::{Path, PathBuf};

use rust_i18n::t;

use super::use_env::is_venv_symlink_to;
use crate::cli::confirm::Confirmation;
use crate::core::VirtualenvService;
use crate::error::Result;
use crate::output::{Output, RemoveData};

/// Execute the remove command
pub fn execute(output: &Output, name: &str, force: bool) -> Result<()> {
    let service = VirtualenvService::auto()?;

    // Verify environment exists
    let path = service.get_path(name)?;

    // JSON mode always implies force (no interactive confirmation)
    if !force && !output.is_json() {
        // Show what will be deleted
        output.info(&t!(
            "remove.path",
            path = crate::paths::abbreviate_home(&path)
        ));

        // No terminal to ask on (a script without --force) is an error, not
        // a "no": it used to print "Cancelled" and exit 0.
        let confirmed = match crate::cli::confirm::confirm(&t!("remove.confirm", name = name)) {
            Confirmation::Yes => true,
            Confirmation::No => false,
            Confirmation::Unavailable => {
                return Err(crate::error::ScoopError::ConfirmationUnavailable {
                    flag: "--force".to_string(),
                });
            }
        };

        if !confirmed {
            output.info(&t!("remove.cancelled"));
            return Ok(());
        }
    }

    // Decided before the delete, while the link still resolves (#202).
    let link = linked_venv(&path);

    output.info(&t!("remove.removing", name = name));
    service.delete(name)?;

    let (unlinked, unlink_error) = match link {
        Some(link) => unlink_venv(output, link),
        None => (None, None),
    };

    // JSON output
    if output.is_json() {
        output.json_success(
            "remove",
            RemoveData {
                name: name.to_string(),
                path: path.display().to_string(),
                unlinked: unlinked.map(|l| l.display().to_string()),
                unlink_error,
            },
        );
        return Ok(());
    }

    output.success(&t!("remove.success", name = name));
    if let Some(link) = unlinked {
        output.info(&t!(
            "remove.unlinked",
            path = crate::paths::abbreviate_home(&link)
        ));
    }

    Ok(())
}

/// A `.venv` link that `remove` will be responsible for.
struct VenvLink {
    path: PathBuf,
    /// Its raw target when judged, or why it could not be read back.
    recorded: std::io::Result<PathBuf>,
}

/// The current directory's `.venv`, if it is the link `scuv use --link`
/// made to `env_path`.
///
/// Such a link would dangle once the env is gone, and uv fails on it in
/// that project (#202). Only the current directory is checked: link
/// locations are not recorded anywhere. The raw target is recorded too:
/// the env deletion in between can take a while, and the unlink must only
/// touch the same link it judged.
fn linked_venv(env_path: &Path) -> Option<VenvLink> {
    let path = std::env::current_dir().ok()?.join(".venv");
    if !is_venv_symlink_to(&path, env_path) {
        return None;
    }
    match std::fs::read_link(&path) {
        Ok(target) => Some(VenvLink {
            path,
            recorded: Ok(target),
        }),
        // Gone or no longer a symlink since it was judged: not ours.
        Err(e)
            if matches!(
                e.kind(),
                std::io::ErrorKind::NotFound | std::io::ErrorKind::InvalidInput
            ) =>
        {
            None
        }
        Err(e) => Some(VenvLink {
            path,
            recorded: Err(e),
        }),
    }
}

/// Removes the judged link: `(Some(path), None)` when removed, `(None,
/// Some(error))` when it could not be.
///
/// The env is already gone, so a link we cannot remove (or could not even
/// read back) is a warning, not a failed `remove`. JSON carries it as
/// `unlink_error`: `warn` is silent there, and a missing `unlinked` alone
/// would read as "there was no link".
fn unlink_venv(output: &Output, link: VenvLink) -> (Option<PathBuf>, Option<String>) {
    let result = link
        .recorded
        .and_then(|recorded| crate::paths::remove_symlink_if_unchanged(&link.path, &recorded));
    match result {
        Ok(true) => (Some(link.path), None),
        Ok(false) => (None, None),
        Err(e) => {
            output.warn(&t!(
                "remove.unlink_failed",
                path = crate::paths::abbreviate_home(&link.path),
                error = e.to_string()
            ));
            (None, Some(e.to_string()))
        }
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use crate::output::Colors;
    use crate::test_utils::{FakeUv, env_guard};
    use serial_test::serial;

    /// With a fake uv on PATH and a fresh SCUV_HOME holding env `e`.
    fn with_env(f: impl FnOnce(&std::path::Path)) {
        let uv = FakeUv::new(&[]);
        let home = tempfile::tempdir().unwrap();
        let path = uv.path_var();
        let _env = env_guard(&[
            ("PATH", Some(path.as_str())),
            ("SCUV_HOME", Some(home.path().to_str().unwrap())),
        ]);
        let env = home.path().join("virtualenvs").join("e");
        std::fs::create_dir_all(&env).unwrap();
        f(&env);
    }

    /// Yes removes, no keeps (and succeeds), a question that could not be
    /// asked keeps and fails naming `--force`. What fails this: any answer
    /// other than yes removing the env.
    #[test]
    #[serial]
    fn remove_acts_only_on_yes() {
        let out = Output::new(0, true, Colors::NONE, false);
        with_env(|env| {
            crate::cli::confirm::answering(Some(false), || execute(&out, "e", false)).unwrap();
            assert!(env.exists());
            let err =
                crate::cli::confirm::answering(None, || execute(&out, "e", false)).unwrap_err();
            assert!(
                matches!(err, crate::error::ScoopError::ConfirmationUnavailable { ref flag } if flag == "--force"),
                "{err:?}"
            );
            assert!(env.exists());
            crate::cli::confirm::answering(Some(true), || execute(&out, "e", false)).unwrap();
            assert!(!env.exists());
        });
    }
}
