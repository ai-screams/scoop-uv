//! `.scuv.toml` project manifest — opt-in declarative env definition
//! consumed by `scuv sync`.
//!
//! Resolution walks cwd → parents; within a single directory `.scuv.toml`
//! wins over a legacy `.scoop.toml` (deprecated; emits a one-shot warning).
//! A legacy file in a *nearer* directory still beats a new-named file in a
//! parent directory — nearest-directory-first is unchanged by the rename.
//!
//! DEPRECATION(0.16.0): remove the legacy branch.
//!
//! The schema is intentionally minimal for v1:
//!
//! ```toml
//! [environment]
//! name = "myproject"
//! python = "3.12"
//!
//! [packages]
//! default = ["pytest", "black", "mypy"]
//! dev = ["ipython", "debugpy"]
//! ```
//!
//! `[hooks]` and a `python_path` field are intentionally out of scope until a
//! separate threat-model / interpreter-override design lands.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::error::{Result, ScoopError};

/// Project manifest filename — searched cwd→parent like `.scuv-version`.
pub const MANIFEST_FILE: &str = ".scuv.toml";
/// DEPRECATION(0.16.0): remove the legacy manifest-filename fallback.
pub const LEGACY_MANIFEST_FILE: &str = ".scoop.toml";

/// Parsed `.scuv.toml`. Fields are validated at parse time so callers receive
/// a struct that's already meaningful (`name` passes `is_valid_env_name`,
/// `python` is non-empty).
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ScoopManifest {
    pub environment: Environment,
    #[serde(default)]
    pub packages: Packages,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Environment {
    pub name: String,
    pub python: String,
}

/// Package groups. `default` is always installed by `scuv sync`; any extra
/// keys become opt-in groups selected via `--with <group>`.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
pub struct Packages {
    #[serde(default)]
    pub default: Vec<String>,
    /// Arbitrary named groups (e.g. `dev`, `docs`). Captured via `flatten` so
    /// the TOML shape stays one nested table instead of `[packages.groups]`.
    #[serde(flatten)]
    pub groups: BTreeMap<String, Vec<String>>,
}

impl ScoopManifest {
    /// Parse a `.scuv.toml` document from a string and validate.
    pub fn parse(content: &str) -> Result<Self> {
        let manifest: ScoopManifest =
            toml::from_str(content).map_err(|e| ScoopError::InvalidArgument {
                message: format!("{MANIFEST_FILE}: {}", e.message()),
            })?;
        manifest.validate()?;
        Ok(manifest)
    }

    /// Load and parse from a file path.
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Self::parse(&content)
    }

    /// Returns the deduplicated package list for `default` plus the named
    /// `extra_groups`, preserving first-seen order so users can predict pin
    /// resolution. Unknown group names are silently ignored — the sync
    /// command is responsible for surfacing them to the user before calling
    /// in (see [`Self::group`]).
    pub fn packages_for(&self, extra_groups: &[String]) -> Vec<String> {
        let mut all = self.packages.default.clone();
        for g in extra_groups {
            if let Some(pkgs) = self.packages.groups.get(g) {
                all.extend(pkgs.iter().cloned());
            }
        }
        let mut seen = std::collections::HashSet::new();
        all.into_iter().filter(|p| seen.insert(p.clone())).collect()
    }

    /// Look up a group by name. Returns the `default` list when asked for
    /// `"default"` so callers can validate `--with default` without special
    /// casing.
    pub fn group(&self, name: &str) -> Option<&Vec<String>> {
        if name == "default" {
            return Some(&self.packages.default);
        }
        self.packages.groups.get(name)
    }

    fn validate(&self) -> Result<()> {
        if !crate::validate::is_valid_env_name(&self.environment.name) {
            return Err(ScoopError::InvalidEnvName {
                name: self.environment.name.clone(),
                reason: format!("invalid name in {MANIFEST_FILE} [environment]"),
            });
        }
        if self.environment.python.trim().is_empty() {
            return Err(ScoopError::InvalidPythonVersion {
                version: self.environment.python.clone(),
            });
        }
        Ok(())
    }
}

