//! Version file service

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
        let version_file = paths::local_version_file(dir);
        Self::read_version_file(&version_file)
    }

    /// Get the global version
    pub fn get_global() -> Option<String> {
        let version_file = paths::global_version_file().ok()?;
        Self::read_version_file(&version_file)
    }

    /// Resolve the version for a directory (local -> parent -> global)
    pub fn resolve(dir: &Path) -> Option<String> {
        // Check current and parent directories for local version
        let mut current = dir.to_path_buf();
        loop {
            if let Some(version) = Self::get_local(&current) {
                return Some(version);
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
    fn read_version_file(path: &PathBuf) -> Option<String> {
        fs::read_to_string(path)
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
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
    use tempfile::TempDir;

    #[test]
    fn test_set_and_get_local() {
        let temp = TempDir::new().unwrap();
        let dir = temp.path();

        VersionService::set_local(dir, "myenv").unwrap();
        assert_eq!(VersionService::get_local(dir), Some("myenv".to_string()));
    }
}
