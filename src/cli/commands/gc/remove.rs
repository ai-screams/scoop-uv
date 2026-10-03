//! `--yes`: removing the candidates, re-checking each one first.

use std::path::PathBuf;

use chrono::{DateTime, Utc};
use rust_i18n::t;

use crate::output::Output;
use crate::uv::UvClient;

use super::scan::{classify, recheck_stale, scan_unused_pythons};
use super::types::{
    EnvGcReason, EnvOutcome, EnvRecord, GcCandidate, PythonOutcome, PythonRecord, UnusedPython,
};

/// Apply the deletions, mutating `env_records` / `python_records` in
/// place so each entry's `outcome` reflects what actually happened.
///
/// Both record vecs are assumed to be initialised as Pending in 1:1
/// order with `envs` and `pythons`. We continue past per-item errors so
/// a single failure doesn't hide the rest of the cleanup; the per-record
/// `error` field carries the detail for JSON consumers, and human-mode
/// users still get inline warn lines as before.
pub(super) fn remove_candidates(
    output: &Output,
    envs: &[GcCandidate],
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
        // protects venvs from `scuv create`-racing; without this
        // Python-level recheck a concurrent `scuv create` could
        // pull a venv onto a Python we're about to nuke, leaving
        // that env broken.
        // Fails closed: if the re-scan errors or cannot tell what is in
        // use, nothing counts as still unused and every uninstall is
        // skipped. (It used to assume all of them still were.)
        let still_unused: std::collections::HashSet<String> = match scan_unused_pythons(envs) {
            Ok((current, None)) => current.into_iter().map(|p| p.version).collect(),
            _ => std::collections::HashSet::new(),
        };

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
