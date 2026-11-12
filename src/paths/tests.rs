//! Unit tests for `paths`.

/// The decision-to-delete gap: whatever replaced the link after the
/// caller recorded it must survive. Fails if the helper stops comparing
/// the recorded target, or deletes whatever is at the path.
#[cfg(unix)]
#[test]
fn remove_symlink_if_unchanged_spares_a_replaced_link() {
    use std::os::unix::fs::symlink;
    let tmp = tempfile::tempdir().unwrap();
    let link = tmp.path().join(".venv");
    let target = tmp.path().join("env");

    symlink(&target, &link).unwrap();
    assert!(remove_symlink_if_unchanged(&link, &target).unwrap());
    assert!(!link.is_symlink());

    // Replaced by a regular file between decision and delete.
    std::fs::write(&link, b"user data").unwrap();
    assert!(!remove_symlink_if_unchanged(&link, &target).unwrap());
    assert_eq!(std::fs::read(&link).unwrap(), b"user data");
    std::fs::remove_file(&link).unwrap();

    // Re-pointed somewhere else.
    symlink(tmp.path().join("other"), &link).unwrap();
    assert!(!remove_symlink_if_unchanged(&link, &target).unwrap());
    assert!(link.is_symlink());
}

/// A read failure that is not "changed" is reported, not mistaken for a
/// replaced link. Fails if every read error turns into `Ok(false)`.
#[cfg(unix)]
#[test]
fn remove_symlink_if_unchanged_reports_other_read_errors() {
    let tmp = tempfile::tempdir().unwrap();
    let file = tmp.path().join("file");
    std::fs::write(&file, b"").unwrap();
    // A path through a regular file: ENOTDIR, for any user.
    let link = file.join(".venv");
    assert!(remove_symlink_if_unchanged(&link, Path::new("/x")).is_err());
}

use super::*;
use crate::test_utils::{with_no_scoop_home, with_temp_scoop_home};
use serial_test::serial;

#[test]
fn test_scoop_home_default() {
    with_no_scoop_home(|| {
        let home = scoop_home().unwrap();
        assert!(home.ends_with(".scuv"));
    });
}

/// The scoop-era `SCOOP_HOME` is not read any more: with only the legacy
/// name set, resolution falls through to the default directory.
/// Fails if a `SCOOP_HOME` fallback is reintroduced in `scoop_home()`.
#[test]
#[serial]
fn scoop_home_ignores_legacy_home_env() {
    let _g = crate::test_utils::env_guard(&[
        (SCUV_HOME_ENV, None),
        ("SCOOP_HOME", Some("/tmp/oldhome")),
    ]);
    let home = scoop_home().unwrap();
    assert_ne!(home, PathBuf::from("/tmp/oldhome"));
    assert!(home.ends_with(".scuv"));
}

#[test]
#[serial]
fn default_home_is_dot_scuv() {
    let _g = crate::test_utils::env_guard(&[(SCUV_HOME_ENV, None)]);
    let home = scoop_home().unwrap();
    assert!(home.ends_with(".scuv"));
}

/// `dirs::home_dir()` reads `$HOME` on Unix (not on Windows, hence the
/// cfg), so `env_guard` can steer it at a tempdir. A leftover `~/.scoop`
/// (and no `~/.scuv`) must not change the answer: the directory fallback
/// ended with 0.16.0.
/// Fails if a `~/.scoop` fallback is reintroduced in `scoop_home()`.
#[cfg(unix)]
#[test]
#[serial]
fn scoop_home_ignores_legacy_home_dir_even_when_scuv_dir_is_missing() {
    let tmp = tempfile::TempDir::new().unwrap();
    std::fs::create_dir(tmp.path().join(".scoop")).unwrap();
    let home_str = tmp.path().to_str().unwrap();
    let _g = crate::test_utils::env_guard(&[(SCUV_HOME_ENV, None), ("HOME", Some(home_str))]);
    assert_eq!(scoop_home().unwrap(), tmp.path().join(".scuv"));
}

#[cfg(unix)]
#[test]
#[serial]
fn default_home_when_neither_dir_exists() {
    let tmp = tempfile::TempDir::new().unwrap();
    let home_str = tmp.path().to_str().unwrap();
    let _g = crate::test_utils::env_guard(&[(SCUV_HOME_ENV, None), ("HOME", Some(home_str))]);
    assert_eq!(scoop_home().unwrap(), tmp.path().join(".scuv"));
}

#[test]
fn test_scoop_home_env() {
    with_temp_scoop_home(|temp_dir| {
        let home = scoop_home().unwrap();
        assert_eq!(home, temp_dir.path());
    });
}

