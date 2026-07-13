//! Data structures for `scuv diff`.
//!
//! Layered per Clean Architecture: this module is pure data with
//! no I/O, no error returns, and no behaviour. Compute, render, and
//! orchestration depend on these shapes; this module depends only on
//! `serde` and `std`.

use serde::Serialize;

/// Options collected from the CLI parse, forwarded into `execute`.
#[derive(Debug, Clone)]
pub struct DiffOpts {
    pub env_a: String,
    pub env_b: String,
    pub mode: DiffMode,
    pub strict: bool,
}

/// Which sections to compare.
///
/// `clap`'s `conflicts_with` on the corresponding flag pair prevents
/// the caller from constructing both `PackagesOnly` and
/// `MetadataOnly` simultaneously; the enum just makes the three
/// reachable states explicit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffMode {
    All,
    PackagesOnly,
    MetadataOnly,
}

/// One normalised package entry for diff input.
///
/// `name` is the canonical PEP 503 key used for diff matching;
/// `display_name` preserves the original spelling (case, extras,
/// underscores) so user-facing output stays readable.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PackageEntry {
    pub name: String,
    pub version: String,
    pub display_name: String,
}

/// One package that changed version between two envs.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PackageChanged {
    pub name: String,
    pub version_a: String,
    pub version_b: String,
}

/// Package-set diff: added (only in b), removed (only in a),
/// changed (in both, different version).
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PackageDiff {
    pub added: Vec<PackageEntry>,
    pub removed: Vec<PackageEntry>,
    pub changed: Vec<PackageChanged>,
}

/// Per-side comparison of an optional scalar metadata field.
///
/// **2-state contract:** each side carries `Some(value)` if the
/// value is observable on that side, `None` if it is not —
/// regardless of *why* (metadata file missing, field absent in
/// metadata, or field present-as-null). Diff intentionally does
/// not surface the cause; callers needing that signal should
/// inspect the env directly with `scuv info`.
///
/// `changed = a != b` under `Option` equality: both-`None` is not
/// a change; exactly-one-`None` is.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ScalarDiff<T: Serialize + PartialEq> {
    pub a: Option<T>,
    pub b: Option<T>,
    pub changed: bool,
}

impl<T: Serialize + PartialEq> ScalarDiff<T> {
    /// Build a diff from two optional sides, deriving `changed`.
    pub fn from_sides(a: Option<T>, b: Option<T>) -> Self {
        let changed = a != b;
        Self { a, b, changed }
    }
}

/// Comparison of every metadata field scuv tracks.
///
/// See [`ScalarDiff`] for the 2-state nullable contract that
/// applies to every field here.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct MetadataDiff {
    pub python_version: ScalarDiff<String>,
    pub created_at: ScalarDiff<String>,
    pub last_used: ScalarDiff<String>,
    pub uv_version: ScalarDiff<String>,
}

/// Top-level diff payload — what the JSON envelope's `data` carries.
#[derive(Debug, Serialize)]
pub struct DiffData {
    pub env_a: String,
    pub env_b: String,
    pub identical: bool,
    pub python: ScalarDiff<String>,
    pub packages: Option<PackageDiff>,
    pub metadata: Option<MetadataDiff>,
    pub summary: DiffSummary,
}

/// Pre-computed counts so consumers don't have to walk every nested
/// vector to know "did anything change and how much".
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub struct DiffSummary {
    pub differences: usize,
    pub python_changed: bool,
    pub packages_added: usize,
    pub packages_removed: usize,
    pub packages_changed: usize,
    pub metadata_fields_changed: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scalar_diff_both_none_is_not_changed() {
        let d: ScalarDiff<String> = ScalarDiff::from_sides(None, None);
        assert!(!d.changed);
    }

    #[test]
    fn scalar_diff_one_none_is_changed() {
        let d = ScalarDiff::from_sides(Some("x".to_string()), None);
        assert!(d.changed);
    }

    #[test]
    fn scalar_diff_same_values_is_not_changed() {
        let d = ScalarDiff::from_sides(Some("x".to_string()), Some("x".to_string()));
        assert!(!d.changed);
    }

    #[test]
    fn scalar_diff_different_values_is_changed() {
        let d = ScalarDiff::from_sides(Some("x".to_string()), Some("y".to_string()));
        assert!(d.changed);
    }
}
