//! Package enumeration boundary.
//!
//! [`PackageEnumerator`] is the trait that the orchestrator depends
//! on; production uses [`UvPipEnumerator`] (spawns `uv pip list`),
//! tests use a mock implementation. Keeping the trait at this seam
//! lets `execute()` be tested with deterministic fixtures.

use std::path::Path;

use crate::core::packages::read_installed_packages_with;
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

/// Production enumerator: `uv pip list --format=json` for the inventory,
/// the env's interpreter for each package's origin
/// ([`read_installed_packages_with`]).
///
/// Wraps an existing [`UvClient`] borrow rather than owning one so
/// the same client constructed at the command boundary can be
/// shared across both sides of the diff.
pub struct UvPipEnumerator<'a> {
    pub uv: &'a UvClient,
}

impl PackageEnumerator for UvPipEnumerator<'_> {
    fn list(&self, venv_path: &Path) -> Result<Vec<PackageEntry>> {
        Ok(read_installed_packages_with(self.uv, venv_path)?
            .into_iter()
            .map(|p| PackageEntry {
                name: canonical_name(&p.name),
                version: p.version,
                display_name: p.name,
                source: p.origin,
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The production enumerator carries each package's origin into the
    /// diff. What fails this: dropping `origin` when building the entry.
    #[cfg(unix)]
    #[test]
    #[serial_test::serial]
    fn uv_pip_enumerator_carries_origins() {
        let fake = crate::test_utils::FakeUv::new(&[]);
        let env = tempfile::tempdir().unwrap();
        crate::test_utils::write_fake_python(
            env.path(),
            r#"[["six","1.16.0","{\"url\":\"file:///w/six\",\"dir_info\":{\"editable\":true}}"]]"#,
        );
        let uv = UvClient::with_path(fake.exe());
        let got = UvPipEnumerator { uv: &uv }.list(env.path()).unwrap();
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].name, "six");
        assert!(
            matches!(
                got[0].source,
                Some(crate::core::packages::Origin::Dir { editable: true, .. })
            ),
            "{got:?}"
        );
    }
}