#[test]
#[serial]
fn test_virtualenvs_dir() {
    with_temp_scoop_home(|temp_dir| {
        let venvs = virtualenvs_dir().unwrap();
        assert_eq!(venvs, temp_dir.path().join("virtualenvs"));
    });
}

#[test]
#[serial]
fn test_virtualenv_path() {
    with_temp_scoop_home(|temp_dir| {
        let path = virtualenv_path("myenv").unwrap();
        assert_eq!(path, temp_dir.path().join("virtualenvs").join("myenv"));
    });
}

#[test]
#[serial]
fn test_virtualenv_bin() {
    with_temp_scoop_home(|temp_dir| {
        let bin = virtualenv_bin("myenv").unwrap();
        assert_eq!(
            bin,
            temp_dir
                .path()
                .join("virtualenvs")
                .join("myenv")
                .join("bin")
        );
    });
}

#[test]
#[serial]
fn test_virtualenv_exists() {
    with_temp_scoop_home(|temp_dir| {
        // Create the virtualenvs directory
        let venv_path = temp_dir.path().join("virtualenvs").join("existing");
        std::fs::create_dir_all(&venv_path).unwrap();

        assert!(virtualenv_exists("existing").unwrap());
        assert!(!virtualenv_exists("nonexistent").unwrap());
    });
}

#[test]
fn test_local_version_file() {
    // This test doesn't use environment variables
    let dir = PathBuf::from("/some/project");
    let version_file = local_version_file(&dir);
    assert_eq!(version_file, dir.join(".scuv-version"));
}

#[test]
#[serial]
fn test_global_version_file() {
    with_temp_scoop_home(|temp_dir| {
        let version_file = global_version_file().unwrap();
        assert_eq!(version_file, temp_dir.path().join("version"));
    });
}

#[cfg(unix)]
#[test]
#[serial]
fn test_virtualenv_activate_unix() {
    with_temp_scoop_home(|temp_dir| {
        let activate = virtualenv_activate("myenv").unwrap();
        assert_eq!(
            activate,
            temp_dir
                .path()
                .join("virtualenvs")
                .join("myenv")
                .join("bin")
                .join("activate")
        );
    });
}

// ==========================================================================
// Symlink and Edge Case Tests
// ==========================================================================

#[cfg(unix)]
#[test]
#[serial]
fn test_virtualenv_exists_with_symlink() {
    with_temp_scoop_home(|temp_dir| {
        use std::os::unix::fs::symlink;

        // Create a real directory
        let real_dir = temp_dir.path().join("real_venv");
        std::fs::create_dir_all(&real_dir).unwrap();

        // Create virtualenvs directory and symlink
        let venvs_dir = temp_dir.path().join("virtualenvs");
        std::fs::create_dir_all(&venvs_dir).unwrap();
        let symlink_path = venvs_dir.join("symlinked");
        symlink(&real_dir, &symlink_path).unwrap();

        // Symlinked virtualenv should be detected as existing
        assert!(virtualenv_exists("symlinked").unwrap());
    });
}

#[cfg(unix)]
#[test]
#[serial]
fn test_virtualenv_exists_with_broken_symlink() {
    with_temp_scoop_home(|temp_dir| {
        use std::os::unix::fs::symlink;

        // Create virtualenvs directory
        let venvs_dir = temp_dir.path().join("virtualenvs");
        std::fs::create_dir_all(&venvs_dir).unwrap();

        // Create a symlink to non-existent target
        let broken_symlink = venvs_dir.join("broken");
        symlink("/nonexistent/path", &broken_symlink).unwrap();

        // Broken symlink should NOT be detected as existing directory
        assert!(!virtualenv_exists("broken").unwrap());
    });
}

#[cfg(unix)]
#[test]
#[serial]
fn test_virtualenv_exists_symlink_to_file() {
    with_temp_scoop_home(|temp_dir| {
        use std::os::unix::fs::symlink;

        // Create a regular file
        let file_path = temp_dir.path().join("regular_file");
        std::fs::write(&file_path, "test").unwrap();

        // Create virtualenvs directory and symlink to file
        let venvs_dir = temp_dir.path().join("virtualenvs");
        std::fs::create_dir_all(&venvs_dir).unwrap();
        let symlink_path = venvs_dir.join("filelink");
        symlink(&file_path, &symlink_path).unwrap();

        // Symlink to file should NOT be detected as existing (needs to be directory)
        assert!(!virtualenv_exists("filelink").unwrap());
    });
}

