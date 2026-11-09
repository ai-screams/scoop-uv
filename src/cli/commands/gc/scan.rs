//! Finding candidates: orphan and stale virtualenvs, unused Pythons, and the
//! re-checks `--yes` runs right before deleting.

use std::collections::HashSet;
use std::path::Path;

use chrono::{DateTime, Utc};

use crate::core::Metadata;
use crate::error::Result;
use crate::paths;
use crate::uv::UvClient;

use super::types::{EnvGcReason, EnvOutcome, GcCandidate, PythonSkip, UnusedPython};

/// Walk `~/.scuv/virtualenvs/` and flag any directory that fails the
/// "looks like a working env" sniff test.
///
/// Symlinks are intentionally NOT considered. `is_dir()` follows
/// symlinks, so a hostile (or accidental) symlink under `virtualenvs/`
/// would look like an orphan directory and downstream code (verify
/// shelling out to `<target>/bin/python`, or future tooling that does
/// not assume rustc 1.78+'s non-following `remove_dir_all`) would act
/// on the target. We reject every symlink up front via
/// `entry.file_type().is_symlink()`, regardless of whether the target
/// is a directory, so the threat doesn't reach those code paths in the
/// first place.
///
/// Note: as of Rust 1.78 `std::fs::remove_dir_all` does NOT follow
/// symlinks itself, so a missed symlink wouldn't directly nuke the
/// target *via gc*. The defense above is still correct policy —
/// classify/exec paths would still touch the target — but don't
/// cargo-cult the old "remove_dir_all follows symlinks" rationale.
///
/// Per-entry IO errors (transient permission / disappearing file) are
/// swallowed so one bad entry doesn't abort the entire scan and hide
/// other orphans from the user.
pub(super) fn scan_orphan_envs() -> Result<Vec<GcCandidate>> {
    let mut orphans = Vec::new();
    // `env_dirs` skips symlinks (see the note above) and unreadable entries.
    for (name, path) in crate::core::env_dirs()? {
        // Skip dotfiles — `gc` should leave alone anything that doesn't look
        // like an env name (e.g. `.DS_Store`, `.cache/`).
        if name.starts_with('.') {
            continue;
        }

        // An env whose files cannot be inspected is not a candidate: it is
        // kept, as a healthy one is.
        if let Health::Orphan(reason) = classify(&path) {
            orphans.push(GcCandidate {
                name,
                path: path.display().to_string(),
                reason,
                age_days: None,
            });
        }
    }
    orphans.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(orphans)
}

/// Walk the env list and flag any env whose `last_used` is older than
/// `cutoff`.
///
/// Crucial design decisions baked in:
///
/// * **Only healthy envs are considered.** `classify()` returning
///   `Some(_)` means the env is already an orphan — it'll be removed
///   via the orphan path. Flagging it twice would double-count and
///   confuse the JSON envelope.
/// * **`last_used = None` never matches.** A missing `last_used` could mean "never activated"
///   (fresh env) *or* "predates the field" (legacy metadata). Either
///   way we have no positive evidence the env is unused, so we refuse
///   to flag it. Users who really want to nuke un-activated envs can
///   `scuv verify` + manual removal.
/// * **Corrupt metadata never matches.** Unreadable or unparseable
///   metadata reads as no `last_used`, so the same else-guard that
///   protects "never activated" skips it too.
///
/// # Errors
///
/// When the env directories cannot be walked at all.
pub(super) fn scan_stale_envs(cutoff: DateTime<Utc>) -> Result<Vec<GcCandidate>> {
    // Files only: no uv needed. A walk that cannot start is an error, not
    // "nothing is stale".
    let mut stale = Vec::new();
    for (name, path) in crate::core::env_dirs()? {
        // Only healthy envs: orphans are handled by the orphan path (and
        // must not be recorded twice), and an env that cannot be inspected
        // is kept.
        if classify(&path) != Health::Healthy {
            continue;
        }

        // last_used = None → no match, full stop. See function doc. Corrupt
        // metadata reads as None too.
        let last_used = crate::core::read_metadata_file(&path)
            .ok()
            .flatten()
            .and_then(|m| m.last_used);
        let Some(last_used) = last_used else {
            continue;
        };

        if last_used >= cutoff {
            continue;
        }

        let age_days = (Utc::now() - last_used).num_days().max(0) as u64;
        stale.push(GcCandidate {
            name,
            path: path.display().to_string(),
            reason: EnvGcReason::Stale,
            age_days: Some(age_days),
        });
    }
    Ok(stale)
}

