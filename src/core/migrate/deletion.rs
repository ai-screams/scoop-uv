//! What a migration may delete: the directories it removes (a source under
//! `--delete-source`, an existing target under `--force`) against every
//! path that must still work afterwards.
//!
//! Paths are walked the way the kernel resolves them
//! ([`resolve_visiting_existing`]), so a path that reaches a directory
//! through a symlink inside it depends on it even when the place it ends up
//! is elsewhere.

use std::path::{Path, PathBuf};

use crate::core::packages::{is_within, resolve_visiting_existing};

/// A directory a run removes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Deletion {
    pub(crate) path: PathBuf,
    /// Index of the env (in the batch) that removes it.
    pub(crate) owner: usize,
    pub(crate) kind: DeletionKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DeletionKind {
    /// `--delete-source`: the source env, after its migration.
    Source,
    /// `--force` over an existing scuv env: removed before the new one is
    /// created.
    Target,
}

/// A path that must still work after the run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Survivor {
    pub(crate) path: PathBuf,
    /// What it is, for the message (`"web's source"`, `"package proj"`).
    pub(crate) what: String,
    /// The env whose own source deletion this survivor does not block: a
    /// source and the interpreter inside it go with that source.
    pub(crate) gone_with_source_of: Option<usize>,
}

/// A deletion that would break a survivor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Blocked {
    /// Index into the `deletions` given to [`check_deletions`].
    pub(crate) deletion: usize,
    /// The survivor's `what`.
    pub(crate) because: String,
}

/// Every deletion some survivor reaches (or might: a path that cannot be
/// walked counts as reaching, so an unreadable path blocks rather than
/// risks). A deletion whose path does not exist blocks nothing.
pub(crate) fn check_deletions(deletions: &[Deletion], survivors: &[Survivor]) -> Vec<Blocked> {
    let mut blocked = Vec::new();
    for (i, deletion) in deletions.iter().enumerate() {
        if !deletion.path.exists() {
            continue;
        }
        let exempt = |s: &Survivor| {
            deletion.kind == DeletionKind::Source && s.gone_with_source_of == Some(deletion.owner)
        };
        if let Some(survivor) = survivors
            .iter()
            .find(|s| !exempt(s) && path_reaches(&s.path, &deletion.path).unwrap_or(true))
        {
            blocked.push(Blocked {
                deletion: i,
                because: survivor.what.clone(),
            });
        }
    }
    blocked
}

/// Whether walking `path` (as far as it exists) visits `root` or anything
/// under it.
///
/// # Errors
///
/// A relative path, too many symlinks, or unreadable metadata.
pub(crate) fn path_reaches(path: &Path, root: &Path) -> std::io::Result<bool> {
    if !root.exists() {
        return Ok(false);
    }
    for entry in resolve_visiting_existing(path)? {
        if is_within(&entry, root)? {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Whether two existing directories are the same or one lies under the
/// other, by the filesystem's own identity (symlink aliases and
/// differently cased paths included).
///
/// # Errors
///
/// Either path cannot be canonicalized.
pub(crate) fn overlaps(a: &Path, b: &Path) -> std::io::Result<bool> {
    let (ca, cb) = (std::fs::canonicalize(a)?, std::fs::canonicalize(b)?);
    Ok(is_within(&ca, b)? || is_within(&cb, a)?)
}

/// Whether `prospective`, which may not exist yet, would be created inside
/// `dir`: its nearest existing ancestor is `dir` or lies under it. A
/// sibling under a shared parent is not.
///
/// # Errors
///
/// No ancestor exists, or one cannot be canonicalized.
pub(crate) fn would_be_within(prospective: &Path, dir: &Path) -> std::io::Result<bool> {
    let existing = prospective
        .ancestors()
        .find(|a| a.exists())
        .ok_or_else(|| std::io::Error::other("no existing ancestor"))?;
    is_within(&std::fs::canonicalize(existing)?, dir)
}

/// The paths an env's interpreter depends on: `bin/python` (walked, so the
/// links it goes through count) and the `home` its `pyvenv.cfg` records
/// (uv may link an env to a managed Python inside another env).
pub(crate) fn interpreter_paths(env: &Path) -> Vec<PathBuf> {
    let mut paths = vec![crate::paths::virtualenv_python_exe(env)];
    let home = std::fs::read_to_string(env.join("pyvenv.cfg"))
        .ok()
        .and_then(|cfg| {
            cfg.lines().find_map(|line| {
                let (key, value) = line.split_once('=')?;
                (key.trim() == "home").then(|| PathBuf::from(value.trim()))
            })
        });
    paths.extend(home.filter(|h| h.is_absolute()));
    paths
}

#[cfg(all(test, unix))]
mod tests;
