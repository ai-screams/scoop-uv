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
    pub fn migration_exit_code(&self) -> MigrationExitCode {
        match self {
            Self::PyenvNotFound
            | Self::PyenvEnvNotFound { .. }
            | Self::VenvWrapperEnvNotFound { .. }
            | Self::CondaEnvNotFound { .. }
            | Self::CorruptedEnvironment { .. } => MigrationExitCode::SourceError,
            Self::MigrationFailed { .. } | Self::MigrationNameConflict { .. } => {
                MigrationExitCode::CompleteFailure
            }
            _ => MigrationExitCode::CompleteFailure,
        }
    }
}
