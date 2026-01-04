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
