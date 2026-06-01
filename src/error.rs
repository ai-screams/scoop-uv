//! Error types for scoop

use std::fmt;
use std::path::PathBuf;

use rust_i18n::t;
use serde::Serialize;
use thiserror::Error;

/// Result type alias using ScoopError
pub type Result<T> = std::result::Result<T, ScoopError>;

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

/// Main error type for scoop
#[derive(Error, Debug)]
pub enum ScoopError {
    /// Virtual environment not found
    VirtualenvNotFound { name: String },

    /// Virtual environment already exists
    VirtualenvExists { name: String },

    /// Invalid environment name
    InvalidEnvName { name: String, reason: String },

    /// Invalid Python version
    InvalidPythonVersion { version: String },

    /// uv not found
    UvNotFound,

    /// uv command failed
    UvCommandFailed { command: String, message: String },

    /// Path error
    PathError(String),

    /// Home directory not found
    HomeNotFound,

    /// IO error
    Io(#[from] std::io::Error),

    /// JSON error
    Json(#[from] serde_json::Error),

    /// Version file not found
    VersionFileNotFound { path: PathBuf },

    /// Shell not supported
    UnsupportedShell { shell: String },

    /// Python version not installed
    PythonNotInstalled { version: String },

    /// Python installation failed
    PythonInstallFailed { version: String, message: String },

    /// Python uninstallation failed
    PythonUninstallFailed { version: String, message: String },

    /// No Python versions available
    NoPythonVersions { pattern: String },

    /// Invalid argument combination
    InvalidArgument { message: String },

    /// pyenv not found
    PyenvNotFound,

    /// pyenv environment not found
    PyenvEnvNotFound { name: String },

    /// virtualenvwrapper environment not found
    VenvWrapperEnvNotFound { name: String },

    /// conda environment not found
    CondaEnvNotFound { name: String },

    /// Corrupted environment
    CorruptedEnvironment { name: String, reason: String },

    /// Package extraction failed
    PackageExtractionFailed { reason: String },

    /// Migration failed
    MigrationFailed { reason: String },

    /// Name conflict with existing scoop environment
    MigrationNameConflict { name: String, existing: PathBuf },

    /// Invalid Python path (not found, not executable, not a Python binary)
    InvalidPythonPath { path: PathBuf, reason: String },

    /// Cascade uninstall aborted by user
    CascadeAborted,

    /// `scoop self update` failed (search, install, or post-install verify).
    SelfUpdateFailed { message: String },

    /// No environment is currently active and none was specified.
    NoActiveEnvironment,

    /// Executable not found within an environment's bin directory.
    ExecutableNotFound { exe: String, env: String },

    /// `.scoop.toml` could not be located walking up from `start_dir`.
    ManifestNotFound { start_dir: PathBuf },

    /// Export file failed to parse / load (invalid JSON or schema mismatch).
    InvalidExportFile { path: PathBuf, reason: String },

    /// Export file's `scoop_export_version` is not one this binary supports.
    UnsupportedExportVersion { version: String, supported: String },
}

// ============================================================================
// Display Implementation (i18n-aware)
// ============================================================================

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
        }
    }
}

impl fmt::Display for ScoopError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let locale = rust_i18n::locale();
        write!(f, "{}", self.message_in(&locale))
    }
}

