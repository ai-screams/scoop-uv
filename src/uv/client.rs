//! uv CLI client

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::{Result, ScoopError};
use crate::validate::PythonVersion;

/// Information about an installed Python version
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PythonInfo {
    /// The version string (e.g., "3.12.0")
    pub version: String,
    /// Path to the Python executable
    pub path: Option<PathBuf>,
    /// Whether this version is installed locally by uv
    pub installed: bool,
    /// Implementation (cpython, pypy, etc.)
    pub implementation: String,
}

/// Client for interacting with the uv CLI
pub struct UvClient {
    /// Path to the uv executable
    path: PathBuf,
}

impl UvClient {
    /// Create a new UvClient, finding uv in PATH
    pub fn new() -> Result<Self> {
        let path = which::which("uv").map_err(|_| ScoopError::UvNotFound)?;
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
            .map_err(|e| ScoopError::UvCommandFailed {
                command: "uv --version".to_string(),
                message: e.to_string(),
            })?;

        if !output.status.success() {
            return Err(ScoopError::UvCommandFailed {
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
            .map_err(|e| ScoopError::UvCommandFailed {
                command: format!("uv venv {} --python {}", path.display(), python_version),
                message: e.to_string(),
            })?;

        if !output.status.success() {
            return Err(ScoopError::UvCommandFailed {
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
            .map_err(|e| ScoopError::UvCommandFailed {
                command: format!("uv python install {version}"),
                message: e.to_string(),
            })?;

        if !output.status.success() {
            return Err(ScoopError::UvCommandFailed {
                command: format!("uv python install {version}"),
                message: String::from_utf8_lossy(&output.stderr).to_string(),
            });
        }

        // TODO: Parse output to get installed path
        Ok(PathBuf::new())
    }

    /// List installed Python versions (raw output)
    pub fn list_pythons(&self) -> Result<Vec<String>> {
        let output = Command::new(&self.path)
            .arg("python")
            .arg("list")
            .output()
            .map_err(|e| ScoopError::UvCommandFailed {
                command: "uv python list".to_string(),
                message: e.to_string(),
            })?;

        if !output.status.success() {
            return Err(ScoopError::UvCommandFailed {
                command: "uv python list".to_string(),
                message: String::from_utf8_lossy(&output.stderr).to_string(),
            });
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let versions: Vec<String> = stdout.lines().map(|s| s.trim().to_string()).collect();

        Ok(versions)
    }

    /// List installed Python versions with structured info
    pub fn list_installed_pythons(&self) -> Result<Vec<PythonInfo>> {
        let output = Command::new(&self.path)
            .arg("python")
            .arg("list")
            .arg("--only-installed")
            .output()
            .map_err(|e| ScoopError::UvCommandFailed {
                command: "uv python list --only-installed".to_string(),
                message: e.to_string(),
            })?;

        if !output.status.success() {
            return Err(ScoopError::UvCommandFailed {
                command: "uv python list --only-installed".to_string(),
                message: String::from_utf8_lossy(&output.stderr).to_string(),
            });
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let pythons = parse_python_list(&stdout);

        Ok(pythons)
    }

    /// Uninstall a Python version
    pub fn uninstall_python(&self, version: &str) -> Result<()> {
        let output = Command::new(&self.path)
            .arg("python")
            .arg("uninstall")
            .arg(version)
            .output()
            .map_err(|e| ScoopError::PythonUninstallFailed {
                version: version.to_string(),
                message: e.to_string(),
            })?;

        if !output.status.success() {
            return Err(ScoopError::PythonUninstallFailed {
                version: version.to_string(),
                message: String::from_utf8_lossy(&output.stderr).to_string(),
            });
        }

        Ok(())
    }

    /// Find an installed Python matching the version pattern
    pub fn find_python(&self, version_pattern: &str) -> Result<Option<PythonInfo>> {
        let installed = self.list_installed_pythons()?;

        if let Some(pattern) = PythonVersion::parse(version_pattern) {
            for info in installed {
                if let Some(ver) = PythonVersion::parse(&info.version) {
                    if pattern.matches(&ver) {
                        return Ok(Some(info));
                    }
                }
            }
        }

        Ok(None)
    }

    /// Get the latest installed Python version
    pub fn latest_installed_python(&self) -> Result<Option<PythonInfo>> {
        let mut installed = self.list_installed_pythons()?;

        // Sort by version descending
        installed.sort_by(|a, b| {
            let va = PythonVersion::parse(&a.version);
            let vb = PythonVersion::parse(&b.version);
            match (va, vb) {
                (Some(a), Some(b)) => match b.major.cmp(&a.major) {
                    std::cmp::Ordering::Equal => match (b.minor, a.minor) {
                        (Some(bm), Some(am)) => bm.cmp(&am),
                        _ => std::cmp::Ordering::Equal,
                    },
                    other => other,
                },
                _ => std::cmp::Ordering::Equal,
            }
        });

        Ok(installed.into_iter().next())
    }
}

/// Parse uv python list output into structured info
fn parse_python_list(output: &str) -> Vec<PythonInfo> {
    let mut pythons = Vec::new();

    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // uv python list output format varies:
        // "cpython-3.12.0-macos-aarch64-none    /path/to/python"
        // "cpython-3.12.0    <download available>"
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        let version_part = parts[0];
        let path = if parts.len() > 1 && !parts[1].starts_with('<') {
            Some(PathBuf::from(parts[1]))
        } else {
            None
        };

        // Parse "cpython-3.12.0-macos-aarch64-none" format
        let segments: Vec<&str> = version_part.split('-').collect();
        if segments.is_empty() {
            continue;
        }

        let implementation = segments[0].to_string();
        let version = if segments.len() > 1 {
            segments[1].to_string()
        } else {
            version_part.to_string()
        };

        let installed = path.is_some();

        pythons.push(PythonInfo {
            version,
            path,
            installed,
            implementation,
        });
    }

    pythons
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

    #[test]
    fn test_parse_python_list_with_paths() {
        let output = r#"
cpython-3.12.0-macos-aarch64-none    /Users/test/.local/share/uv/python/cpython-3.12.0-macos-aarch64-none/bin/python3
cpython-3.11.8-macos-aarch64-none    /Users/test/.local/share/uv/python/cpython-3.11.8-macos-aarch64-none/bin/python3
"#;
        let pythons = parse_python_list(output);
        assert_eq!(pythons.len(), 2);

        assert_eq!(pythons[0].version, "3.12.0");
        assert_eq!(pythons[0].implementation, "cpython");
        assert!(pythons[0].installed);
        assert!(pythons[0].path.is_some());

        assert_eq!(pythons[1].version, "3.11.8");
        assert_eq!(pythons[1].implementation, "cpython");
    }

    #[test]
    fn test_parse_python_list_without_paths() {
        let output = r#"
cpython-3.13.0    <download available>
cpython-3.12.0    <download available>
"#;
        let pythons = parse_python_list(output);
        assert_eq!(pythons.len(), 2);

        assert_eq!(pythons[0].version, "3.13.0");
        assert!(!pythons[0].installed);
        assert!(pythons[0].path.is_none());
    }

    #[test]
    fn test_parse_python_list_mixed() {
        let output = r#"
cpython-3.12.0-macos-aarch64-none    /path/to/python3
cpython-3.11.0    <download available>
pypy-3.10.0-macos-aarch64-none    /path/to/pypy
"#;
        let pythons = parse_python_list(output);
        assert_eq!(pythons.len(), 3);

        assert_eq!(pythons[0].implementation, "cpython");
        assert!(pythons[0].installed);

        assert_eq!(pythons[1].implementation, "cpython");
        assert!(!pythons[1].installed);

        assert_eq!(pythons[2].implementation, "pypy");
        assert!(pythons[2].installed);
    }

    #[test]
    fn test_parse_python_list_empty() {
        let output = "";
        let pythons = parse_python_list(output);
        assert!(pythons.is_empty());
    }

    #[test]
    fn test_python_info_equality() {
        let info1 = PythonInfo {
            version: "3.12.0".to_string(),
            path: Some(PathBuf::from("/path/to/python")),
            installed: true,
            implementation: "cpython".to_string(),
        };

        let info2 = PythonInfo {
            version: "3.12.0".to_string(),
            path: Some(PathBuf::from("/path/to/python")),
            installed: true,
            implementation: "cpython".to_string(),
        };

        assert_eq!(info1, info2);
    }

    #[test]
    fn test_uv_client_with_path() {
        let client = UvClient::with_path(PathBuf::from("/usr/bin/uv"));
        assert_eq!(client.path, PathBuf::from("/usr/bin/uv"));
    }
}
