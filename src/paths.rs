//! Path utilities for scoop

use std::path::PathBuf;

use crate::error::{Result, ScoopError};

/// Environment variable for scoop home directory
pub const SCOOP_HOME_ENV: &str = "SCOOP_HOME";

/// Default scoop home directory name
const SCOOP_HOME_DIR: &str = ".scoop";

/// Version file name
pub const VERSION_FILE: &str = ".scoop-version";

/// Get the scoop home directory (~/.scoop or $SCOOP_HOME)
pub fn scoop_home() -> Result<PathBuf> {
    if let Ok(home) = std::env::var(SCOOP_HOME_ENV) {
        return Ok(PathBuf::from(home));
    }

    dirs::home_dir()
        .map(|h| h.join(SCOOP_HOME_DIR))
        .ok_or(ScoopError::HomeNotFound)
}

/// Get the virtualenvs directory (~/.scoop/virtualenvs)
pub fn virtualenvs_dir() -> Result<PathBuf> {
    Ok(scoop_home()?.join("virtualenvs"))
}

/// Get the pythons directory (~/.scoop/pythons)
pub fn pythons_dir() -> Result<PathBuf> {
    Ok(scoop_home()?.join("pythons"))
}

/// Get the global version file path (~/.scoop/version)
pub fn global_version_file() -> Result<PathBuf> {
    Ok(scoop_home()?.join("version"))
}

/// Get the local version file path in the given directory
pub fn local_version_file(dir: &std::path::Path) -> PathBuf {
    dir.join(VERSION_FILE)
}

/// Get the path to a specific virtualenv
pub fn virtualenv_path(name: &str) -> Result<PathBuf> {
    Ok(virtualenvs_dir()?.join(name))
}

/// Get the bin directory of a virtualenv
pub fn virtualenv_bin(name: &str) -> Result<PathBuf> {
    Ok(virtualenv_path(name)?.join("bin"))
}

/// Get the python executable in a virtualenv
pub fn virtualenv_python(name: &str) -> Result<PathBuf> {
    Ok(virtualenv_bin(name)?.join("python"))
}

/// Ensure all scoop directories exist
///
/// Creates the following directory structure:
/// - ~/.scoop/
/// - ~/.scoop/virtualenvs/
/// - ~/.scoop/pythons/
pub fn ensure_scoop_dirs() -> Result<()> {
    let home = scoop_home()?;
    std::fs::create_dir_all(&home)?;
    std::fs::create_dir_all(home.join("virtualenvs"))?;
    std::fs::create_dir_all(home.join("pythons"))?;
    Ok(())
}

/// Check if a virtualenv exists
pub fn virtualenv_exists(name: &str) -> Result<bool> {
    let path = virtualenv_path(name)?;
    Ok(path.exists() && path.is_dir())
}

/// Get the activate script path for a virtualenv
#[cfg(unix)]
pub fn virtualenv_activate(name: &str) -> Result<PathBuf> {
    Ok(virtualenv_bin(name)?.join("activate"))
}

/// Get the activate script path for a virtualenv (Windows)
#[cfg(windows)]
pub fn virtualenv_activate(name: &str) -> Result<PathBuf> {
    Ok(virtualenv_path(name)?.join("Scripts").join("activate.bat"))
}

/// Calculate directory size recursively
///
/// Symlinks are skipped to prevent infinite loops.
///
/// # Errors
///
/// Returns `std::io::Error` if:
/// - Directory cannot be read (permission denied)
/// - File metadata cannot be accessed
///
/// # Examples
///
/// ```no_run
/// use std::path::Path;
/// use scoop_uv::paths::calculate_dir_size;
///
/// let size = calculate_dir_size(Path::new("/tmp/mydir"))?;
/// println!("Directory size: {} bytes", size);
/// # Ok::<(), std::io::Error>(())
/// ```
pub fn calculate_dir_size(path: &std::path::Path) -> std::io::Result<u64> {
    let mut total: u64 = 0;
    // Skip symlinks to prevent infinite loops
    if path.is_dir() && !path.is_symlink() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path(); // Fixed: avoid variable shadowing
            // Skip symlinks in size calculation
            if entry_path.is_symlink() {
                continue;
            }
            if entry_path.is_dir() {
                total += calculate_dir_size(&entry_path)?;
            } else {
                total += entry.metadata()?.len();
            }
        }
    }
    Ok(total)
}

/// Abbreviate home directory to `~` for display.
///
/// # Examples
///
/// ```
/// use std::path::Path;
/// use scoop_uv::paths::abbreviate_home;
///
/// // Home directory paths get abbreviated
/// let home = dirs::home_dir().unwrap();
/// let path = home.join(".scoop/virtualenvs/myenv");
/// let abbreviated = abbreviate_home(&path);
/// assert!(abbreviated.starts_with("~/"));
/// ```
pub fn abbreviate_home(path: &std::path::Path) -> String {
    if let Some(home) = dirs::home_dir() {
        if let Ok(stripped) = path.strip_prefix(&home) {
            return format!("~/{}", stripped.display());
        }
    }
    path.display().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{with_no_scoop_home, with_temp_scoop_home};
    use serial_test::serial;

    #[test]
    fn test_scoop_home_default() {
        with_no_scoop_home(|| {
            let home = scoop_home().unwrap();
            assert!(home.ends_with(".scoop"));
        });
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
    fn test_pythons_dir() {
        with_temp_scoop_home(|temp_dir| {
            let pythons = pythons_dir().unwrap();
            assert_eq!(pythons, temp_dir.path().join("pythons"));
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
    fn test_virtualenv_python() {
        with_temp_scoop_home(|temp_dir| {
            let python = virtualenv_python("myenv").unwrap();
            assert_eq!(
                python,
                temp_dir
                    .path()
                    .join("virtualenvs")
                    .join("myenv")
                    .join("bin")
                    .join("python")
            );
        });
    }

    #[test]
    #[serial]
    fn test_ensure_scoop_dirs() {
        with_temp_scoop_home(|temp_dir| {
            ensure_scoop_dirs().unwrap();
            assert!(temp_dir.path().exists());
            assert!(temp_dir.path().join("virtualenvs").exists());
            assert!(temp_dir.path().join("pythons").exists());
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
        assert_eq!(version_file, dir.join(".scoop-version"));
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
    fn test_abbreviate_home_with_home_path() {
        // Path under home directory should be abbreviated
        if let Some(home) = dirs::home_dir() {
            let path = home.join(".scoop").join("virtualenvs").join("myenv");
            let result = abbreviate_home(&path);
            assert!(result.starts_with("~/"));
            assert!(result.contains(".scoop/virtualenvs/myenv"));
        }
    }

    #[test]
    fn test_abbreviate_home_outside_home() {
        // Path outside home directory should remain unchanged
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
}