// ============================================================================
// JSON Error Support
// ============================================================================

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
        }
    }

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

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use std::io;

    #[test]
    fn test_virtualenv_not_found_message() {
        let err = ScoopError::VirtualenvNotFound {
            name: "myenv".to_string(),
        };
        assert_eq!(err.message_in("en"), "Can't find 'myenv' environment");
    }

    // ==========================================================================
    // Localized rendering (per-call locale, no global mutation → no #[serial])
    // ==========================================================================

    #[test]
    fn message_renders_in_korean() {
        let err = ScoopError::VirtualenvNotFound {
            name: "myenv".to_string(),
        };
        let ko = err.message_in("ko");
        // Structural assertion: keeps the interpolated name, and is not English.
        assert!(ko.contains("myenv"));
        assert_ne!(ko, err.message_in("en"));
    }

    /// Every supported locale renders a non-empty message that preserves the
    /// interpolated environment name. Runs in parallel — no global locale touched.
    #[rstest]
    #[case::en("en")]
    #[case::ko("ko")]
    #[case::ja("ja")]
    #[case::pt_br("pt-BR")]
    fn message_in_all_locales_keeps_name(#[case] locale: &str) {
        let err = ScoopError::VirtualenvNotFound {
            name: "proj-env".to_string(),
        };
        let msg = err.message_in(locale);
        assert!(!msg.is_empty(), "[{locale}] message must not be empty");
        assert!(msg.contains("proj-env"), "[{locale}] must keep the name");
    }

    #[test]
    fn suggestion_renders_in_korean() {
        let err = ScoopError::VirtualenvNotFound {
            name: "myenv".to_string(),
        };
        let ko = err.suggestion_in("ko").expect("has suggestion");
        assert!(ko.starts_with("→"));
        assert!(ko.contains("myenv"));
        assert_ne!(ko, err.suggestion_in("en").unwrap());
    }

    #[test]
    fn test_virtualenv_exists_message() {
        let err = ScoopError::VirtualenvExists {
            name: "existing".to_string(),
        };
        assert_eq!(err.message_in("en"), "'existing' already exists");
    }

    #[test]
    fn test_invalid_env_name_message() {
        let err = ScoopError::InvalidEnvName {
            name: "123bad".to_string(),
            reason: "must start with a letter".to_string(),
        };
        assert!(err.message_in("en").contains("123bad"));
        assert!(err.message_in("en").contains("must start with a letter"));
    }

    #[test]
    fn test_invalid_python_version_message() {
        let err = ScoopError::InvalidPythonVersion {
            version: "abc".to_string(),
        };
        assert_eq!(err.message_in("en"), "Invalid Python version: abc");
    }

    #[test]
    fn test_uv_not_found_message() {
        let err = ScoopError::UvNotFound;
        let msg = err.message_in("en");
        assert!(msg.contains("uv not found"));
        assert!(msg.contains("core engine"));
    }

    #[test]
    fn test_uv_command_failed_message() {
        let err = ScoopError::UvCommandFailed {
            command: "venv".to_string(),
            message: "Python not found".to_string(),
        };
        assert!(err.message_in("en").contains("uv venv failed"));
        assert!(err.message_in("en").contains("Python not found"));
    }

    #[test]
    fn test_path_error_message() {
        let err = ScoopError::PathError("invalid UTF-8".to_string());
        assert_eq!(err.message_in("en"), "Path error: invalid UTF-8");
    }

    #[test]
    fn test_home_not_found_message() {
        let err = ScoopError::HomeNotFound;
        assert!(err.message_in("en").contains("Can't find home directory"));
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file missing");
        let err: ScoopError = io_err.into();
        assert!(matches!(err, ScoopError::Io(_)));
        assert!(err.message_in("en").contains("file missing"));
    }

    #[test]
    fn test_json_error_conversion() {
        let json_str = "{ invalid json }";
        let json_err: serde_json::Error =
            serde_json::from_str::<serde_json::Value>(json_str).expect_err("should fail");
        let err: ScoopError = json_err.into();
        assert!(matches!(err, ScoopError::Json(_)));
    }

    #[test]
    fn test_version_file_not_found_message() {
        let err = ScoopError::VersionFileNotFound {
            path: PathBuf::from("/some/path"),
        };
        assert!(err.message_in("en").contains("/some/path"));
        assert!(err.message_in("en").contains("parent directories"));
    }

    #[test]
    fn test_unsupported_shell_message() {
        let err = ScoopError::UnsupportedShell {
            shell: "fish".to_string(),
        };
        assert_eq!(err.message_in("en"), "Shell 'fish' not supported");
    }

    #[test]
    fn test_python_not_installed_message() {
        let err = ScoopError::PythonNotInstalled {
            version: "3.13".to_string(),
        };
        let msg = err.message_in("en");
        assert!(msg.contains("3.13"));
        assert!(msg.contains("not installed"));
    }

    #[test]
    fn test_python_install_failed_message() {
        let err = ScoopError::PythonInstallFailed {
            version: "3.12".to_string(),
            message: "network error".to_string(),
        };
        assert!(err.message_in("en").contains("Couldn't install"));
        assert!(err.message_in("en").contains("3.12"));
        assert!(err.message_in("en").contains("network error"));
    }

    #[test]
    fn test_python_uninstall_failed_message() {
        let err = ScoopError::PythonUninstallFailed {
            version: "3.11".to_string(),
            message: "in use".to_string(),
        };
        assert!(err.message_in("en").contains("Couldn't uninstall"));
        assert!(err.message_in("en").contains("3.11"));
        assert!(err.message_in("en").contains("in use"));
    }

    #[test]
    fn test_no_python_versions_message() {
        let err = ScoopError::NoPythonVersions {
            pattern: "2.7".to_string(),
        };
        assert!(err.message_in("en").contains("2.7"));
    }

    #[test]
    fn test_invalid_argument_message() {
        let err = ScoopError::InvalidArgument {
            message: "Cannot use --stable and --latest together".to_string(),
        };
        assert_eq!(
            err.message_in("en"),
            "Cannot use --stable and --latest together"
        );
    }

    // ==========================================================================
    // IO Error Propagation Tests
    // ==========================================================================

    #[test]
    fn test_io_error_not_found() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let err: ScoopError = io_err.into();
        assert!(matches!(err, ScoopError::Io(_)));
        assert!(err.message_in("en").contains("file not found"));
    }

    #[test]
    fn test_io_error_permission_denied() {
        let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "access denied");
        let err: ScoopError = io_err.into();
        assert!(matches!(err, ScoopError::Io(_)));
        assert!(err.message_in("en").contains("access denied"));
    }

    #[test]
    fn test_io_error_already_exists() {
        let io_err = io::Error::new(io::ErrorKind::AlreadyExists, "file exists");
        let err: ScoopError = io_err.into();
        assert!(matches!(err, ScoopError::Io(_)));
    }

    #[test]
    fn test_io_error_preserves_kind() {
        let original = io::Error::new(io::ErrorKind::TimedOut, "operation timed out");
        let err: ScoopError = original.into();

        if let ScoopError::Io(inner) = err {
            assert_eq!(inner.kind(), io::ErrorKind::TimedOut);
        } else {
            panic!("Expected ScoopError::Io");
        }
    }

    #[test]
    fn test_json_error_details() {
        // Invalid JSON syntax
        let json_err: serde_json::Error =
            serde_json::from_str::<serde_json::Value>("{ invalid }").expect_err("should fail");
        let err: ScoopError = json_err.into();
        assert!(matches!(err, ScoopError::Json(_)));

        // The error message should contain useful info
        let msg = err.message_in("en");
        assert!(msg.contains("JSON"));
    }

    #[test]
    fn test_result_type_alias() {
        // Verify that Result<T> is an alias for std::result::Result<T, ScoopError>
        fn returns_result() -> Result<i32> {
            Ok(42)
        }

        fn returns_error() -> Result<i32> {
            Err(ScoopError::HomeNotFound)
        }

        assert_eq!(returns_result().unwrap(), 42);
        assert!(returns_error().is_err());
    }

    #[test]
    fn test_error_source_chain() {
        use std::error::Error;

        // IO error should have source
        let io_err = io::Error::new(io::ErrorKind::NotFound, "original error");
        let err: ScoopError = io_err.into();

        // ScoopError::Io wraps the original error
        if let ScoopError::Io(inner) = &err {
            assert!(inner.source().is_none()); // Simple io::Error has no source
        }

        // JSON error should also work
        let json_err: serde_json::Error =
            serde_json::from_str::<serde_json::Value>("invalid").expect_err("should fail");
        let err: ScoopError = json_err.into();
        assert!(err.source().is_some()); // JSON error has source
    }

    // ==========================================================================
    // Error Message Quality Tests
    // ==========================================================================

    #[test]
    fn test_error_messages_are_user_friendly() {
        // All error messages should be complete sentences or clear phrases
        let errors = vec![
            ScoopError::VirtualenvNotFound {
                name: "test".to_string(),
            },
            ScoopError::VirtualenvExists {
                name: "test".to_string(),
            },
            ScoopError::HomeNotFound,
            ScoopError::UvNotFound,
        ];

        for err in errors {
            let msg = err.message_in("en");
            // Messages should not be empty
            assert!(!msg.is_empty(), "Error message should not be empty");
            // Messages should start with uppercase, quote, or 'u' (for 'uv')
            let first_char = msg.chars().next().unwrap();
            assert!(
                first_char.is_uppercase() || first_char == '\'' || first_char == 'u',
                "Error message should start with uppercase, quote, or 'u': {}",
                msg
            );
        }
    }

    #[test]
    fn test_error_messages_include_context() {
        // Errors with context should include that context in the message
        let err = ScoopError::VirtualenvNotFound {
            name: "myenv".to_string(),
        };
        assert!(
            err.message_in("en").contains("myenv"),
            "Error should include the env name"
        );

        let err = ScoopError::InvalidPythonVersion {
            version: "abc".to_string(),
        };
        assert!(
            err.message_in("en").contains("abc"),
            "Error should include the invalid version"
        );

        let err = ScoopError::UnsupportedShell {
            shell: "fish".to_string(),
        };
        assert!(
            err.message_in("en").contains("fish"),
            "Error should include the shell name"
        );
    }

    #[test]
    fn test_error_suggestions_provide_hints() {
        // UvNotFound suggestion should include installation instructions
        let err = ScoopError::UvNotFound;
        let suggestion = err.suggestion_in("en").expect("should have suggestion");
        assert!(
            suggestion.contains("curl") && suggestion.contains("astral.sh"),
            "UvNotFound suggestion should include install command"
        );

        // PythonNotInstalled suggestion should suggest install command
        let err = ScoopError::PythonNotInstalled {
            version: "3.13".to_string(),
        };
        let suggestion = err.suggestion_in("en").expect("should have suggestion");
        assert!(
            suggestion.contains("scoop install") && suggestion.contains("3.13"),
            "PythonNotInstalled suggestion should include scoop install"
        );
    }

    #[test]
    fn test_error_messages_no_sensitive_info() {
        // Ensure error messages don't leak sensitive paths or info
        let err = ScoopError::PathError("test path error".to_string());
        let msg = err.message_in("en");
        // Should not contain home directory patterns
        assert!(
            !msg.contains("/Users/") && !msg.contains("/home/"),
            "Error should not leak full paths"
        );
    }

    #[test]
    fn test_invalid_env_name_provides_reason() {
        let err = ScoopError::InvalidEnvName {
            name: "123".to_string(),
            reason: "must start with a letter".to_string(),
        };
        let msg = err.message_in("en");
        assert!(msg.contains("123"), "Should include the invalid name");
        assert!(
            msg.contains("must start with a letter"),
            "Should include the reason"
        );
    }

    #[test]
    fn test_uv_command_failed_includes_details() {
        let err = ScoopError::UvCommandFailed {
            command: "venv".to_string(),
            message: "Python 3.15 not found".to_string(),
        };
        let msg = err.message_in("en");
        assert!(
            msg.contains("uv venv failed"),
            "Should indicate uv command failure"
        );
        assert!(
            msg.contains("Python 3.15 not found"),
            "Should include the error message"
        );
    }

    #[test]
    fn test_version_file_not_found_shows_path() {
        let err = ScoopError::VersionFileNotFound {
            path: PathBuf::from("/project/dir"),
        };
        let msg = err.message_in("en");
        assert!(msg.contains("/project/dir"), "Should include the path");
        assert!(
            msg.contains("parent directories"),
            "Should mention parent directory search"
        );
    }

    // ==========================================================================
    // Error Code Tests (JSON API)
    // ==========================================================================

    #[test]
    fn test_error_code_env_not_found() {
        let err = ScoopError::VirtualenvNotFound { name: "x".into() };
        assert_eq!(err.code(), "ENV_NOT_FOUND");
    }

    #[test]
    fn test_error_code_env_already_exists() {
        let err = ScoopError::VirtualenvExists { name: "x".into() };
        assert_eq!(err.code(), "ENV_ALREADY_EXISTS");
    }

    #[test]
    fn test_error_code_env_invalid_name() {
        let err = ScoopError::InvalidEnvName {
            name: "x".into(),
            reason: "r".into(),
        };
        assert_eq!(err.code(), "ENV_INVALID_NAME");
    }

    #[test]
    fn test_error_code_python_invalid_version() {
        let err = ScoopError::InvalidPythonVersion {
            version: "x".into(),
        };
        assert_eq!(err.code(), "PYTHON_INVALID_VERSION");
    }

    #[test]
    fn test_error_code_uv_not_installed() {
        let err = ScoopError::UvNotFound;
        assert_eq!(err.code(), "UV_NOT_INSTALLED");
    }

    #[test]
    fn test_error_code_uv_command_failed() {
        let err = ScoopError::UvCommandFailed {
            command: "x".into(),
            message: "m".into(),
        };
        assert_eq!(err.code(), "UV_COMMAND_FAILED");
    }

    #[test]
    fn test_error_code_io_path_error() {
        let err = ScoopError::PathError("x".into());
        assert_eq!(err.code(), "IO_PATH_ERROR");
    }

    #[test]
    fn test_error_code_io_home_not_found() {
        let err = ScoopError::HomeNotFound;
        assert_eq!(err.code(), "IO_HOME_NOT_FOUND");
    }

    #[test]
    fn test_error_code_io_error() {
        let err = ScoopError::Io(io::Error::other("test"));
        assert_eq!(err.code(), "IO_ERROR");
    }

    #[test]
    fn test_error_code_internal_json_error() {
        let json_err: serde_json::Error =
            serde_json::from_str::<serde_json::Value>("invalid").expect_err("should fail");
        let err: ScoopError = json_err.into();
        assert_eq!(err.code(), "INTERNAL_JSON_ERROR");
    }

    #[test]
    fn test_error_code_config_version_file_not_found() {
        let err = ScoopError::VersionFileNotFound {
            path: PathBuf::new(),
        };
        assert_eq!(err.code(), "CONFIG_VERSION_FILE_NOT_FOUND");
    }

    #[test]
    fn test_error_code_shell_not_supported() {
        let err = ScoopError::UnsupportedShell { shell: "x".into() };
        assert_eq!(err.code(), "SHELL_NOT_SUPPORTED");
    }

    #[test]
    fn test_error_code_python_not_installed() {
        let err = ScoopError::PythonNotInstalled {
            version: "x".into(),
        };
        assert_eq!(err.code(), "PYTHON_NOT_INSTALLED");
    }

    #[test]
    fn test_error_code_python_install_failed() {
        let err = ScoopError::PythonInstallFailed {
            version: "x".into(),
            message: "m".into(),
        };
        assert_eq!(err.code(), "PYTHON_INSTALL_FAILED");
    }

    #[test]
    fn test_error_code_python_uninstall_failed() {
        let err = ScoopError::PythonUninstallFailed {
            version: "x".into(),
            message: "m".into(),
        };
        assert_eq!(err.code(), "PYTHON_UNINSTALL_FAILED");
    }

    #[test]
    fn test_error_code_python_no_matching_version() {
        let err = ScoopError::NoPythonVersions {
            pattern: "x".into(),
        };
        assert_eq!(err.code(), "PYTHON_NO_MATCHING_VERSION");
    }

    #[test]
    fn test_error_code_arg_invalid() {
        let err = ScoopError::InvalidArgument {
            message: "x".into(),
        };
        assert_eq!(err.code(), "ARG_INVALID");
    }

    #[test]
    fn test_error_code_source_pyenv_not_found() {
        let err = ScoopError::PyenvNotFound;
        assert_eq!(err.code(), "SOURCE_PYENV_NOT_FOUND");
    }

    #[test]
    fn test_error_code_source_pyenv_env_not_found() {
        let err = ScoopError::PyenvEnvNotFound { name: "x".into() };
        assert_eq!(err.code(), "SOURCE_PYENV_ENV_NOT_FOUND");
    }

    #[test]
    fn test_error_code_source_venvwrapper_env_not_found() {
        let err = ScoopError::VenvWrapperEnvNotFound { name: "x".into() };
        assert_eq!(err.code(), "SOURCE_VENVWRAPPER_ENV_NOT_FOUND");
    }

    #[test]
    fn test_error_code_source_conda_env_not_found() {
        let err = ScoopError::CondaEnvNotFound { name: "x".into() };
        assert_eq!(err.code(), "SOURCE_CONDA_ENV_NOT_FOUND");
    }

    #[test]
    fn test_error_code_migrate_corrupted() {
        let err = ScoopError::CorruptedEnvironment {
            name: "x".into(),
            reason: "r".into(),
        };
        assert_eq!(err.code(), "MIGRATE_CORRUPTED");
    }

    #[test]
    fn test_error_code_migrate_extraction_failed() {
        let err = ScoopError::PackageExtractionFailed { reason: "x".into() };
        assert_eq!(err.code(), "MIGRATE_EXTRACTION_FAILED");
    }

    #[test]
    fn test_error_code_migrate_failed() {
        let err = ScoopError::MigrationFailed { reason: "x".into() };
        assert_eq!(err.code(), "MIGRATE_FAILED");
    }

    #[test]
    fn test_error_code_migrate_name_conflict() {
        let err = ScoopError::MigrationNameConflict {
            name: "x".into(),
            existing: PathBuf::from("/path"),
        };
        assert_eq!(err.code(), "MIGRATE_NAME_CONFLICT");
    }

    #[test]
    fn test_error_code_invalid_python_path() {
        let err = ScoopError::InvalidPythonPath {
            path: PathBuf::from("/bad/python"),
            reason: "not found".into(),
        };
        assert_eq!(err.code(), "PYTHON_INVALID_PATH");
    }

    #[test]
    fn test_invalid_python_path_message() {
        let err = ScoopError::InvalidPythonPath {
            path: PathBuf::from("/usr/bin/fake-python"),
            reason: "file not found".into(),
        };
        let msg = err.message_in("en");
        assert!(msg.contains("/usr/bin/fake-python"));
        assert!(msg.contains("file not found"));
    }

    #[test]
    fn test_invalid_python_path_suggestion() {
        let err = ScoopError::InvalidPythonPath {
            path: PathBuf::from("/bad/path"),
            reason: "not executable".into(),
        };
        let suggestion = err.suggestion_in("en").expect("should have suggestion");
        assert!(suggestion.starts_with("→"));
        assert!(suggestion.contains("Python executable"));
    }

    #[test]
    fn test_all_error_codes_are_unique() {
        use std::collections::HashSet;

        let codes: Vec<&str> = vec![
            ScoopError::VirtualenvNotFound { name: "".into() }.code(),
            ScoopError::VirtualenvExists { name: "".into() }.code(),
            ScoopError::InvalidEnvName {
                name: "".into(),
                reason: "".into(),
            }
            .code(),
            ScoopError::InvalidPythonVersion { version: "".into() }.code(),
            ScoopError::UvNotFound.code(),
            ScoopError::UvCommandFailed {
                command: "".into(),
                message: "".into(),
            }
            .code(),
            ScoopError::PathError("".into()).code(),
            ScoopError::HomeNotFound.code(),
            ScoopError::Io(io::Error::other("")).code(),
            ScoopError::VersionFileNotFound {
                path: PathBuf::new(),
            }
            .code(),
            ScoopError::UnsupportedShell { shell: "".into() }.code(),
            ScoopError::PythonNotInstalled { version: "".into() }.code(),
            ScoopError::PythonInstallFailed {
                version: "".into(),
                message: "".into(),
            }
            .code(),
            ScoopError::PythonUninstallFailed {
                version: "".into(),
                message: "".into(),
            }
            .code(),
            ScoopError::NoPythonVersions { pattern: "".into() }.code(),
            ScoopError::InvalidArgument { message: "".into() }.code(),
            // Migration error codes
            ScoopError::PyenvNotFound.code(),
            ScoopError::PyenvEnvNotFound { name: "".into() }.code(),
            ScoopError::VenvWrapperEnvNotFound { name: "".into() }.code(),
            ScoopError::CondaEnvNotFound { name: "".into() }.code(),
            ScoopError::CorruptedEnvironment {
                name: "".into(),
                reason: "".into(),
            }
            .code(),
            ScoopError::PackageExtractionFailed { reason: "".into() }.code(),
            ScoopError::MigrationFailed { reason: "".into() }.code(),
            ScoopError::MigrationNameConflict {
                name: "".into(),
                existing: PathBuf::new(),
            }
            .code(),
            ScoopError::InvalidPythonPath {
                path: PathBuf::new(),
                reason: "".into(),
            }
            .code(),
            ScoopError::CascadeAborted.code(),
        ];

        let unique: HashSet<_> = codes.iter().collect();
        assert_eq!(
            codes.len(),
            unique.len(),
            "All error codes must be unique. Found duplicates."
        );
    }

    #[test]
    fn test_error_codes_follow_naming_convention() {
        // All codes should be SCREAMING_SNAKE_CASE
        let codes = vec![
            ScoopError::VirtualenvNotFound { name: "".into() }.code(),
            ScoopError::UvNotFound.code(),
            ScoopError::HomeNotFound.code(),
            ScoopError::InvalidArgument { message: "".into() }.code(),
            // Migration error codes
            ScoopError::PyenvNotFound.code(),
            ScoopError::PyenvEnvNotFound { name: "".into() }.code(),
            ScoopError::VenvWrapperEnvNotFound { name: "".into() }.code(),
            ScoopError::CondaEnvNotFound { name: "".into() }.code(),
            ScoopError::CorruptedEnvironment {
                name: "".into(),
                reason: "".into(),
            }
            .code(),
            ScoopError::PackageExtractionFailed { reason: "".into() }.code(),
            ScoopError::MigrationFailed { reason: "".into() }.code(),
            ScoopError::MigrationNameConflict {
                name: "".into(),
                existing: PathBuf::new(),
            }
            .code(),
            ScoopError::InvalidPythonPath {
                path: PathBuf::new(),
                reason: "".into(),
            }
            .code(),
        ];

        for code in codes {
            assert!(
                code.chars().all(|c| c.is_uppercase() || c == '_'),
                "Error code '{}' should be SCREAMING_SNAKE_CASE",
                code
            );
        }
    }

    // ==========================================================================
    // Suggestion Tests (JSON API)
    // ==========================================================================

    #[test]
    fn test_suggestion_virtualenv_not_found_includes_name() {
        let err = ScoopError::VirtualenvNotFound {
            name: "myenv".into(),
        };
        let suggestion = err.suggestion_in("en").unwrap();
        assert!(suggestion.starts_with("→"));
        assert!(suggestion.contains("myenv"));
        assert!(suggestion.contains("scoop create"));
    }

    #[test]
    fn test_suggestion_virtualenv_exists() {
        let err = ScoopError::VirtualenvExists {
            name: "existing".into(),
        };
        let suggestion = err.suggestion_in("en").unwrap();
        assert!(suggestion.starts_with("→"));
        assert!(suggestion.contains("--force"));
    }

    #[test]
    fn test_suggestion_invalid_env_name() {
        let err = ScoopError::InvalidEnvName {
            name: "123".into(),
            reason: "must start with letter".into(),
        };
        let suggestion = err.suggestion_in("en").unwrap();
        assert!(suggestion.starts_with("→"));
        assert!(suggestion.contains("letter"));
    }

    #[test]
    fn test_suggestion_uv_not_found() {
        let err = ScoopError::UvNotFound;
        let suggestion = err.suggestion_in("en").unwrap();
        assert!(suggestion.starts_with("→"));
        assert!(suggestion.contains("curl"));
        assert!(suggestion.contains("astral.sh"));
    }

    #[test]
    fn test_suggestion_python_not_installed_includes_version() {
        let err = ScoopError::PythonNotInstalled {
            version: "3.13".into(),
        };
        let suggestion = err.suggestion_in("en").unwrap();
        assert!(suggestion.starts_with("→"));
        assert!(suggestion.contains("3.13"));
        assert!(suggestion.contains("scoop install"));
    }

    #[test]
    fn test_suggestion_no_python_versions() {
        let err = ScoopError::NoPythonVersions {
            pattern: "2.7".into(),
        };
        let suggestion = err.suggestion_in("en").unwrap();
        assert!(suggestion.starts_with("→"));
        assert!(suggestion.contains("scoop list --pythons"));
    }

    #[test]
    fn test_suggestion_pyenv_not_found() {
        let suggestion = ScoopError::PyenvNotFound.suggestion_in("en").unwrap();
        assert!(suggestion.starts_with("→"));
    }

    #[test]
    fn test_suggestion_source_env_not_found() {
        let err = ScoopError::PyenvEnvNotFound { name: "x".into() };
        assert!(err.suggestion_in("en").unwrap().starts_with("→"));
    }

    #[test]
    fn test_suggestion_migration_name_conflict() {
        let err = ScoopError::MigrationNameConflict {
            name: "x".into(),
            existing: PathBuf::from("/p"),
        };
        assert!(err.suggestion_in("en").unwrap().starts_with("→"));
    }

    #[test]
    fn test_suggestion_wrapper_delegates_to_current_locale() {
        // suggestion() delegates to suggestion_in(current locale); for an error
        // that has a suggestion it must not collapse to None.
        let err = ScoopError::VirtualenvNotFound {
            name: "myenv".into(),
        };
        assert!(err.suggestion().is_some());
    }

    #[test]
    fn test_no_suggestion_for_io_error() {
        let err = ScoopError::Io(io::Error::other("test"));
        assert!(err.suggestion_in("en").is_none());
    }

    #[test]
    fn test_no_suggestion_for_json_error() {
        let json_err: serde_json::Error =
            serde_json::from_str::<serde_json::Value>("invalid").expect_err("should fail");
        let err: ScoopError = json_err.into();
        assert!(err.suggestion_in("en").is_none());
    }

    #[test]
    fn test_suggestion_uv_command_failed_points_to_doctor() {
        let err = ScoopError::UvCommandFailed {
            command: "venv".into(),
            message: "failed".into(),
        };
        let suggestion = err.suggestion_in("en").unwrap();
        assert!(suggestion.starts_with("→"));
        assert!(suggestion.contains("scoop doctor"));
    }

    #[test]
    fn test_no_suggestion_for_path_error() {
        let err = ScoopError::PathError("invalid path".into());
        assert!(err.suggestion_in("en").is_none());
    }

    #[test]
    fn test_no_suggestion_for_home_not_found() {
        let err = ScoopError::HomeNotFound;
        assert!(err.suggestion_in("en").is_none());
    }

    #[test]
    fn test_no_suggestion_for_version_file_not_found() {
        let err = ScoopError::VersionFileNotFound {
            path: PathBuf::from("/project"),
        };
        assert!(err.suggestion_in("en").is_none());
    }

    #[test]
    fn test_no_suggestion_for_unsupported_shell() {
        let err = ScoopError::UnsupportedShell {
            shell: "fish".into(),
        };
        assert!(err.suggestion_in("en").is_none());
    }

    #[test]
    fn test_suggestion_python_install_failed_points_to_doctor() {
        let err = ScoopError::PythonInstallFailed {
            version: "3.12".into(),
            message: "network error".into(),
        };
        assert!(err.suggestion_in("en").unwrap().contains("scoop doctor"));
    }

    #[test]
    fn test_suggestion_python_uninstall_failed_points_to_doctor() {
        let err = ScoopError::PythonUninstallFailed {
            version: "3.11".into(),
            message: "in use".into(),
        };
        assert!(err.suggestion_in("en").unwrap().contains("scoop doctor"));
    }

    #[test]
    fn test_no_suggestion_for_invalid_python_version() {
        let err = ScoopError::InvalidPythonVersion {
            version: "abc".into(),
        };
        assert!(err.suggestion_in("en").is_none());
    }

    #[test]
    fn test_no_suggestion_for_invalid_argument() {
        let err = ScoopError::InvalidArgument {
            message: "conflicting flags".into(),
        };
        assert!(err.suggestion_in("en").is_none());
    }

    #[test]
    fn test_suggestion_no_active_environment_hints_use_or_env_flag() {
        // The hint must point the user at a concrete next step. Deleting the
        // match arm would collapse this to `None`, which the assertions catch.
        let s = ScoopError::NoActiveEnvironment.suggestion_in("en").unwrap();
        assert!(s.starts_with("→"));
        assert!(s.contains("scoop use"));
        assert!(s.contains("--env"));
    }

    #[test]
    fn test_suggestion_executable_not_found_includes_env_name() {
        let err = ScoopError::ExecutableNotFound {
            exe: "pytest".into(),
            env: "myenv".into(),
        };
        let s = err.suggestion_in("en").unwrap();
        assert!(s.starts_with("→"));
        // Must interpolate the env name so the user knows where to look.
        assert!(s.contains("myenv"));
        assert!(s.contains("scoop info"));
    }

    #[test]
    fn test_suggestion_manifest_not_found_points_at_docs() {
        // Deleting the match arm would collapse to `None`; asserting on the
        // hint content kills that mutation and pins the docs pointer.
        let err = ScoopError::ManifestNotFound {
            start_dir: PathBuf::from("/some/project"),
        };
        let s = err.suggestion_in("en").unwrap();
        assert!(s.starts_with("→"));
        assert!(s.contains("project root") || s.contains("scoop-uv"));
    }

    #[test]
    fn test_suggestion_unsupported_export_version_includes_supported_version() {
        // Pinning: deleting the match arm collapses to `None`, and the
        // suggestion must interpolate `supported` so the user knows what
        // version this binary can read.
        let err = ScoopError::UnsupportedExportVersion {
            version: "99".into(),
            supported: "1".into(),
        };
        let s = err.suggestion_in("en").unwrap();
        assert!(s.starts_with("→"));
        assert!(s.contains("'1'") || s.contains("version '1'"));
    }
}
