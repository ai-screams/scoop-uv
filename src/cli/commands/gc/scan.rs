//! Finding candidates: orphan and stale virtualenvs, unused Pythons, and the
//! re-checks `--yes` runs right before deleting.

use std::collections::HashSet;
use std::path::Path;

use chrono::{DateTime, Utc};

use crate::core::VirtualenvService;
use crate::error::Result;
use crate::paths;
use crate::uv::UvClient;

use super::types::{EnvGcReason, EnvOutcome, OrphanEnv, PythonSkip, UnusedPython};

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
pub(super) fn scan_orphan_envs() -> Result<Vec<OrphanEnv>> {
    let dir = paths::virtualenvs_dir()?;
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut orphans = Vec::new();
    for entry in std::fs::read_dir(&dir)? {
        // Per-entry tolerance: don't let a single read_dir item failure
        // (e.g. permission flake, file removed mid-scan) abort the whole
        // pass — that would silently hide orphans further in the listing.
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        // Use file_type() (no traversal) instead of path.is_dir() (which
        // follows symlinks). Skip symlinks unconditionally — see the
        // symlink note in this function's doc comment for the rationale.
        let ft = match entry.file_type() {
            Ok(t) => t,
            Err(_) => continue,
        };
        if !ft.is_dir() || ft.is_symlink() {
            continue;
        }
        let path = entry.path();
        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };
        // Skip dotfiles — `gc` should leave alone anything that doesn't look
        // like an env name (e.g. `.DS_Store`, `.cache/`).
        if name.starts_with('.') {
            continue;
        }

        if let Some(reason) = classify(&path) {
            orphans.push(OrphanEnv {
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
/// * **`last_used = None` never matches.** Codex review on the v2 plan
///   was emphatic: a missing `last_used` could mean "never activated"
///   (fresh env) *or* "predates the field" (legacy metadata). Either
///   way we have no positive evidence the env is unused, so we refuse
///   to flag it. Users who really want to nuke un-activated envs can
///   `scuv verify` + manual removal.
/// * **Corrupt metadata never matches.** Same conservative rule via
///   a different code path: `VirtualenvService::list()` populates
///   `info.last_used` from the legacy `read_metadata`, which collapses
///   parse errors to `None`. The same `Some(last_used)` else-guard
///   that protects "never activated" therefore also skips "unreadable
///   metadata" — we don't need a separate corrupt branch here.
pub(super) fn scan_stale_envs(cutoff: DateTime<Utc>) -> Result<Vec<OrphanEnv>> {
    let service = match VirtualenvService::auto() {
        Ok(s) => s,
        Err(_) => return Ok(Vec::new()),
    };
    let envs = match service.list() {
        Ok(v) => v,
        Err(_) => return Ok(Vec::new()),
    };

    let mut stale = Vec::new();
    for info in envs {
        // Skip envs already flagged as orphans (no metadata / broken
        // python). The orphan path handles them; flagging them as
        // stale too would double-record.
        if classify(&info.path).is_some() {
            continue;
        }

        // last_used = None → no match, full stop. See function doc.
        let Some(last_used) = info.last_used else {
            continue;
        };

        if last_used >= cutoff {
            continue;
        }

        let age_days = (Utc::now() - last_used).num_days().max(0) as u64;
        stale.push(OrphanEnv {
            name: info.name.clone(),
            path: info.path.display().to_string(),
            reason: EnvGcReason::Stale,
            age_days: Some(age_days),
        });
    }
    Ok(stale)
}

/// Return `Some(reason)` if `path` looks like a broken env, `None` if
/// it looks healthy from an *orphan-classifier* point of view. Stale
/// detection lives in `scan_stale_envs` — keeping the two separate
/// (per Codex STOP-1 on the v2 plan) means the orphan-side TOCTOU
/// guard can't accidentally reclassify a stale-but-healthy env as
/// "not really stale" and skip removing it.
pub(super) fn classify(path: &Path) -> Option<EnvGcReason> {
    // `.scoop-metadata.json` is the contract: every env scuv creates has
    // one. Its absence means the directory was made by hand or its metadata
    // was deleted — either way we can't safely interpret it.
    if !path.join(".scoop-metadata.json").exists() {
        return Some(EnvGcReason::OrphanMissingMetadata);
    }
    // Check that the interpreter the env points at still exists. We avoid
    // re-running uv here — a stat on the python executable is enough to
    // catch the common "Python uninstalled out from under the env" case.
    let bin = paths::virtualenv_python_exe(path);
    if !bin.exists() {
        return Some(EnvGcReason::OrphanBrokenPython);
    }
    None
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
/// Pulled out of `remove_orphans` so the per-reason TOCTOU branches
/// stay readable.
pub(super) fn recheck_stale(name: &str, cutoff: DateTime<Utc>) -> Option<EnvOutcome> {
    // Validation guard — see VirtualenvService::delete for the path
    // traversal rationale. recheck_stale's `name` comes from scan
    // results today (disk-walked basenames, can't traverse), but
    // guarding here closes the gap if a future caller passes raw
    // input.
    if crate::validate::validate_env_name(name).is_err() {
        return Some(EnvOutcome::SkippedNoData);
    }
    let path = match paths::virtualenv_path(name) {
        Ok(p) => p,
        Err(_) => return Some(EnvOutcome::SkippedNoData),
    };
    let service = match VirtualenvService::auto() {
        Ok(s) => s,
        // Without the service we can't recheck. Refuse to delete
        // rather than guess.
        Err(_) => return Some(EnvOutcome::SkippedNoData),
    };
    let meta = match service.read_metadata_result(&path) {
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
/// Orphans already slated for removal are *not* counted as references —
/// gc'ing them wouldn't free their Pythons otherwise.
///
/// Safety: whenever it cannot tell which Pythons the surviving envs use (see
/// [`referenced_versions`]), it names none as unused and returns the reason,
/// which the caller surfaces. Claiming a live Python is unused would let
/// `gc --aggressive --yes` uninstall it and break the env.
pub(super) fn scan_unused_pythons(
    orphans: &[OrphanEnv],
) -> Result<(Vec<UnusedPython>, Option<PythonSkip>)> {
    let uv = match UvClient::new() {
        Ok(u) => u,
        // No uv on PATH → nothing we can do here. Skip aggressive mode
        // silently instead of failing the whole command.
        Err(_) => return Ok((Vec::new(), None)),
    };
    let installed = uv.list_installed_pythons().unwrap_or_default();
    if installed.is_empty() {
        return Ok((Vec::new(), None));
    }

    let used = match referenced_versions(orphans) {
        Ok(used) => used,
        Err(skip) => return Ok((Vec::new(), Some(skip))),
    };

    Ok((
        installed
            .into_iter()
            .filter(|p| !used.contains(&p.version))
            .map(|p| UnusedPython {
                version: p.version,
                path: p.path.map(|p| p.display().to_string()),
            })
            .collect(),
        None,
    ))
}

/// The Python versions the surviving (non-candidate) envs depend on.
///
/// Fails closed: if the env list cannot be read, or any surviving env's
/// metadata cannot, the answer is unknown and the caller must not treat any
/// Python as unused. An empty set here would mean exactly that.
pub(super) fn referenced_versions(
    candidates: &[OrphanEnv],
) -> std::result::Result<HashSet<String>, PythonSkip> {
    let service = VirtualenvService::auto().map_err(|_| PythonSkip::EnvListUnavailable)?;
    let envs = service.list().map_err(|_| PythonSkip::EnvListUnavailable)?;

    let mut used = HashSet::new();
    let mut unreadable = 0usize;
    for info in envs {
        // Skip envs we're about to remove — they shouldn't protect their
        // Pythons from cleanup.
        if candidates.iter().any(|o| o.name == info.name) {
            continue;
        }
        // Read metadata for the python_version field — info.python_version
        // is sniffed from the venv layout and can be missing.
        let meta = paths::virtualenv_path(&info.name)
            .ok()
            .and_then(|path| service.read_metadata(&path));
        match meta {
            Some(meta) => {
                used.insert(meta.python_version);
            }
            // Metadata exists (the env wasn't classified as a candidate) but
            // failed to parse: we can't tell which Python it depends on.
            None => unreadable += 1,
        }
    }

    if unreadable > 0 {
        return Err(PythonSkip::UnreadableMetadata(unreadable));
    }
    Ok(used)
}
