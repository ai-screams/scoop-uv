//! Pure diff logic — no I/O, no error returns.
//!
//! This module is the core of `scuv diff`: it takes two slices of
//! [`PackageEntry`] and produces a [`PackageDiff`]. Keeping the
//! algorithm pure means it's trivially unit-tested with inline
//! fixtures and never blocked on filesystem / process state.

use std::collections::BTreeMap;

use crate::core::packages::origins_differ;

use super::types::{PackageChanged, PackageDiff, PackageEntry};

/// PEP 503 name normalisation used as the diff key; the one
/// `core::packages` matches origins with.
pub(super) use crate::core::packages::canonical_name;

/// Compute the package-set diff between two envs.
///
/// O((n+m) log(n+m)) via [`BTreeMap`] — deterministic iteration
/// order doubles as the output sort, so callers get a stable
/// alphabetic listing without an extra sort pass.
///
/// Side-A-only entries appear in `removed`; side-B-only in `added`;
/// entries whose version string or origin differs (same canonical
/// name) in `changed`; an origin-only change counts like a version one. Same canonical name with the same
/// version string is silently a no-op even if the original
/// [`PackageEntry::display_name`] differs (e.g. `requests` vs
/// `Requests`).
pub fn compute_package_diff(a: &[PackageEntry], b: &[PackageEntry]) -> PackageDiff {
    // First-wins on canonical-name collision (e.g. `Foo` and `foo` in
    // the same input). `uv pip list` doesn't emit duplicates in
    // practice, but `.collect::<BTreeMap>()` would be silently
    // last-wins — explicit `or_insert_with` keeps the behaviour
    // deterministic regardless of input order so tests and JSON
    // output stay stable. If duplicates ever do appear we'd want to
    // know about them as a separate diff signal; for now, "ignore
    // the second occurrence" is the conservative default.
    fn index(entries: &[PackageEntry]) -> BTreeMap<&str, &PackageEntry> {
        let mut map = BTreeMap::new();
        for entry in entries {
            map.entry(entry.name.as_str()).or_insert(entry);
        }
        map
    }
    let map_a = index(a);
    let map_b = index(b);

    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut changed = Vec::new();

    for (name, pkg_a) in &map_a {
        match map_b.get(name) {
            None => removed.push((*pkg_a).clone()),
            Some(pkg_b)
                if pkg_a.version != pkg_b.version
                    || origins_differ(pkg_a.source.as_ref(), pkg_b.source.as_ref()) =>
            {
                changed.push(PackageChanged {
                    name: (*name).to_string(),
                    version_a: pkg_a.version.clone(),
                    version_b: pkg_b.version.clone(),
                    source_a: pkg_a.source.clone(),
                    source_b: pkg_b.source.clone(),
                });
            }
            Some(_) => {}
        }
    }
    for (name, pkg_b) in &map_b {
        if !map_a.contains_key(name) {
            added.push((*pkg_b).clone());
        }
    }

    PackageDiff {
        added,
        removed,
        changed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pkg(name: &str, version: &str) -> PackageEntry {
        PackageEntry {
            name: canonical_name(name),
            version: version.to_string(),
            display_name: name.to_string(),
            source: None,
        }
    }

    fn editable(path: &str) -> crate::core::packages::Origin {
        crate::core::packages::Origin::Dir {
            url: format!("file://{path}"),
            editable: true,
            subdirectory: None,
        }
    }

    /// Same version, different origin: changed, with both origins. What
    /// fails this: comparing versions only.
    #[test]
    fn an_origin_only_change_is_a_change() {
        let mut local = pkg("idna", "3.4");
        local.source = Some(editable("/w/idna"));
        let diff = compute_package_diff(&[local.clone()], &[pkg("idna", "3.4")]);
        assert_eq!(diff.changed.len(), 1);
        assert_eq!(diff.changed[0].source_a, local.source);
        assert_eq!(diff.changed[0].source_b, None);
        // Same origin on both sides is no change.
        assert!(
            compute_package_diff(&[local.clone()], &[local])
                .changed
                .is_empty()
        );
    }

    /// `/repo` + `pkg` and `/repo/pkg` are one origin, so an editable
    /// subdirectory that round-trips is not reported. What fails this:
    /// comparing recorded fields instead of the identity.
    #[test]
    fn equivalent_origins_are_not_a_change() {
        let mut split = pkg("p", "1");
        split.source = Some(crate::core::packages::Origin::Dir {
            url: "file:///repo".into(),
            editable: true,
            subdirectory: Some("pkg".into()),
        });
        let mut joined = pkg("p", "1");
        joined.source = Some(editable("/repo/pkg"));
        assert!(compute_package_diff(&[split], &[joined]).changed.is_empty());
    }

    // ====== canonical_name table tests ======

    #[test]
    fn canonical_name_lowercases() {
        assert_eq!(canonical_name("Requests"), "requests");
    }

    #[test]
    fn canonical_name_collapses_separators() {
        assert_eq!(canonical_name("zope.interface"), "zope-interface");
        assert_eq!(canonical_name("flask_sqlalchemy"), "flask-sqlalchemy");
        assert_eq!(canonical_name("foo..bar"), "foo-bar");
        assert_eq!(canonical_name("foo._-bar"), "foo-bar");
    }

    #[test]
    fn canonical_name_strips_extras() {
        assert_eq!(canonical_name("requests[socks]"), "requests");
        assert_eq!(canonical_name("Django[bcrypt,argon2]"), "django");
    }

    #[test]
    fn canonical_name_passes_through_already_canonical() {
        assert_eq!(canonical_name("numpy"), "numpy");
        assert_eq!(canonical_name("pytest-cov"), "pytest-cov");
    }

    // ====== compute_package_diff tests ======

    #[test]
    fn diff_empty_inputs_produce_empty_diff() {
        let d = compute_package_diff(&[], &[]);
        assert!(d.added.is_empty() && d.removed.is_empty() && d.changed.is_empty());
    }

    #[test]
    fn diff_identical_envs_produce_empty_diff() {
        let a = vec![pkg("requests", "2.31.0"), pkg("numpy", "1.26.0")];
        let b = a.clone();
        let d = compute_package_diff(&a, &b);
        assert!(d.added.is_empty() && d.removed.is_empty() && d.changed.is_empty());
    }

    #[test]
    fn diff_detects_added() {
        let a = vec![pkg("requests", "2.31.0")];
        let b = vec![pkg("requests", "2.31.0"), pkg("pandas", "2.2.0")];
        let d = compute_package_diff(&a, &b);
        assert_eq!(d.added.len(), 1);
        assert_eq!(d.added[0].name, "pandas");
        assert!(d.removed.is_empty());
    }

    #[test]
    fn diff_detects_removed() {
        let a = vec![pkg("requests", "2.31.0"), pkg("numpy", "1.26.0")];
        let b = vec![pkg("requests", "2.31.0")];
        let d = compute_package_diff(&a, &b);
        assert_eq!(d.removed.len(), 1);
        assert_eq!(d.removed[0].name, "numpy");
        assert!(d.added.is_empty());
    }

    #[test]
    fn diff_detects_version_change() {
        let a = vec![pkg("requests", "2.31.0")];
        let b = vec![pkg("requests", "2.32.0")];
        let d = compute_package_diff(&a, &b);
        assert_eq!(d.changed.len(), 1);
        assert_eq!(d.changed[0].version_a, "2.31.0");
        assert_eq!(d.changed[0].version_b, "2.32.0");
    }

    #[test]
    fn diff_normalises_name_match() {
        // 'Requests' on a, 'requests' on b — canonical key matches.
        let a = vec![pkg("Requests", "2.31.0")];
        let b = vec![pkg("requests", "2.31.0")];
        let d = compute_package_diff(&a, &b);
        assert!(d.added.is_empty() && d.removed.is_empty() && d.changed.is_empty());
    }

    #[test]
    fn diff_treats_extras_as_same_package() {
        // requests[socks] and requests share canonical key.
        let a = vec![pkg("requests[socks]", "2.31.0")];
        let b = vec![pkg("requests", "2.31.0")];
        let d = compute_package_diff(&a, &b);
        assert!(d.added.is_empty() && d.removed.is_empty() && d.changed.is_empty());
    }

    #[test]
    fn diff_duplicate_canonical_names_first_wins() {
        // Two entries with the same canonical name on side A: the second
        // is dropped deterministically (first-wins), not silently
        // overwritten by BTreeMap last-wins. Without this, a diff
        // against a clean side could either see "no change" or
        // "version: 1.0 → 2.0" depending on input order.
        let a = vec![pkg("Foo", "1.0"), pkg("foo", "2.0")]; // both canonicalise to "foo"
        let b = vec![pkg("foo", "1.0")];
        let d = compute_package_diff(&a, &b);
        // First-wins: "1.0" is the surviving A version → matches B → no change.
        assert!(d.added.is_empty(), "no additions expected");
        assert!(d.removed.is_empty(), "no removals expected");
        assert!(d.changed.is_empty(), "first-wins kept 1.0, identical to B");
    }

    #[test]
    fn diff_mixed_add_remove_change_alphabetic() {
        let a = vec![pkg("alpha", "1.0"), pkg("beta", "2.0"), pkg("gamma", "3.0")];
        let b = vec![
            pkg("alpha", "1.0"), // unchanged
            pkg("beta", "2.1"),  // changed
            pkg("delta", "4.0"), // added
        ]; // gamma removed
        let d = compute_package_diff(&a, &b);
        assert_eq!(d.added.len(), 1);
        assert_eq!(d.added[0].name, "delta");
        assert_eq!(d.removed.len(), 1);
        assert_eq!(d.removed[0].name, "gamma");
        assert_eq!(d.changed.len(), 1);
        assert_eq!(d.changed[0].name, "beta");
        // BTreeMap iteration is alphabetic; output ordering follows.
    }
}
