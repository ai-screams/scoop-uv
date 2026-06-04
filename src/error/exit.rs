//! Process exit policy for [`ScoopError`].
//!
//! Owns two policy decisions made before `main.rs` returns control to the OS:
//!
//! - [`ScoopError::exit_code`] — the numeric process exit code (0..=255).
//! - [`ScoopError::render_policy`] — whether `main.rs` should print the
//!   global `error:` prefix and suggestion, or stay quiet because the
//!   command already rendered its own output (verify report, etc.).
//!
//! Kept separate from `migrate.rs` (which owns *migration-domain*
//! exit semantics via [`super::MigrationExitCode`]) so that process-exit
//! decisions live in one place instead of being scattered across
//! per-feature modules.
//!
//! # Narrow operational policy
//!
//! `exit_code()` deliberately keeps existing operational variants
//! (`UvCommandFailed`, `UvNotFound`, `Io`, `PathError`,
//! `PythonInstallFailed`, etc.) at exit `1`. Only new probe/tool
//! failures introduced for `diff`/`audit` will get exit `2`, and
//! migration-batch failures get exit `2`/`3` per migration semantics.
//! This avoids breaking the implicit exit-1 contract that every other
//! command already follows.

use super::ScoopError;

/// How `main.rs` should render an error before exiting.
///
/// Determined per-variant by [`ScoopError::render_policy`].
///
/// `Quiet` means the command has already rendered its own output (a
/// verify report, an audit summary, a diff table) and the global
/// `error: …` prefix would be redundant noise. `Default` keeps the
/// existing behaviour of printing the error message and any suggestion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorRenderPolicy {
    /// Print `error: {msg}` + suggestion (the historical behaviour).
    Default,
    /// Suppress the global prefix — the command already wrote its own report.
    Quiet,
}

impl ScoopError {
    /// Returns the process exit code for this error.
    ///
    /// Contract:
    ///
    /// - `1` — semantic finding (verify failed) or generic operational error.
    /// - `2` — migration-batch failure / migration-internal failure
    ///   (`MigrationFailed`, `MigrationNameConflict`).
    /// - `3` — migration source-discovery error (pyenv/conda/venvwrapper
    ///   missing, corrupted source env).
    ///
    /// Diff/audit-specific exit codes are added alongside their feature
    /// variants in later increments; until those land, existing variants
    /// fall through to the catchall `1`.
    ///
    /// # Examples
    ///
    /// ```
    /// use scoop_uv::error::ScoopError;
    /// assert_eq!(ScoopError::UvNotFound.exit_code(), 1);
    /// assert_eq!(ScoopError::VerifyFailed { issues: 3 }.exit_code(), 1);
    /// assert_eq!(ScoopError::PyenvNotFound.exit_code(), 3);
    /// ```
    pub fn exit_code(&self) -> u8 {
        match self {
            // Semantic findings — the command has already rendered output.
            // Both VerifyFailed and DiffMismatch follow the same precedent:
            // exit 1 (signal "I found something") and Quiet render policy.
            Self::VerifyFailed { .. } | Self::DiffMismatch { .. } => 1,

            // Migration source-discovery (existing MigrationExitCode == 3).
            // MigrationSourcesNotFound joined this arm in Inc 4 so that
            // `scoop migrate all` on a system with no source tools
            // installed surfaces exit 3, distinct from generic exit 1.
            Self::PyenvNotFound
            | Self::PyenvEnvNotFound { .. }
            | Self::VenvWrapperEnvNotFound { .. }
            | Self::CondaEnvNotFound { .. }
            | Self::CorruptedEnvironment { .. }
            | Self::MigrationSourcesNotFound { .. } => 3,

            // Migration-internal failures (existing MigrationExitCode == 2).
            // MigrationBatchFailed is the new aggregate variant returned
            // by `migrate all` when per-env failures or unresolved name
            // conflicts (without --force) occurred.
            Self::MigrationFailed { .. }
            | Self::MigrationNameConflict { .. }
            | Self::MigrationBatchFailed { .. } => 2,

            // Narrow policy: every other operational variant exits 1 to
            // preserve the historical contract every CI script already
            // expects from scoop. New diff/audit variants will be added
            // here as their feature increments land.
            _ => 1,
        }
    }

