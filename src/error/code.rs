//! Stable error codes for JSON output.
//!
//! Each variant maps to a SCREAMING_SNAKE_CASE string. These codes are
//! part of the public JSON contract — renames are breaking changes for
//! scripts that pattern-match on them. The `test_error_codes_follow_naming_convention`
//! and `test_all_error_codes_are_unique` tests guard against drift.

use super::ScoopError;

impl ScoopError {
    /// Returns the error code for JSON output
    pub fn code(&self) -> &'static str {
        match self {
            Self::VirtualenvNotFound { .. } => "ENV_NOT_FOUND",
            Self::VirtualenvExists { .. } => "ENV_ALREADY_EXISTS",
            Self::InvalidEnvName { .. } => "ENV_INVALID_NAME",
            Self::InvalidPythonVersion { .. } => "PYTHON_INVALID_VERSION",
            Self::UvNotFound => "UV_NOT_INSTALLED",
            Self::UvCommandFailed { .. } => "UV_COMMAND_FAILED",
            Self::PathError(_) => "IO_PATH_ERROR",
            Self::HomeNotFound => "IO_HOME_NOT_FOUND",
            Self::Io(_) => "IO_ERROR",
            Self::Json(_) => "INTERNAL_JSON_ERROR",
            Self::VersionFileNotFound { .. } => "CONFIG_VERSION_FILE_NOT_FOUND",
            Self::UnsupportedShell { .. } => "SHELL_NOT_SUPPORTED",
            Self::PythonNotInstalled { .. } => "PYTHON_NOT_INSTALLED",
            Self::PythonInstallFailed { .. } => "PYTHON_INSTALL_FAILED",
            Self::PythonUninstallFailed { .. } => "PYTHON_UNINSTALL_FAILED",
            Self::NoPythonVersions { .. } => "PYTHON_NO_MATCHING_VERSION",
            Self::InvalidArgument { .. } => "ARG_INVALID",
            Self::PyenvNotFound => "SOURCE_PYENV_NOT_FOUND",
            Self::PyenvEnvNotFound { .. } => "SOURCE_PYENV_ENV_NOT_FOUND",
            Self::VenvWrapperEnvNotFound { .. } => "SOURCE_VENVWRAPPER_ENV_NOT_FOUND",
            Self::CondaEnvNotFound { .. } => "SOURCE_CONDA_ENV_NOT_FOUND",
            Self::CorruptedEnvironment { .. } => "MIGRATE_CORRUPTED",
            Self::PackageExtractionFailed { .. } => "MIGRATE_EXTRACTION_FAILED",
            Self::MigrationFailed { .. } => "MIGRATE_FAILED",
            Self::MigrationNameConflict { .. } => "MIGRATE_NAME_CONFLICT",
            Self::InvalidPythonPath { .. } => "PYTHON_INVALID_PATH",
            Self::CascadeAborted => "UNINSTALL_CASCADE_ABORTED",
            Self::SelfUpdateFailed { .. } => "SELF_UPDATE_FAILED",
            Self::NoActiveEnvironment => "NO_ACTIVE_ENV",
            Self::ExecutableNotFound { .. } => "EXE_NOT_FOUND",
            Self::ManifestNotFound { .. } => "MANIFEST_NOT_FOUND",
            Self::InvalidExportFile { .. } => "EXPORT_INVALID_FILE",
            Self::UnsupportedExportVersion { .. } => "EXPORT_UNSUPPORTED_VERSION",
            Self::VerifyFailed { .. } => "VERIFY_FAILED",
            Self::SitePackagesNotFound { .. } => "IO_SITE_PACKAGES_NOT_FOUND",
        }
    }
}
