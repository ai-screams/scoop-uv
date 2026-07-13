//! Portable JSON schema for `scuv export` / `scuv import`.
//!
//! Versioned on purpose: bumping the format requires bumping
//! [`EXPORT_SCHEMA_VERSION`] and either keeping back-compat parsing or letting
//! the import command surface a clear "unsupported version" error. The
//! `scoop_export_version` field is parsed *before* the rest of the document
//! is interpreted so the error path is reliable even for malformed payloads.

use serde::{Deserialize, Serialize};

use crate::error::{Result, ScoopError};

/// Format version embedded in every exported file. Bump on any
/// backwards-incompatible change (field removal/rename, semantic shift).
pub const EXPORT_SCHEMA_VERSION: &str = "1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExportSchema {
    pub scoop_export_version: String,
    pub environment: ExportEnvironment,
    #[serde(default)]
    pub packages: Vec<ExportPackage>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExportEnvironment {
    pub name: String,
    pub python: String,
    /// RFC 3339 timestamp from the source env's metadata, if known. Optional
    /// so hand-authored exports stay valid.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExportPackage {
    pub name: String,
    pub version: String,
}

impl ExportSchema {
    /// Build a v1 schema from raw inputs.
    pub fn new(
        name: String,
        python: String,
        created_at: Option<String>,
        packages: Vec<(String, String)>,
    ) -> Self {
        Self {
            scoop_export_version: EXPORT_SCHEMA_VERSION.to_string(),
            environment: ExportEnvironment {
                name,
                python,
                created_at,
            },
            packages: packages
                .into_iter()
                .map(|(name, version)| ExportPackage { name, version })
                .collect(),
        }
    }

    /// Validate version + env name. Run after `from_json` (or its equivalent)
    /// to surface schema-level problems before any side effects.
    pub fn validate(&self) -> Result<()> {
        if self.scoop_export_version != EXPORT_SCHEMA_VERSION {
            return Err(ScoopError::UnsupportedExportVersion {
                version: self.scoop_export_version.clone(),
                supported: EXPORT_SCHEMA_VERSION.to_string(),
            });
        }
        if !crate::validate::is_valid_env_name(&self.environment.name) {
            return Err(ScoopError::InvalidEnvName {
                name: self.environment.name.clone(),
                reason: "invalid name in export schema".to_string(),
            });
        }
        if self.environment.python.trim().is_empty() {
            return Err(ScoopError::InvalidPythonVersion {
                version: self.environment.python.clone(),
            });
        }
        Ok(())
    }

    /// Parse + validate a JSON document.
    pub fn from_json(raw: &str, source_path: &std::path::Path) -> Result<Self> {
        let schema: ExportSchema =
            serde_json::from_str(raw).map_err(|e| ScoopError::InvalidExportFile {
                path: source_path.to_path_buf(),
                reason: e.to_string(),
            })?;
        schema.validate()?;
        Ok(schema)
    }

    /// Render as pretty JSON suitable for stdout or a file.
    pub fn to_json_pretty(&self) -> String {
        // Serialization cannot fail for these owned String / Vec inputs.
        serde_json::to_string_pretty(self).expect("ExportSchema is always serializable")
    }

    /// uv pip install specs (`name==version`) so the importer can reinstall
    /// pinned versions in one shot.
    pub fn pip_specs(&self) -> Vec<String> {
        self.packages
            .iter()
            .map(|p| format!("{}=={}", p.name, p.version))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn sample_schema() -> ExportSchema {
        ExportSchema::new(
            "myproject".to_string(),
            "3.12".to_string(),
            Some("2026-05-29T12:00:00+00:00".to_string()),
            vec![
                ("pytest".to_string(), "8.0.0".to_string()),
                ("black".to_string(), "24.1.0".to_string()),
            ],
        )
    }

    #[test]
    fn new_sets_current_version_and_fields() {
        let s = sample_schema();
        assert_eq!(s.scoop_export_version, EXPORT_SCHEMA_VERSION);
        assert_eq!(s.environment.name, "myproject");
        assert_eq!(s.environment.python, "3.12");
        assert_eq!(s.packages.len(), 2);
    }

    #[test]
    fn json_roundtrip_preserves_all_fields() {
        let original = sample_schema();
        let json = original.to_json_pretty();
        let parsed = ExportSchema::from_json(&json, Path::new("test.json")).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn from_json_rejects_invalid_json() {
        let err = ExportSchema::from_json("not json", Path::new("bad.json")).unwrap_err();
        assert!(matches!(err, ScoopError::InvalidExportFile { .. }));
    }

    #[test]
    fn from_json_rejects_unsupported_version() {
        let payload = r#"{
            "scoop_export_version": "99",
            "environment": { "name": "x", "python": "3.12" },
            "packages": []
        }"#;
        let err = ExportSchema::from_json(payload, Path::new("future.json")).unwrap_err();
        assert!(matches!(err, ScoopError::UnsupportedExportVersion { .. }));
    }

    #[test]
    fn from_json_rejects_invalid_env_name() {
        let payload = r#"{
            "scoop_export_version": "1",
            "environment": { "name": "list", "python": "3.12" },
            "packages": []
        }"#;
        let err = ExportSchema::from_json(payload, Path::new("bad-name.json")).unwrap_err();
        assert!(matches!(err, ScoopError::InvalidEnvName { .. }));
    }

    #[test]
    fn from_json_rejects_empty_python() {
        let payload = r#"{
            "scoop_export_version": "1",
            "environment": { "name": "ok", "python": "" },
            "packages": []
        }"#;
        let err = ExportSchema::from_json(payload, Path::new("bad-py.json")).unwrap_err();
        assert!(matches!(err, ScoopError::InvalidPythonVersion { .. }));
    }

    #[test]
    fn from_json_accepts_packages_default_omitted() {
        // `packages` is `#[serde(default)]` — old exports that lacked the
        // field should still load as an empty list.
        let payload = r#"{
            "scoop_export_version": "1",
            "environment": { "name": "minimal", "python": "3.12" }
        }"#;
        let schema = ExportSchema::from_json(payload, Path::new("min.json")).unwrap();
        assert!(schema.packages.is_empty());
    }

    #[test]
    fn pip_specs_pins_each_package() {
        let s = sample_schema();
        assert_eq!(
            s.pip_specs(),
            vec!["pytest==8.0.0".to_string(), "black==24.1.0".to_string()]
        );
    }

    #[test]
    fn json_omits_created_at_when_none() {
        let s = ExportSchema::new("noTimestamp".to_string(), "3.12".to_string(), None, vec![]);
        let json = s.to_json_pretty();
        assert!(!json.contains("created_at"), "json was:\n{json}");
    }
}
