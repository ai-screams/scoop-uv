//! Portable JSON schema for `scuv export` / `scuv import`.
//!
//! Versioned on purpose: a change in what the file means bumps
//! [`EXPORT_SCHEMA_VERSION`], and [`ExportSchema::from_json`] reads
//! `scoop_export_version` first, then parses the rest as that version, so an
//! unsupported version is reported as such even when the rest does not fit.
//!
//! - `"1"`: packages are `{name, version}`, reinstalled as `name==version`.
//! - `"2"`: every package also has `source`, its PEP 610 origin object or
//!   `null` (no recorded direct origin, which does not mean "from PyPI").
//!   scuv 0.21 and older reject version 2 instead of dropping `source`.

use serde::{Deserialize, Serialize};

use crate::core::packages::{InstalledPackage, Origin};
use crate::error::{Result, ScoopError};

/// Format version written by `scuv export`.
pub const EXPORT_SCHEMA_VERSION: &str = "2";

/// Every version `scuv import` reads.
pub const SUPPORTED_EXPORT_VERSIONS: &[&str] = &["1", "2"];

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

/// One package. `source` is required in version 2 (`null` allowed) and
/// absent in version 1, which reads as `None`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExportPackage {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub source: Option<Origin>,
}

/// Just the version field, read before the rest.
#[derive(Deserialize)]
struct VersionHeader {
    scoop_export_version: String,
}

