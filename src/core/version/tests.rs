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

/// Fail-fast on an unwritable directory: the removal errors and
/// `unset_local` returns Err without silently claiming success — the
/// file stays in place.
#[cfg(unix)]
#[test]
fn test_unset_local_fails_fast_on_unwritable_dir() {
    use std::os::unix::fs::PermissionsExt;

    let temp = TempDir::new().unwrap();
    let dir = temp.path();
    std::fs::write(dir.join(".scuv-version"), "newenv\n").unwrap();

    // Restore permissions from a Drop guard so a panic between chmod and
    // the assertions can't strand a read-only dir that TempDir then fails
    // to clean up.
    struct RestorePerms<'a>(&'a std::path::Path);
    impl Drop for RestorePerms<'_> {
        fn drop(&mut self) {
            let _ = std::fs::set_permissions(self.0, std::fs::Permissions::from_mode(0o755));
        }
    }

    std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o555)).unwrap();
    let _restore = RestorePerms(dir);

    // root (e.g. the Docker integration CI) bypasses the read-only bit, so
    // the unwritable-dir scenario is unreproducible there. Probe with a
    // real write and skip rather than false-fail when perms aren't enforced.
    let probe = dir.join(".perm-probe");
    let perms_enforced = std::fs::write(&probe, b"x").is_err();
    let _ = std::fs::remove_file(&probe);
    if !perms_enforced {
        return;
    }

    let result = VersionService::unset_local(dir);

    assert!(result.is_err(), "read-only dir must surface an error");
    assert!(dir.join(".scuv-version").exists());
}

#[test]
fn test_read_version_file_normalizes_system_case() {
    let temp = TempDir::new().unwrap();
    let dir = temp.path();
    let version_file = dir.join(".scuv-version");

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
// Version-File Walk Tests (.scuv-version; the scoop-era name is ignored)
// =========================================================================

/// A directory holding only the scoop-era `.scoop-version` has no local
/// pin. Fails if a `.scoop-version` fallback is reintroduced in
/// `get_local`.
#[test]
fn get_local_ignores_legacy_version_file() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join(".scoop-version"), "oldenv").unwrap();
    assert_eq!(VersionService::get_local(dir.path()), None);
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

/// The scoop-era `SCOOP_RESOLVE_MAX_DEPTH` is ignored: with only the
/// legacy name set to `0`, the walk still reaches the parent directory.
/// `env_guard` alone isolates the home (SCUV_HOME at a tempdir) and
/// restores every variable on drop.
/// Fails if a `SCOOP_RESOLVE_MAX_DEPTH` fallback is reintroduced.
#[test]
#[serial]
fn test_resolve_max_depth_ignores_legacy_env() {
    let home = TempDir::new().unwrap();
    let _g = crate::test_utils::env_guard(&[
        (paths::SCUV_HOME_ENV, Some(home.path().to_str().unwrap())),
        ("SCUV_RESOLVE_MAX_DEPTH", None),
        ("SCOOP_RESOLVE_MAX_DEPTH", Some("0")),
    ]);
    let temp = TempDir::new().unwrap();
    let parent = temp.path().join("a").join("b");
    let deep = parent.join("c");
    std::fs::create_dir_all(&deep).unwrap();

    VersionService::set_local(&parent, "parentenv").unwrap();
    VersionService::set_global("globalenv").unwrap();

    assert_eq!(
        VersionService::resolve(&deep),
        Some("parentenv".to_string()),
        "SCOOP_RESOLVE_MAX_DEPTH=0 must not limit the walk"
    );
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
        }
        assert_eq!(VersionService::resolve(dir), Some("envenv".to_string()));
        // SAFETY: serial test, no concurrent env access.
        unsafe {
            std::env::remove_var("SCUV_VERSION");
        }
    });
}

/// The scoop-era `SCOOP_VERSION` is ignored: with only the legacy name
/// set, resolution falls through to the version file. `env_guard` alone
/// isolates the home and restores every variable on drop.
/// Fails if a `SCOOP_VERSION` fallback is reintroduced in
/// `resolve_env_version`.
#[test]
#[serial]
fn resolve_env_ignores_legacy_scoop_version() {
    let home = TempDir::new().unwrap();
    let _g = crate::test_utils::env_guard(&[
        (paths::SCUV_HOME_ENV, Some(home.path().to_str().unwrap())),
        ("SCUV_VERSION", None),
        ("SCOOP_VERSION", Some("legacyenv")),
    ]);
    let temp = TempDir::new().unwrap();
    let dir = temp.path();
    VersionService::set_local(dir, "fileenv").unwrap();

    assert_eq!(VersionService::resolve(dir), Some("fileenv".to_string()));
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
    let version_file = dir.join(".scuv-version");

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
