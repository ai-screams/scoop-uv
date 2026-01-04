//! Error types for uvenv

use std::path::PathBuf;
use thiserror::Error;

/// Result type alias using UvenvError
pub type Result<T> = std::result::Result<T, UvenvError>;

/// Main error type for uvenv
#[derive(Error, Debug)]
pub enum UvenvError {
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
}
