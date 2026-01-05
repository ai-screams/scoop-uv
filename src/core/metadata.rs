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

    // ==========================================================================
    // Version Compatibility Tests
    // ==========================================================================

    #[test]
    fn test_metadata_backwards_compatibility_minimal() {
        // Simulate a minimal v1 metadata file (only required fields)
        let v1_json = r#"{
            "name": "oldenv",
            "python_version": "3.11",
            "created_at": "2024-01-01T00:00:00Z",
            "created_by": "scoop 0.1.0",
            "uv_version": null
        }"#;

        let meta: Metadata = serde_json::from_str(v1_json).expect("should parse v1 metadata");
        assert_eq!(meta.name, "oldenv");
        assert_eq!(meta.python_version, "3.11");
        assert_eq!(meta.uv_version, None);
    }

    #[test]
    fn test_metadata_backwards_compatibility_with_uv_version() {
        // Metadata with uv_version field
        let json = r#"{
            "name": "withuvenv",
            "python_version": "3.12",
            "created_at": "2024-06-15T12:30:00Z",
            "created_by": "scoop 0.2.0",
            "uv_version": "0.4.5"
        }"#;

        let meta: Metadata = serde_json::from_str(json).expect("should parse metadata");
        assert_eq!(meta.uv_version, Some("0.4.5".to_string()));
    }

    #[test]
    fn test_metadata_malformed_json_handling() {
        // Missing required field
        let invalid_json = r#"{"name": "test"}"#;
        let result: Result<Metadata, _> = serde_json::from_str(invalid_json);
        assert!(result.is_err(), "should fail on missing required fields");

        // Invalid date format
        let invalid_date = r#"{
            "name": "test",
            "python_version": "3.12",
            "created_at": "not-a-date",
            "created_by": "scoop",
            "uv_version": null
        }"#;
        let result: Result<Metadata, _> = serde_json::from_str(invalid_date);
        assert!(result.is_err(), "should fail on invalid date format");

        // Empty JSON
        let result: Result<Metadata, _> = serde_json::from_str("{}");
        assert!(result.is_err(), "should fail on empty JSON");

        // Null values for required fields
        let null_name = r#"{
            "name": null,
            "python_version": "3.12",
            "created_at": "2024-01-01T00:00:00Z",
            "created_by": "scoop",
            "uv_version": null
        }"#;
        let result: Result<Metadata, _> = serde_json::from_str(null_name);
        assert!(result.is_err(), "should fail on null required field");
    }

    #[test]
    fn test_metadata_extra_fields_ignored() {
        // Future versions might add new fields - they should be ignored
        let future_json = r#"{
            "name": "futureenv",
            "python_version": "3.15",
            "created_at": "2025-01-01T00:00:00Z",
            "created_by": "scoop 1.0.0",
            "uv_version": "1.0.0",
            "new_field": "should be ignored",
            "another_field": 42
        }"#;

        let meta: Metadata =
            serde_json::from_str(future_json).expect("should parse with extra fields");
        assert_eq!(meta.name, "futureenv");
        assert_eq!(meta.python_version, "3.15");
    }

    #[test]
    fn test_metadata_unicode_values() {
        // Unicode in values should work
        let meta = Metadata::new(
            "testenv".to_string(),
            "3.12".to_string(),
            Some("uv 한글".to_string()), // Korean characters in uv version note
        );

        let json = serde_json::to_string(&meta).expect("should serialize");
        let restored: Metadata = serde_json::from_str(&json).expect("should deserialize");
        assert_eq!(restored.uv_version, Some("uv 한글".to_string()));
    }
}
