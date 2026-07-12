//! Version file service
//!
//! # Precedence contract
//!
//! Resolution order, highest priority first:
//!
//! 1. `SCUV_VERSION` environment variable.
//! 2. Legacy `SCOOP_VERSION` environment variable (deprecated; emits a
//!    one-shot warning).
//! 3. Nearest directory walking up from the target directory. Within a
//!    single directory, `.scuv-version` wins over a legacy `.scoop-version`
//!    (deprecated; emits a one-shot warning). A legacy file in a *nearer*
//!    directory still beats a new-named file in a parent directory —
//!    nearest-directory-first is unchanged by the file rename.
//! 4. The global version file (`~/.scuv/version`).
//!
//! DEPRECATION(0.16.0): remove both legacy branches (env var and file name).

use std::fs;
use std::path::{Path, PathBuf};

use crate::error::Result;
use crate::paths;

/// Service for managing version files
pub struct VersionService;

impl VersionService {
    /// Set the local version for a directory
    pub fn set_local(dir: &Path, env_name: &str) -> Result<()> {
        let version_file = paths::local_version_file(dir);
        fs::write(&version_file, format!("{env_name}\n"))?;
        Ok(())
    }

    /// Set the global version
    pub fn set_global(env_name: &str) -> Result<()> {
        let version_file = paths::global_version_file()?;
        if let Some(parent) = version_file.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&version_file, format!("{env_name}\n"))?;
        Ok(())
    }

    /// Get the local version for a directory
    pub fn get_local(dir: &Path) -> Option<String> {
        let version_file = Self::resolve_local_version_file(dir);
        Self::read_version_file(&version_file)
    }

    /// Resolve the local version-file path for a directory: the new
    /// `.scuv-version` name wins when present, otherwise falls back to the
    /// legacy `.scoop-version` name (warning once).
    ///
    /// DEPRECATION(0.16.0): remove the legacy fallback branch.
    fn resolve_local_version_file(dir: &Path) -> PathBuf {
        let version_file = dir.join(paths::VERSION_FILE);
        if version_file.exists() {
            version_file
        } else {
            let legacy = dir.join(paths::LEGACY_VERSION_FILE);
            if legacy.exists() {
                crate::output::deprecation::warn_once(&rust_i18n::t!("deprecation.version_file"));
                legacy
            } else {
                version_file
            }
        }
    }

    /// Get the global version
    pub fn get_global() -> Option<String> {
        let version_file = paths::global_version_file().ok()?;
        Self::read_version_file(&version_file)
    }

    /// Resolve the version for a directory (env var -> local -> parent -> global)
    ///
    /// See the module-level precedence contract for the full ordering,
    /// including the `.scuv-version` / legacy `.scoop-version` per-directory
    /// rule.
    ///
    /// # Environment Variables
    ///
    /// - `SCUV_VERSION` (or legacy `SCOOP_VERSION`): overrides file-based
    ///   resolution entirely when set to a valid environment name or
    ///   `system`.
    /// - `SCUV_RESOLVE_MAX_DEPTH` (or legacy `SCOOP_RESOLVE_MAX_DEPTH`):
    ///   Limits parent directory traversal depth.
    ///   Useful for slow network filesystems (NFS, SSHFS, etc).
    ///   - `0` = current directory only
    ///   - `3` = current + up to 3 parent directories
    ///   - unset = unlimited (default behavior)
    pub fn resolve(dir: &Path) -> Option<String> {
        // Priority 1: SCUV_VERSION / legacy SCOOP_VERSION environment variable.
        if let Some(name) = Self::resolve_env_version() {
            return Some(name);
        }

        // Get max depth from environment variable (None = unlimited).
        // DEPRECATION(0.16.0): remove legacy env fallback.
        let max_depth = std::env::var("SCUV_RESOLVE_MAX_DEPTH")
            .ok()
            .or_else(|| {
                std::env::var("SCOOP_RESOLVE_MAX_DEPTH").ok().inspect(|_| {
                    crate::output::deprecation::warn_once(&rust_i18n::t!(
                        "deprecation.env_var",
                        old = "SCOOP_RESOLVE_MAX_DEPTH",
                        new = "SCUV_RESOLVE_MAX_DEPTH"
                    ));
                })
            })
            .and_then(|s| s.parse::<usize>().ok());

        // Check current and parent directories for local version
        let mut current = dir.to_path_buf();
        let mut depth = 0;

        loop {
            if let Some(version) = Self::get_local(&current) {
                return Some(version);
            }

            // Check depth limit for network filesystem optimization
            if let Some(max) = max_depth {
                depth += 1;
                if depth > max {
                    break;
                }
            }

            if !current.pop() {
                break;
            }
        }

        // Fall back to global
        Self::get_global()
    }

    /// Resolve from current directory
    pub fn resolve_current() -> Option<String> {
        let cwd = std::env::current_dir().ok()?;
        Self::resolve(&cwd)
    }

    /// Read a version file
    ///
    /// Returns `None` if:
    /// - File doesn't exist or can't be read
    /// - Content is empty after trimming
    /// - Content is not a valid environment name or "system" (security: prevents command injection)
    fn read_version_file(path: &PathBuf) -> Option<String> {
        fs::read_to_string(path)
            .ok()
            .and_then(|s| Self::normalize_version_value(&s))
    }

    /// Validate and normalize a raw version value (from a file or an
    /// environment variable).
    ///
    /// Returns `None` if the trimmed value is empty or is not a valid
    /// environment name / `system` (security: prevents command injection).
    /// `system` is normalized to lowercase for consistent shell hook
    /// comparison, regardless of source-value casing.
    fn normalize_version_value(raw: &str) -> Option<String> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return None;
        }
        if trimmed.eq_ignore_ascii_case("system") {
            return Some("system".to_string());
        }
        crate::validate::is_valid_env_name(trimmed).then(|| trimmed.to_string())
    }

    /// Priority-1 environment-variable override: `SCUV_VERSION`, then legacy
    /// `SCOOP_VERSION` (deprecated; emits a one-shot warning). Falls through
    /// to `None` (i.e. file-based resolution) when neither is set to a
    /// valid value.
    ///
    /// DEPRECATION(0.16.0): remove the legacy `SCOOP_VERSION` branch.
    fn resolve_env_version() -> Option<String> {
        if let Ok(raw) = std::env::var("SCUV_VERSION") {
            if let Some(name) = Self::normalize_version_value(&raw) {
                return Some(name);
            }
        }
        if let Ok(raw) = std::env::var("SCOOP_VERSION") {
            if let Some(name) = Self::normalize_version_value(&raw) {
                crate::output::deprecation::warn_once(&rust_i18n::t!(
                    "deprecation.env_var",
                    old = "SCOOP_VERSION",
                    new = "SCUV_VERSION"
                ));
                return Some(name);
            }
        }
        None
    }

    /// Unset local version
    pub fn unset_local(dir: &Path) -> Result<()> {
        let version_file = paths::local_version_file(dir);
        if version_file.exists() {
            fs::remove_file(&version_file)?;
        }
        Ok(())
    }

    /// Unset global version
    pub fn unset_global() -> Result<()> {
        let version_file = paths::global_version_file()?;
        if version_file.exists() {
            fs::remove_file(&version_file)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::with_temp_scoop_home;
    use serial_test::serial;
    use tempfile::TempDir;

    // =========================================================================
    // Local Version Tests
    // =========================================================================

    #[test]
    fn test_set_and_get_local() {
        let temp = TempDir::new().unwrap();
        let dir = temp.path();

        VersionService::set_local(dir, "myenv").unwrap();
        assert_eq!(VersionService::get_local(dir), Some("myenv".to_string()));
    }

    #[test]
    fn test_get_local_nonexistent() {
        let temp = TempDir::new().unwrap();
        let dir = temp.path();

        // No version file set
        assert_eq!(VersionService::get_local(dir), None);
    }

    #[test]
    fn test_unset_local() {
        let temp = TempDir::new().unwrap();
        let dir = temp.path();

        // Set then unset
        VersionService::set_local(dir, "myenv").unwrap();
        assert!(VersionService::get_local(dir).is_some());

        VersionService::unset_local(dir).unwrap();
        assert_eq!(VersionService::get_local(dir), None);
    }

    #[test]
    fn test_unset_local_nonexistent() {
        let temp = TempDir::new().unwrap();
        let dir = temp.path();

        // Unset on non-existent file should succeed
        assert!(VersionService::unset_local(dir).is_ok());
    }

    #[test]
    fn test_read_version_file_normalizes_system_case() {
        let temp = TempDir::new().unwrap();
        let dir = temp.path();
        // DEPRECATION(0.16.0): kept on the legacy filename deliberately — this
        // doubles as a legacy-shim regression test that normalization still
        // applies when the value comes through the fallback path.
        let version_file = dir.join(".scoop-version");

        // Test various case combinations - all should normalize to lowercase "system"
        for variant in ["system", "System", "SYSTEM", "sYsTeM"] {
            std::fs::write(&version_file, format!("{variant}\n")).unwrap();
            assert_eq!(
                VersionService::get_local(dir),
                Some("system".to_string()),
                "'{variant}' should normalize to 'system'"
            );
        }
    }

    // =========================================================================
    // Dual Version-File Walk Tests (.scuv-version / legacy .scoop-version)
    // =========================================================================

    #[test]
    fn scuv_version_file_wins_within_same_dir() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join(".scuv-version"), "newenv").unwrap();
        std::fs::write(dir.path().join(".scoop-version"), "oldenv").unwrap();
        assert_eq!(VersionService::get_local(dir.path()).unwrap(), "newenv");
    }

    #[test]
    fn legacy_version_file_still_resolves() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join(".scoop-version"), "oldenv").unwrap();
        assert_eq!(VersionService::get_local(dir.path()).unwrap(), "oldenv");
    }

    // =========================================================================
    // Global Version Tests
    // =========================================================================

    #[test]
    #[serial]
    fn test_set_and_get_global() {
        with_temp_scoop_home(|_temp_dir| {
            VersionService::set_global("globalenv").unwrap();
            assert_eq!(VersionService::get_global(), Some("globalenv".to_string()));
        });
    }

    #[test]
    #[serial]
    fn test_get_global_nonexistent() {
        with_temp_scoop_home(|_temp_dir| {
            // No global version set
            assert_eq!(VersionService::get_global(), None);
        });
    }

    #[test]
    #[serial]
    fn test_unset_global() {
        with_temp_scoop_home(|_temp_dir| {
            VersionService::set_global("globalenv").unwrap();
            assert!(VersionService::get_global().is_some());

            VersionService::unset_global().unwrap();
            assert_eq!(VersionService::get_global(), None);
        });
    }

    #[test]
    #[serial]
    fn test_unset_global_nonexistent() {
        with_temp_scoop_home(|_temp_dir| {
            // Unset on non-existent file should succeed
            assert!(VersionService::unset_global().is_ok());
        });
    }

    // =========================================================================
    // Version Resolution Tests (local -> parent -> global)
    // =========================================================================

    #[test]
    #[serial]
    fn test_resolve_local_priority() {
        with_temp_scoop_home(|_temp_dir| {
            let temp = TempDir::new().unwrap();
            let dir = temp.path();

            // Set both local and global
            VersionService::set_local(dir, "localenv").unwrap();
            VersionService::set_global("globalenv").unwrap();

            // Local should take priority
            assert_eq!(VersionService::resolve(dir), Some("localenv".to_string()));
        });
    }

    #[test]
    #[serial]
    fn test_resolve_parent_directory() {
        with_temp_scoop_home(|_temp_dir| {
            let temp = TempDir::new().unwrap();
            let parent = temp.path();
            let child = parent.join("subdir");
            std::fs::create_dir(&child).unwrap();

            // Set version in parent only
            VersionService::set_local(parent, "parentenv").unwrap();

            // Child should resolve to parent's version
            assert_eq!(
                VersionService::resolve(&child),
                Some("parentenv".to_string())
            );
        });
    }

    #[test]
    #[serial]
    fn test_resolve_deep_nested() {
        with_temp_scoop_home(|_temp_dir| {
            let temp = TempDir::new().unwrap();
            let root = temp.path();
            let deep = root.join("a").join("b").join("c").join("d");
            std::fs::create_dir_all(&deep).unwrap();

            // Set version at root
            VersionService::set_local(root, "rootenv").unwrap();

            // Deep directory should resolve to root's version
            assert_eq!(VersionService::resolve(&deep), Some("rootenv".to_string()));
        });
    }

    #[test]
    #[serial]
    fn test_resolve_max_depth_limits_traversal() {
        with_temp_scoop_home(|_temp_dir| {
            let temp = TempDir::new().unwrap();
            let root = temp.path();
            let deep = root.join("a").join("b").join("c");
            std::fs::create_dir_all(&deep).unwrap();

            // Set version at root (3 levels up from deep)
            VersionService::set_local(root, "rootenv").unwrap();
            VersionService::set_global("globalenv").unwrap();

            // SAFETY: This test runs in serial mode, so no concurrent access
            unsafe {
                // Without limit: should find rootenv
                std::env::remove_var("SCUV_RESOLVE_MAX_DEPTH");
                assert_eq!(VersionService::resolve(&deep), Some("rootenv".to_string()));

                // With limit=1: should not find rootenv (only checks deep and deep/a)
                // and fall back to global
                std::env::set_var("SCUV_RESOLVE_MAX_DEPTH", "1");
                assert_eq!(
                    VersionService::resolve(&deep),
                    Some("globalenv".to_string())
                );

                // With limit=0: only checks current directory, falls back to global
                std::env::set_var("SCUV_RESOLVE_MAX_DEPTH", "0");
                assert_eq!(
                    VersionService::resolve(&deep),
                    Some("globalenv".to_string())
                );

                // Cleanup
                std::env::remove_var("SCUV_RESOLVE_MAX_DEPTH");
            }
        });
    }

    /// Pins the exact depth boundary: with limit=1, traversal must still reach
    /// the *immediate* parent. Distinguishes `depth > max` from `depth >= max`
    /// (the latter would stop one directory too early).
    #[test]
    #[serial]
    fn test_resolve_max_depth_reaches_immediate_parent() {
        with_temp_scoop_home(|_temp_dir| {
            let temp = TempDir::new().unwrap();
            let parent = temp.path().join("a").join("b");
            let deep = parent.join("c");
            std::fs::create_dir_all(&deep).unwrap();

            VersionService::set_local(&parent, "parentenv").unwrap();
            VersionService::set_global("globalenv").unwrap();

            // SAFETY: serial test, no concurrent env access.
            unsafe {
                std::env::set_var("SCUV_RESOLVE_MAX_DEPTH", "1");
                assert_eq!(
                    VersionService::resolve(&deep),
                    Some("parentenv".to_string()),
                    "limit=1 must still reach the immediate parent"
                );
                std::env::remove_var("SCUV_RESOLVE_MAX_DEPTH");
            }
        });
    }

    /// DEPRECATION(0.16.0): exercises the legacy `SCOOP_RESOLVE_MAX_DEPTH`
    /// fallback; remove alongside the fallback itself.
    #[test]
    #[serial]
    fn test_resolve_max_depth_legacy_env_still_read() {
        with_temp_scoop_home(|_temp_dir| {
            let temp = TempDir::new().unwrap();
            let parent = temp.path().join("a").join("b");
            let deep = parent.join("c");
            std::fs::create_dir_all(&deep).unwrap();

            VersionService::set_local(&parent, "parentenv").unwrap();
            VersionService::set_global("globalenv").unwrap();

            // SAFETY: serial test, no concurrent env access.
            unsafe {
                std::env::remove_var("SCUV_RESOLVE_MAX_DEPTH");
                std::env::set_var("SCOOP_RESOLVE_MAX_DEPTH", "1");
                assert_eq!(
                    VersionService::resolve(&deep),
                    Some("parentenv".to_string()),
                    "legacy SCOOP_RESOLVE_MAX_DEPTH must still be honored"
                );
                std::env::remove_var("SCOOP_RESOLVE_MAX_DEPTH");
            }
        });
    }

    #[test]
    #[serial]
    fn resolve_env_scuv_version_wins_over_files() {
        with_temp_scoop_home(|_temp_dir| {
            let temp = TempDir::new().unwrap();
            let dir = temp.path();
            VersionService::set_local(dir, "fileenv").unwrap();
            VersionService::set_global("globalenv").unwrap();

            // SAFETY: serial test, no concurrent env access.
            unsafe {
                std::env::set_var("SCUV_VERSION", "envenv");
                std::env::remove_var("SCOOP_VERSION");
            }
            assert_eq!(VersionService::resolve(dir), Some("envenv".to_string()));
            // SAFETY: serial test, no concurrent env access.
            unsafe {
                std::env::remove_var("SCUV_VERSION");
            }
        });
    }

    /// DEPRECATION(0.16.0): exercises the legacy `SCOOP_VERSION` env-var
    /// fallback; remove alongside the fallback itself.
    #[test]
    #[serial]
    fn resolve_env_legacy_scoop_version_still_read_when_scuv_unset() {
        with_temp_scoop_home(|_temp_dir| {
            let temp = TempDir::new().unwrap();
            let dir = temp.path();
            VersionService::set_local(dir, "fileenv").unwrap();

            // SAFETY: serial test, no concurrent env access.
            unsafe {
                std::env::remove_var("SCUV_VERSION");
                std::env::set_var("SCOOP_VERSION", "legacyenv");
            }
            assert_eq!(VersionService::resolve(dir), Some("legacyenv".to_string()));
            // SAFETY: serial test, no concurrent env access.
            unsafe {
                std::env::remove_var("SCOOP_VERSION");
            }
        });
    }

    #[test]
    #[serial]
    fn test_resolve_fallback_to_global() {
        with_temp_scoop_home(|_temp_dir| {
            let temp = TempDir::new().unwrap();
            let dir = temp.path();

            // Only set global
            VersionService::set_global("globalenv").unwrap();

            // Should fall back to global
            assert_eq!(VersionService::resolve(dir), Some("globalenv".to_string()));
        });
    }

    #[test]
    #[serial]
    fn test_resolve_none_when_no_version() {
        with_temp_scoop_home(|_temp_dir| {
            let temp = TempDir::new().unwrap();
            let dir = temp.path();

            // No version set anywhere
            assert_eq!(VersionService::resolve(dir), None);
        });
    }

    #[test]
    #[serial]
    fn test_resolve_child_overrides_parent() {
        with_temp_scoop_home(|_temp_dir| {
            let temp = TempDir::new().unwrap();
            let parent = temp.path();
            let child = parent.join("subdir");
            std::fs::create_dir(&child).unwrap();

            // Set version in both parent and child
            VersionService::set_local(parent, "parentenv").unwrap();
            VersionService::set_local(&child, "childenv").unwrap();

            // Child should use its own version
            assert_eq!(
                VersionService::resolve(&child),
                Some("childenv".to_string())
            );

            // Parent should use its own version
            assert_eq!(
                VersionService::resolve(parent),
                Some("parentenv".to_string())
            );
        });
    }

    // =========================================================================
    // Edge Cases and File Format Tests
    // =========================================================================

    #[test]
    fn test_version_file_trimmed() {
        let temp = TempDir::new().unwrap();
        let dir = temp.path();
        let version_file = dir.join(".scuv-version");

        // Write with extra whitespace
        std::fs::write(&version_file, "  myenv  \n\n").unwrap();

        // Should be trimmed
        assert_eq!(VersionService::get_local(dir), Some("myenv".to_string()));
    }

    #[test]
    fn test_version_file_empty_returns_none() {
        let temp = TempDir::new().unwrap();
        let dir = temp.path();
        let version_file = dir.join(".scuv-version");

        // Write empty content
        std::fs::write(&version_file, "").unwrap();

        assert_eq!(VersionService::get_local(dir), None);
    }

    #[test]
    fn test_version_file_whitespace_only_returns_none() {
        let temp = TempDir::new().unwrap();
        let dir = temp.path();
        let version_file = dir.join(".scuv-version");

        // Write whitespace only
        std::fs::write(&version_file, "   \n\t\n  ").unwrap();

        assert_eq!(VersionService::get_local(dir), None);
    }

    #[test]
    fn test_version_file_preserves_env_name() {
        let temp = TempDir::new().unwrap();
        let dir = temp.path();

        // Test various valid env names
        let names = ["myenv", "my-project", "test_env", "Env123"];

        for name in names {
            VersionService::set_local(dir, name).unwrap();
            assert_eq!(
                VersionService::get_local(dir),
                Some(name.to_string()),
                "Failed for env name: {}",
                name
            );
        }
    }

    #[test]
    fn test_set_local_creates_file_with_newline() {
        let temp = TempDir::new().unwrap();
        let dir = temp.path();
        let version_file = dir.join(".scuv-version");

        VersionService::set_local(dir, "myenv").unwrap();

        let content = std::fs::read_to_string(&version_file).unwrap();
        assert_eq!(content, "myenv\n", "Version file should end with newline");
    }

    // =========================================================================
    // Security Tests: Command Injection Prevention
    // =========================================================================

    #[test]
    fn test_read_version_file_rejects_command_injection() {
        let temp = TempDir::new().unwrap();
        let dir = temp.path();
        // DEPRECATION(0.16.0): kept on the legacy filename deliberately — this
        // doubles as a legacy-shim regression test that security validation
        // still applies when the value comes through the fallback path.
        let version_file = dir.join(".scoop-version");

        // Write malicious content (command injection attempt)
        std::fs::write(&version_file, "\"; echo INJECTED; #\n").unwrap();

        // Should return None because content is not a valid env name
        assert_eq!(
            VersionService::get_local(dir),
            None,
            "Malicious content should be rejected"
        );
    }

    #[test]
    fn test_read_version_file_rejects_backtick_injection() {
        let temp = TempDir::new().unwrap();
        let dir = temp.path();
        let version_file = dir.join(".scuv-version");

        // Write backtick command substitution attempt
        std::fs::write(&version_file, "`rm -rf /`\n").unwrap();

        assert_eq!(
            VersionService::get_local(dir),
            None,
            "Backtick injection should be rejected"
        );
    }

    #[test]
    fn test_read_version_file_rejects_dollar_expansion() {
        let temp = TempDir::new().unwrap();
        let dir = temp.path();
        let version_file = dir.join(".scuv-version");

        // Write variable expansion attempt
        std::fs::write(&version_file, "$(whoami)\n").unwrap();

        assert_eq!(
            VersionService::get_local(dir),
            None,
            "Dollar expansion should be rejected"
        );
    }

    #[test]
    fn test_read_version_file_rejects_path_traversal() {
        let temp = TempDir::new().unwrap();
        let dir = temp.path();
        let version_file = dir.join(".scuv-version");

        // Write path traversal attempt
        std::fs::write(&version_file, "../../../etc/passwd\n").unwrap();

        assert_eq!(
            VersionService::get_local(dir),
            None,
            "Path traversal should be rejected"
        );
    }

    #[test]
    fn test_read_version_file_rejects_newline_injection() {
        let temp = TempDir::new().unwrap();
        let dir = temp.path();
        let version_file = dir.join(".scuv-version");

        // Write multiline injection attempt
        std::fs::write(&version_file, "safe\nrm -rf /\n").unwrap();

        // After trim(), this becomes "safe\nrm -rf /" which contains newline
        // is_valid_env_name should reject this
        assert_eq!(
            VersionService::get_local(dir),
            None,
            "Newline injection should be rejected"
        );
    }

    #[test]
    fn test_read_version_file_accepts_valid_names() {
        let temp = TempDir::new().unwrap();
        let dir = temp.path();
        let version_file = dir.join(".scuv-version");

        // Test various valid environment names
        let valid_names = ["myenv", "my-project", "test_env", "Env123", "a"];

        for name in valid_names {
            std::fs::write(&version_file, format!("{name}\n")).unwrap();
            assert_eq!(
                VersionService::get_local(dir),
                Some(name.to_string()),
                "Valid name '{}' should be accepted",
                name
            );
        }
    }
}