/// What gc can tell about an env directory from its files.
#[derive(Debug, PartialEq)]
pub(super) enum Health {
    /// Metadata and interpreter are both there.
    Healthy,
    /// Confirmed broken: one of them is missing.
    Orphan(EnvGcReason),
    /// A check failed (an unreadable path, a looping symlink): neither
    /// healthy nor confirmed broken, so never deleted.
    Unknown,
}

/// Classifies `path` from an *orphan-classifier* point of view. Stale
/// detection lives in `scan_stale_envs` — keeping the two separate means
/// the orphan-side TOCTOU guard can't accidentally reclassify a
/// stale-but-healthy env as "not really stale" and skip removing it.
///
/// Uses `try_exists`, not `exists`: `exists` reads every error as
/// "missing", which would turn an env gc cannot inspect into an orphan to
/// delete.
pub(super) fn classify(path: &Path) -> Health {
    // `.scoop-metadata.json` is the contract: every env scuv creates has
    // one. Its absence means the directory was made by hand or its metadata
    // was deleted — either way we can't safely interpret it.
    match path.join(Metadata::FILE_NAME).try_exists() {
        Ok(true) => {}
        Ok(false) => return Health::Orphan(EnvGcReason::OrphanMissingMetadata),
        Err(_) => return Health::Unknown,
    }
    // Check that the interpreter the env points at still exists. We avoid
    // re-running uv here — a stat on the python executable is enough to
    // catch the common "Python uninstalled out from under the env" case. A
    // dangling interpreter link is `Ok(false)`, so it is still an orphan.
    match paths::virtualenv_python_exe(path).try_exists() {
        Ok(true) => Health::Healthy,
        Ok(false) => Health::Orphan(EnvGcReason::OrphanBrokenPython),
        Err(_) => Health::Unknown,
    }
}

/// Re-check that an env recorded as `Stale` at scan time *still* qualifies.
///
/// Returns the outcome that should be written to the env record:
/// - `None` — actually remove (still stale after recheck).
/// - `Some(SkippedRecentlyUsed)` — env was touched between scan and
///   remove; `last_used` is now >= original cutoff.
/// - `Some(SkippedNoData)` — metadata became unreadable or its
///   `last_used` is suddenly None.
///
/// Pulled out of `remove_candidates` so the per-reason TOCTOU branches
/// stay readable.
pub(super) fn recheck_stale(name: &str, cutoff: DateTime<Utc>) -> Option<EnvOutcome> {
    // Validation guard — see VirtualenvService::delete for the path
    // traversal rationale. recheck_stale's `name` comes from scan
    // results today (disk-walked basenames, can't traverse), but
    // guarding here closes the gap if a future caller passes raw
    // input.
    if crate::validate::validate_existing_env_name(name).is_err() {
        return Some(EnvOutcome::SkippedNoData);
    }
    let path = match paths::virtualenv_path(name) {
        Ok(p) => p,
        Err(_) => return Some(EnvOutcome::SkippedNoData),
    };
    // Files only, like the scan: no uv needed.
    let meta = match crate::core::read_metadata_file(&path) {
        Ok(Some(m)) => m,
        // Missing or corrupt — see scan_stale_envs's rationale.
        _ => return Some(EnvOutcome::SkippedNoData),
    };
    let Some(last_used) = meta.last_used else {
        return Some(EnvOutcome::SkippedNoData);
    };
    if last_used >= cutoff {
        return Some(EnvOutcome::SkippedRecentlyUsed);
    }
    None
}

