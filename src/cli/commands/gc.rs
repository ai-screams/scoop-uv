//! Handler for the `scoop gc` command.
//!
//! Detects orphan virtualenvs (directories under `~/.scoop/virtualenvs/`
//! that no longer look like usable environments) and, when run with
//! `--aggressive`, also flags uv-managed Python versions that are not
//! referenced by any surviving env's metadata.
//!
//! Default behaviour is **dry-run** — destructive removal happens only when
//! the caller passes `--yes`. This mirrors how most package managers' `gc`
//! commands behave (cargo, nix, dnf): preview by default, opt-in to delete.

use std::path::{Path, PathBuf};

use rust_i18n::t;
use serde::Serialize;

use crate::core::VirtualenvService;
use crate::error::Result;
use crate::output::Output;
use crate::paths::{self, abbreviate_home};
use crate::uv::UvClient;

/// Why a virtualenv directory was classified as an orphan.
///
/// Strings stay stable — they are part of the JSON contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum OrphanReason {
    /// `.scoop-metadata.json` is missing — the env wasn't created by scoop,
    /// or its metadata file was deleted.
    MissingMetadata,
    /// The Python interpreter the env points at is gone (uninstalled out
    /// from under us, or the symlink target was deleted).
    BrokenPython,
}

#[derive(Debug, Serialize)]
struct OrphanEnv {
    name: String,
    path: String,
    reason: OrphanReason,
}

#[derive(Debug, Serialize)]
struct UnusedPython {
    version: String,
    path: Option<String>,
}

/// What actually happened to an orphan env at `--yes` time.
///
/// JSON consumers parse `outcome` to detect partial failure: a green
/// envelope ("status": "success") with `outcome: "failed"` envs is
/// still a partial failure the caller needs to handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum EnvOutcome {
    /// Dry-run: would remove if `--yes` were given.
    Pending,
    /// `--yes`: directory successfully removed.
    Removed,
    /// `--yes`: env re-classified as healthy between scan and remove;
    /// destructive action skipped on purpose (TOCTOU guard fired).
    SkippedHealthy,
    /// `--yes`: removal returned an IO error. See `error` for details.
    Failed,
}

#[derive(Debug, Serialize)]
struct EnvRecord {
    name: String,
    path: String,
    reason: OrphanReason,
    outcome: EnvOutcome,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

/// What actually happened to a `--aggressive` candidate Python.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum PythonOutcome {
    /// Dry-run: would uninstall.
    Pending,
    /// `--yes`: `uv python uninstall` succeeded.
    Removed,
    /// `--yes`: re-scan showed an env now references this version; skipped.
    SkippedInUse,
    /// `--yes`: uv binary disappeared between scan and uninstall; skipped.
    SkippedNoUv,
    /// `--yes`: uninstall returned an error. See `error` for details.
    Failed,
}

#[derive(Debug, Serialize)]
struct PythonRecord {
    version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<String>,
    outcome: PythonOutcome,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Debug, Serialize)]
struct GcData {
    /// `true` if nothing was actually removed (preview only).
    dry_run: bool,
    /// Orphan virtualenvs and their actual outcome.
    envs: Vec<EnvRecord>,
    /// Unused Python versions (populated only when `--aggressive`) and
    /// their actual outcome.
    pythons: Vec<PythonRecord>,
}

