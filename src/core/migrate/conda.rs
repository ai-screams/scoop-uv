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
                // Reads both `version` (stdlib venv) and `version_info` (uv) keys.
                if let Some(version) = content
                    .lines()
                    .find_map(crate::core::pyvenv_version_from_line)
                {
                    return Some(version);
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

    /// Build a directory that `is_conda_env` should accept: `conda-meta/`
    /// plus a `bin/python`.
    fn make_conda_env(root: &Path, name: &str) -> PathBuf {
        let env = root.join(name);
        fs::create_dir_all(env.join("conda-meta")).unwrap();
        fs::create_dir_all(env.join("bin")).unwrap();
        fs::write(env.join("bin").join("python"), b"").unwrap();
        env
    }

    /// The existing negative test passes even when the `conda-meta` guard is
    /// inverted, because a bare tempdir has no `bin/python` either and both
    /// branches end up false. Pin the positive case so the guard is actually
    /// exercised.
    /// `default_roots` walks `$HOME` for the four well-known conda layouts
    /// and de-duplicates against `$CONDA_PREFIX/envs`. Both the discovery and
    /// the dedup guard need a controlled `$HOME` to exercise, which is why
    /// nothing covered them before.
    #[test]
    #[serial_test::serial]
    fn default_roots_finds_every_known_layout() {
        let temp = tempfile::tempdir().unwrap();
        let home = temp.path();
        for layout in [".conda", "anaconda3", "miniconda3", "miniforge3"] {
            fs::create_dir_all(home.join(layout).join("envs")).unwrap();
        }

        let _g = crate::test_utils::env_guard(&[
            ("HOME", Some(home.to_str().unwrap())),
            ("CONDA_PREFIX", None),
        ]);

        let found = CondaDiscovery::default_roots().expect("layouts exist, so Some");
        assert_eq!(
            found.roots.len(),
            4,
            "every known layout should be picked up"
        );
    }

    /// `$CONDA_PREFIX/envs` is added first, and the candidate loop must not
    /// add it a second time. Dropping the `!` on that guard stops the loop
    /// adding anything at all, which this length check catches.
    #[test]
    #[serial_test::serial]
    fn default_roots_dedups_conda_prefix_against_layouts() {
        let temp = tempfile::tempdir().unwrap();
        let home = temp.path();
        fs::create_dir_all(home.join(".conda").join("envs")).unwrap();
        fs::create_dir_all(home.join("miniconda3").join("envs")).unwrap();

        let _g = crate::test_utils::env_guard(&[
            ("HOME", Some(home.to_str().unwrap())),
            // Points at the same directory as the `.conda` candidate.
            ("CONDA_PREFIX", Some(home.join(".conda").to_str().unwrap())),
        ]);

        let found = CondaDiscovery::default_roots().expect("layouts exist, so Some");
        assert_eq!(
            found.roots.len(),
            2,
            "CONDA_PREFIX and .conda are the same path and must appear once, \
             alongside miniconda3: {:?}",
            found.roots
        );
    }

    /// No conda layout anywhere means no discovery at all.
    #[test]
    #[serial_test::serial]
    fn default_roots_none_when_nothing_installed() {
        let temp = tempfile::tempdir().unwrap();
        let _g = crate::test_utils::env_guard(&[
            ("HOME", Some(temp.path().to_str().unwrap())),
            ("CONDA_PREFIX", None),
        ]);

        assert!(CondaDiscovery::default_roots().is_none());
    }

    #[test]
    fn is_conda_env_true_for_conda_layout() {
        let temp = tempfile::tempdir().unwrap();
        let env = make_conda_env(temp.path(), "web");
        assert!(CondaDiscovery::is_conda_env(&env));
    }

    /// `conda-meta` alone is not enough — we only migrate Python envs.
    #[test]
    fn is_conda_env_false_without_python_binary() {
        let temp = tempfile::tempdir().unwrap();
        let env = temp.path().join("web");
        fs::create_dir_all(env.join("conda-meta")).unwrap();
        assert!(!CondaDiscovery::is_conda_env(&env));
    }

    /// The filename filter is `starts_with("python-") && ends_with(".json")`.
    /// Weakened to `||`, a short name like `a.json` reaches `&name_str[7..]`
    /// and panics on the byte slice, so this covers more than a wrong answer.
    #[test]
    fn get_python_version_ignores_unrelated_conda_meta_files() {
        let temp = tempfile::tempdir().unwrap();
        let meta = temp.path().join("conda-meta");
        fs::create_dir(&meta).unwrap();
        fs::write(meta.join("a.json"), b"{}").unwrap();
        fs::write(meta.join("python-notes.txt"), b"").unwrap();

        assert_eq!(CondaDiscovery::get_python_version(temp.path()), None);
    }

    /// A directory that is not a conda env must not become a SourceEnvironment.
    #[test]
    fn parse_environment_rejects_non_conda_directory() {
        let temp = tempfile::tempdir().unwrap();
        let plain = temp.path().join("notconda");
        fs::create_dir(&plain).unwrap();

        let d = CondaDiscovery::new(vec![temp.path().to_path_buf()]);
        assert!(d.parse_environment(&plain).is_none());
    }

    #[test]
    fn parse_environment_accepts_conda_directory() {
        let temp = tempfile::tempdir().unwrap();
        let env = make_conda_env(temp.path(), "web");

        let d = CondaDiscovery::new(vec![temp.path().to_path_buf()]);
        let parsed = d.parse_environment(&env).expect("conda env should parse");
        assert_eq!(parsed.name, "web");
        assert_eq!(parsed.source_type, SourceType::Conda);
    }

    /// Scanning must read roots that exist and skip those that do not —
    /// inverting that guard silently yields nothing.
    #[test]
    fn scan_environments_reads_existing_root_and_skips_missing() {
        let temp = tempfile::tempdir().unwrap();
        make_conda_env(temp.path(), "web");

        let d = CondaDiscovery::new(vec![
            temp.path().to_path_buf(),
            temp.path().join("does-not-exist"),
        ]);
        let envs = d.scan_environments().unwrap();
        assert_eq!(envs.len(), 1);
        assert_eq!(envs[0].name, "web");
    }

    /// A symlink under a conda root is skipped whatever it points at. The
    /// guard is `is_symlink() || !is_dir()`; with `&&` a symlink to a real
    /// conda env would be scanned, which is how a link outside the root
    /// gets pulled in.
    #[cfg(unix)]
    #[test]
    fn scan_environments_skips_symlinked_entries() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("root");
        fs::create_dir(&root).unwrap();
        let outside = make_conda_env(temp.path(), "outside");
        std::os::unix::fs::symlink(&outside, root.join("linked")).unwrap();

        let d = CondaDiscovery::new(vec![root]);
        assert!(d.scan_environments().unwrap().is_empty());
    }

    /// Plain files in a root are not environments either.
    #[test]
    fn scan_environments_skips_plain_files() {
        let temp = tempfile::tempdir().unwrap();
        fs::write(temp.path().join("stray.txt"), b"").unwrap();

        let d = CondaDiscovery::new(vec![temp.path().to_path_buf()]);
        assert!(d.scan_environments().unwrap().is_empty());
    }

    /// `find_environment` requires the path to exist *and* be a directory;
    /// weakened to `||` a same-named file would be parsed.
    #[test]
    fn find_environment_ignores_file_with_matching_name() {
        let temp = tempfile::tempdir().unwrap();
        fs::write(temp.path().join("web"), b"").unwrap();

        let d = CondaDiscovery::new(vec![temp.path().to_path_buf()]);
        assert!(d.find_environment("web").is_err());
    }

    #[test]
    fn find_environment_returns_conda_env() {
        let temp = tempfile::tempdir().unwrap();
        make_conda_env(temp.path(), "web");

        let d = CondaDiscovery::new(vec![temp.path().to_path_buf()]);
        assert_eq!(d.find_environment("web").unwrap().name, "web");
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