/// List uv-installed Python versions that aren't referenced by any healthy
/// env's `.scoop-metadata.json`.
///
/// `leaving` names the envs that will not survive (the candidates at
/// preview time, the ones actually removed at `--yes` time); they are not
/// counted as references — gc'ing them wouldn't free their Pythons
/// otherwise. An env that stays must keep protecting its Python.
///
/// Safety: whenever it cannot tell which Pythons the surviving envs use (see
/// [`referenced_versions`]), it names none as unused and returns the reason,
/// which the caller surfaces. Claiming a live Python is unused would let
/// `gc --aggressive --yes` uninstall it and break the env.
///
/// # Errors
///
/// Returns uv's error when `uv python list` fails.
pub(super) fn scan_unused_pythons(
    leaving: &[&str],
) -> Result<(Vec<UnusedPython>, Option<PythonSkip>)> {
    let uv = match UvClient::new() {
        Ok(u) => u,
        // No uv on PATH → nothing we can do here. Skip aggressive mode
        // silently instead of failing the whole command.
        Err(_) => return Ok((Vec::new(), None)),
    };
    // Only uv-managed Pythons: a system interpreter is not uv's to uninstall.
    // A failing listing ends the command with uv's error, before anything is
    // removed: reading it as "no Pythons" would report nothing to clean.
    let installed = uv.list_managed_pythons()?;
    if installed.is_empty() {
        return Ok((Vec::new(), None));
    }

    let used = match referenced_versions(leaving) {
        Ok(used) => used,
        Err(skip) => return Ok((Vec::new(), Some(skip))),
    };

    Ok((
        installed
            .into_iter()
            .filter(|p| !used.iter().any(|v| version_covers(v, &p.version)))
            .map(|p| UnusedPython {
                version: p.version,
                path: p.path.map(|p| p.display().to_string()),
            })
            .collect(),
        None,
    ))
}

/// Whether an env's recorded Python version refers to `installed`.
///
/// uv links an env to a minor version (`cpython-3.12-…`) and records only
/// `3.12` in pyvenv.cfg, which scuv copies into the metadata; that env uses
/// whichever 3.12.x the link points at. So the recorded version covers
/// every installed version it is a component-wise prefix of: `3.12` covers
/// `3.12.14`, `3.12.1` covers only `3.12.1`, never `3.12.10`.
pub(super) fn version_covers(recorded: &str, installed: &str) -> bool {
    let mut installed = installed.split('.');
    recorded
        .split('.')
        .all(|part| installed.next() == Some(part))
}

/// Whether a recorded version is the dotted numbers `version_covers` can
/// compare (`3.12`, `3.12.1`); anything else (`unknown`) is unreadable.
fn is_dotted_version(version: &str) -> bool {
    version
        .split('.')
        .all(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit()))
}

/// The Python versions the surviving (non-candidate) envs depend on.
///
/// Fails closed: if the env list cannot be read, or any surviving env's
/// metadata cannot, the answer is unknown and the caller must not treat any
/// Python as unused. An empty set here would mean exactly that.
pub(super) fn referenced_versions(
    leaving: &[&str],
) -> std::result::Result<HashSet<String>, PythonSkip> {
    // The strict walk: an env it could not see would stop protecting its
    // Python, so any entry it cannot read makes the answer unknown.
    let envs = crate::core::env_dirs_complete().map_err(|_| PythonSkip::EnvListUnavailable)?;

    let mut used = HashSet::new();
    let mut unreadable = 0usize;
    for (name, path) in envs {
        // Skip envs that are going away — they shouldn't protect their
        // Pythons from cleanup.
        if leaving.contains(&name.as_str()) {
            continue;
        }
        let meta = crate::core::read_metadata_file(&path).ok().flatten();
        match meta {
            Some(meta) if is_dotted_version(&meta.python_version) => {
                used.insert(meta.python_version);
            }
            // Metadata exists (the env wasn't classified as a candidate) but
            // failed to parse, or names no version we can compare (`unknown`
            // from an unreadable --python-path): we can't tell which Python
            // it depends on.
            _ => unreadable += 1,
        }
    }

    if unreadable > 0 {
        return Err(PythonSkip::UnreadableMetadata(unreadable));
    }
    Ok(used)
}
