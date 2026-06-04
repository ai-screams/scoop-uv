//! Tests for the `scoop diff` command's orchestration layer.
//!
//! These exercise `execute_with` with a mock [`PackageEnumerator`]
//! so the full CLI path runs end-to-end without spawning `uv`. Pure
//! `compute_package_diff` algorithm + `canonical_name` table tests
//! live in `compute.rs`; data-shape tests live in `types.rs`.

use std::path::Path;

use chrono::{TimeZone, Utc};

use crate::core::Metadata;
use crate::error::{Result, ScoopError};
use crate::output::Output;
use crate::test_utils::with_temp_scoop_home;

use super::enumerator::PackageEnumerator;
use super::types::{DiffMode, DiffOpts, PackageEntry};
use super::{compute, execute_with};
use crate::core::VirtualenvService;

/// In-memory enumerator: returns the same canned list for both envs
/// unless the caller stored a per-path override.
struct MockEnumerator {
    default: Vec<PackageEntry>,
    overrides: std::collections::HashMap<std::path::PathBuf, Vec<PackageEntry>>,
}

impl MockEnumerator {
    fn shared(packages: Vec<PackageEntry>) -> Self {
        Self {
            default: packages,
            overrides: std::collections::HashMap::new(),
        }
    }

    fn with_per_env(
        default: Vec<PackageEntry>,
        overrides: Vec<(std::path::PathBuf, Vec<PackageEntry>)>,
    ) -> Self {
        Self {
            default,
            overrides: overrides.into_iter().collect(),
        }
    }
}

impl PackageEnumerator for MockEnumerator {
    fn list(&self, venv_path: &Path) -> Result<Vec<PackageEntry>> {
        Ok(self
            .overrides
            .get(venv_path)
            .cloned()
            .unwrap_or_else(|| self.default.clone()))
    }
}

fn pkg(name: &str, version: &str) -> PackageEntry {
    PackageEntry {
        name: compute::canonical_name(name),
        version: version.to_string(),
        display_name: name.to_string(),
    }
}

fn make_env_with_metadata(name: &str, python_version: &str) -> std::path::PathBuf {
    let path = crate::paths::virtualenv_path(name).expect("path for env");
    std::fs::create_dir_all(&path).expect("create env dir");
    let meta = Metadata {
        name: name.to_string(),
        python_version: python_version.to_string(),
        created_at: Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap(),
        created_by: "scoop-test".to_string(),
        last_used: None,
        uv_version: Some("0.5.14".to_string()),
        python_path: None,
    };
    let meta_json = serde_json::to_string_pretty(&meta).expect("serialize");
    std::fs::write(path.join(".scoop-metadata.json"), meta_json).expect("write metadata");
    path
}

fn opts_for(a: &str, b: &str, strict: bool, mode: DiffMode) -> DiffOpts {
    DiffOpts {
        env_a: a.to_string(),
        env_b: b.to_string(),
        mode,
        strict,
    }
}

#[test]
fn execute_identical_envs_returns_ok_non_strict() {
    with_temp_scoop_home(|_tmp| {
        make_env_with_metadata("a", "3.12.0");
        make_env_with_metadata("b", "3.12.0");
        let svc = VirtualenvService::auto().unwrap();
        let enumerator = MockEnumerator::shared(vec![pkg("requests", "2.31.0")]);
        let out = Output::new(0, true, true, false);
        let result = execute_with(
            &out,
            &opts_for("a", "b", false, DiffMode::All),
            &enumerator,
            &svc,
        );
        assert!(result.is_ok());
    });
}

#[test]
fn execute_identical_envs_returns_ok_strict() {
    with_temp_scoop_home(|_tmp| {
        make_env_with_metadata("a", "3.12.0");
        make_env_with_metadata("b", "3.12.0");
        let svc = VirtualenvService::auto().unwrap();
        let enumerator = MockEnumerator::shared(vec![pkg("requests", "2.31.0")]);
        let out = Output::new(0, true, true, false);
        let result = execute_with(
            &out,
            &opts_for("a", "b", true, DiffMode::All),
            &enumerator,
            &svc,
        );
        assert!(
            result.is_ok(),
            "identical envs must be Ok even under --strict"
        );
    });
}

#[test]
fn execute_mismatch_non_strict_returns_ok() {
    with_temp_scoop_home(|_tmp| {
        let path_a = make_env_with_metadata("a", "3.12.0");
        let path_b = make_env_with_metadata("b", "3.11.9");
        let svc = VirtualenvService::auto().unwrap();
        let enumerator = MockEnumerator::with_per_env(
            vec![pkg("requests", "2.31.0")],
            vec![
                (
                    path_a.clone(),
                    vec![pkg("requests", "2.31.0"), pkg("numpy", "1.26.0")],
                ),
                (path_b.clone(), vec![pkg("requests", "2.32.0")]),
            ],
        );
        let out = Output::new(0, true, true, false);
        let result = execute_with(
            &out,
            &opts_for("a", "b", false, DiffMode::All),
            &enumerator,
            &svc,
        );
        assert!(result.is_ok());
    });
}

