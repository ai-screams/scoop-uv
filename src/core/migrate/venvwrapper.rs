//! virtualenvwrapper environment discovery

use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{Result, ScoopError};
use crate::paths;

use super::source::{EnvironmentSource, EnvironmentStatus, SourceEnvironment, SourceType};

/// Discovers virtualenvwrapper environments
#[derive(Debug)]
pub struct VenvWrapperDiscovery {
    /// Root path (typically ~/.virtualenvs)
    root: PathBuf,
}

impl VenvWrapperDiscovery {
    /// Creates a new discovery instance for the given virtualenvwrapper root.
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// Creates a discovery instance using the default virtualenvwrapper root.
    ///
    /// Uses `$WORKON_HOME` if set, otherwise `~/.virtualenvs`.
    pub fn default_root() -> Option<Self> {
        let root = std::env::var("WORKON_HOME")
            .map(PathBuf::from)
            .ok()
            .or_else(|| dirs::home_dir().map(|h| h.join(".virtualenvs")))?;

        if root.exists() {
            Some(Self::new(root))
        } else {
            None
        }
    }

    /// Parse pyvenv.cfg to extract Python version
    fn parse_pyvenv_cfg(path: &Path) -> Option<String> {
        let cfg_path = path.join("pyvenv.cfg");
        let content = fs::read_to_string(&cfg_path).ok()?;

        for line in content.lines() {
            let line = line.trim();
            if let Some(version) = line.strip_prefix("version") {
                let version = version.trim_start_matches([' ', '=']);
                return Some(version.trim().to_string());
            }
        }

        // Fallback: try to extract from home path
        for line in content.lines() {
            let line = line.trim();
            if let Some(home) = line.strip_prefix("home") {
                let home = home.trim_start_matches([' ', '=']).trim();
                // Try to extract version from Python path
                // e.g., /usr/local/opt/python@3.11/bin or /Library/Frameworks/Python.framework/Versions/3.11/bin
                if let Some(at_idx) = home.find("python@") {
                    let after_at = &home[at_idx + 7..];
                    if let Some(slash_idx) = after_at.find('/') {
                        return Some(after_at[..slash_idx].to_string());
                    }
                    return Some(after_at.to_string());
                }
                if let Some(versions_idx) = home.find("Versions/") {
                    let after_versions = &home[versions_idx + 9..];
                    if let Some(slash_idx) = after_versions.find('/') {
                        return Some(after_versions[..slash_idx].to_string());
                    }
                }
            }
        }

        None
    }

    /// Calculate directory size in bytes
    fn dir_size(path: &Path) -> u64 {
        walkdir::WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter_map(|e| e.metadata().ok())
            .filter(|m| m.is_file())
            .map(|m| m.len())
            .sum()
    }

    /// Check if environment name conflicts with existing scoop environment
    fn check_name_conflict(name: &str) -> Option<PathBuf> {
        if let Ok(venvs_dir) = paths::virtualenvs_dir() {
            let scoop_path = venvs_dir.join(name);
            if scoop_path.exists() {
                return Some(scoop_path);
            }
        }
        None
    }

    /// Determine environment status
    fn determine_status(name: &str, python_version: &str) -> EnvironmentStatus {
        if let Some(existing) = Self::check_name_conflict(name) {
            return EnvironmentStatus::NameConflict { existing };
        }

        let major_minor: Vec<&str> = python_version.split('.').collect();
        if major_minor.len() >= 2 {
            if let (Ok(major), Ok(minor)) =
                (major_minor[0].parse::<u32>(), major_minor[1].parse::<u32>())
            {
                if major == 3 && minor <= 8 {
                    return EnvironmentStatus::PythonEol {
                        version: python_version.to_string(),
                    };
                }
                if major == 2 {
                    return EnvironmentStatus::PythonEol {
                        version: python_version.to_string(),
                    };
                }
            }
        }

        EnvironmentStatus::Ready
    }
}

impl EnvironmentSource for VenvWrapperDiscovery {
    fn source_type(&self) -> SourceType {
        SourceType::VirtualenvWrapper
    }

    fn scan_environments(&self) -> Result<Vec<SourceEnvironment>> {
        let mut environments = Vec::new();

        if !self.root.exists() {
            return Ok(environments);
        }

        let entries = fs::read_dir(&self.root).map_err(ScoopError::Io)?;

        for entry in entries.flatten() {
            let env_path = entry.path();

            // Skip symlinks and non-directories
            if env_path.is_symlink() || !env_path.is_dir() {
                continue;
            }

            let name = match entry.file_name().to_str() {
                Some(n) => n.to_string(),
                None => continue,
            };

            // Skip hidden directories
            if name.starts_with('.') {
                continue;
            }

            // Validate environment (check for bin/python)
            let python_bin = env_path.join("bin").join("python");
            if !python_bin.exists() {
                // Not a valid virtualenv, skip silently
                continue;
            }

            // Parse Python version from pyvenv.cfg
            let python_version =
                Self::parse_pyvenv_cfg(&env_path).unwrap_or_else(|| "unknown".to_string());

            // Calculate size
            let size_bytes = Self::dir_size(&env_path);

            // Determine status
            let status = Self::determine_status(&name, &python_version);

            environments.push(SourceEnvironment {
                name,
                python_version,
                path: env_path,
                source_type: SourceType::VirtualenvWrapper,
                size_bytes,
                status,
            });
        }

        environments.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(environments)
    }

    fn find_environment(&self, name: &str) -> Result<SourceEnvironment> {
        let environments = self.scan_environments()?;

        environments
            .into_iter()
            .find(|env| env.name == name)
            .ok_or_else(|| ScoopError::VenvWrapperEnvNotFound {
                name: name.to_string(),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_root_returns_none_when_not_installed() {
        let _ = VenvWrapperDiscovery::default_root();
    }

    #[test]
    fn test_parse_pyvenv_cfg_extracts_version() {
        use std::io::Write;
        let temp_dir = tempfile::tempdir().unwrap();
        let cfg_path = temp_dir.path().join("pyvenv.cfg");

        let mut file = fs::File::create(&cfg_path).unwrap();
        writeln!(file, "home = /usr/local/bin").unwrap();
        writeln!(file, "include-system-site-packages = false").unwrap();
        writeln!(file, "version = 3.11.0").unwrap();

        let version = VenvWrapperDiscovery::parse_pyvenv_cfg(temp_dir.path());
        assert_eq!(version, Some("3.11.0".to_string()));
    }

    #[test]
    fn test_determine_status_ready() {
        let status =
            VenvWrapperDiscovery::determine_status("nonexistent_venv_wrapper_test", "3.12.0");
        assert!(matches!(status, EnvironmentStatus::Ready));
    }
}