/// Execute the `gc` command.
///
/// * `yes` — actually remove the orphans (otherwise dry-run only).
/// * `aggressive` — also consider Python versions that no env uses.
pub fn execute(output: &Output, yes: bool, aggressive: bool) -> Result<()> {
    let envs = scan_orphan_envs()?;
    let (pythons, unreadable_envs) = if aggressive {
        scan_unused_pythons(&envs)?
    } else {
        (Vec::new(), 0)
    };

    // Surface the conservative bail-out before any destructive work so the
    // user understands why `--aggressive` turned up nothing.
    if aggressive && unreadable_envs > 0 {
        output.warn(&t!(
            "gc.unreadable_metadata_warn",
            count = unreadable_envs.to_string()
        ));
    }

    // Build records up-front. Dry-run leaves everything Pending; `--yes`
    // mutates outcomes in remove_orphans so the JSON envelope reflects
    // what actually happened, not just the original scan snapshot. (The
    // old JSON shape always claimed success — partial failures were only
    // visible in human warn output, which scripts can't see.)
    let mut env_records: Vec<EnvRecord> = envs
        .iter()
        .map(|o| EnvRecord {
            name: o.name.clone(),
            path: o.path.clone(),
            reason: o.reason,
            outcome: EnvOutcome::Pending,
            error: None,
        })
        .collect();
    let mut python_records: Vec<PythonRecord> = pythons
        .iter()
        .map(|p| PythonRecord {
            version: p.version.clone(),
            path: p.path.clone(),
            outcome: PythonOutcome::Pending,
            error: None,
        })
        .collect();

    if yes {
        remove_orphans(
            output,
            &envs,
            &pythons,
            &mut env_records,
            &mut python_records,
        );
    }

    let data = GcData {
        dry_run: !yes,
        envs: env_records,
        pythons: python_records,
    };

    if output.is_json() {
        output.json_success("gc", data);
        return Ok(());
    }

    render_human(output, &data, aggressive);
    Ok(())
}

/// Walk `~/.scoop/virtualenvs/` and flag any directory that fails the
/// "looks like a working env" sniff test.
///
/// Symlinks are intentionally NOT considered. `is_dir()` follows symlinks,
/// so a hostile (or accidental) symlink under `virtualenvs/` would look
/// like an orphan directory, and `fs::remove_dir_all` would then follow
/// the symlink and delete the *target*'s contents under the user's UID.
/// We use `entry.file_type().is_symlink()` to reject every symlink up
/// front, regardless of whether it points to a directory.
///
/// Per-entry IO errors (transient permission / disappearing file) are
/// swallowed so one bad entry doesn't abort the entire scan and hide
/// other orphans from the user.
fn scan_orphan_envs() -> Result<Vec<OrphanEnv>> {
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
            });
        }
    }
    orphans.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(orphans)
}

/// Return `Some(reason)` if `path` looks like a broken env, `None` if it
/// looks healthy.
fn classify(path: &Path) -> Option<OrphanReason> {
    // `.scoop-metadata.json` is the contract: every env scoop creates has
    // one. Its absence means the directory was made by hand or its metadata
    // was deleted — either way we can't safely interpret it.
    if !path.join(".scoop-metadata.json").exists() {
        return Some(OrphanReason::MissingMetadata);
    }
    // Check that the interpreter the env points at still exists. We avoid
    // re-running uv here — a stat on the bin dir is enough to catch the
    // common "Python uninstalled out from under the env" case.
    let bin = if cfg!(windows) {
        path.join("Scripts").join("python.exe")
    } else {
        path.join("bin").join("python")
    };
    if !bin.exists() {
        return Some(OrphanReason::BrokenPython);
    }
    None
}