/// Walk from `start` up to filesystem root looking for `.scuv.toml`, falling
/// back to the legacy `.scoop.toml` name per directory (deprecated; emits a
/// one-shot warning). Mirrors the version-file resolution model so users get
/// the same mental model for "this directory is configured" detection.
///
/// DEPRECATION(0.16.0): remove the legacy fallback branch.
pub fn find_manifest(start: &Path) -> Option<PathBuf> {
    let mut current = start.to_path_buf();
    loop {
        let candidate = current.join(MANIFEST_FILE);
        if candidate.is_file() {
            return Some(candidate);
        }
        let legacy = current.join(LEGACY_MANIFEST_FILE);
        if legacy.is_file() {
            crate::output::deprecation::warn_once(&rust_i18n::t!("deprecation.manifest_file"));
            return Some(legacy);
        }
        if !current.pop() {
            return None;
        }
    }
}

/// Convenience: find from the current working directory.
pub fn find_manifest_from_cwd() -> Option<PathBuf> {
    std::env::current_dir().ok().and_then(|d| find_manifest(&d))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use tempfile::TempDir;

    // ==========================================================================
    // Parse — valid documents
    // ==========================================================================

    #[rstest]
    #[case::minimal(
        r#"
        [environment]
        name = "myproject"
        python = "3.12"
        "#,
        "myproject",
        "3.12",
        0
    )]
    #[case::with_default_packages(
        r#"
        [environment]
        name = "webapp"
        python = "3.11"

        [packages]
        default = ["fastapi", "uvicorn"]
        "#,
        "webapp",
        "3.11",
        2
    )]
    #[case::with_extra_groups(
        r#"
        [environment]
        name = "ds"
        python = "3.12"

        [packages]
        default = ["numpy"]
        dev = ["ipython", "pytest"]
        docs = ["mkdocs"]
        "#,
        "ds",
        "3.12",
        1
    )]
    fn parse_valid(
        #[case] toml_src: &str,
        #[case] expected_name: &str,
        #[case] expected_python: &str,
        #[case] expected_default_count: usize,
    ) {
        let m = ScoopManifest::parse(toml_src).expect("valid manifest");
        assert_eq!(m.environment.name, expected_name);
        assert_eq!(m.environment.python, expected_python);
        assert_eq!(m.packages.default.len(), expected_default_count);
    }

    // ==========================================================================
    // Parse — invalid documents
    // ==========================================================================

    #[test]
    fn parse_rejects_missing_environment_section() {
        let err = ScoopManifest::parse(
            r#"
            [packages]
            default = ["pytest"]
            "#,
        )
        .unwrap_err();
        assert!(
            matches!(err, ScoopError::InvalidArgument { .. }),
            "expected InvalidArgument, got {err:?}"
        );
    }

    #[test]
    fn parse_rejects_missing_python_field() {
        let err = ScoopManifest::parse(
            r#"
            [environment]
            name = "myproject"
            "#,
        )
        .unwrap_err();
        assert!(matches!(err, ScoopError::InvalidArgument { .. }));
    }

    #[test]
    fn parse_rejects_reserved_env_name() {
        // `list` is a CLI subcommand and reserved by `validate::RESERVED_NAMES`.
        let err = ScoopManifest::parse(
            r#"
            [environment]
            name = "list"
            python = "3.12"
            "#,
        )
        .unwrap_err();
        assert!(matches!(err, ScoopError::InvalidEnvName { .. }));
    }

    #[test]
    fn parse_rejects_empty_python_version() {
        let err = ScoopManifest::parse(
            r#"
            [environment]
            name = "myproject"
            python = ""
            "#,
        )
        .unwrap_err();
        assert!(matches!(err, ScoopError::InvalidPythonVersion { .. }));
    }

    #[test]
    fn parse_rejects_unknown_top_level_key() {
        // `deny_unknown_fields` on the root prevents typo'd sections from
        // silently no-op'ing (e.g. `[package]` instead of `[packages]`).
        let err = ScoopManifest::parse(
            r#"
            [environment]
            name = "myproject"
            python = "3.12"

            [hooks]
            post-create = "echo hi"
            "#,
        )
        .unwrap_err();
        assert!(matches!(err, ScoopError::InvalidArgument { .. }));
    }

    // ==========================================================================
    // Group resolution
    // ==========================================================================

    fn manifest_with_groups() -> ScoopManifest {
        ScoopManifest::parse(
            r#"
            [environment]
            name = "ds"
            python = "3.12"

            [packages]
            default = ["numpy", "pandas"]
            dev = ["ipython", "pytest"]
            docs = ["mkdocs"]
            "#,
        )
        .unwrap()
    }

    #[test]
    fn packages_for_default_only() {
        let m = manifest_with_groups();
        assert_eq!(m.packages_for(&[]), vec!["numpy", "pandas"]);
    }

    #[test]
    fn packages_for_includes_extra_group() {
        let m = manifest_with_groups();
        assert_eq!(
            m.packages_for(&["dev".to_string()]),
            vec!["numpy", "pandas", "ipython", "pytest"]
        );
    }

    #[test]
    fn packages_for_combines_multiple_groups() {
        let m = manifest_with_groups();
        let pkgs = m.packages_for(&["dev".to_string(), "docs".to_string()]);
        assert_eq!(pkgs, vec!["numpy", "pandas", "ipython", "pytest", "mkdocs"]);
    }

    #[test]
    fn packages_for_dedups_first_seen_order() {
        let m = ScoopManifest::parse(
            r#"
            [environment]
            name = "x"
            python = "3.12"

            [packages]
            default = ["pkg-a", "pkg-b"]
            extra = ["pkg-b", "pkg-c"]
            "#,
        )
        .unwrap();
        // pkg-b appears in both groups — keep the earliest position only.
        assert_eq!(
            m.packages_for(&["extra".to_string()]),
            vec!["pkg-a", "pkg-b", "pkg-c"]
        );
    }

    #[test]
    fn packages_for_silently_skips_unknown_group() {
        // Sync is responsible for surfacing unknown groups; the lookup itself
        // must not error so callers can probe.
        let m = manifest_with_groups();
        assert_eq!(
            m.packages_for(&["ghost".to_string()]),
            vec!["numpy", "pandas"]
        );
    }

    #[test]
    fn group_lookup_returns_default() {
        let m = manifest_with_groups();
        assert_eq!(
            m.group("default"),
            Some(&vec!["numpy".to_string(), "pandas".to_string()])
        );
    }

    #[test]
    fn group_lookup_returns_named_group() {
        let m = manifest_with_groups();
        assert!(m.group("dev").is_some());
        assert!(m.group("ghost").is_none());
    }

    // ==========================================================================
    // find_manifest — directory walk
    // ==========================================================================

    #[test]
    fn find_manifest_locates_in_current_dir() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join(MANIFEST_FILE), "").unwrap();
        assert_eq!(
            find_manifest(dir.path()),
            Some(dir.path().join(MANIFEST_FILE))
        );
    }

    #[test]
    fn find_manifest_walks_to_parent() {
        let root = TempDir::new().unwrap();
        let child = root.path().join("nested").join("deep");
        std::fs::create_dir_all(&child).unwrap();
        std::fs::write(root.path().join(MANIFEST_FILE), "").unwrap();

        let found = find_manifest(&child).expect("should walk up to root");
        assert_eq!(found, root.path().join(MANIFEST_FILE));
    }

    #[test]
    fn find_manifest_walks_terminate_at_root() {
        // Search a path that definitely has no .scuv.toml ancestor: the
        // system root. find_manifest must terminate (not loop forever) and
        // return either Some (if root happens to have one — unusual but legal)
        // or None.
        let _ = find_manifest(Path::new("/"));
    }

    // ==========================================================================
    // find_manifest — dual-name walk (.scuv.toml / legacy .scoop.toml)
    // ==========================================================================

    /// New name wins when both are present in the same directory.
    #[test]
    fn find_manifest_new_name_wins_over_legacy_in_same_dir() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join(MANIFEST_FILE), "").unwrap();
        std::fs::write(dir.path().join(LEGACY_MANIFEST_FILE), "").unwrap();
        assert_eq!(
            find_manifest(dir.path()),
            Some(dir.path().join(MANIFEST_FILE))
        );
    }

    /// DEPRECATION(0.16.0): legacy-shim regression test — a directory with
    /// only the legacy `.scoop.toml` name must still resolve.
    #[test]
    fn find_manifest_legacy_only_still_resolves() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join(LEGACY_MANIFEST_FILE), "").unwrap();
        assert_eq!(
            find_manifest(dir.path()),
            Some(dir.path().join(LEGACY_MANIFEST_FILE))
        );
    }
}
