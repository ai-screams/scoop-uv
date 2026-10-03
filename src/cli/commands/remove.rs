//! Remove command

use std::path::{Path, PathBuf};

use dialoguer::Confirm;
use rust_i18n::t;

use super::use_env::is_venv_symlink_to;
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

        let confirmed = Confirm::new()
            .with_prompt(t!("remove.confirm", name = name).to_string())
            .default(false)
            .interact()
            // No terminal to ask on (a script without --force) is an error,
            // not a "no": it used to print "Cancelled" and exit 0.
            .map_err(|e| crate::error::ScoopError::Io(std::io::Error::other(e)))?;

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