#[test]
#[serial]
fn test_path_with_special_characters() {
    with_temp_scoop_home(|temp_dir| {
        // Environment names with allowed special characters
        let venvs_dir = temp_dir.path().join("virtualenvs");
        std::fs::create_dir_all(venvs_dir.join("my-env")).unwrap();
        std::fs::create_dir_all(venvs_dir.join("my_env")).unwrap();
        std::fs::create_dir_all(venvs_dir.join("env123")).unwrap();

        assert!(virtualenv_exists("my-env").unwrap());
        assert!(virtualenv_exists("my_env").unwrap());
        assert!(virtualenv_exists("env123").unwrap());
    });
}

// ==========================================================================
// calculate_dir_size Tests
// ==========================================================================

#[test]
fn test_calculate_dir_size_empty_dir() {
    let dir = tempfile::tempdir().unwrap();
    let size = calculate_dir_size(dir.path()).unwrap();
    assert_eq!(size, 0);
}

#[test]
fn test_calculate_dir_size_with_file() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("test.txt");
    std::fs::write(&file_path, b"hello").unwrap();

    let size = calculate_dir_size(dir.path()).unwrap();
    assert_eq!(size, 5);
}

#[test]
fn test_calculate_dir_size_nested_dirs() {
    let dir = tempfile::tempdir().unwrap();
    let subdir = dir.path().join("subdir");
    std::fs::create_dir(&subdir).unwrap();

    std::fs::write(subdir.join("test.txt"), b"hello world").unwrap();

    let size = calculate_dir_size(dir.path()).unwrap();
    assert_eq!(size, 11);
}

#[test]
fn test_calculate_dir_size_nonexistent() {
    // is_dir() returns false for nonexistent, so returns 0
    let result = calculate_dir_size(std::path::Path::new("/nonexistent/path"));
    assert_eq!(result.unwrap(), 0);
}

#[cfg(unix)]
#[test]
fn test_calculate_dir_size_skips_symlinks() {
    use std::os::unix::fs::symlink;

    let dir = tempfile::tempdir().unwrap();

    // Create a file
    std::fs::write(dir.path().join("file.txt"), b"test").unwrap();

    // Create a symlink to the file (should be skipped)
    symlink(dir.path().join("file.txt"), dir.path().join("link")).unwrap();

    // Size should only include the file, not the symlink
    let size = calculate_dir_size(dir.path()).unwrap();
    assert_eq!(size, 4); // Only "test" (4 bytes)
}

#[cfg(unix)]
#[test]
fn test_calculate_dir_size_circular_symlink() {
    use std::os::unix::fs::symlink;

    let dir = tempfile::tempdir().unwrap();
    let subdir = dir.path().join("sub");
    std::fs::create_dir(&subdir).unwrap();

    // Create circular symlink: sub/loop -> ..
    symlink(dir.path(), subdir.join("loop")).unwrap();

    // Should not hang or overflow - symlinks are skipped
    let result = calculate_dir_size(dir.path());
    assert!(result.is_ok());
}

// ==========================================================================
// abbreviate_home Tests
// ==========================================================================

#[test]
#[serial]
fn test_abbreviate_home_with_home_path() {
    // Pin HOME so parallel tests that mutate it can't race this one
    // (dirs::home_dir() reads the process-wide env live).
    let tmp = tempfile::tempdir().unwrap();
    let _g = crate::test_utils::env_guard(&[("HOME", tmp.path().to_str())]);
    let path = tmp.path().join(".scuv").join("virtualenvs").join("myenv");
    let result = abbreviate_home(&path);
    assert!(result.starts_with("~/"));
    assert!(result.contains(".scuv/virtualenvs/myenv"));
}

#[test]
#[serial]
fn test_abbreviate_home_outside_home() {
    // Path outside home directory should remain unchanged. HOME is pinned
    // away from /tmp because parallel tests point HOME at tempdirs (which
    // live under /tmp on Linux) and would make this path abbreviable.
    let home = tempfile::tempdir().unwrap();
    let _g = crate::test_utils::env_guard(&[("HOME", home.path().to_str())]);
    let path = PathBuf::from("/tmp/some/path");
    let result = abbreviate_home(&path);
    assert_eq!(result, "/tmp/some/path");
}

#[test]
fn test_abbreviate_home_root_path() {
    // Root path should remain unchanged
    let path = PathBuf::from("/");
    let result = abbreviate_home(&path);
    assert_eq!(result, "/");
}

// ==========================================================================
// find_executable_in / executable_candidates Tests
// ==========================================================================

#[test]
fn find_executable_in_locates_existing_file() {
    let dir = tempfile::tempdir().unwrap();
    let exe = dir.path().join("python");
    std::fs::write(&exe, b"").unwrap();
    assert_eq!(find_executable_in(dir.path(), "python"), Some(exe));
}

#[test]
fn find_executable_in_returns_none_for_missing() {
    let dir = tempfile::tempdir().unwrap();
    assert!(find_executable_in(dir.path(), "python").is_none());
}