impl ExportSchema {
    /// Builds a current-version schema from an env's packages.
    pub fn new(
        name: String,
        python: String,
        created_at: Option<String>,
        packages: Vec<InstalledPackage>,
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
                .map(|p| ExportPackage {
                    name: p.name,
                    version: p.version,
                    source: p.origin,
                })
                .collect(),
        }
    }

    /// Validate version + env name. Run after `from_json` (or its equivalent)
    /// to surface schema-level problems before any side effects.
    ///
    /// # Errors
    ///
    /// [`ScoopError::UnsupportedExportVersion`],
    /// [`ScoopError::InvalidEnvName`] or
    /// [`ScoopError::InvalidPythonVersion`].
    pub fn validate(&self) -> Result<()> {
        check_version(&self.scoop_export_version)?;
        if !crate::validate::is_valid_existing_env_name(&self.environment.name) {
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

    /// Parses and validates a JSON document: the version first, then the
    /// rest as that version.
    ///
    /// # Errors
    ///
    /// [`ScoopError::InvalidExportFile`] for JSON that does not parse, has
    /// no version, does not fit its version, or (version 2) has a package
    /// without `source`; [`ScoopError::UnsupportedExportVersion`] for any
    /// other version; and the errors of [`validate`](Self::validate).
    pub fn from_json(raw: &str, source_path: &std::path::Path) -> Result<Self> {
        let invalid = |reason: String| ScoopError::InvalidExportFile {
            path: source_path.to_path_buf(),
            reason,
        };
        let value: serde_json::Value =
            serde_json::from_str(raw).map_err(|e| invalid(e.to_string()))?;
        let header: VersionHeader =
            serde_json::from_value(value.clone()).map_err(|e| invalid(e.to_string()))?;
        check_version(&header.scoop_export_version)?;
        // The key's presence decides, not its value: version 2 needs it on
        // every package (`null` for the registry), version 1 has none.
        let packages = value.get("packages").and_then(|p| p.as_array());
        let mut with_source = packages
            .into_iter()
            .flatten()
            .map(|p| p.get("source").is_some());
        if header.scoop_export_version == "2" {
            if let Some(missing) = with_source.position(|has| !has) {
                return Err(invalid(format!(
                    "package {} has no \"source\"",
                    missing + 1
                )));
            }
        } else if with_source.any(|has| has) {
            return Err(invalid(
                "version 1 has no \"source\"; use version 2".to_string(),
            ));
        }
        let schema: ExportSchema =
            serde_json::from_value(value).map_err(|e| invalid(e.to_string()))?;
        schema.validate()?;
        Ok(schema)
    }

    /// Render as pretty JSON suitable for stdout or a file. A version 1
    /// schema (read from a version 1 file) is written without `source`, so
    /// [`from_json`](Self::from_json) reads it back.
    pub fn to_json_pretty(&self) -> String {
        // Serialization cannot fail for these owned String / Vec inputs.
        if self.scoop_export_version != "1" {
            return serde_json::to_string_pretty(self)
                .expect("ExportSchema is always serializable");
        }
        // Only here through a `Value`: without serde_json's `preserve_order`
        // that sorts the keys, which a version 2 export must not change.
        let mut value = serde_json::to_value(self).expect("ExportSchema is always serializable");
        for package in value["packages"]
            .as_array_mut()
            .into_iter()
            .flatten()
            .filter_map(|p| p.as_object_mut())
        {
            package.remove("source");
        }
        serde_json::to_string_pretty(&value).expect("a JSON value is always serializable")
    }

    /// The packages as [`InstalledPackage`]s, for
    /// [`InstallPlan`](crate::core::packages::InstallPlan).
    pub fn installed_packages(&self) -> Vec<InstalledPackage> {
        self.packages
            .iter()
            .map(|p| InstalledPackage {
                name: p.name.clone(),
                version: p.version.clone(),
                origin: p.source.clone(),
            })
            .collect()
    }
}

fn check_version(version: &str) -> Result<()> {
    if SUPPORTED_EXPORT_VERSIONS.contains(&version) {
        return Ok(());
    }
    Err(ScoopError::UnsupportedExportVersion {
        version: version.to_string(),
        supported: SUPPORTED_EXPORT_VERSIONS.join(", "),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn package(name: &str, version: &str, origin: Option<Origin>) -> InstalledPackage {
        InstalledPackage {
            name: name.to_string(),
            version: version.to_string(),
            origin,
        }
    }

    fn editable(path: &str) -> Origin {
        Origin::Dir {
            url: format!("file://{path}"),
            editable: true,
            subdirectory: None,
        }
    }

    fn sample_schema() -> ExportSchema {
        ExportSchema::new(
            "myproject".to_string(),
            "3.12".to_string(),
            Some("2026-05-29T12:00:00+00:00".to_string()),
            vec![
                package("pytest", "8.0.0", None),
                package("black", "24.1.0", Some(editable("/work/black"))),
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
            "environment": { "name": "../escape", "python": "3.12" },
            "packages": []
        }"#;
        let err = ExportSchema::from_json(payload, Path::new("bad-name.json")).unwrap_err();
        assert!(matches!(err, ScoopError::InvalidEnvName { .. }));
    }

    /// A file exported from an env named before its word was reserved
    /// still loads; `import` checks the name of the env it creates.
    #[test]
    fn from_json_accepts_a_reserved_env_name() {
        let payload = r#"{
            "scoop_export_version": "1",
            "environment": { "name": "list", "python": "3.12" },
            "packages": []
        }"#;
        let schema = ExportSchema::from_json(payload, Path::new("list.json")).unwrap();
        assert_eq!(schema.environment.name, "list");
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

    /// Version 2 writes `source` for every package, `null` included.
    #[test]
    fn new_writes_version_2_with_a_source_for_every_package() {
        let json: serde_json::Value =
            serde_json::from_str(&sample_schema().to_json_pretty()).unwrap();
        assert_eq!(json["scoop_export_version"], "2");
        assert!(json["packages"][0]["source"].is_null());
        assert_eq!(
            json["packages"][1]["source"],
            serde_json::json!({"url": "file:///work/black", "dir_info": {"editable": true}})
        );
    }

    #[test]
    fn installed_packages_carry_the_origins() {
        let got = sample_schema().installed_packages();
        assert_eq!(got[0], package("pytest", "8.0.0", None));
        assert_eq!(got[1].origin, Some(editable("/work/black")));
    }

    /// Version 1 still loads, its packages without an origin.
    #[test]
    fn from_json_reads_version_1() {
        let payload = r#"{
            "scoop_export_version": "1",
            "environment": { "name": "old", "python": "3.12" },
            "packages": [{ "name": "six", "version": "1.16.0" }]
        }"#;
        let schema = ExportSchema::from_json(payload, Path::new("v1.json")).unwrap();
        assert_eq!(
            schema.installed_packages(),
            [package("six", "1.16.0", None)]
        );
    }

    /// A version 2 package must say where it came from, even if `null`.
    /// What fails this: treating a missing `source` as `null`.
    #[test]
    fn from_json_refuses_a_version_2_package_without_source() {
        let payload = r#"{
            "scoop_export_version": "2",
            "environment": { "name": "x", "python": "3.12" },
            "packages": [{ "name": "six", "version": "1.16.0", "source": null }, { "name": "p", "version": "1" }]
        }"#;
        let err = ExportSchema::from_json(payload, Path::new("v2.json")).unwrap_err();
        assert!(
            matches!(err, ScoopError::InvalidExportFile { ref reason, .. } if reason.contains("package 2")),
            "{err:?}"
        );
    }

    /// A version 1 file written back stays a valid version 1 file. What
    /// fails this: writing `"source": null`, which version 1 refuses.
    #[test]
    fn a_version_1_schema_round_trips() {
        let payload = r#"{
            "scoop_export_version": "1",
            "environment": { "name": "x", "python": "3.12" },
            "packages": [{ "name": "six", "version": "1.16.0" }]
        }"#;
        let schema = ExportSchema::from_json(payload, Path::new("v1.json")).unwrap();
        let written = schema.to_json_pretty();
        assert!(!written.contains("source"), "{written}");
        assert_eq!(
            ExportSchema::from_json(&written, Path::new("v1.json")).unwrap(),
            schema
        );
    }

    /// A version 1 file cannot carry origins: that is what version 2 is for.
    /// The key is refused even as `null`. What fails this: testing the
    /// parsed value instead of the key.
    #[rstest::rstest]
    #[case::origin(r#"{"url": "file:///p", "dir_info": {}}"#)]
    #[case::null("null")]
    fn from_json_refuses_a_source_in_version_1(#[case] source: &str) {
        let payload = format!(
            r#"{{
            "scoop_export_version": "1",
            "environment": {{ "name": "x", "python": "3.12" }},
            "packages": [{{ "name": "p", "version": "1", "source": {source} }}]
        }}"#
        );
        let err = ExportSchema::from_json(&payload, Path::new("v1.json")).unwrap_err();
        assert!(
            matches!(err, ScoopError::InvalidExportFile { ref reason, .. } if reason.contains("version 1")),
            "{err:?}"
        );
    }

    /// The version is read first: an unknown version is reported as such
    /// even when the rest is not a version 1 or 2 document. What fails
    /// this: deserialising everything before looking at the version.
    #[test]
    fn from_json_reports_an_unknown_version_before_the_shape() {
        let payload = r#"{ "scoop_export_version": "3", "environment": 42 }"#;
        let err = ExportSchema::from_json(payload, Path::new("v3.json")).unwrap_err();
        assert!(
            matches!(err, ScoopError::UnsupportedExportVersion { ref supported, .. } if supported == "1, 2"),
            "{err:?}"
        );
    }

    #[test]
    fn from_json_refuses_a_malformed_source() {
        let payload = r#"{
            "scoop_export_version": "2",
            "environment": { "name": "x", "python": "3.12" },
            "packages": [{ "name": "p", "version": "1", "source": {"url": "file:///p"} }]
        }"#;
        let err = ExportSchema::from_json(payload, Path::new("v2.json")).unwrap_err();
        assert!(
            matches!(err, ScoopError::InvalidExportFile { .. }),
            "{err:?}"
        );
    }

    #[test]
    fn json_omits_created_at_when_none() {
        let s = ExportSchema::new("noTimestamp".to_string(), "3.12".to_string(), None, vec![]);
        let json = s.to_json_pretty();
        assert!(!json.contains("created_at"), "json was:\n{json}");
    }
}