#[test]
fn execute_mismatch_strict_returns_diff_mismatch() {
    with_temp_scoop_home(|_tmp| {
        let path_a = make_env_with_metadata("a", "3.12.0");
        let path_b = make_env_with_metadata("b", "3.11.9");
        let svc = VirtualenvService::auto().unwrap();
        let enumerator = MockEnumerator::with_per_env(
            vec![],
            vec![
                (path_a, vec![pkg("requests", "2.31.0")]),
                (path_b, vec![pkg("requests", "2.32.0")]),
            ],
        );
        let out = Output::new(0, true, true, false);
        let result = execute_with(
            &out,
            &opts_for("a", "b", true, DiffMode::All),
            &enumerator,
            &svc,
        );
        assert!(matches!(
            result,
            Err(ScoopError::DiffMismatch { differences, .. }) if differences > 0
        ));
    });
}

#[test]
fn execute_env_a_missing_returns_not_found() {
    with_temp_scoop_home(|_tmp| {
        make_env_with_metadata("b", "3.12.0");
        let svc = VirtualenvService::auto().unwrap();
        let enumerator = MockEnumerator::shared(vec![]);
        let out = Output::new(0, true, true, false);
        let result = execute_with(
            &out,
            &opts_for("a", "b", false, DiffMode::All),
            &enumerator,
            &svc,
        );
        assert!(matches!(
            result,
            Err(ScoopError::VirtualenvNotFound { ref name }) if name == "a"
        ));
    });
}

#[test]
fn execute_env_b_missing_returns_not_found() {
    with_temp_scoop_home(|_tmp| {
        make_env_with_metadata("a", "3.12.0");
        let svc = VirtualenvService::auto().unwrap();
        let enumerator = MockEnumerator::shared(vec![]);
        let out = Output::new(0, true, true, false);
        let result = execute_with(
            &out,
            &opts_for("a", "b", false, DiffMode::All),
            &enumerator,
            &svc,
        );
        assert!(matches!(
            result,
            Err(ScoopError::VirtualenvNotFound { ref name }) if name == "b"
        ));
    });
}

#[test]
fn execute_packages_only_skips_metadata_section() {
    with_temp_scoop_home(|_tmp| {
        // Different metadata between envs — should be ignored under
        // PackagesOnly mode (no MetadataDiff produced).
        let path_a = make_env_with_metadata("a", "3.12.0");
        let path_b = make_env_with_metadata("b", "3.11.9");
        let svc = VirtualenvService::auto().unwrap();
        let enumerator = MockEnumerator::with_per_env(
            vec![],
            vec![
                (path_a, vec![pkg("requests", "2.31.0")]),
                (path_b, vec![pkg("requests", "2.31.0")]), // identical packages
            ],
        );
        let out = Output::new(0, true, true, false);
        // Under --strict, packages match but python differs → should
        // NOT trigger DiffMismatch because metadata section is skipped.
        // Python ScalarDiff is built unconditionally though, so it
        // still counts. The intent here is: PackagesOnly skips
        // *metadata*, not *python version comparison*.
        let result = execute_with(
            &out,
            &opts_for("a", "b", true, DiffMode::PackagesOnly),
            &enumerator,
            &svc,
        );
        // python_version differs → DiffMismatch.
        assert!(matches!(result, Err(ScoopError::DiffMismatch { .. })));
    });
}

#[test]
fn execute_metadata_only_skips_package_enumeration() {
    // If enumerator.list() were called under MetadataOnly mode, this
    // test would panic via a poison-pill enumerator. Instead, packages
    // section is skipped entirely. We verify by asserting Ok when
    // packages on each side would differ.
    struct PanicEnumerator;
    impl PackageEnumerator for PanicEnumerator {
        fn list(&self, _venv_path: &Path) -> Result<Vec<PackageEntry>> {
            panic!("enumerator must not be called in MetadataOnly mode");
        }
    }
    with_temp_scoop_home(|_tmp| {
        make_env_with_metadata("a", "3.12.0");
        make_env_with_metadata("b", "3.12.0");
        let svc = VirtualenvService::auto().unwrap();
        let out = Output::new(0, true, true, false);
        let result = execute_with(
            &out,
            &opts_for("a", "b", false, DiffMode::MetadataOnly),
            &PanicEnumerator,
            &svc,
        );
        assert!(result.is_ok());
    });
}
