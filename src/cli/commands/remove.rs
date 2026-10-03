//! Remove command

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

    // A `.venv` that `scuv use --link` pointed at this env would dangle once
    // the env is gone, and uv fails on it in that project (#202). Decide now,
    // while the link still resolves. Only the current directory is checked:
    // link locations are not recorded anywhere.
    let link = std::env::current_dir()
        .ok()
        .map(|cwd| cwd.join(".venv"))
        .filter(|link| is_venv_symlink_to(link, &path));

    output.info(&t!("remove.removing", name = name));
    service.delete(name)?;

    // The env is already gone, so a link we cannot remove is a warning, not a
    // failed `remove`. JSON carries it as `unlink_error`: `warn` is silent
    // there, and a missing `unlinked` alone would read as "there was no link".
    let mut unlink_error = None;
    let unlinked = link.filter(|link| match std::fs::remove_file(link) {
        Ok(()) => true,
        Err(e) => {
            output.warn(&t!(
                "remove.unlink_failed",
                path = crate::paths::abbreviate_home(link),
                error = e.to_string()
            ));
            unlink_error = Some(e.to_string());
            false
        }
    });

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
