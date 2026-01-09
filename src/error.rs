//! Error types for scoop

use std::path::PathBuf;
use thiserror::Error;

/// Result type alias using ScoopError
pub type Result<T> = std::result::Result<T, ScoopError>;

/// Main error type for scoop
#[derive(Error, Debug)]
pub enum ScoopError {
    /// Virtual environment not found
    #[error("Virtual environment '{name}' not found")]
    VirtualenvNotFound { name: String },

    /// Virtual environment already exists
    #[error("Virtual environment '{name}' already exists")]
    VirtualenvExists { name: String },

    /// Invalid environment name
    #[error("Invalid environment name '{name}': {reason}")]
    InvalidEnvName { name: String, reason: String },

    /// Invalid Python version
    #[error("Invalid Python version '{version}'")]
    InvalidPythonVersion { version: String },

    /// uv not found
    #[error(
        "uv is not installed. Install it with: curl -LsSf https://astral.sh/uv/install.sh | sh"
    )]
    UvNotFound,

    /// uv command failed
    #[error("uv command failed: {message}")]
    UvCommandFailed { command: String, message: String },

    /// Path error
    #[error("Path error: {0}")]
    PathError(String),

    /// Home directory not found
    #[error("Could not determine home directory")]
    HomeNotFound,

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Version file not found
    #[error("No version file found in {path} or its parent directories")]
    VersionFileNotFound { path: PathBuf },

    /// Shell not supported
    #[error("Shell '{shell}' is not supported")]
    UnsupportedShell { shell: String },

    /// Python version not installed
    #[error("Python {version} is not installed. Install it with: scoop install {version}")]
    PythonNotInstalled { version: String },

    /// Python installation failed
    #[error("Failed to install Python {version}: {message}")]
    PythonInstallFailed { version: String, message: String },

    /// Python uninstallation failed
    #[error("Failed to uninstall Python {version}: {message}")]
    PythonUninstallFailed { version: String, message: String },

    /// No Python versions available
    #[error("No Python versions available matching '{pattern}'")]
    NoPythonVersions { pattern: String },

    /// Invalid argument combination
    #[error("{message}")]
    InvalidArgument { message: String },
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
        }
    }

    /// Returns a suggested fix for the error (if available)
    pub fn suggestion(&self) -> Option<String> {
        match self {
            Self::VirtualenvNotFound { name } => {
                Some(format!("Create it with: scoop create {name} 3.12"))
            }
            Self::VirtualenvExists { .. } => Some("Use --force to overwrite".to_string()),
            Self::InvalidEnvName { .. } => {
                Some("Names must start with a letter and contain only [a-zA-Z0-9_-]".to_string())
            }
            Self::UvNotFound => {
                Some("Install: curl -LsSf https://astral.sh/uv/install.sh | sh".to_string())
            }
            Self::PythonNotInstalled { version } => {
                Some(format!("Install it with: scoop install {version}"))
            }
            Self::NoPythonVersions { .. } => {
                Some("Use 'scoop list --pythons' to see available versions".to_string())
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_virtualenv_not_found_message() {
        let err = ScoopError::VirtualenvNotFound {
            name: "myenv".to_string(),
        };
        assert_eq!(err.to_string(), "Virtual environment 'myenv' not found");
    }

    #[test]
    fn test_virtualenv_exists_message() {
        let err = ScoopError::VirtualenvExists {
            name: "existing".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "Virtual environment 'existing' already exists"
        );
    }

    #[test]
    fn test_invalid_env_name_message() {
        let err = ScoopError::InvalidEnvName {
            name: "123bad".to_string(),
            reason: "must start with a letter".to_string(),
        };
        assert!(err.to_string().contains("123bad"));
        assert!(err.to_string().contains("must start with a letter"));
    }

    #[test]
    fn test_invalid_python_version_message() {
        let err = ScoopError::InvalidPythonVersion {
            version: "abc".to_string(),
        };
        assert_eq!(err.to_string(), "Invalid Python version 'abc'");
    }

    #[test]
    fn test_uv_not_found_contains_install_hint() {
        let err = ScoopError::UvNotFound;
        let msg = err.to_string();
        assert!(msg.contains("uv is not installed"));
        assert!(msg.contains("curl"));
        assert!(msg.contains("astral.sh"));
    }

    #[test]
    fn test_uv_command_failed_message() {
        let err = ScoopError::UvCommandFailed {
            command: "venv".to_string(),
            message: "Python not found".to_string(),
        };
        assert!(err.to_string().contains("uv command failed"));
        assert!(err.to_string().contains("Python not found"));
    }

    #[test]
    fn test_path_error_message() {
        let err = ScoopError::PathError("invalid UTF-8".to_string());
        assert_eq!(err.to_string(), "Path error: invalid UTF-8");
    }

    #[test]
    fn test_home_not_found_message() {
        let err = ScoopError::HomeNotFound;
        assert!(err.to_string().contains("home directory"));
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file missing");
        let err: ScoopError = io_err.into();
        assert!(matches!(err, ScoopError::Io(_)));
        assert!(err.to_string().contains("file missing"));
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
        assert!(err.to_string().contains("/some/path"));
        assert!(err.to_string().contains("parent directories"));
    }

    #[test]
    fn test_unsupported_shell_message() {
        let err = ScoopError::UnsupportedShell {
            shell: "fish".to_string(),
        };
        assert_eq!(err.to_string(), "Shell 'fish' is not supported");
    }

    #[test]
    fn test_python_not_installed_contains_hint() {
        let err = ScoopError::PythonNotInstalled {
            version: "3.13".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("3.13"));
        assert!(msg.contains("scoop install"));
    }

    #[test]
    fn test_python_install_failed_message() {
        let err = ScoopError::PythonInstallFailed {
            version: "3.12".to_string(),
            message: "network error".to_string(),
        };
        assert!(err.to_string().contains("3.12"));
        assert!(err.to_string().contains("network error"));
    }

    #[test]
    fn test_python_uninstall_failed_message() {
        let err = ScoopError::PythonUninstallFailed {
            version: "3.11".to_string(),
            message: "in use".to_string(),
        };
        assert!(err.to_string().contains("3.11"));
        assert!(err.to_string().contains("in use"));
    }

    #[test]
    fn test_no_python_versions_message() {
        let err = ScoopError::NoPythonVersions {
            pattern: "2.7".to_string(),
        };
        assert!(err.to_string().contains("2.7"));
    }

    #[test]
    fn test_invalid_argument_message() {
        let err = ScoopError::InvalidArgument {
            message: "Cannot use --stable and --latest together".to_string(),
        };
        assert_eq!(err.to_string(), "Cannot use --stable and --latest together");
    }

    #[test]
    fn test_error_is_debug() {
        let err = ScoopError::HomeNotFound;
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("HomeNotFound"));
    }

    // ==========================================================================
    // IO Error Propagation Tests
    // ==========================================================================

    #[test]
    fn test_io_error_not_found() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let err: ScoopError = io_err.into();
        assert!(matches!(err, ScoopError::Io(_)));
        assert!(err.to_string().contains("file not found"));
    }

    #[test]
    fn test_io_error_permission_denied() {
        let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "access denied");
        let err: ScoopError = io_err.into();
        assert!(matches!(err, ScoopError::Io(_)));
        assert!(err.to_string().contains("access denied"));
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
        let msg = err.to_string();
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
            let msg = err.to_string();
            // Messages should not be empty
            assert!(!msg.is_empty(), "Error message should not be empty");
            // Messages should not start with lowercase (except for special cases)
            let first_char = msg.chars().next().unwrap();
            assert!(
                first_char.is_uppercase() || first_char == 'u', // 'uv' is valid
                "Error message should start with uppercase: {}",
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
            err.to_string().contains("myenv"),
            "Error should include the env name"
        );

        let err = ScoopError::InvalidPythonVersion {
            version: "abc".to_string(),
        };
        assert!(
            err.to_string().contains("abc"),
            "Error should include the invalid version"
        );

        let err = ScoopError::UnsupportedShell {
            shell: "fish".to_string(),
        };
        assert!(
            err.to_string().contains("fish"),
            "Error should include the shell name"
        );
    }

    #[test]
    fn test_error_messages_suggest_solutions() {
        // UvNotFound should include installation instructions
        let err = ScoopError::UvNotFound;
        let msg = err.to_string();
        assert!(
            msg.contains("curl") || msg.contains("install"),
            "UvNotFound should suggest installation"
        );

        // PythonNotInstalled should suggest install command
        let err = ScoopError::PythonNotInstalled {
            version: "3.13".to_string(),
        };
        let msg = err.to_string();
        assert!(
            msg.contains("scoop install"),
            "PythonNotInstalled should suggest scoop install"
        );
    }

    #[test]
    fn test_error_messages_no_sensitive_info() {
        // Ensure error messages don't leak sensitive paths or info
        let err = ScoopError::PathError("test path error".to_string());
        let msg = err.to_string();
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
        let msg = err.to_string();
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
        let msg = err.to_string();
        assert!(msg.contains("failed"), "Should indicate failure");
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
        let msg = err.to_string();
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
        let suggestion = err.suggestion().unwrap();
        assert!(suggestion.contains("myenv"));
        assert!(suggestion.contains("scoop create"));
    }

    #[test]
    fn test_suggestion_virtualenv_exists() {
        let err = ScoopError::VirtualenvExists {
            name: "existing".into(),
        };
        let suggestion = err.suggestion().unwrap();
        assert!(suggestion.contains("--force"));
    }

    #[test]
    fn test_suggestion_invalid_env_name() {
        let err = ScoopError::InvalidEnvName {
            name: "123".into(),
            reason: "must start with letter".into(),
        };
        let suggestion = err.suggestion().unwrap();
        assert!(suggestion.contains("letter"));
        assert!(suggestion.contains("[a-zA-Z0-9_-]"));
    }

    #[test]
    fn test_suggestion_uv_not_found() {
        let err = ScoopError::UvNotFound;
        let suggestion = err.suggestion().unwrap();
        assert!(suggestion.contains("curl"));
        assert!(suggestion.contains("astral.sh"));
    }

    #[test]
    fn test_suggestion_python_not_installed_includes_version() {
        let err = ScoopError::PythonNotInstalled {
            version: "3.13".into(),
        };
        let suggestion = err.suggestion().unwrap();
        assert!(suggestion.contains("3.13"));
        assert!(suggestion.contains("scoop install"));
    }

    #[test]
    fn test_suggestion_no_python_versions() {
        let err = ScoopError::NoPythonVersions {
            pattern: "2.7".into(),
        };
        let suggestion = err.suggestion().unwrap();
        assert!(suggestion.contains("scoop list --pythons"));
    }

    #[test]
    fn test_no_suggestion_for_io_error() {
        let err = ScoopError::Io(io::Error::other("test"));
        assert!(err.suggestion().is_none());
    }

    #[test]
    fn test_no_suggestion_for_json_error() {
        let json_err: serde_json::Error =
            serde_json::from_str::<serde_json::Value>("invalid").expect_err("should fail");
        let err: ScoopError = json_err.into();
        assert!(err.suggestion().is_none());
    }

    #[test]
    fn test_no_suggestion_for_uv_command_failed() {
        let err = ScoopError::UvCommandFailed {
            command: "venv".into(),
            message: "failed".into(),
        };
        assert!(err.suggestion().is_none());
    }

    #[test]
    fn test_no_suggestion_for_path_error() {
        let err = ScoopError::PathError("invalid path".into());
        assert!(err.suggestion().is_none());
    }

    #[test]
    fn test_no_suggestion_for_home_not_found() {
        let err = ScoopError::HomeNotFound;
        assert!(err.suggestion().is_none());
    }

    #[test]
    fn test_no_suggestion_for_version_file_not_found() {
        let err = ScoopError::VersionFileNotFound {
            path: PathBuf::from("/project"),
        };
        assert!(err.suggestion().is_none());
    }

    #[test]
    fn test_no_suggestion_for_unsupported_shell() {
        let err = ScoopError::UnsupportedShell {
            shell: "fish".into(),
        };
        assert!(err.suggestion().is_none());
    }

    #[test]
    fn test_no_suggestion_for_python_install_failed() {
        let err = ScoopError::PythonInstallFailed {
            version: "3.12".into(),
            message: "network error".into(),
        };
        assert!(err.suggestion().is_none());
    }

    #[test]
    fn test_no_suggestion_for_python_uninstall_failed() {
        let err = ScoopError::PythonUninstallFailed {
            version: "3.11".into(),
            message: "in use".into(),
        };
        assert!(err.suggestion().is_none());
    }

    #[test]
    fn test_no_suggestion_for_invalid_python_version() {
        let err = ScoopError::InvalidPythonVersion {
            version: "abc".into(),
        };
        assert!(err.suggestion().is_none());
    }

    #[test]
    fn test_no_suggestion_for_invalid_argument() {
        let err = ScoopError::InvalidArgument {
            message: "conflicting flags".into(),
        };
        assert!(err.suggestion().is_none());
    }
}
