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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::with_temp_scoop_home;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_scoop_home_default() {
        // SAFETY: serial_test ensures exclusive access
        unsafe { std::env::remove_var(SCOOP_HOME_ENV) };
        let home = scoop_home().unwrap();
        assert!(home.ends_with(".scoop"));
    }

    #[test]
    #[serial]
    fn test_scoop_home_env() {
        // SAFETY: serial_test ensures exclusive access
        unsafe { std::env::set_var(SCOOP_HOME_ENV, "/tmp/test-scoop") };
        let home = scoop_home().unwrap();
        assert_eq!(home, PathBuf::from("/tmp/test-scoop"));
        // SAFETY: cleanup
        unsafe { std::env::remove_var(SCOOP_HOME_ENV) };
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
}
