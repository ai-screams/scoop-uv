//! Virtual environment service

use std::fs;
use std::path::{Path, PathBuf};

use crate::core::Metadata;
use crate::error::{Result, ScoopError};
use crate::paths;
use crate::uv::UvClient;
use crate::validate;

/// Information about a virtual environment
#[derive(Debug, Clone)]
pub struct VirtualenvInfo {
    /// Name of the environment
    pub name: String,
    /// Path to the environment
    pub path: PathBuf,
    /// Python version (if metadata exists)
    pub python_version: Option<String>,
}

/// Service for managing virtual environments
pub struct VirtualenvService {
    uv: UvClient,
}

impl VirtualenvService {
    /// Create a new service with the given uv client
    pub fn new(uv: UvClient) -> Self {
        Self { uv }
    }

    /// Create a new service, finding uv automatically
    pub fn auto() -> Result<Self> {
        Ok(Self::new(UvClient::new()?))
    }

    /// List all virtual environments
    pub fn list(&self) -> Result<Vec<VirtualenvInfo>> {
        let venvs_dir = paths::virtualenvs_dir()?;

        if !venvs_dir.exists() {
            return Ok(Vec::new());
        }

        let mut envs = Vec::new();

        for entry in fs::read_dir(&venvs_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    let metadata = self.read_metadata(&path);
                    envs.push(VirtualenvInfo {
                        name: name.to_string(),
                        path: path.clone(),
                        python_version: metadata.map(|m| m.python_version),
                    });
                }
            }
        }

        envs.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(envs)
    }

    /// Create a new virtual environment
    pub fn create(&self, name: &str, python_version: &str) -> Result<PathBuf> {
        validate::validate_env_name(name)?;

        let path = paths::virtualenv_path(name)?;

        if path.exists() {
            return Err(ScoopError::VirtualenvExists {
                name: name.to_string(),
            });
        }

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Create the virtual environment
        self.uv.create_venv(&path, python_version)?;

        // Write metadata
        let uv_version = self.uv.version().ok();
        let metadata = Metadata::new(name.to_string(), python_version.to_string(), uv_version);
        self.write_metadata(&path, &metadata)?;

        Ok(path)
    }

    /// Delete a virtual environment
    pub fn delete(&self, name: &str) -> Result<()> {
        let path = paths::virtualenv_path(name)?;

        if !path.exists() {
            return Err(ScoopError::VirtualenvNotFound {
                name: name.to_string(),
            });
        }

        fs::remove_dir_all(&path)?;
        Ok(())
    }

    /// Check if a virtual environment exists
    pub fn exists(&self, name: &str) -> Result<bool> {
        let path = paths::virtualenv_path(name)?;
        Ok(path.exists())
    }

    /// Get the path to a virtual environment
    pub fn get_path(&self, name: &str) -> Result<PathBuf> {
        let path = paths::virtualenv_path(name)?;
        if !path.exists() {
            return Err(ScoopError::VirtualenvNotFound {
                name: name.to_string(),
            });
        }
        Ok(path)
    }

    /// Read metadata from a virtual environment
    fn read_metadata(&self, path: &Path) -> Option<Metadata> {
        let metadata_path = path.join(Metadata::FILE_NAME);
        let content = fs::read_to_string(metadata_path).ok()?;
        serde_json::from_str(&content).ok()
    }

    /// Write metadata to a virtual environment
    fn write_metadata(&self, path: &Path, metadata: &Metadata) -> Result<()> {
        let metadata_path = path.join(Metadata::FILE_NAME);
        let content = serde_json::to_string_pretty(metadata)?;
        fs::write(metadata_path, content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{create_mock_venv, with_temp_scoop_home};
    use serial_test::serial;

    /// Helper to get VirtualenvService, skipping test if uv not available.
    /// Returns None if uv is not installed, allowing graceful test skip.
    fn get_service() -> Option<VirtualenvService> {
        crate::uv::UvClient::new().ok().map(VirtualenvService::new)
    }

    /// Macro to skip test if uv is not available.
    /// This makes the skip explicit in test output.
    macro_rules! require_uv {
        () => {
            match get_service() {
                Some(service) => service,
                None => {
                    eprintln!("SKIPPED: uv not installed");
                    return;
                }
            }
        };
    }

    #[test]
    fn test_virtualenv_info_struct() {
        let info = VirtualenvInfo {
            name: "testenv".to_string(),
            path: PathBuf::from("/path/to/env"),
            python_version: Some("3.12".to_string()),
        };

        assert_eq!(info.name, "testenv");
        assert_eq!(info.path, PathBuf::from("/path/to/env"));
        assert_eq!(info.python_version, Some("3.12".to_string()));
    }

    #[test]
    fn test_virtualenv_info_clone() {
        let info = VirtualenvInfo {
            name: "clonetest".to_string(),
            path: PathBuf::from("/clone"),
            python_version: None,
        };
        let cloned = info.clone();

        assert_eq!(cloned.name, info.name);
        assert_eq!(cloned.path, info.path);
        assert_eq!(cloned.python_version, info.python_version);
    }

    #[test]
    #[serial]
    fn test_list_empty_when_no_venvs_dir() {
        with_temp_scoop_home(|_temp_dir| {
            let service = require_uv!();
            let result = service.list().unwrap();
            assert!(result.is_empty());
        });
    }

    #[test]
    #[serial]
    fn test_list_returns_envs_sorted() {
        with_temp_scoop_home(|temp_dir| {
            // Arrange: Create mock venvs in reverse alphabetical order
            create_mock_venv(temp_dir, "zeta", Some("3.11"));
            create_mock_venv(temp_dir, "alpha", Some("3.12"));
            create_mock_venv(temp_dir, "beta", None);

            // Act
            let service = require_uv!();
            let envs = service.list().unwrap();

            // Assert
            assert_eq!(envs.len(), 3);
            assert_eq!(envs[0].name, "alpha");
            assert_eq!(envs[1].name, "beta");
            assert_eq!(envs[2].name, "zeta");
        });
    }

    #[test]
    #[serial]
    fn test_list_reads_python_version_from_metadata() {
        with_temp_scoop_home(|temp_dir| {
            create_mock_venv(temp_dir, "withversion", Some("3.12.1"));
            create_mock_venv(temp_dir, "noversion", None);

            let service = require_uv!();
            let envs = service.list().unwrap();

            let with_ver = envs.iter().find(|e| e.name == "withversion").unwrap();
            let no_ver = envs.iter().find(|e| e.name == "noversion").unwrap();

            assert_eq!(with_ver.python_version, Some("3.12.1".to_string()));
            assert_eq!(no_ver.python_version, None);
        });
    }

    #[test]
    #[serial]
    fn test_exists_returns_false_for_nonexistent() {
        with_temp_scoop_home(|_temp_dir| {
            let service = require_uv!();
            assert!(!service.exists("nonexistent").unwrap());
        });
    }

    #[test]
    #[serial]
    fn test_exists_returns_true_for_existing() {
        with_temp_scoop_home(|temp_dir| {
            create_mock_venv(temp_dir, "exists", None);

            let service = require_uv!();
            assert!(service.exists("exists").unwrap());
        });
    }

    #[test]
    #[serial]
    fn test_get_path_returns_error_for_nonexistent() {
        with_temp_scoop_home(|_temp_dir| {
            let service = require_uv!();
            let result = service.get_path("nonexistent");

            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(matches!(err, ScoopError::VirtualenvNotFound { .. }));
        });
    }

    #[test]
    #[serial]
    fn test_get_path_returns_path_for_existing() {
        with_temp_scoop_home(|temp_dir| {
            create_mock_venv(temp_dir, "myenv", None);

            let service = require_uv!();
            let path = service.get_path("myenv").unwrap();
            assert!(path.ends_with("myenv"));
            assert!(path.exists());
        });
    }

    #[test]
    #[serial]
    fn test_delete_removes_directory() {
        with_temp_scoop_home(|temp_dir| {
            create_mock_venv(temp_dir, "todelete", Some("3.12"));
            let venv_path = temp_dir.path().join("virtualenvs").join("todelete");
            assert!(venv_path.exists());

            let service = require_uv!();
            service.delete("todelete").unwrap();
            assert!(!venv_path.exists());
        });
    }

    #[test]
    #[serial]
    fn test_delete_returns_error_for_nonexistent() {
        with_temp_scoop_home(|temp_dir| {
            // Arrange: Create virtualenvs dir but not the specific venv
            fs::create_dir_all(temp_dir.path().join("virtualenvs")).unwrap();

            // Act
            let service = require_uv!();
            let result = service.delete("nonexistent");

            // Assert
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(matches!(err, ScoopError::VirtualenvNotFound { .. }));
        });
    }

    #[test]
    #[serial]
    fn test_list_ignores_files() {
        with_temp_scoop_home(|temp_dir| {
            let venvs_dir = temp_dir.path().join("virtualenvs");
            fs::create_dir_all(&venvs_dir).unwrap();

            // Create a file (not directory) - should be ignored
            fs::write(venvs_dir.join("notadir"), "test").unwrap();
            // Create a real venv directory
            create_mock_venv(temp_dir, "realenv", None);

            let service = require_uv!();
            let envs = service.list().unwrap();

            assert_eq!(envs.len(), 1);
            assert_eq!(envs[0].name, "realenv");
        });
    }
}