    /// Returns the render policy for this error.
    ///
    /// `Quiet` for variants whose command already rendered a report or
    /// table, so `main.rs` should not append the global `error:` prefix.
    /// `Default` for everything else.
    ///
    /// # Examples
    ///
    /// ```
    /// use scoop_uv::error::{ErrorRenderPolicy, ScoopError};
    /// assert_eq!(
    ///     ScoopError::VerifyFailed { issues: 1 }.render_policy(),
    ///     ErrorRenderPolicy::Quiet
    /// );
    /// assert_eq!(
    ///     ScoopError::UvNotFound.render_policy(),
    ///     ErrorRenderPolicy::Default
    /// );
    /// ```
    pub fn render_policy(&self) -> ErrorRenderPolicy {
        match self {
            // The command has already rendered its full report (human
            // table or JSON envelope) before returning these variants,
            // so `main.rs` must not append the generic `error:` prefix.
            Self::VerifyFailed { .. }
            | Self::MigrationBatchFailed { .. }
            | Self::DiffMismatch { .. } => ErrorRenderPolicy::Quiet,
            _ => ErrorRenderPolicy::Default,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_failed_is_quiet_and_exits_one() {
        let err = ScoopError::VerifyFailed { issues: 2 };
        assert_eq!(err.exit_code(), 1);
        assert_eq!(err.render_policy(), ErrorRenderPolicy::Quiet);
    }

    #[test]
    fn source_discovery_variants_exit_three() {
        let cases = [
            ScoopError::PyenvNotFound,
            ScoopError::PyenvEnvNotFound {
                name: "x".to_string(),
            },
            ScoopError::VenvWrapperEnvNotFound {
                name: "x".to_string(),
            },
            ScoopError::CondaEnvNotFound {
                name: "x".to_string(),
            },
            ScoopError::CorruptedEnvironment {
                name: "x".to_string(),
                reason: "y".to_string(),
            },
        ];
        for err in cases {
            assert_eq!(err.exit_code(), 3, "{err:?} should exit 3");
            assert_eq!(err.render_policy(), ErrorRenderPolicy::Default);
        }
    }

    #[test]
    fn migration_internal_failures_exit_two() {
        use std::path::PathBuf;
        let cases = [
            ScoopError::MigrationFailed {
                reason: "boom".to_string(),
            },
            ScoopError::MigrationNameConflict {
                name: "x".to_string(),
                existing: PathBuf::from("/y"),
            },
            ScoopError::MigrationBatchFailed {
                failed_count: 2,
                conflict_count: 1,
            },
        ];
        for err in cases {
            assert_eq!(err.exit_code(), 2, "{err:?} should exit 2");
        }
    }

    #[test]
    fn migration_batch_failed_renders_quiet() {
        let err = ScoopError::MigrationBatchFailed {
            failed_count: 1,
            conflict_count: 0,
        };
        // Quiet because batch.rs prints the full per-env summary itself
        // before returning Err; the global `error:` prefix would be noise.
        assert_eq!(err.render_policy(), ErrorRenderPolicy::Quiet);
    }

    #[test]
    fn diff_mismatch_exits_one_and_renders_quiet() {
        // Same precedent as VerifyFailed: --strict opt-in to non-zero
        // exit on semantic finding; render policy Quiet because the
        // command already wrote its own report.
        let err = ScoopError::DiffMismatch {
            env_a: "a".to_string(),
            env_b: "b".to_string(),
            differences: 3,
        };
        assert_eq!(err.exit_code(), 1);
        assert_eq!(err.render_policy(), ErrorRenderPolicy::Quiet);
    }

    #[test]
    fn migration_sources_not_found_exits_three() {
        let cases = [
            ScoopError::MigrationSourcesNotFound { requested: None },
            ScoopError::MigrationSourcesNotFound {
                requested: Some("pyenv".to_string()),
            },
        ];
        for err in cases {
            assert_eq!(err.exit_code(), 3, "{err:?} should exit 3");
            // Default render — main.rs prints the localized "no source
            // tools" message and the install-a-source-tool suggestion.
            assert_eq!(err.render_policy(), ErrorRenderPolicy::Default);
        }
    }

    #[test]
    fn generic_operational_variants_exit_one() {
        let cases = [
            ScoopError::UvNotFound,
            ScoopError::UvCommandFailed {
                command: "venv".to_string(),
                message: "m".to_string(),
            },
            ScoopError::HomeNotFound,
            ScoopError::VirtualenvNotFound {
                name: "x".to_string(),
            },
            ScoopError::InvalidArgument {
                message: "m".to_string(),
            },
        ];
        for err in cases {
            assert_eq!(
                err.exit_code(),
                1,
                "{err:?} should exit 1 under narrow policy"
            );
            assert_eq!(err.render_policy(), ErrorRenderPolicy::Default);
        }
    }

    #[test]
    fn render_policy_is_default_for_non_semantic() {
        assert_eq!(
            ScoopError::UvNotFound.render_policy(),
            ErrorRenderPolicy::Default
        );
        assert_eq!(
            ScoopError::PyenvNotFound.render_policy(),
            ErrorRenderPolicy::Default
        );
    }

    #[test]
    fn render_policy_enum_is_value_type() {
        // Make sure callers can assert equality directly without &.
        let a = ErrorRenderPolicy::Quiet;
        let b = ErrorRenderPolicy::Quiet;
        assert_eq!(a, b);
        let c: ErrorRenderPolicy = a;
        assert_eq!(c, ErrorRenderPolicy::Quiet);
    }
}