/// List uv-installed Python versions that aren't referenced by any healthy
/// env's `.scoop-metadata.json`.
///
/// Orphans already slated for removal are *not* counted as references —
/// gc'ing them wouldn't free their Pythons otherwise.
///
/// Safety: if any surviving (non-orphan) env has unreadable metadata, we
/// can't tell what Python it depends on. Silent-dropping it would treat
/// its Python as unused and `gc --aggressive --yes` would uninstall a
/// Python that's actually live — leaving the env broken. To prevent this
/// destructive misclassification, the function bails out conservatively
/// (returns an empty list) the moment it encounters any unreadable
/// metadata, and surfaces a warning to the caller via the second tuple
/// element.
fn scan_unused_pythons(orphans: &[OrphanEnv]) -> Result<(Vec<UnusedPython>, usize)> {
    let uv = match UvClient::new() {
        Ok(u) => u,
        // No uv on PATH → nothing we can do here. Skip aggressive mode
        // silently instead of failing the whole command.
        Err(_) => return Ok((Vec::new(), 0)),
    };
    let installed = uv.list_installed_pythons().unwrap_or_default();
    if installed.is_empty() {
        return Ok((Vec::new(), 0));
    }

    let service = VirtualenvService::auto().ok();
    let mut used: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut unreadable_envs: usize = 0;

    if let Some(svc) = service {
        for info in svc.list().unwrap_or_default() {
            // Skip envs we're about to remove — they shouldn't protect
            // their Pythons from cleanup.
            if orphans.iter().any(|o| o.name == info.name) {
                continue;
            }
            let path = match paths::virtualenv_path(&info.name) {
                Ok(p) => p,
                Err(_) => {
                    unreadable_envs += 1;
                    continue;
                }
            };
            // Read metadata for the python_version field — info.python_version
            // is sniffed from the venv layout and can be missing.
            match svc.read_metadata(&path) {
                Some(meta) => {
                    used.insert(meta.python_version);
                }
                None => {
                    // Metadata file exists (the env wasn't classified as
                    // an orphan) but failed to parse. We can't tell which
                    // Python it depends on — see this function's doc for
                    // why we then bail conservatively.
                    unreadable_envs += 1;
                }
            }
        }
    }

    if unreadable_envs > 0 {
        // Conservative bail: refuse to claim *any* Python is unused.
        // Caller surfaces the warning so the user knows why aggressive
        // cleanup turned up nothing.
        return Ok((Vec::new(), unreadable_envs));
    }

    Ok((
        installed
            .into_iter()
            .filter(|p| !used.contains(&p.version))
            .map(|p| UnusedPython {
                version: p.version,
                path: p.path.map(|p| p.display().to_string()),
            })
            .collect(),
        0,
    ))
}

/// Apply the deletions, mutating `env_records` / `python_records` in
/// place so each entry's `outcome` reflects what actually happened.
///
/// Both record vecs are assumed to be initialised as Pending in 1:1
/// order with `envs` and `pythons`. We continue past per-item errors so
/// a single failure doesn't hide the rest of the cleanup; the per-record
/// `error` field carries the detail for JSON consumers, and human-mode
/// users still get inline warn lines as before.
fn remove_orphans(
    output: &Output,
    envs: &[OrphanEnv],
    pythons: &[UnusedPython],
    env_records: &mut [EnvRecord],
    python_records: &mut [PythonRecord],
) {
    for (env, record) in envs.iter().zip(env_records.iter_mut()) {
        let path = PathBuf::from(&env.path);

        // TOCTOU guard: re-run classify() right before destruction. The
        // gap between scan_orphan_envs and here is usually milliseconds,
        // but a concurrent `scoop create` (or a user manually populating
        // the directory) can make the env healthy again. We refuse to
        // delete a healthy env that just happened to be empty at scan
        // time, and surface the skip both inline (human) and in the
        // record (JSON).
        if classify(&path).is_none() {
            output.warn(&t!("gc.skipped_now_healthy", name = &env.name));
            record.outcome = EnvOutcome::SkippedHealthy;
            continue;
        }

        match std::fs::remove_dir_all(&path) {
            Ok(()) => {
                if !output.is_json() {
                    output.info(&t!("gc.removed_env", name = &env.name));
                }
                record.outcome = EnvOutcome::Removed;
            }
            Err(e) => {
                let detail = e.to_string();
                output.warn(&t!(
                    "gc.remove_env_failed",
                    name = &env.name,
                    error = detail.clone()
                ));
                record.outcome = EnvOutcome::Failed;
                record.error = Some(detail);
            }
        }
    }

    if !pythons.is_empty() {
        // Best-effort: if uv is missing here we just leave the Pythons
        // alone (scan_unused_pythons already returned `[]` in that
        // case; this branch is defensive against transient PATH issues).
        let uv = match UvClient::new() {
            Ok(u) => u,
            Err(_) => {
                for rec in python_records.iter_mut() {
                    rec.outcome = PythonOutcome::SkippedNoUv;
                }
                return;
            }
        };

        // TOCTOU guard for Pythons: re-scan unused versions right
        // before uninstall. The env-level reclassify above only
        // protects venvs from `scoop create`-racing; without this
        // Python-level recheck a concurrent `scoop create` could
        // pull a venv onto a Python we're about to nuke, leaving
        // that env broken.
        let still_unused: std::collections::HashSet<String> = scan_unused_pythons(envs)
            .map_or_else(
                |_| pythons.iter().map(|p| p.version.clone()).collect(),
                |(current, _)| current.into_iter().map(|p| p.version).collect(),
            );

        for (py, record) in pythons.iter().zip(python_records.iter_mut()) {
            if !still_unused.contains(&py.version) {
                output.warn(&t!("gc.skipped_python_now_in_use", version = &py.version));
                record.outcome = PythonOutcome::SkippedInUse;
                continue;
            }
            match uv.uninstall_python(&py.version) {
                Ok(()) => {
                    if !output.is_json() {
                        output.info(&t!("gc.removed_python", version = &py.version));
                    }
                    record.outcome = PythonOutcome::Removed;
                }
                Err(e) => {
                    let detail = e.to_string();
                    output.warn(&t!(
                        "gc.remove_python_failed",
                        version = &py.version,
                        error = detail.clone()
                    ));
                    record.outcome = PythonOutcome::Failed;
                    record.error = Some(detail);
                }
            }
        }
    }
}

