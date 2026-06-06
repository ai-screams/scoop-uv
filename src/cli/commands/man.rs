//! Handler for the `scoop man` command.
//!
//! Generates groff-formatted man pages from the live `clap::Command` tree,
//! so the man pages always reflect the actual `--help` text. Two modes:
//!
//! - No arg: render the top-level `scoop(1)` to stdout (pipe to `man -l -`).
//! - `<dir>`: render `scoop.1` + one `scoop-<sub>.1` per subcommand into `<dir>`.
//!
//! Distro packagers consume the directory form; `man -l` is the convenient
//! local-preview path.

use std::io::Write;
use std::path::Path;

use clap::CommandFactory;
use rust_i18n::t;

use crate::cli::Cli;
use crate::error::{Result, ScoopError};
use crate::output::Output;

/// Execute the `man` command.
pub fn execute(output: &Output, output_dir: Option<&Path>) -> Result<()> {
    let cmd = Cli::command();

    match output_dir {
        None => render_stdout(&cmd),
        Some(dir) => render_to_dir(output, &cmd, dir),
    }
}

fn render_stdout(cmd: &clap::Command) -> Result<()> {
    let man = clap_mangen::Man::new(cmd.clone());
    let mut buf: Vec<u8> = Vec::new();
    man.render(&mut buf)
        .map_err(|e| ScoopError::InvalidArgument {
            message: format!("failed to render man page: {e}"),
        })?;
    std::io::stdout().write_all(&buf)?;
    Ok(())
}

fn render_to_dir(output: &Output, cmd: &clap::Command, dir: &Path) -> Result<()> {
    // Refuse to write into a directory that's actually a symlink. Packager
    // scripts that take an attacker-influenced `--output-dir` (e.g. via a
    // tainted env var while running as root) would otherwise follow the
    // symlink and silently write into the target. We use
    // `symlink_metadata` so we inspect the link itself, not its target.
    if dir.exists() {
        let meta = std::fs::symlink_metadata(dir)?;
        if meta.file_type().is_symlink() {
            return Err(ScoopError::InvalidArgument {
                message: t!("man.refuse_symlink", path = dir.display()).to_string(),
            });
        }
    }

    std::fs::create_dir_all(dir)?;

    // Top-level `scoop.1`
    write_page(dir, "scoop.1", cmd)?;
    let mut count = 1;

    // One file per immediate subcommand. Hidden subcommands (`activate`,
    // `deactivate`, `resolve`) carry `Hide=true` so we skip them — they're
    // implementation details of the shell wrapper, not user-facing.
    for sub in cmd.get_subcommands() {
        if sub.is_hide_set() {
            continue;
        }
        let filename = format!("scoop-{}.1", sub.get_name());
        write_page(dir, &filename, sub)?;
        count += 1;
    }

    if output.is_json() {
        output.json_success(
            "man",
            serde_json::json!({
                "directory": dir.display().to_string(),
                "pages": count,
            }),
        );
        return Ok(());
    }

    output.success(&t!(
        "man.written",
        dir = dir.display(),
        count = count.to_string()
    ));
    Ok(())
}

fn write_page(dir: &Path, filename: &str, cmd: &clap::Command) -> Result<()> {
    let target = dir.join(filename);

    // Reject symlinks at the file level too. The dir-level check upstream
    // refuses to descend into a symlinked DIR, but a packager script that
    // pre-creates `scoop.1` or `scoop-*.1` as a symlink to some other
    // path would otherwise have `fs::write` follow the link — an arbitrary
    // truncate/write under whatever UID is running `scoop man`. Use
    // `symlink_metadata` so we inspect the link itself, not its target.
    if let Ok(meta) = std::fs::symlink_metadata(&target) {
        if meta.file_type().is_symlink() {
            return Err(ScoopError::InvalidArgument {
                message: t!("man.refuse_symlink", path = target.display()).to_string(),
            });
        }
    }

    let man = clap_mangen::Man::new(cmd.clone());
    let mut buf: Vec<u8> = Vec::new();
    man.render(&mut buf)
        .map_err(|e| ScoopError::InvalidArgument {
            message: format!("failed to render {filename}: {e}"),
        })?;
    std::fs::write(&target, buf)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn render_to_dir_emits_top_level_and_subcommand_pages() {
        let tmp = TempDir::new().unwrap();
        let output = Output::new(0, true, true, false);
        execute(&output, Some(tmp.path())).unwrap();

        // Top-level page must exist
        assert!(
            tmp.path().join("scoop.1").exists(),
            "scoop.1 should be written"
        );

        // At least one well-known subcommand page must exist. We don't
        // hardcode all of them — this guards against the renderer breaking
        // wholesale, not against subcommand renames (which clap_mangen will
        // pick up automatically anyway).
        for sub in ["scoop-list.1", "scoop-create.1", "scoop-doctor.1"] {
            assert!(
                tmp.path().join(sub).exists(),
                "expected man page {sub} to be written"
            );
        }
    }

    /// Future-proof regression guard: every non-hidden subcommand in
    /// the clap tree must get its own man page. This is what catches
    /// "we added a new command but the man emission missed it"
    /// before reviewers do. The previous static list
    /// (list/create/doctor) caught the renderer crashing wholesale
    /// but not a per-command miss; this one does.
    #[test]
    fn render_to_dir_emits_page_for_every_non_hidden_subcommand() {
        let tmp = TempDir::new().unwrap();
        let output = Output::new(0, true, true, false);
        execute(&output, Some(tmp.path())).unwrap();

        let cmd = Cli::command();
        for sub in cmd.get_subcommands() {
            if sub.is_hide_set() {
                continue;
            }
            let filename = format!("scoop-{}.1", sub.get_name());
            assert!(
                tmp.path().join(&filename).exists(),
                "expected man page {filename} for subcommand '{}'",
                sub.get_name()
            );
        }
    }

    #[test]
    fn render_to_dir_skips_hidden_subcommands() {
        let tmp = TempDir::new().unwrap();
        let output = Output::new(0, true, true, false);
        execute(&output, Some(tmp.path())).unwrap();

        // `activate`, `deactivate`, `resolve` are hidden (shell-internal).
        for hidden in ["scoop-activate.1", "scoop-deactivate.1", "scoop-resolve.1"] {
            assert!(
                !tmp.path().join(hidden).exists(),
                "hidden subcommand {hidden} should not get a man page"
            );
        }
    }

    // ==========================================================================
    // C1 regression — file-level symlink hardening. A pre-existing
    // `scoop.1` symlink inside a non-symlinked DIR must NOT be followed
    // by fs::write; the dir-level check is not enough.
    // ==========================================================================
    #[cfg(unix)]
    #[test]
    fn render_to_dir_rejects_pre_existing_symlink_file() {
        let tmp = TempDir::new().unwrap();

        // Canary: target file that we don't want written through.
        let target_dir = TempDir::new().unwrap();
        let canary = target_dir.path().join("important.txt");
        std::fs::write(&canary, b"do not truncate").unwrap();

        // Plant a symlink inside the (non-symlinked) output dir pointing
        // at the canary. Then ask man to write. The symlink check should
        // fire and the canary content must stay intact.
        std::os::unix::fs::symlink(&canary, tmp.path().join("scoop.1")).unwrap();

        let output = Output::new(0, true, true, false);
        let result = execute(&output, Some(tmp.path()));
        assert!(result.is_err(), "must refuse to write through symlink");
        assert_eq!(
            std::fs::read(&canary).unwrap(),
            b"do not truncate",
            "canary content was modified — symlink was followed"
        );
    }
}