#[test]
fn find_executable_in_ignores_directories() {
    // A subdirectory whose name matches the executable must not satisfy
    // the lookup — we want files only.
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(dir.path().join("python")).unwrap();
    assert!(find_executable_in(dir.path(), "python").is_none());
}

#[test]
fn find_executable_in_returns_none_when_dir_missing() {
    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().join("does-not-exist");
    assert!(find_executable_in(&missing, "python").is_none());
}

#[cfg(not(windows))]
#[test]
fn executable_candidates_unix_is_single_entry() {
    assert_eq!(executable_candidates("python"), vec!["python".to_string()]);
}

#[cfg(windows)]
#[test]
fn executable_candidates_windows_probes_standard_extensions() {
    let candidates = executable_candidates("python");
    assert!(candidates.contains(&"python".to_string()));
    assert!(candidates.contains(&"python.exe".to_string()));
    assert!(candidates.contains(&"python.bat".to_string()));
    assert!(candidates.contains(&"python.cmd".to_string()));
}

#[cfg(windows)]
#[test]
fn executable_candidates_windows_respects_explicit_extension() {
    assert_eq!(executable_candidates("python.exe"), vec!["python.exe"]);
}

// ==========================================================================
// Cross-platform venv path helpers
// ==========================================================================

#[cfg(unix)]
mod cross_platform_unix {
    use super::*;

    #[test]
    fn virtualenv_bin_dir_returns_bin_on_unix() {
        let root = std::path::PathBuf::from("/tmp/fake");
        assert_eq!(virtualenv_bin_dir(&root), root.join("bin"));
    }

    #[test]
    fn virtualenv_python_exe_returns_bin_python_on_unix() {
        let root = std::path::PathBuf::from("/tmp/fake");
        assert_eq!(
            virtualenv_python_exe(&root),
            root.join("bin").join("python")
        );
    }

    #[test]
    fn virtualenv_activate_script_returns_bin_activate_on_unix() {
        let root = std::path::PathBuf::from("/tmp/fake");
        assert_eq!(
            virtualenv_activate_script(&root),
            root.join("bin").join("activate")
        );
    }

    #[test]
    fn virtualenv_bin_wrapper_delegates_to_helper() {
        with_temp_scoop_home(|_| {
            let bin = virtualenv_bin("anyname").unwrap();
            assert!(bin.ends_with("bin"));
        });
    }
}

#[cfg(windows)]
mod cross_platform_windows {
    use super::*;

    #[test]
    fn virtualenv_bin_dir_returns_scripts_on_windows() {
        let root = std::path::PathBuf::from("C:\\fake");
        assert_eq!(virtualenv_bin_dir(&root), root.join("Scripts"));
    }

    #[test]
    fn virtualenv_python_exe_returns_scripts_python_exe_on_windows() {
        let root = std::path::PathBuf::from("C:\\fake");
        assert_eq!(
            virtualenv_python_exe(&root),
            root.join("Scripts").join("python.exe")
        );
    }

    #[test]
    fn virtualenv_activate_script_returns_ps1_on_windows() {
        let root = std::path::PathBuf::from("C:\\fake");
        assert_eq!(
            virtualenv_activate_script(&root),
            root.join("Scripts").join("Activate.ps1")
        );
    }
}

/// A failed clone or import removes only the directory it made: one put
/// under the name meanwhile survives, and one already gone is no error.
/// What fails this: removing by path without comparing the identity, or
/// never removing.
#[test]
fn remove_dir_if_unchanged_spares_a_replaced_directory() {
    let tmp = tempfile::tempdir().unwrap();
    let env = tmp.path().join("work");
    std::fs::create_dir(&env).unwrap();
    let created = super::dir_identity(&env).unwrap();
    // Replaced: a new directory under the same name, kept alive by a file
    // so the old one's inode is not reused for it.
    let old = tmp.path().join("old");
    std::fs::rename(&env, &old).unwrap();
    std::fs::create_dir(&env).unwrap();
    std::fs::write(env.join("saved"), "x").unwrap();
    assert!(!super::remove_dir_if_unchanged(&env, created).unwrap());
    assert!(env.join("saved").exists());
    // The original, back in place, is removed.
    std::fs::remove_dir_all(&env).unwrap();
    std::fs::rename(&old, &env).unwrap();
    assert!(super::remove_dir_if_unchanged(&env, created).unwrap());
    assert!(!env.exists());
    assert!(!super::remove_dir_if_unchanged(&env, created).unwrap());
    // Unreadable for another reason (a file where a directory should be):
    // an error, not "gone".
    let file = tmp.path().join("file");
    std::fs::write(&file, "x").unwrap();
    assert!(super::remove_dir_if_unchanged(&file.join("work"), created).is_err());
}
