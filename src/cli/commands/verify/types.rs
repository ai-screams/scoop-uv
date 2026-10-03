//! What `verify` reports: per-check results, per-env reports and the
//! summary. The serialized names are part of the `--json` contract.

use serde::Serialize;

/// Outcome of a single check.
///
/// The strings stay stable — they're part of the JSON contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum CheckStatus {
    /// Everything as expected.
    Pass,
    /// Check is irrelevant for this env (e.g. manifest_match with no manifest).
    Skip,
    /// Something is off but the env might still work (e.g. version drift).
    Warn,
    /// Hard breakage — env almost certainly unusable.
    Fail,
}

/// Which check a result belongs to.
///
/// Serializes to the stable snake_case id `--json` clients read; the
/// human-readable label comes from [`CheckKind::label_key`] at print time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum CheckKind {
    Metadata,
    PythonBinary,
    PyvenvCfg,
    ActivateScript,
    PythonExecutes,
    ManifestMatch,
}

impl CheckKind {
    /// The i18n key of this check's label. Exhaustive, so a new check
    /// cannot ship without one.
    pub(super) fn label_key(self) -> &'static str {
        match self {
            Self::Metadata => "verify.check_metadata",
            Self::PythonBinary => "verify.check_python_binary",
            Self::PyvenvCfg => "verify.check_pyvenv_cfg",
            Self::ActivateScript => "verify.check_activate_script",
            Self::PythonExecutes => "verify.check_python_executes",
            Self::ManifestMatch => "verify.check_manifest_match",
        }
    }
}

#[derive(Debug, Serialize)]
pub(super) struct CheckResult {
    /// Which check this is (serialized as its stable id).
    pub(super) name: CheckKind,
    pub(super) status: CheckStatus,
    /// Populated for Warn/Fail with a short hint. Pass/Skip leave this null.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) message: Option<String>,
}

impl CheckResult {
    pub(super) fn pass(name: CheckKind) -> Self {
        Self {
            name,
            status: CheckStatus::Pass,
            message: None,
        }
    }
    pub(super) fn skip(name: CheckKind) -> Self {
        Self {
            name,
            status: CheckStatus::Skip,
            message: None,
        }
    }
    pub(super) fn warn(name: CheckKind, message: impl Into<String>) -> Self {
        Self {
            name,
            status: CheckStatus::Warn,
            message: Some(message.into()),
        }
    }
    pub(super) fn fail(name: CheckKind, message: impl Into<String>) -> Self {
        Self {
            name,
            status: CheckStatus::Fail,
            message: Some(message.into()),
        }
    }
}

#[derive(Debug, Serialize)]
pub(super) struct EnvReport {
    pub(super) name: String,
    /// `true` when every check is Pass or Skip.
    pub(super) healthy: bool,
    /// Python version pulled from `.scoop-metadata.json` (null if metadata is
    /// missing or invalid).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) python: Option<String>,
    pub(super) checks: Vec<CheckResult>,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub(super) struct Summary {
    pub(super) total: usize,
    /// All checks Pass or Skip (no Warn, no Fail). The "perfect" bucket.
    pub(super) healthy: usize,
    /// At least one Warn check but no Fail. Informational drift (e.g.
    /// `.scuv.toml` python version mismatch) lives here — the env still
    /// works, the user should just know about it. Does NOT trigger
    /// `--strict` exit.
    pub(super) warnings: usize,
    /// At least one Fail check. Real breakage. `--strict` exits non-zero
    /// when this is > 0.
    pub(super) issues: usize,
}

#[derive(Debug, Serialize)]
pub(super) struct VerifyData {
    pub(super) envs: Vec<EnvReport>,
    pub(super) summary: Summary,
}

/// Bucket each report into exactly one of healthy / warnings / issues so
/// the three counts sum to `total`. `--strict` reads `summary.issues`
/// only — Warn-only envs (e.g. manifest drift) must NOT trip the exit.
pub(super) fn compute_summary(reports: &[EnvReport]) -> Summary {
    let mut healthy = 0usize;
    let mut warnings = 0usize;
    let mut issues = 0usize;
    for r in reports {
        let has_fail = r.checks.iter().any(|c| c.status == CheckStatus::Fail);
        let has_warn = r.checks.iter().any(|c| c.status == CheckStatus::Warn);
        if has_fail {
            issues += 1;
        } else if has_warn {
            warnings += 1;
        } else {
            healthy += 1;
        }
    }
    Summary {
        total: reports.len(),
        healthy,
        warnings,
        issues,
    }
}
