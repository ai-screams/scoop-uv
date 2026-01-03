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
    use std::sync::Mutex;
    use tempfile::TempDir;

    /// Mutex to synchronize tests that manipulate SCOOP_HOME environment variable.
    /// Environment variables are process-global state, so concurrent access causes race conditions.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn with_temp_scoop_home<F, T>(f: F) -> T
    where
        F: FnOnce(&TempDir) -> T,
    {
        let _guard = ENV_LOCK.lock().unwrap();
        let temp_dir = TempDir::new().unwrap();
        // SAFETY: Protected by ENV_LOCK mutex
        unsafe { std::env::set_var(SCOOP_HOME_ENV, temp_dir.path()) };
        let result = f(&temp_dir);
        // SAFETY: Protected by ENV_LOCK mutex
        unsafe { std::env::remove_var(SCOOP_HOME_ENV) };
        result
    }

    #[test]
    fn test_scoop_home_default() {
        let _guard = ENV_LOCK.lock().unwrap();
        // SAFETY: Protected by ENV_LOCK mutex
        unsafe { std::env::remove_var(SCOOP_HOME_ENV) };
        let home = scoop_home().unwrap();
        assert!(home.ends_with(".scoop"));
    }

    #[test]
    fn test_scoop_home_env() {
        let _guard = ENV_LOCK.lock().unwrap();
        // SAFETY: Protected by ENV_LOCK mutex
        unsafe { std::env::set_var(SCOOP_HOME_ENV, "/tmp/test-scoop") };
        let home = scoop_home().unwrap();
        assert_eq!(home, PathBuf::from("/tmp/test-scoop"));
        // SAFETY: Protected by ENV_LOCK mutex
        unsafe { std::env::remove_var(SCOOP_HOME_ENV) };
    }

    #[test]
    fn test_virtualenvs_dir() {
        with_temp_scoop_home(|temp_dir| {
            let venvs = virtualenvs_dir().unwrap();
            assert_eq!(venvs, temp_dir.path().join("virtualenvs"));
        });
    }

    #[test]
    fn test_pythons_dir() {
        with_temp_scoop_home(|temp_dir| {
            let pythons = pythons_dir().unwrap();
            assert_eq!(pythons, temp_dir.path().join("pythons"));
        });
    }

    #[test]
    fn test_virtualenv_path() {
        with_temp_scoop_home(|temp_dir| {
            let path = virtualenv_path("myenv").unwrap();
            assert_eq!(path, temp_dir.path().join("virtualenvs").join("myenv"));
        });
    }

    #[test]
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
    fn test_ensure_scoop_dirs() {
        with_temp_scoop_home(|temp_dir| {
            ensure_scoop_dirs().unwrap();
            assert!(temp_dir.path().exists());
            assert!(temp_dir.path().join("virtualenvs").exists());
            assert!(temp_dir.path().join("pythons").exists());
        });
    }

    #[test]
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
        let dir = PathBuf::from("/some/project");
        let version_file = local_version_file(&dir);
        assert_eq!(version_file, dir.join(".scoop-version"));
    }

    #[test]
    fn test_global_version_file() {
        with_temp_scoop_home(|temp_dir| {
            let version_file = global_version_file().unwrap();
            assert_eq!(version_file, temp_dir.path().join("version"));
        });
    }

    #[cfg(unix)]
    #[test]
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
}
