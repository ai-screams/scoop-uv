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

    /// Version of scoop that created this environment
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
            created_by: format!("scoop {}", env!("CARGO_PKG_VERSION")),
            uv_version,
        }
    }

    /// Metadata file name
    pub const FILE_NAME: &'static str = ".scoop-metadata.json";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata_new_sets_fields() {
        let meta = Metadata::new(
            "testenv".to_string(),
            "3.12".to_string(),
            Some("0.5.0".to_string()),
        );

        assert_eq!(meta.name, "testenv");
        assert_eq!(meta.python_version, "3.12");
        assert_eq!(meta.uv_version, Some("0.5.0".to_string()));
        assert!(meta.created_by.starts_with("scoop "));
    }

    #[test]
    fn test_metadata_new_without_uv_version() {
        let meta = Metadata::new("myenv".to_string(), "3.11".to_string(), None);

        assert_eq!(meta.name, "myenv");
        assert_eq!(meta.uv_version, None);
    }

    #[test]
    fn test_metadata_created_at_is_recent() {
        let before = Utc::now();
        let meta = Metadata::new("env".to_string(), "3.12".to_string(), None);
        let after = Utc::now();

        assert!(meta.created_at >= before);
        assert!(meta.created_at <= after);
    }

    #[test]
    fn test_metadata_serialization_roundtrip() {
        let meta = Metadata::new(
            "roundtrip".to_string(),
            "3.12.1".to_string(),
            Some("0.4.0".to_string()),
        );

        let json = serde_json::to_string(&meta).expect("serialize");
        let restored: Metadata = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(restored.name, meta.name);
        assert_eq!(restored.python_version, meta.python_version);
        assert_eq!(restored.uv_version, meta.uv_version);
        assert_eq!(restored.created_at, meta.created_at);
        assert_eq!(restored.created_by, meta.created_by);
    }

    #[test]
    fn test_metadata_json_format() {
        let meta = Metadata::new("jsontest".to_string(), "3.10".to_string(), None);

        let json = serde_json::to_string_pretty(&meta).expect("serialize");

        assert!(json.contains("\"name\": \"jsontest\""));
        assert!(json.contains("\"python_version\": \"3.10\""));
        assert!(json.contains("\"created_at\""));
        assert!(json.contains("\"created_by\""));
        assert!(json.contains("\"uv_version\": null"));
    }

    #[test]
    fn test_metadata_file_name_constant() {
        assert_eq!(Metadata::FILE_NAME, ".scoop-metadata.json");
        assert!(Metadata::FILE_NAME.starts_with("."));
    }

    #[test]
    fn test_metadata_clone() {
        let meta = Metadata::new("clonetest".to_string(), "3.12".to_string(), None);
        let cloned = meta.clone();

        assert_eq!(cloned.name, meta.name);
        assert_eq!(cloned.python_version, meta.python_version);
    }

    #[test]
    fn test_metadata_debug_format() {
        let meta = Metadata::new("debugtest".to_string(), "3.12".to_string(), None);
        let debug_str = format!("{:?}", meta);

        assert!(debug_str.contains("Metadata"));
        assert!(debug_str.contains("debugtest"));
    }
}
