//! virtualenvwrapper environment discovery

use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{Result, ScoopError};

use super::common;
use super::source::{EnvironmentSource, SourceEnvironment, SourceType};

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

        // Reads both `version` (stdlib venv) and `version_info` (uv) keys.
        if let Some(version) = content
            .lines()
            .find_map(crate::core::pyvenv_version_from_line)
        {
            return Some(version);
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

    /// Parse a single environment directory into SourceEnvironment
    fn parse_environment(&self, env_path: &Path) -> Option<SourceEnvironment> {
        let name = env_path.file_name()?.to_str()?.to_string();

        // Skip hidden directories
        if name.starts_with('.') {
            return None;
        }

        // Validate environment (check for bin/python)
        let python_bin = env_path.join("bin").join("python");
        if !python_bin.exists() {
            return None; // Not a valid virtualenv, skip silently
        }

        // Parse Python version from pyvenv.cfg
        let python_version =
            Self::parse_pyvenv_cfg(env_path).unwrap_or_else(|| "unknown".to_string());

        // Determine status
        let status = common::determine_status(&name, &python_version);

        Some(SourceEnvironment {
            name,
            python_version,
            path: env_path.to_path_buf(),
            source_type: SourceType::VirtualenvWrapper,
            size_bytes: None, // Lazy: calculated only when needed
            status,
        })
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

            if let Some(env) = self.parse_environment(&env_path) {
                environments.push(env);
            }
        }

        environments.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(environments)
    }

    /// Find a specific environment by name using O(1) direct path access.
    fn find_environment(&self, name: &str) -> Result<SourceEnvironment> {
        let env_path = self.root.join(name);

        if !env_path.exists() || !env_path.is_dir() {
            return Err(ScoopError::VenvWrapperEnvNotFound {
                name: name.to_string(),
            });
        }

        self.parse_environment(&env_path)
            .ok_or_else(|| ScoopError::VenvWrapperEnvNotFound {
                name: name.to_string(),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::migrate::EnvironmentStatus;

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

    /// `parse_pyvenv_cfg` falls back to reading the version out of the `home`
    /// path when no `version` key is present. The existing test only covers
    /// the `version` key, so the two offset arithmetic sites below
    /// (`at_idx + 7` past `python@`, `versions_idx + 9` past `Versions/`)
    /// were never executed.
    #[test]
    fn parse_pyvenv_cfg_reads_version_from_homebrew_style_home() {
        let temp = tempfile::tempdir().unwrap();
        fs::write(
            temp.path().join("pyvenv.cfg"),
            "home = /usr/local/opt/python@3.11/bin\n",
        )
        .unwrap();

        assert_eq!(
            VenvWrapperDiscovery::parse_pyvenv_cfg(temp.path()),
            Some("3.11".to_string())
        );
    }

    /// Same fallback, framework layout: the offset must land exactly past
    /// `Versions/`.
    #[test]
    fn parse_pyvenv_cfg_reads_version_from_framework_home() {
        let temp = tempfile::tempdir().unwrap();
        fs::write(
            temp.path().join("pyvenv.cfg"),
            "home = /Library/Frameworks/Python.framework/Versions/3.12/bin\n",
        )
        .unwrap();

        assert_eq!(
            VenvWrapperDiscovery::parse_pyvenv_cfg(temp.path()),
            Some("3.12".to_string())
        );
    }

    /// `python@` with nothing after it still has to produce the trailing
    /// segment rather than slicing out of bounds.
    #[test]
    fn parse_pyvenv_cfg_handles_home_ending_at_version() {
        let temp = tempfile::tempdir().unwrap();
        fs::write(
            temp.path().join("pyvenv.cfg"),
            "home = /usr/local/opt/python@3.13\n",
        )
        .unwrap();

        assert_eq!(
            VenvWrapperDiscovery::parse_pyvenv_cfg(temp.path()),
            Some("3.13".to_string())
        );
    }

    /// `default_root` prefers `$WORKON_HOME` and falls back to
    /// `~/.virtualenvs`, in both cases only when the directory exists.
    #[test]
    #[serial_test::serial]
    fn default_root_prefers_workon_home_when_it_exists() {
        let temp = tempfile::tempdir().unwrap();
        let workon = temp.path().join("envs");
        fs::create_dir_all(&workon).unwrap();

        let _g = crate::test_utils::env_guard(&[
            ("WORKON_HOME", Some(workon.to_str().unwrap())),
            ("HOME", Some(temp.path().to_str().unwrap())),
        ]);

        let found = VenvWrapperDiscovery::default_root().expect("WORKON_HOME exists");
        assert_eq!(found.root, workon);
    }

    #[test]
    #[serial_test::serial]
    fn default_root_falls_back_to_dot_virtualenvs() {
        let temp = tempfile::tempdir().unwrap();
        let fallback = temp.path().join(".virtualenvs");
        fs::create_dir_all(&fallback).unwrap();

        let _g = crate::test_utils::env_guard(&[
            ("WORKON_HOME", None),
            ("HOME", Some(temp.path().to_str().unwrap())),
        ]);

        let found = VenvWrapperDiscovery::default_root().expect("~/.virtualenvs exists");
        assert_eq!(found.root, fallback);
    }

    #[test]
    #[serial_test::serial]
    fn default_root_none_when_nothing_exists() {
        let temp = tempfile::tempdir().unwrap();
        let _g = crate::test_utils::env_guard(&[
            ("WORKON_HOME", None),
            ("HOME", Some(temp.path().to_str().unwrap())),
        ]);

        assert!(VenvWrapperDiscovery::default_root().is_none());
    }

    /// A directory virtualenvwrapper would accept: `bin/python` present.
    fn make_venv(root: &Path, name: &str) -> PathBuf {
        let env = root.join(name);
        fs::create_dir_all(env.join("bin")).unwrap();
        fs::write(env.join("bin").join("python"), b"").unwrap();
        env
    }

    /// `parse_environment` requires `bin/python`; inverting that guard makes
    /// every non-virtualenv directory look like one.
    #[test]
    fn parse_environment_requires_python_binary() {
        let temp = tempfile::tempdir().unwrap();
        let bare = temp.path().join("web");
        fs::create_dir(&bare).unwrap();

        let d = VenvWrapperDiscovery::new(temp.path().to_path_buf());
        assert!(d.parse_environment(&bare).is_none());

        let real = make_venv(temp.path(), "api");
        assert_eq!(d.parse_environment(&real).unwrap().name, "api");
    }

    /// Hidden directories are skipped.
    #[test]
    fn parse_environment_skips_hidden_directories() {
        let temp = tempfile::tempdir().unwrap();
        let hidden = make_venv(temp.path(), ".cache");

        let d = VenvWrapperDiscovery::new(temp.path().to_path_buf());
        assert!(d.parse_environment(&hidden).is_none());
    }

    /// Scanning a missing root yields nothing rather than erroring, and a
    /// present root is actually read — inverting the guard swaps both.
    #[test]
    fn scan_environments_handles_present_and_missing_root() {
        let temp = tempfile::tempdir().unwrap();
        make_venv(temp.path(), "web");

        let present = VenvWrapperDiscovery::new(temp.path().to_path_buf());
        let envs = present.scan_environments().unwrap();
        assert_eq!(envs.len(), 1);
        assert_eq!(envs[0].name, "web");

        let missing = VenvWrapperDiscovery::new(temp.path().join("nope"));
        assert!(missing.scan_environments().unwrap().is_empty());
    }

    /// The scan guard is `is_symlink() || !is_dir()`. Weakened to `&&`, a
    /// symlink pointing at a real virtualenv outside the root gets scanned.
    #[cfg(unix)]
    #[test]
    fn scan_environments_skips_symlinked_entries() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("root");
        fs::create_dir(&root).unwrap();
        let outside = make_venv(temp.path(), "outside");
        std::os::unix::fs::symlink(&outside, root.join("linked")).unwrap();

        let d = VenvWrapperDiscovery::new(root);
        assert!(d.scan_environments().unwrap().is_empty());
    }

    #[test]
    fn scan_environments_skips_plain_files() {
        let temp = tempfile::tempdir().unwrap();
        fs::write(temp.path().join("stray"), b"").unwrap();

        let d = VenvWrapperDiscovery::new(temp.path().to_path_buf());
        assert!(d.scan_environments().unwrap().is_empty());
    }

    /// `find_environment` rejects a missing path and a same-named file; the
    /// guard is `!exists() || !is_dir()`, and every weakening of it lets one
    /// of those through.
    #[test]
    fn find_environment_rejects_missing_and_non_directory() {
        let temp = tempfile::tempdir().unwrap();
        let d = VenvWrapperDiscovery::new(temp.path().to_path_buf());

        assert!(d.find_environment("absent").is_err());

        fs::write(temp.path().join("afile"), b"").unwrap();
        assert!(d.find_environment("afile").is_err());

        make_venv(temp.path(), "web");
        assert_eq!(d.find_environment("web").unwrap().name, "web");
    }

    #[test]
    fn test_determine_status_ready() {
        let status = common::determine_status("nonexistent_venv_wrapper_test", "3.12.0");
        assert!(matches!(status, EnvironmentStatus::Ready));
    }
}
