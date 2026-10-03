//! The data `gc` works with and reports: what was flagged, why, and what
//! happened to it. These shapes are the `--json` wire format.

use serde::Serialize;

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
pub(super) enum EnvGcReason {
    /// `.scoop-metadata.json` is missing — the env wasn't created by scuv,
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

/// An env `gc` flagged: an orphan (unusable) or, with `--older-than`, a
/// healthy env that has not been used for longer than the cutoff.
#[derive(Debug, Serialize)]
pub(super) struct GcCandidate {
    pub(super) name: String,
    pub(super) path: String,
    pub(super) reason: EnvGcReason,
    /// Only populated when `reason == Stale`. Frozen at scan time;
    /// recheck before removal may see a slightly different value if
    /// the env was touched concurrently. Hidden from JSON unless set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) age_days: Option<u64>,
}

/// Why `--aggressive` named no Python as unused even though some may be.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PythonSkip {
    /// This many surviving envs have metadata that cannot be read.
    UnreadableMetadata(usize),
    /// The env list itself could not be read.
    EnvListUnavailable,
}

#[derive(Debug, Serialize)]
pub(super) struct UnusedPython {
    pub(super) version: String,
    pub(super) path: Option<String>,
}

/// What actually happened to an orphan env at `--yes` time.
///
/// JSON consumers parse `outcome` to detect partial failure: a green
/// envelope ("status": "success") with `outcome: "failed"` envs is
/// still a partial failure the caller needs to handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum EnvOutcome {
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
pub(super) struct EnvRecord {
    pub(super) name: String,
    pub(super) path: String,
    pub(super) reason: EnvGcReason,
    /// Days since last activation at scan time. Populated only for
    /// `Stale` records; orphan records leave it `None` and serde
    /// omits the field entirely. Carrying this alongside (not inside)
    /// `reason` is what keeps the JSON envelope additive: the old
    /// `reason: "missing_metadata"` shape is untouched for orphans.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) age_days: Option<u64>,
    pub(super) outcome: EnvOutcome,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) error: Option<String>,
}

/// What actually happened to a `--aggressive` candidate Python.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum PythonOutcome {
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
pub(super) struct PythonRecord {
    pub(super) version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) path: Option<String>,
    pub(super) outcome: PythonOutcome,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) error: Option<String>,
}

#[derive(Debug, Serialize)]
pub(super) struct GcData {
    /// `true` if nothing was actually removed (preview only).
    pub(super) dry_run: bool,
    /// Orphan virtualenvs and their actual outcome.
    pub(super) envs: Vec<EnvRecord>,
    /// Unused Python versions (populated only when `--aggressive`) and
    /// their actual outcome.
    pub(super) pythons: Vec<PythonRecord>,
}
