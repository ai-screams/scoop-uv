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

use chrono::{DateTime, Utc};
use rust_i18n::t;
use serde::Serialize;

use crate::core::VirtualenvService;
use crate::error::Result;
use crate::output::Output;
use crate::paths::{self, abbreviate_home};
use crate::uv::UvClient;

use super::duration::parse_duration;

/// Why a virtualenv directory was flagged by `gc`.
///
/// **Wire-format kind only.** Used as the `reason` discriminator in the
/// JSON envelope. The two `Orphan*` variants serialize to their pre-
/// Stale string values (`"missing_metadata"` / `"broken_python"`) via
/// `#[serde(rename)]` so adding `Stale` is a pure additive schema
/// change — old consumers parse `reason: "missing_metadata"` unchanged
/// instead of suddenly seeing `reason: {kind: "missing_metadata"}` if
/// we'd used `#[serde(tag = "kind")]`. The Stale-specific `age_days`
/// rides next to `reason` in the record, not inside it (see [`EnvRecord`]).
///
/// Codex review on the v2 plan flagged this separation as STOP-1: a
/// single flat enum would have let the orphan-side TOCTOU guard
/// reclassify stale-but-healthy envs as "fine" and skip removing them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum EnvGcReason {
    /// `.scoop-metadata.json` is missing — the env wasn't created by scoop,
    /// or its metadata file was deleted. Pre-Stale JSON value preserved.
    #[serde(rename = "missing_metadata")]
    OrphanMissingMetadata,
    /// The Python interpreter the env points at is gone (uninstalled out
    /// from under us, or the symlink target was deleted). Pre-Stale JSON
    /// value preserved.
    #[serde(rename = "broken_python")]
    OrphanBrokenPython,
    /// `last_used` is older than the `--older-than <DURATION>` cutoff.
    /// The actual day count rides separately as `EnvRecord.age_days`
    /// so the JSON envelope stays flat (no nested tag/object form).
    Stale,
}

#[derive(Debug, Serialize)]
struct OrphanEnv {
    name: String,
    path: String,
    reason: EnvGcReason,
    /// Only populated when `reason == Stale`. Frozen at scan time;
    /// recheck before removal may see a slightly different value if
    /// the env was touched concurrently. Hidden from JSON unless set.
    #[serde(skip_serializing_if = "Option::is_none")]
    age_days: Option<u64>,
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
    /// `--yes`: orphan re-classified as healthy between scan and
    /// remove; destructive action skipped on purpose (TOCTOU guard
    /// fired for an `Orphan*` reason).
    SkippedHealthy,
    /// `--yes`: stale env was touched between scan and remove (the
    /// recheck against the original cutoff now passes); destructive
    /// action skipped on purpose (TOCTOU guard fired for `Stale`).
    SkippedRecentlyUsed,
    /// `--yes`: stale env's metadata became unreadable between scan
    /// and remove. Refusing to delete an env we can no longer reason
    /// about is the conservative move — surfaces the situation
    /// instead of guessing.
    SkippedNoData,
    /// `--yes`: removal returned an IO error. See `error` for details.
    Failed,
}