fn render_human(output: &Output, data: &GcData, aggressive: bool) {
    if data.envs.is_empty() && data.pythons.is_empty() {
        output.success(&t!("gc.nothing_to_remove"));
        return;
    }

    if !data.envs.is_empty() {
        output.info(&t!("gc.envs_header", count = data.envs.len().to_string()));
        for env in &data.envs {
            let reason = match env.reason {
                OrphanReason::MissingMetadata => t!("gc.reason_missing_metadata"),
                OrphanReason::BrokenPython => t!("gc.reason_broken_python"),
            };
            println!(
                "  - {} ({})  {}",
                env.name,
                reason,
                abbreviate_home(Path::new(&env.path))
            );
        }
    }

    if aggressive && !data.pythons.is_empty() {
        output.info(&t!(
            "gc.pythons_header",
            count = data.pythons.len().to_string()
        ));
        for py in &data.pythons {
            println!("  - Python {}", py.version);
        }
    }

    if data.dry_run {
        output.info(&t!("gc.dry_run_hint"));
    } else {
        output.success(&t!("gc.done"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::with_temp_scoop_home;
    use serial_test::serial;
    use std::fs;

    fn make_env(dir: &Path, name: &str, with_metadata: bool, with_python: bool) {
        let env_dir = dir.join(name);
        fs::create_dir_all(&env_dir).unwrap();
        if with_metadata {
            fs::write(env_dir.join(".scoop-metadata.json"), "{}").unwrap();
        }
        if with_python {
            let bin = if cfg!(windows) {
                env_dir.join("Scripts")
            } else {
                env_dir.join("bin")
            };
            fs::create_dir_all(&bin).unwrap();
            let py = if cfg!(windows) {
                bin.join("python.exe")
            } else {
                bin.join("python")
            };
            fs::write(&py, "").unwrap();
        }
    }

    #[test]
    #[serial]
    fn classifies_missing_metadata_as_orphan() {
        with_temp_scoop_home(|_| {
            let dir = paths::virtualenvs_dir().unwrap();
            fs::create_dir_all(&dir).unwrap();
            make_env(&dir, "no-meta", false, true);

            let orphans = scan_orphan_envs().unwrap();
            assert_eq!(orphans.len(), 1);
            assert_eq!(orphans[0].name, "no-meta");
            assert_eq!(orphans[0].reason, OrphanReason::MissingMetadata);
        });
    }

    #[test]
    #[serial]
    fn classifies_broken_python_as_orphan() {
        with_temp_scoop_home(|_| {
            let dir = paths::virtualenvs_dir().unwrap();
            fs::create_dir_all(&dir).unwrap();
            make_env(&dir, "no-python", true, false);

            let orphans = scan_orphan_envs().unwrap();
            assert_eq!(orphans.len(), 1);
            assert_eq!(orphans[0].name, "no-python");
            assert_eq!(orphans[0].reason, OrphanReason::BrokenPython);
        });
    }

    #[test]
    #[serial]
    fn healthy_env_is_not_an_orphan() {
        with_temp_scoop_home(|_| {
            let dir = paths::virtualenvs_dir().unwrap();
            fs::create_dir_all(&dir).unwrap();
            make_env(&dir, "ok", true, true);

            let orphans = scan_orphan_envs().unwrap();
            assert_eq!(orphans.len(), 0);
        });
    }

    #[test]
    #[serial]
    fn dotfile_entries_are_skipped() {
        with_temp_scoop_home(|_| {
            let dir = paths::virtualenvs_dir().unwrap();
            fs::create_dir_all(&dir).unwrap();
            fs::create_dir_all(dir.join(".cache")).unwrap();

            let orphans = scan_orphan_envs().unwrap();
            assert!(orphans.is_empty());
        });
    }

    #[test]
    #[serial]
    fn dry_run_does_not_remove() {
        with_temp_scoop_home(|_| {
            let dir = paths::virtualenvs_dir().unwrap();
            fs::create_dir_all(&dir).unwrap();
            make_env(&dir, "no-meta", false, true);

            let output = Output::new(0, true, true, false);
            execute(&output, false, false).unwrap();

            assert!(
                dir.join("no-meta").exists(),
                "dry-run must not delete orphans"
            );
        });
    }

    #[test]
    #[serial]
    fn yes_actually_removes_orphans() {
        with_temp_scoop_home(|_| {
            let dir = paths::virtualenvs_dir().unwrap();
            fs::create_dir_all(&dir).unwrap();
            make_env(&dir, "no-meta", false, true);

            let output = Output::new(0, true, true, false);
            execute(&output, true, false).unwrap();

            assert!(!dir.join("no-meta").exists(), "--yes should remove orphans");
        });
    }

    // ==========================================================================
    // S1 regression — symlinks must never be classified as orphans, even
    // when their target lacks .scoop-metadata.json. Otherwise gc --yes
    // would follow the symlink via remove_dir_all and delete the target.
    // ==========================================================================
    #[cfg(unix)]
    #[test]
    #[serial]
    fn scan_skips_symlink_entries() {
        with_temp_scoop_home(|_| {
            let dir = paths::virtualenvs_dir().unwrap();
            fs::create_dir_all(&dir).unwrap();

            // Set up a real directory OUTSIDE the venvs dir — the would-be
            // deletion target. It deliberately lacks .scoop-metadata.json
            // so if the symlink were followed it would classify as
            // MissingMetadata.
            let outside = tempfile::TempDir::new().unwrap();
            let canary = outside.path().join("important.txt");
            fs::write(&canary, b"do not delete").unwrap();

            // Symlink "evil" inside virtualenvs/ → outside dir.
            std::os::unix::fs::symlink(outside.path(), dir.join("evil")).unwrap();

            let orphans = scan_orphan_envs().unwrap();
            assert!(
                orphans.iter().all(|o| o.name != "evil"),
                "symlink must not be classified as an orphan: {:?}",
                orphans
            );

            // Defense-in-depth: even the full --yes path must leave the
            // canary intact.
            let output = Output::new(0, true, true, false);
            execute(&output, true, false).unwrap();
            assert!(
                canary.exists(),
                "gc --yes followed the symlink and deleted the target"
            );
        });
    }

    // ==========================================================================
    // Q2 regression — unreadable metadata on a surviving env must NOT
    // cause gc --aggressive to claim that env's Python is unused. The
    // scan must bail conservatively and return zero unused pythons.
    // ==========================================================================
    #[test]
    #[serial]
    fn aggressive_bails_when_metadata_unreadable() {
        with_temp_scoop_home(|_| {
            let dir = paths::virtualenvs_dir().unwrap();
            fs::create_dir_all(&dir).unwrap();

            // Healthy-shaped env (metadata file + python binary present)
            // so classify() doesn't flag it as an orphan. But the
            // metadata content is garbage so read_metadata returns None.
            let env_path = dir.join("corrupt");
            fs::create_dir_all(env_path.join("bin")).unwrap();
            fs::write(env_path.join("bin/python"), "").unwrap();
            fs::write(env_path.join(".scoop-metadata.json"), "{ not json").unwrap();

            // Sanity: this env is healthy by classify(), so it's not in
            // orphans — exactly the dangerous case where the old code
            // silently dropped it from the "used" set.
            let orphans = scan_orphan_envs().unwrap();
            assert!(orphans.iter().all(|o| o.name != "corrupt"));

            // The scan must bail with `unreadable_envs > 0` and return
            // an empty pythons list — refusing to mark any Python as
            // unused, no matter what `uv python list` reports.
            let (pythons, unreadable_envs) = scan_unused_pythons(&orphans).unwrap();
            assert_eq!(unreadable_envs, 1, "should count one unreadable env");
            assert!(
                pythons.is_empty(),
                "must not claim any Python is unused when metadata is unreadable; got {:?}",
                pythons
            );
        });
    }

    // ==========================================================================
    // Q3 regression — TOCTOU between scan and remove. We simulate by
    // building a fake orphan record that points at a path which is
    // currently healthy. remove_orphans must re-classify and skip.
    // ==========================================================================
    #[test]
    #[serial]
    fn remove_skips_env_that_became_healthy() {
        with_temp_scoop_home(|_| {
            let dir = paths::virtualenvs_dir().unwrap();
            fs::create_dir_all(&dir).unwrap();
            // Make a fully healthy env at the path the fake orphan record
            // will reference. This simulates the racing `scoop create`
            // that ran between scan and remove.
            let env_path = dir.join("racy");
            fs::create_dir_all(env_path.join("bin")).unwrap();
            fs::write(env_path.join("bin/python"), "").unwrap();
            fs::write(env_path.join(".scoop-metadata.json"), "{}").unwrap();

            // Hand-construct an orphan record as if the original scan had
            // flagged it (before the user re-populated the dir).
            let stale_orphan = OrphanEnv {
                name: "racy".to_string(),
                path: env_path.display().to_string(),
                reason: OrphanReason::MissingMetadata,
            };
            let mut env_records = vec![EnvRecord {
                name: stale_orphan.name.clone(),
                path: stale_orphan.path.clone(),
                reason: stale_orphan.reason,
                outcome: EnvOutcome::Pending,
                error: None,
            }];

            let output = Output::new(0, true, true, false);
            remove_orphans(
                &output,
                &[stale_orphan],
                &[],
                &mut env_records,
                &mut Vec::<PythonRecord>::new(),
            );

            // The env was healthy at remove-time so the destructive path
            // must not have run — both the disk state AND the JSON-bound
            // outcome record need to agree on that.
            assert!(
                env_path.exists(),
                "remove_orphans deleted an env that re-classified as healthy"
            );
            assert_eq!(
                env_records[0].outcome,
                EnvOutcome::SkippedHealthy,
                "outcome must be SkippedHealthy so JSON consumers see the skip"
            );
        });
    }

    // ==========================================================================
    // C3 regression — JSON envelope must reflect actual outcomes, not the
    // pre-action scan snapshot. We don't run the full `execute` here (uv
    // would need to be present for --aggressive paths); we verify the
    // record-mutation contract directly.
    // ==========================================================================
    #[test]
    #[serial]
    fn remove_records_actual_outcomes_for_each_env() {
        with_temp_scoop_home(|_| {
            let dir = paths::virtualenvs_dir().unwrap();
            fs::create_dir_all(&dir).unwrap();
            // Two orphans we expect to be successfully removed.
            make_env(&dir, "ghost-a", false, true);
            make_env(&dir, "ghost-b", false, true);

            let orphans = scan_orphan_envs().unwrap();
            assert_eq!(orphans.len(), 2);
            let mut env_records: Vec<EnvRecord> = orphans
                .iter()
                .map(|o| EnvRecord {
                    name: o.name.clone(),
                    path: o.path.clone(),
                    reason: o.reason,
                    outcome: EnvOutcome::Pending,
                    error: None,
                })
                .collect();

            let output = Output::new(0, true, true, false);
            remove_orphans(
                &output,
                &orphans,
                &[],
                &mut env_records,
                &mut Vec::<PythonRecord>::new(),
            );

            for record in &env_records {
                assert_eq!(
                    record.outcome,
                    EnvOutcome::Removed,
                    "expected Removed outcome for {}, got {:?}",
                    record.name,
                    record.outcome
                );
                assert!(record.error.is_none());
            }
        });
    }
}
