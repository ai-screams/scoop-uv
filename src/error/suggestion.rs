//! Locale-aware user-facing fix hints for [`ScoopError`].
//!
//! [`ScoopError::suggestion_in`] is the locale-explicit primary, used by
//! both `--json` output (machine-readable hint) and human formatting via
//! `main.rs`. [`ScoopError::suggestion`] delegates to the current locale.
//!
//! Variants that genuinely have no actionable suggestion (e.g. raw IO
//! errors, JSON parse failures) return `None` so callers can omit the
//! hint line entirely instead of rendering an empty bullet.

use rust_i18n::t;

use super::ScoopError;

impl ScoopError {
    /// Returns a suggested fix for the error in an explicit `locale`.
    ///
    /// Locale-explicit sibling of [`suggestion`](Self::suggestion); use it in
    /// tests to assert hint text without depending on the process-global locale.
    pub fn suggestion_in(&self, locale: &str) -> Option<String> {
        match self {
            Self::VirtualenvNotFound { name } => Some(
                t!(
                    "suggestion.virtualenv_not_found",
                    locale = locale,
                    name = name
                )
                .to_string(),
            ),
            Self::VirtualenvExists { .. } => {
                Some(t!("suggestion.virtualenv_exists", locale = locale).to_string())
            }
            Self::InvalidEnvName { .. } => {
                Some(t!("suggestion.invalid_env_name", locale = locale).to_string())
            }
            Self::UvNotFound => Some(t!("suggestion.uv_not_found", locale = locale).to_string()),
            Self::PythonNotInstalled { version } => Some(
                t!(
                    "suggestion.python_not_installed",
                    locale = locale,
                    version = version
                )
                .to_string(),
            ),
            Self::NoPythonVersions { .. } => {
                Some(t!("suggestion.no_python_versions", locale = locale).to_string())
            }
            Self::PyenvNotFound => {
                Some(t!("suggestion.pyenv_not_found", locale = locale).to_string())
            }
            Self::PyenvEnvNotFound { .. }
            | Self::VenvWrapperEnvNotFound { .. }
            | Self::CondaEnvNotFound { .. } => {
                Some(t!("suggestion.source_env_not_found", locale = locale).to_string())
            }
            Self::MigrationNameConflict { .. } => {
                Some(t!("suggestion.migration_name_conflict", locale = locale).to_string())
            }
            Self::InvalidPythonPath { .. } => {
                Some(t!("suggestion.invalid_python_path", locale = locale).to_string())
            }
            // uv-backed operations failing mid-command often trace back to an
            // unhealthy uv (missing, too old, broken PATH). Point users at the
            // one command that diagnoses all of those, since normal commands
            // don't run the version/health checks that doctor and self update do.
            Self::UvCommandFailed { .. }
            | Self::PythonInstallFailed { .. }
            | Self::PythonUninstallFailed { .. } => {
                Some(t!("suggestion.run_doctor", locale = locale).to_string())
            }
            Self::NoActiveEnvironment => {
                Some(t!("suggestion.no_active_environment", locale = locale).to_string())
            }
            Self::ExecutableNotFound { env, .. } => Some(
                t!(
                    "suggestion.executable_not_found",
                    locale = locale,
                    env = env
                )
                .to_string(),
            ),
            Self::ManifestNotFound { .. } => {
                Some(t!("suggestion.manifest_not_found", locale = locale).to_string())
            }
            Self::UnsupportedExportVersion { supported, .. } => Some(
                t!(
                    "suggestion.unsupported_export_version",
                    locale = locale,
                    supported = supported
                )
                .to_string(),
            ),
            _ => None,
        }
    }

    /// Returns a suggested fix for the error (if available), in the current locale.
    pub fn suggestion(&self) -> Option<String> {
        let locale = rust_i18n::locale();
        self.suggestion_in(&locale)
    }
}
