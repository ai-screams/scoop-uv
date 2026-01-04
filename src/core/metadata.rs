//! Virtualenv metadata

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Metadata for a virtual environment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    /// Name of the virtual environment
    pub name: String,

    /// Python version used
    pub python_version: String,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Version of uvenv that created this environment
    pub created_by: String,

    /// Version of uv used
    pub uv_version: Option<String>,
}

impl Metadata {
    /// Create new metadata
    pub fn new(name: String, python_version: String, uv_version: Option<String>) -> Self {
        Self {
            name,
            python_version,
            created_at: Utc::now(),
            created_by: format!("uvenv {}", env!("CARGO_PKG_VERSION")),
            uv_version,
        }
    }

    /// Metadata file name
    pub const FILE_NAME: &'static str = ".uvenv-metadata.json";
}
