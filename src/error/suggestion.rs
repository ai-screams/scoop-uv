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
            Self::MigrationSourcesNotFound { .. } => {
                Some(t!("suggestion.migration_sources_not_found", locale = locale).to_string())
            }
            _ => None,
        }
    }

    /// Returns a suggested fix for the error (if available), in the current locale.
    pub fn suggestion(&self) -> Option<String> {
        let locale = rust_i18n::locale();
        self.suggestion_in(&locale)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// MigrationSourcesNotFound must surface an install-a-source-tool
    /// hint. Pinning this kills the cargo-mutants `delete match arm`
    /// mutation (which would have collapsed the variant into the
    /// `_ => None` catchall and silently dropped the suggestion).
    #[test]
    fn migration_sources_not_found_has_install_suggestion() {
        let err_any = ScoopError::MigrationSourcesNotFound { requested: None };
        let hint_en = err_any
            .suggestion_in("en")
            .expect("must surface a suggestion");
        assert!(hint_en.contains("pyenv"));
        assert!(hint_en.contains("conda") || hint_en.contains("virtualenvwrapper"));

        let err_filtered = ScoopError::MigrationSourcesNotFound {
            requested: Some("pyenv".to_string()),
        };
        // Same suggestion key regardless of the requested-filter — the
        // user still needs to install at least one tool.
        assert!(err_filtered.suggestion_in("en").is_some());

        // All four supported locales must render a non-empty hint so a
        // locale-specific msgstr regression is caught at unit level.
        for locale in ["en", "ko", "ja", "pt-BR"] {
            let s = err_any
                .suggestion_in(locale)
                .unwrap_or_else(|| panic!("locale {locale} returned None"));
            assert!(
                !s.is_empty(),
                "locale {locale} returned empty suggestion: {s:?}"
            );
        }
    }
}
