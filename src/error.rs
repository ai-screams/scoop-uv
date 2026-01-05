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
}
