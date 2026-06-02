//! Localized rendering for [`ScoopError`].
//!
//! [`ScoopError::message_in`] is the locale-explicit primary; [`Display`]
//! delegates to it via the rust-i18n global locale so callers that
//! `println!("{err}")` still get the right language. Tests should prefer
//! `message_in("en")` to avoid depending on the process-wide locale.

use std::fmt;

use rust_i18n::t;

use super::ScoopError;

impl ScoopError {
    /// Render the localized message in an explicit `locale`, bypassing the
    /// process-global current locale.
    ///
    /// rust-i18n stores the active locale as process-global state, so tests
    /// that assert on [`Display`] output must serialize against any test that
    /// flips the locale. Asserting via `message_in("en")` / `message_in("ko")`
    /// instead is race-free and needs no `#[serial]`.
    ///
    /// [`Display`]: std::fmt::Display
    pub fn message_in(&self, locale: &str) -> String {
        match self {
            Self::VirtualenvNotFound { name } => {
                t!("error.virtualenv_not_found", locale = locale, name = name).to_string()
            }
            Self::VirtualenvExists { name } => {
                t!("error.virtualenv_exists", locale = locale, name = name).to_string()
            }
            Self::InvalidEnvName { name, reason } => t!(
                "error.invalid_env_name",
                locale = locale,
                name = name,
                reason = reason
            )
            .to_string(),
            Self::InvalidPythonVersion { version } => t!(
                "error.invalid_python_version",
                locale = locale,
                version = version
            )
            .to_string(),
            Self::UvNotFound => t!("error.uv_not_found", locale = locale).to_string(),
            Self::UvCommandFailed { command, message } => t!(
                "error.uv_command_failed",
                locale = locale,
                command = command,
                message = message
            )
            .to_string(),
            Self::PathError(msg) => {
                t!("error.path_error", locale = locale, message = msg).to_string()
            }
            Self::HomeNotFound => t!("error.home_not_found", locale = locale).to_string(),
            Self::Io(err) => t!("error.io", locale = locale, message = err.to_string()).to_string(),
            Self::Json(err) => {
                t!("error.json", locale = locale, message = err.to_string()).to_string()
            }
            Self::VersionFileNotFound { path } => t!(
                "error.version_file_not_found",
                locale = locale,
                path = path.display()
            )
            .to_string(),
            Self::UnsupportedShell { shell } => {
                t!("error.unsupported_shell", locale = locale, shell = shell).to_string()
            }
            Self::PythonNotInstalled { version } => t!(
                "error.python_not_installed",
                locale = locale,
                version = version
            )
            .to_string(),
            Self::PythonInstallFailed { version, message } => t!(
                "error.python_install_failed",
                locale = locale,
                version = version,
                message = message
            )
            .to_string(),
            Self::PythonUninstallFailed { version, message } => t!(
                "error.python_uninstall_failed",
                locale = locale,
                version = version,
                message = message
            )
            .to_string(),
            Self::NoPythonVersions { pattern } => t!(
                "error.no_python_versions",
                locale = locale,
                pattern = pattern
            )
            .to_string(),
            Self::InvalidArgument { message } => {
                t!("error.invalid_argument", locale = locale, message = message).to_string()
            }
            Self::PyenvNotFound => t!("error.pyenv_not_found", locale = locale).to_string(),
            Self::PyenvEnvNotFound { name } => {
                t!("error.pyenv_env_not_found", locale = locale, name = name).to_string()
            }
            Self::VenvWrapperEnvNotFound { name } => t!(
                "error.venvwrapper_env_not_found",
                locale = locale,
                name = name
            )
            .to_string(),
            Self::CondaEnvNotFound { name } => {
                t!("error.conda_env_not_found", locale = locale, name = name).to_string()
            }
            Self::CorruptedEnvironment { name, reason } => t!(
                "error.corrupted_environment",
                locale = locale,
                name = name,
                reason = reason
            )
            .to_string(),
            Self::PackageExtractionFailed { reason } => t!(
                "error.package_extraction_failed",
                locale = locale,
                reason = reason
            )
            .to_string(),
            Self::MigrationFailed { reason } => {
                t!("error.migration_failed", locale = locale, reason = reason).to_string()
            }
            Self::MigrationNameConflict { name, existing } => t!(
                "error.migration_name_conflict",
                locale = locale,
                name = name,
                path = existing.display()
            )
            .to_string(),
            Self::InvalidPythonPath { path, reason } => t!(
                "error.invalid_python_path",
                locale = locale,
                path = path.display(),
                reason = reason
            )
            .to_string(),
            Self::CascadeAborted => t!("error.cascade_aborted", locale = locale).to_string(),
            Self::SelfUpdateFailed { message } => t!(
                "error.self_update_failed",
                locale = locale,
                message = message
            )
            .to_string(),
            Self::NoActiveEnvironment => {
                t!("error.no_active_environment", locale = locale).to_string()
            }
            Self::ExecutableNotFound { exe, env } => t!(
                "error.executable_not_found",
                locale = locale,
                exe = exe,
                env = env
            )
            .to_string(),
            Self::ManifestNotFound { start_dir } => t!(
                "error.manifest_not_found",
                locale = locale,
                path = start_dir.display()
            )
            .to_string(),
            Self::InvalidExportFile { path, reason } => t!(
                "error.invalid_export_file",
                locale = locale,
                path = path.display(),
                reason = reason
            )
            .to_string(),
            Self::UnsupportedExportVersion { version, supported } => t!(
                "error.unsupported_export_version",
                locale = locale,
                version = version,
                supported = supported
            )
            .to_string(),
            Self::VerifyFailed { issues } => t!(
                "error.verify_failed",
                locale = locale,
                issues = issues.to_string()
            )
            .to_string(),
        }
    }
}

impl fmt::Display for ScoopError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let locale = rust_i18n::locale();
        write!(f, "{}", self.message_in(&locale))
    }
}
