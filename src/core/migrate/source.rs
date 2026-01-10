//! Source environment types and traits

use std::path::PathBuf;

use crate::error::Result;

/// Type of source tool
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceType {
    /// pyenv-virtualenv
    Pyenv,
    /// virtualenvwrapper (future)
    VirtualenvWrapper,
    /// conda (future)
    Conda,
}

impl std::fmt::Display for SourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pyenv => write!(f, "pyenv"),
            Self::VirtualenvWrapper => write!(f, "virtualenvwrapper"),
            Self::Conda => write!(f, "conda"),
        }
    }
}

/// Status of a source environment for migration
#[derive(Debug, Clone)]
pub enum EnvironmentStatus {
    /// Ready to migrate
    Ready,
    /// Name conflicts with existing scoop environment
    NameConflict { existing: PathBuf },
    /// Python version is EOL
    PythonEol { version: String },
    /// Environment is corrupted
    Corrupted { reason: String },
}

/// Information about a source environment
#[derive(Debug, Clone)]
pub struct SourceEnvironment {
    /// Environment name
    pub name: String,
    /// Python version (e.g., "3.11.0")
    pub python_version: String,
    /// Path to the environment directory
    pub path: PathBuf,
    /// Source type
    pub source_type: SourceType,
    /// Size in bytes
    pub size_bytes: u64,
    /// Migration status
    pub status: EnvironmentStatus,
}

/// Trait for discovering environments from different sources
pub trait EnvironmentSource: Send + Sync {
    /// Get the source type
    fn source_type(&self) -> SourceType;

    /// Scan all environments
    fn scan_environments(&self) -> Result<Vec<SourceEnvironment>>;

    /// Find a specific environment by name
    fn find_environment(&self, name: &str) -> Result<SourceEnvironment>;
}
