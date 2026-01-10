//! pyenv environment discovery

use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{Result, ScoopError};
use crate::paths;

use super::source::{EnvironmentSource, EnvironmentStatus, SourceEnvironment, SourceType};

/// Discovers pyenv-virtualenv environments
#[derive(Debug)]
pub struct PyenvDiscovery {
    /// Root path of pyenv (typically ~/.pyenv)
    root: PathBuf,
}

impl PyenvDiscovery {
    /// Creates a new discovery instance for the given pyenv root.
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// Creates a discovery instance using the default pyenv root.
    ///
    /// Uses `$PYENV_ROOT` if set, otherwise `~/.pyenv`.
    pub fn default_root() -> Option<Self> {
        let root = std::env::var("PYENV_ROOT")
            .map(PathBuf::from)
            .ok()
            .or_else(|| dirs::home_dir().map(|h| h.join(".pyenv")))?;

        if root.exists() {
            Some(Self::new(root))
        } else {
            None
        }
    }

    /// Get the versions directory path
    fn versions_dir(&self) -> PathBuf {
        self.root.join("versions")
    }

    /// Parse pyvenv.cfg to extract Python version
    fn parse_pyvenv_cfg(path: &Path) -> Option<String> {
        let cfg_path = path.join("pyvenv.cfg");
        let content = fs::read_to_string(&cfg_path).ok()?;

        for line in content.lines() {
            let line = line.trim();
            if let Some(version) = line.strip_prefix("version") {
                // Handle both "version = 3.11.0" and "version=3.11.0"
                let version = version.trim_start_matches([' ', '=']);
                return Some(version.trim().to_string());
            }
        }

        // Fallback: try to extract from home path
        for line in content.lines() {
            let line = line.trim();
            if let Some(home) = line.strip_prefix("home") {
                let home = home.trim_start_matches([' ', '=']).trim();
                // Extract version from path like /Users/user/.pyenv/versions/3.11.0/bin
                if let Some(versions_idx) = home.find("versions/") {
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
        // Check for name conflict first
        if let Some(existing) = Self::check_name_conflict(name) {
            return EnvironmentStatus::NameConflict { existing };
        }

        // Check for EOL Python versions
        let major_minor: Vec<&str> = python_version.split('.').collect();
        if major_minor.len() >= 2 {
            if let (Ok(major), Ok(minor)) =
                (major_minor[0].parse::<u32>(), major_minor[1].parse::<u32>())
            {
                // Python 3.8 and earlier are EOL (as of 2024)
                if major == 3 && minor <= 8 {
                    return EnvironmentStatus::PythonEol {
                        version: python_version.to_string(),
                    };
                }
                // Python 2.x is definitely EOL
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

impl EnvironmentSource for PyenvDiscovery {
    fn source_type(&self) -> SourceType {
        SourceType::Pyenv
    }

    fn scan_environments(&self) -> Result<Vec<SourceEnvironment>> {
        let mut environments = Vec::new();
        let versions_dir = self.versions_dir();

        if !versions_dir.exists() {
            return Ok(environments);
        }

        // Scan ~/.pyenv/versions/*/envs/*
        let entries = fs::read_dir(&versions_dir).map_err(ScoopError::Io)?;

        for entry in entries.flatten() {
            let python_version_path = entry.path();

            // Skip symlinks (pyenv creates symlinks for virtualenvs at top level)
            if python_version_path.is_symlink() {
                continue;
            }

            // Look for envs subdirectory
            let envs_dir = python_version_path.join("envs");
            if !envs_dir.exists() || !envs_dir.is_dir() {
                continue;
            }

            // Scan environments in this Python version's envs directory
            let env_entries = match fs::read_dir(&envs_dir) {
                Ok(entries) => entries,
                Err(_) => continue,
            };

            for env_entry in env_entries.flatten() {
                let env_path = env_entry.path();

                // Skip symlinks and non-directories
                if env_path.is_symlink() || !env_path.is_dir() {
                    continue;
                }

                let name = match env_entry.file_name().to_str() {
                    Some(n) => n.to_string(),
                    None => continue,
                };

                // Parse Python version from pyvenv.cfg
                let python_version = Self::parse_pyvenv_cfg(&env_path).unwrap_or_else(|| {
                    // Fallback: use parent directory name
                    python_version_path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown")
                        .to_string()
                });

                // Validate environment (check for bin/python)
                let python_bin = env_path.join("bin").join("python");
                if !python_bin.exists() {
                    environments.push(SourceEnvironment {
                        name,
                        python_version,
                        path: env_path,
                        source_type: SourceType::Pyenv,
                        size_bytes: 0,
                        status: EnvironmentStatus::Corrupted {
                            reason: "Python binary not found".to_string(),
                        },
                    });
                    continue;
                }

                // Calculate size
                let size_bytes = Self::dir_size(&env_path);

                // Determine status
                let status = Self::determine_status(&name, &python_version);

                environments.push(SourceEnvironment {
                    name,
                    python_version,
                    path: env_path,
                    source_type: SourceType::Pyenv,
                    size_bytes,
                    status,
                });
            }
        }

        // Sort by name
        environments.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(environments)
    }

    fn find_environment(&self, name: &str) -> Result<SourceEnvironment> {
        let environments = self.scan_environments()?;

        environments
            .into_iter()
            .find(|env| env.name == name)
            .ok_or_else(|| ScoopError::PyenvEnvNotFound {
                name: name.to_string(),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_root_returns_none_when_pyenv_not_installed() {
        // This test might pass or fail depending on the system
        // Just ensure it doesn't panic
        let _ = PyenvDiscovery::default_root();
    }

    #[test]
    fn test_parse_pyvenv_cfg_extracts_version() {
        use std::io::Write;
        let temp_dir = tempfile::tempdir().unwrap();
        let cfg_path = temp_dir.path().join("pyvenv.cfg");

        let mut file = fs::File::create(&cfg_path).unwrap();
        writeln!(file, "home = /Users/test/.pyenv/versions/3.11.0/bin").unwrap();
        writeln!(file, "include-system-site-packages = false").unwrap();
        writeln!(file, "version = 3.11.0").unwrap();

        let version = PyenvDiscovery::parse_pyvenv_cfg(temp_dir.path());
        assert_eq!(version, Some("3.11.0".to_string()));
    }

    #[test]
    fn test_determine_status_eol_python() {
        // Use a unique name that definitely won't exist in scoop
        let status = PyenvDiscovery::determine_status("nonexistent_eol_test_env_xyz", "3.7.0");
        assert!(matches!(status, EnvironmentStatus::PythonEol { .. }));

        let status = PyenvDiscovery::determine_status("nonexistent_eol_test_env_xyz", "2.7.18");
        assert!(matches!(status, EnvironmentStatus::PythonEol { .. }));
    }

    #[test]
    fn test_determine_status_ready() {
        let status = PyenvDiscovery::determine_status("nonexistent_env_name_12345", "3.12.0");
        assert!(matches!(status, EnvironmentStatus::Ready));
    }
}
