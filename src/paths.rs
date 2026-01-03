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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scoop_home_default() {
        // SAFETY: Test runs in a single thread, no concurrent access
        unsafe { std::env::remove_var(SCOOP_HOME_ENV) };
        let home = scoop_home().unwrap();
        assert!(home.ends_with(".scoop"));
    }

    #[test]
    fn test_scoop_home_env() {
        // SAFETY: Test runs in a single thread, no concurrent access
        unsafe { std::env::set_var(SCOOP_HOME_ENV, "/tmp/test-scoop") };
        let home = scoop_home().unwrap();
        assert_eq!(home, PathBuf::from("/tmp/test-scoop"));
        // SAFETY: Test runs in a single thread, no concurrent access
        unsafe { std::env::remove_var(SCOOP_HOME_ENV) };
    }
}
