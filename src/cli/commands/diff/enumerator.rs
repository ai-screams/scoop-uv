//! Package enumeration boundary.
//!
//! [`PackageEnumerator`] is the trait that the orchestrator depends
//! on; production uses [`UvPipEnumerator`] (spawns `uv pip list`),
//! tests use a mock implementation. Keeping the trait at this seam
//! lets `execute()` be tested with deterministic fixtures.

use std::path::Path;

use crate::error::Result;
use crate::uv::UvClient;

use super::compute::canonical_name;
use super::types::PackageEntry;

/// Boundary trait for "give me the packages installed in this venv".
///
/// Production: see [`UvPipEnumerator`]. Tests: implement with an
/// in-memory `Vec<PackageEntry>` to drive `execute()` without real
/// processes.
pub trait PackageEnumerator: Send + Sync {
    /// Return packages installed in `venv_path`.
    fn list(&self, venv_path: &Path) -> Result<Vec<PackageEntry>>;
}

/// Production enumerator backed by `uv pip list --format=json`.
///
/// Wraps an existing [`UvClient`] borrow rather than owning one so
/// the same client constructed at the command boundary can be
/// shared across both sides of the diff.
pub struct UvPipEnumerator<'a> {
    pub uv: &'a UvClient,
}

impl PackageEnumerator for UvPipEnumerator<'_> {
    fn list(&self, venv_path: &Path) -> Result<Vec<PackageEntry>> {
        let raw = self.uv.pip_list(venv_path)?;
        Ok(raw
            .into_iter()
            .map(|e| PackageEntry {
                name: canonical_name(&e.name),
                version: e.version,
                display_name: e.name,
            })
            .collect())
    }
}