#[derive(Debug, Serialize)]
struct EnvRecord {
    name: String,
    path: String,
    reason: EnvGcReason,
    /// Days since last activation at scan time. Populated only for
    /// `Stale` records; orphan records leave it `None` and serde
    /// omits the field entirely. Carrying this alongside (not inside)
    /// `reason` is what keeps the JSON envelope additive: the old
    /// `reason: "missing_metadata"` shape is untouched for orphans.
    #[serde(skip_serializing_if = "Option::is_none")]
    age_days: Option<u64>,
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
/// * `yes` — actually remove the candidates (otherwise dry-run only).
/// * `aggressive` — also consider Python versions that no env uses.
/// * `older_than` — when `Some`, also flag envs whose `last_used` is
///   older than the parsed duration. `None` preserves the pre-flag
///   behaviour (orphans only).
pub fn execute(
    output: &Output,
    yes: bool,
    aggressive: bool,
    older_than: Option<&str>,
) -> Result<()> {
    // Parse the duration up front so a malformed `--older-than` fails
    // before we touch the filesystem. Cutoff is sampled once and
    // shared between scan + recheck — using a fresh `Utc::now()` at
    // recheck time would make borderline envs jitter in/out of "stale".
    let stale_cutoff = match older_than {
        Some(s) => {
            let d = parse_duration(s)?;
            Some(Utc::now().checked_sub_signed(d).ok_or_else(|| {
                crate::error::ScoopError::InvalidArgument {
                    message: format!("cutoff arithmetic overflowed for --older-than {s}"),
                }
            })?)
        }
        None => None,
    };

    let mut envs = scan_orphan_envs()?;
    if let Some(cutoff) = stale_cutoff {
        envs.extend(scan_stale_envs(cutoff)?);
        envs.sort_by(|a, b| a.name.cmp(&b.name));
    }

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
            age_days: o.age_days,
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
            stale_cutoff,
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
///   `scoop verify` + manual removal.
/// * **Corrupt metadata never matches.** Same conservative rule via
///   a different code path: `VirtualenvService::list()` populates
///   `info.last_used` from the legacy `read_metadata`, which collapses
///   parse errors to `None`. The same `Some(last_used)` else-guard
///   that protects "never activated" therefore also skips "unreadable
///   metadata" — we don't need a separate corrupt branch here.
fn scan_stale_envs(cutoff: DateTime<Utc>) -> Result<Vec<OrphanEnv>> {
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
fn classify(path: &Path) -> Option<EnvGcReason> {
    // `.scoop-metadata.json` is the contract: every env scoop creates has
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
fn recheck_stale(name: &str, cutoff: DateTime<Utc>) -> Option<EnvOutcome> {
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
    stale_cutoff: Option<DateTime<Utc>>,
) {
    for (env, record) in envs.iter().zip(env_records.iter_mut()) {
        let path = PathBuf::from(&env.path);

        // Per-reason TOCTOU guard. Codex STOP-1 on the v2 plan caught
        // the trap of running one guard for both kinds: orphans need
        // re-classify (was-orphan → now-healthy means skip), but stale
        // envs need an age recheck — running classify() on a stale-
        // but-healthy env would falsely skip removal.
        match env.reason {
            EnvGcReason::OrphanMissingMetadata | EnvGcReason::OrphanBrokenPython => {
                if classify(&path).is_none() {
                    output.warn(&t!("gc.skipped_now_healthy", name = &env.name));
                    record.outcome = EnvOutcome::SkippedHealthy;
                    continue;
                }
            }
            EnvGcReason::Stale => {
                // Stale recheck needs the original cutoff — re-deriving
                // "now" here would let a touch on the env between scan
                // and remove silently win or lose by milliseconds.
                let cutoff = match stale_cutoff {
                    Some(c) => c,
                    None => {
                        // Defensive: a Stale record without a cutoff is
                        // an internal bug (scan_stale_envs only runs
                        // when older_than is Some). Refuse rather than
                        // delete based on stale-at-scan-time alone.
                        record.outcome = EnvOutcome::SkippedNoData;
                        continue;
                    }
                };
                if let Some(skip_outcome) = recheck_stale(&env.name, cutoff) {
                    let msg_key = match skip_outcome {
                        EnvOutcome::SkippedRecentlyUsed => "gc.skipped_recently_used",
                        EnvOutcome::SkippedNoData => "gc.skipped_no_data",
                        _ => "gc.skipped_now_healthy",
                    };
                    output.warn(&t!(msg_key, name = &env.name));
                    record.outcome = skip_outcome;
                    continue;
                }
            }
        }

        match std::fs::remove_dir_all(&path) {
            Ok(()) => {
                if !output.is_json() {
                    output.info(&t!("gc.removed_env", name = &env.name));
                }
                record.outcome = EnvOutcome::Removed;
            }
            // Treat "already gone" as success-equivalent rather than
            // surfacing it as Failed. Codex MEDIUM-1: two `gc --yes`
            // racing on the same candidate would otherwise return
            // success once and a noisy "no such file" Failed for the
            // second runner, which scripts can't distinguish from a
            // real IO failure. The on-disk goal (env deleted) is met.
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
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
                EnvGcReason::OrphanMissingMetadata => t!("gc.reason_missing_metadata").to_string(),
                EnvGcReason::OrphanBrokenPython => t!("gc.reason_broken_python").to_string(),
                EnvGcReason::Stale => {
                    // Pull age_days out of the record (None means
                    // some future caller produced a Stale record
                    // without one — render "?" rather than panic).
                    let days = env.age_days.map(|n| n.to_string()).unwrap_or("?".into());
                    t!("gc.reason_stale", days = days).to_string()
                }
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
mod tests;
