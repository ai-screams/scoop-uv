//! uv CLI client

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::{Result, UvenvError};

/// Client for interacting with the uv CLI
pub struct UvClient {
    /// Path to the uv executable
    path: PathBuf,
}

impl UvClient {
    /// Create a new UvClient, finding uv in PATH
    pub fn new() -> Result<Self> {
        let path = which::which("uv").map_err(|_| UvenvError::UvNotFound)?;
        Ok(Self { path })
    }

    /// Create a new UvClient with a specific path
    pub fn with_path(path: PathBuf) -> Self {
        Self { path }
    }

    /// Get the uv version
    pub fn version(&self) -> Result<String> {
        let output = Command::new(&self.path)
            .arg("--version")
            .output()
            .map_err(|e| UvenvError::UvCommandFailed {
                command: "uv --version".to_string(),
                message: e.to_string(),
            })?;

        if !output.status.success() {
            return Err(UvenvError::UvCommandFailed {
                command: "uv --version".to_string(),
                message: String::from_utf8_lossy(&output.stderr).to_string(),
            });
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Create a virtual environment
    pub fn create_venv(&self, path: &Path, python_version: &str) -> Result<()> {
        let output = Command::new(&self.path)
            .arg("venv")
            .arg(path)
            .arg("--python")
            .arg(python_version)
            .output()
            .map_err(|e| UvenvError::UvCommandFailed {
                command: format!("uv venv {} --python {}", path.display(), python_version),
                message: e.to_string(),
            })?;

        if !output.status.success() {
            return Err(UvenvError::UvCommandFailed {
                command: format!("uv venv {} --python {}", path.display(), python_version),
                message: String::from_utf8_lossy(&output.stderr).to_string(),
            });
        }

        Ok(())
    }

    /// Install a Python version
    pub fn install_python(&self, version: &str) -> Result<PathBuf> {
        let output = Command::new(&self.path)
            .arg("python")
            .arg("install")
            .arg(version)
            .output()
            .map_err(|e| UvenvError::UvCommandFailed {
                command: format!("uv python install {version}"),
                message: e.to_string(),
            })?;

        if !output.status.success() {
            return Err(UvenvError::UvCommandFailed {
                command: format!("uv python install {version}"),
                message: String::from_utf8_lossy(&output.stderr).to_string(),
            });
        }

        // TODO: Parse output to get installed path
        Ok(PathBuf::new())
    }

    /// List installed Python versions
    pub fn list_pythons(&self) -> Result<Vec<String>> {
        let output = Command::new(&self.path)
            .arg("python")
            .arg("list")
            .output()
            .map_err(|e| UvenvError::UvCommandFailed {
                command: "uv python list".to_string(),
                message: e.to_string(),
            })?;

        if !output.status.success() {
            return Err(UvenvError::UvCommandFailed {
                command: "uv python list".to_string(),
                message: String::from_utf8_lossy(&output.stderr).to_string(),
            });
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let versions: Vec<String> = stdout.lines().map(|s| s.trim().to_string()).collect();

        Ok(versions)
    }
}

impl Default for UvClient {
    fn default() -> Self {
        Self::new().expect("uv not found in PATH")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uv_client_creation() {
        // This test will only pass if uv is installed
        if which::which("uv").is_ok() {
            let client = UvClient::new();
            assert!(client.is_ok());
        }
    }
}
