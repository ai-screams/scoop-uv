//! Migration-specific exit semantics for [`ScoopError`].
//!
//! `scoop migrate` returns different shell exit codes depending on
//! whether the failure was a source-tool problem (3), a complete
//! migration failure (2), partial success (1), or success (0).
//! [`ScoopError::migration_exit_code`] is the mapping from error variant
//! to that code; the binary inspects it in main.rs to set the exit
//! status when the migrate subcommand is the entry point.

use serde::Serialize;

use super::ScoopError;

/// Exit status for migration operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[repr(u8)]
pub enum MigrationExitCode {
    /// Complete success - all packages migrated
    Success = 0,
    /// Partial success - some packages failed to install
    PartialSuccess = 1,
    /// Complete failure - rollback occurred
    CompleteFailure = 2,
    /// Source error - source not found or corrupted
    SourceError = 3,
}

impl ScoopError {
    /// Returns the migration exit code for this error.
    ///
    /// Maps error types to appropriate exit codes for migration operations.
    /// Source-tool problems (missing pyenv install, env discovery
    /// failures, corrupted source envs) map to [`MigrationExitCode::SourceError`].
    /// Everything else — `MigrationFailed`, `MigrationNameConflict`, and
    /// any unanticipated error a migration might bubble up — collapses to
    /// [`MigrationExitCode::CompleteFailure`] via the catchall, which is
    /// why we don't enumerate them explicitly: doing so would create a
    /// mutation-testing dead arm (the catchall already covers it, so
    /// deleting the explicit arm would be an equivalent mutation no test
    /// can kill).
    pub fn migration_exit_code(&self) -> MigrationExitCode {
        match self {
            Self::PyenvNotFound
            | Self::PyenvEnvNotFound { .. }
            | Self::VenvWrapperEnvNotFound { .. }
            | Self::CondaEnvNotFound { .. }
            | Self::CorruptedEnvironment { .. } => MigrationExitCode::SourceError,
            _ => MigrationExitCode::CompleteFailure,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    // Each source-tool variant must map to SourceError so `scoop migrate`
    // can distinguish "your source pyenv/conda setup is broken" (exit 3)
    // from "the migration itself failed" (exit 2). Without these tests,
    // the cargo-mutants `--in-diff` gate flagged the SourceError arm as
    // an unkilled mutant — deleting it would still let every test pass
    // because the catchall returns CompleteFailure.
    #[test]
    fn source_tool_variants_map_to_source_error() {
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
            assert_eq!(
                err.migration_exit_code(),
                MigrationExitCode::SourceError,
                "{err:?} should map to SourceError"
            );
        }
    }

    // Migration-internal failures fall through the catchall to
    // CompleteFailure. Kept here to pin the contract; if anyone adds a
    // dedicated arm for these in the future, the test still passes.
    #[test]
    fn migration_internal_variants_map_to_complete_failure() {
        let cases = [
            ScoopError::MigrationFailed {
                reason: "boom".to_string(),
            },
            ScoopError::MigrationNameConflict {
                name: "x".to_string(),
                existing: PathBuf::from("/y"),
            },
        ];
        for err in cases {
            assert_eq!(
                err.migration_exit_code(),
                MigrationExitCode::CompleteFailure,
                "{err:?} should map to CompleteFailure"
            );
        }
    }

    // Unanticipated error types reaching migration_exit_code (e.g. a UV
    // failure mid-migration) must still map cleanly to CompleteFailure
    // rather than panic. Pins the catchall behaviour.
    #[test]
    fn unrelated_variants_use_catchall_complete_failure() {
        let err = ScoopError::UvNotFound;
        assert_eq!(
            err.migration_exit_code(),
            MigrationExitCode::CompleteFailure
        );
    }
}
