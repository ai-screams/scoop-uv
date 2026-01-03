//! Path utilities for uvenv

use std::path::PathBuf;

use crate::error::{Result, UvenvError};

/// Environment variable for uvenv home directory
pub const UVENV_HOME_ENV: &str = "UVENV_HOME";

/// Default uvenv home directory name
const UVENV_HOME_DIR: &str = ".uvenv";

/// Version file name
pub const VERSION_FILE: &str = ".uvenv-version";

/// Get the uvenv home directory (~/.uvenv or $UVENV_HOME)
pub fn uvenv_home() -> Result<PathBuf> {
    if let Ok(home) = std::env::var(UVENV_HOME_ENV) {
        return Ok(PathBuf::from(home));
    }

    dirs::home_dir()
        .map(|h| h.join(UVENV_HOME_DIR))
        .ok_or(UvenvError::HomeNotFound)
}

/// Get the virtualenvs directory (~/.uvenv/virtualenvs)
pub fn virtualenvs_dir() -> Result<PathBuf> {
    Ok(uvenv_home()?.join("virtualenvs"))
}

/// Get the pythons directory (~/.uvenv/pythons)
pub fn pythons_dir() -> Result<PathBuf> {
    Ok(uvenv_home()?.join("pythons"))
}

/// Get the global version file path (~/.uvenv/version)
pub fn global_version_file() -> Result<PathBuf> {
    Ok(uvenv_home()?.join("version"))
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
    fn test_uvenv_home_default() {
        // SAFETY: Test runs in a single thread, no concurrent access
        unsafe { std::env::remove_var(UVENV_HOME_ENV) };
        let home = uvenv_home().unwrap();
        assert!(home.ends_with(".uvenv"));
    }

    #[test]
    fn test_uvenv_home_env() {
        // SAFETY: Test runs in a single thread, no concurrent access
        unsafe { std::env::set_var(UVENV_HOME_ENV, "/tmp/test-uvenv") };
        let home = uvenv_home().unwrap();
        assert_eq!(home, PathBuf::from("/tmp/test-uvenv"));
        // SAFETY: Test runs in a single thread, no concurrent access
        unsafe { std::env::remove_var(UVENV_HOME_ENV) };
    }
}
