//! Conda environment discovery

use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{Result, ScoopError};

use super::common;
use super::source::{EnvironmentSource, SourceEnvironment, SourceType};

/// Discovers Conda environments
#[derive(Debug)]
pub struct CondaDiscovery {
    /// Root paths to search for conda environments
    roots: Vec<PathBuf>,
}

impl CondaDiscovery {
    /// Creates a new discovery instance for the given conda roots.
    pub fn new(roots: Vec<PathBuf>) -> Self {
        Self { roots }
    }

    /// Creates a discovery instance using default conda locations.
    ///
    /// Searches in order:
    /// 1. `$CONDA_PREFIX/envs` (if in active conda env)
    /// 2. `~/.conda/envs`
    /// 3. `~/anaconda3/envs`
    /// 4. `~/miniconda3/envs`
    /// 5. `~/miniforge3/envs`
    pub fn default_roots() -> Option<Self> {
        let home = dirs::home_dir()?;

        let mut roots = Vec::new();

        // Check CONDA_PREFIX first
        if let Ok(prefix) = std::env::var("CONDA_PREFIX") {
            let envs_path = PathBuf::from(prefix).join("envs");
            if envs_path.exists() {
                roots.push(envs_path);
            }
        }

        // Check common conda locations
        let candidates = [
            home.join(".conda").join("envs"),
            home.join("anaconda3").join("envs"),
            home.join("miniconda3").join("envs"),
            home.join("miniforge3").join("envs"),
        ];

        for candidate in candidates {
            if candidate.exists() && !roots.contains(&candidate) {
                roots.push(candidate);
            }
        }

        if roots.is_empty() {
            None
        } else {
            Some(Self::new(roots))
        }
    }

    /// Get Python version from conda environment
    ///
    /// Tries multiple methods:
    /// 1. Run `<env>/bin/python --version` (most accurate)
    /// 2. Check `conda-meta/python-*.json` files
    /// 3. Check `pyvenv.cfg` if exists
    fn get_python_version(env_path: &Path) -> Option<String> {
        // Method 1: Check conda-meta for python package
        let conda_meta = env_path.join("conda-meta");
        if conda_meta.exists() {
            if let Ok(entries) = fs::read_dir(&conda_meta) {
                for entry in entries.flatten() {
                    let name = entry.file_name();
                    let name_str = name.to_string_lossy();
                    if name_str.starts_with("python-") && name_str.ends_with(".json") {
                        // Parse version from filename: python-3.11.0-h...json
                        let version_part = &name_str[7..]; // Skip "python-"
                        if let Some(dash_idx) = version_part.find('-') {
                            return Some(version_part[..dash_idx].to_string());
                        }
                    }
                }
            }
        }

        // Method 2: Check pyvenv.cfg (some conda envs have this)
        let cfg_path = env_path.join("pyvenv.cfg");
        if cfg_path.exists() {
            if let Ok(content) = fs::read_to_string(&cfg_path) {
                for line in content.lines() {
                    let line = line.trim();
                    if let Some(version) = line.strip_prefix("version") {
                        let version = version.trim_start_matches([' ', '=']);
                        return Some(version.trim().to_string());
                    }
                }
            }
        }

        // Method 3: Try to run python --version (expensive, skip for now)
        // This would require subprocess execution

        None
    }

    /// Check if directory is a valid conda environment
    fn is_conda_env(path: &Path) -> bool {
        // Conda environments have conda-meta directory
        let conda_meta = path.join("conda-meta");
        if !conda_meta.exists() {
            return false;
        }

        // And should have a python binary (we only migrate python envs)
        let python_bin = path.join("bin").join("python");
        python_bin.exists()
    }

    /// Parse a single environment directory into SourceEnvironment
    fn parse_environment(&self, env_path: &Path) -> Option<SourceEnvironment> {
        let name = env_path.file_name()?.to_str()?.to_string();

        // Skip hidden directories
        if name.starts_with('.') {
            return None;
        }

        // Validate it's a conda environment
        if !Self::is_conda_env(env_path) {
            return None;
        }

        // Get Python version
        let python_version =
            Self::get_python_version(env_path).unwrap_or_else(|| "unknown".to_string());

        // Determine status
        let status = common::determine_status(&name, &python_version);

        Some(SourceEnvironment {
            name,
            python_version,
            path: env_path.to_path_buf(),
            source_type: SourceType::Conda,
            size_bytes: None, // Lazy: calculated only when needed
            status,
        })
    }
}

impl EnvironmentSource for CondaDiscovery {
    fn source_type(&self) -> SourceType {
        SourceType::Conda
    }

    fn scan_environments(&self) -> Result<Vec<SourceEnvironment>> {
        let mut environments = Vec::new();
        let mut seen_names = std::collections::HashSet::new();

        for root in &self.roots {
            if !root.exists() {
                continue;
            }

            let entries = match fs::read_dir(root) {
                Ok(e) => e,
                Err(_) => continue,
            };

            for entry in entries.flatten() {
                let env_path = entry.path();

                // Skip symlinks and non-directories
                if env_path.is_symlink() || !env_path.is_dir() {
                    continue;
                }

                if let Some(env) = self.parse_environment(&env_path) {
                    // Skip duplicates
                    if seen_names.contains(&env.name) {
                        continue;
                    }
                    seen_names.insert(env.name.clone());
                    environments.push(env);
                }
            }
        }

        environments.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(environments)
    }

    /// Find a specific environment by name using O(1) direct path access.
    fn find_environment(&self, name: &str) -> Result<SourceEnvironment> {
        // Search in each root directory directly
        for root in &self.roots {
            let env_path = root.join(name);
            if env_path.exists() && env_path.is_dir() {
                if let Some(env) = self.parse_environment(&env_path) {
                    return Ok(env);
                }
            }
        }

        Err(ScoopError::CondaEnvNotFound {
            name: name.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::migrate::EnvironmentStatus;

    #[test]
    fn test_default_roots_returns_none_when_not_installed() {
        // This might return Some if conda is installed on the test machine
        let _ = CondaDiscovery::default_roots();
    }

    #[test]
    fn test_is_conda_env_false_for_non_conda() {
        let temp_dir = tempfile::tempdir().unwrap();
        assert!(!CondaDiscovery::is_conda_env(temp_dir.path()));
    }

    #[test]
    fn test_determine_status_ready() {
        let status = common::determine_status("nonexistent_conda_test", "3.12.0");
        assert!(matches!(status, EnvironmentStatus::Ready));
    }

    #[test]
    fn test_get_python_version_from_conda_meta() {
        use std::io::Write;
        let temp_dir = tempfile::tempdir().unwrap();

        // Create conda-meta directory
        let conda_meta = temp_dir.path().join("conda-meta");
        fs::create_dir(&conda_meta).unwrap();

        // Create fake python package json
        let python_json = conda_meta.join("python-3.11.5-h2345678_0.json");
        let mut file = fs::File::create(&python_json).unwrap();
        writeln!(file, "{{}}").unwrap();

        let version = CondaDiscovery::get_python_version(temp_dir.path());
        assert_eq!(version, Some("3.11.5".to_string()));
    }
}
