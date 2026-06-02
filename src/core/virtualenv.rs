//! Virtual environment service

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use tracing::warn;

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
            // Per-entry tolerance — transient IO errors on a single entry
            // shouldn't hide the rest of the directory from callers.
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };
            // Reject symlinks via file_type() (no traversal) instead of
            // path.is_dir() (which follows symlinks). A symlink under
            // virtualenvs/ would otherwise be enumerated as a normal env,
            // and downstream commands like `scoop verify` would exec the
            // target's bin/python — arbitrary execution under the user's
            // UID. This is the same hardening gc::scan_orphan_envs does;
            // doing it here makes every caller of list() consistent.
            let ft = match entry.file_type() {
                Ok(t) => t,
                Err(_) => continue,
            };
            if !ft.is_dir() || ft.is_symlink() {
                continue;
            }
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                let metadata = self.read_metadata(&path);
                envs.push(VirtualenvInfo {
                    name: name.to_string(),
                    path: path.clone(),
                    python_version: metadata.map(|m| m.python_version),
                });
            }
        }

        envs.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(envs)
    }

    /// Create a new virtual environment
    pub fn create(&self, name: &str, python_version: &str) -> Result<PathBuf> {
        self.create_inner(name, python_version, None)
    }

    /// Create a new virtual environment using a specific Python executable path.
    ///
    /// The `python_path` is passed directly to uv's `--python` flag, which
    /// accepts both version strings and paths. The `python_version` should be
    /// the detected version string from the binary. The canonical path is stored
    /// in metadata.
    pub fn create_with_python_path(
        &self,
        name: &str,
        python_version: &str,
        python_path: &Path,
    ) -> Result<PathBuf> {
        self.create_inner(
            name,
            &python_path.display().to_string(),
            Some((python_version, python_path)),
        )
    }

    /// Internal create implementation shared by both create methods.
    fn create_inner(
        &self,
        name: &str,
        uv_python_arg: &str,
        python_path_info: Option<(&str, &Path)>,
    ) -> Result<PathBuf> {
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
        self.uv.create_venv(&path, uv_python_arg)?;

        // Write metadata
        let uv_version = self.uv.version().ok();
        // Resolve actual version: prefer pyvenv.cfg (handles specifiers like cpython@3.12),
        // then explicit python_path version, then fall back to the raw uv arg.
        let actual_version = super::parse_pyvenv_version(&path)
            .or_else(|| python_path_info.map(|(ver, _)| ver.to_string()))
            .unwrap_or_else(|| uv_python_arg.to_string());
        let mut metadata = Metadata::new(name.to_string(), actual_version, uv_version);

        if let Some((_, pp)) = python_path_info {
            metadata = metadata.with_python_path(pp.display().to_string());
        }

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

    /// Check whether a Python version matching `version` is already installed
    /// via uv. Thin pass-through to [`UvClient::find_python`] so command
    /// handlers don't need direct access to the private `uv` field.
    pub fn is_python_installed(&self, version: &str) -> Result<bool> {
        Ok(self.uv.find_python(version)?.is_some())
    }

    /// Install a Python version through uv. Thin pass-through that lets command
    /// handlers stay decoupled from the private `uv` field.
    pub fn install_python(&self, version: &str) -> Result<()> {
        self.uv.install_python(version)
    }

    /// Install Python packages into the env via uv. Thin pass-through so the
    /// sync handler doesn't need direct access to the private `uv` field.
    pub fn pip_install(&self, venv_path: &Path, packages: &[String]) -> Result<()> {
        self.uv.pip_install(venv_path, packages)
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

    /// Read metadata from a virtual environment.
    ///
    /// Returns `None` for both "file missing" and "file corrupt" — callers
    /// that only need a best-effort view (e.g. `list`, `info`, `status`)
    /// can keep using this. Anything that needs to *act* on the distinction
    /// (gc classification, touch on activation) must use
    /// [`Self::read_metadata_result`] instead.
    pub fn read_metadata(&self, path: &Path) -> Option<Metadata> {
        self.read_metadata_result(path).ok().flatten()
    }

    /// Read metadata distinguishing missing from corrupt.
    ///
    /// - `Ok(Some(m))` — file present, parsed cleanly
    /// - `Ok(None)`    — file does not exist (legitimate "no metadata")
    /// - `Err(e)`      — file present but unreadable / unparseable
    ///
    /// This split exists so [`Self::touch_metadata_best_effort`] can refuse
    /// to overwrite a corrupt file (which would silently destroy the user's
    /// only forensic trace of the corruption), while still updating files
    /// that are merely absent.
    pub fn read_metadata_result(&self, path: &Path) -> Result<Option<Metadata>> {
        let metadata_path = path.join(Metadata::FILE_NAME);
        let content = match fs::read_to_string(&metadata_path) {
            Ok(c) => c,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(ScoopError::Io(e)),
        };
        let parsed: Metadata = serde_json::from_str(&content)?;
        Ok(Some(parsed))
    }

    /// Write metadata to a virtual environment (non-atomic). Used at
    /// creation time where any failure aborts env setup anyway.
    fn write_metadata(&self, path: &Path, metadata: &Metadata) -> Result<()> {
        let metadata_path = path.join(Metadata::FILE_NAME);
        let content = serde_json::to_string_pretty(metadata)?;
        fs::write(metadata_path, content)?;
        Ok(())
    }

    /// Atomically write metadata: write to a sibling tempfile in the same
    /// directory, fsync, then rename over the target.
    ///
    /// Same-directory tempfile is required so the rename stays on one
    /// filesystem (cross-fs rename would degrade to copy+delete and lose
    /// atomicity). A crash mid-write leaves either the old file intact or
    /// the new file fully written — never a truncated half-write that
    /// future reads would reject as corrupt.
    pub fn write_metadata_atomic(&self, path: &Path, metadata: &Metadata) -> Result<()> {
        let metadata_path = path.join(Metadata::FILE_NAME);
        let dir = metadata_path.parent().ok_or_else(|| {
            ScoopError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "metadata path has no parent",
            ))
        })?;

        let content = serde_json::to_string_pretty(metadata)?;
        let mut tmp = tempfile::NamedTempFile::new_in(dir)?;
        tmp.write_all(content.as_bytes())?;
        tmp.as_file_mut().sync_all()?;
        // persist() does the atomic rename. On Unix this is rename(2); on
        // Windows tempfile uses MoveFileEx with MOVEFILE_REPLACE_EXISTING.
        tmp.persist(&metadata_path)
            .map_err(|e| ScoopError::Io(e.error))?;
        Ok(())
    }

    /// Touch an env's `last_used` to `now`, best-effort.
    ///
    /// Never returns an error: activation must not be blocked by metadata
    /// I/O failure. Three documented behaviors:
    ///
    /// 1. **Missing metadata** — skipped silently (legacy env that was
    ///    never `scoop create`d via this binary). No file is created.
    /// 2. **Corrupt metadata** — logged via `warn!` and left untouched.
    ///    Overwriting would destroy the only on-disk evidence of the
    ///    corruption. The user can re-run with logging to see what's wrong.
    /// 3. **Healthy metadata** — `last_used` updated atomically.
    pub fn touch_metadata_best_effort(&self, env_name: &str, now: DateTime<Utc>) {
        let path = match paths::virtualenv_path(env_name) {
            Ok(p) => p,
            Err(e) => {
                warn!("touch_metadata: cannot resolve path for {env_name}: {e}");
                return;
            }
        };

        match self.read_metadata_result(&path) {
            Ok(Some(mut meta)) => {
                meta.touch(now);
                if let Err(e) = self.write_metadata_atomic(&path, &meta) {
                    warn!("touch_metadata: atomic write failed for {env_name}: {e}");
                }
            }
            Ok(None) => {
                // Legacy env with no metadata file. Nothing to touch and
                // we deliberately do NOT synthesize one — that would lie
                // about created_at/created_by.
            }
            Err(e) => {
                warn!("touch_metadata: refusing to overwrite corrupt metadata for {env_name}: {e}");
            }
        }
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

    // ==========================================================================
    // last_used / atomic write / touch_metadata_best_effort
    // ==========================================================================

    fn seed_metadata_file(env_path: &Path, json: &str) {
        fs::create_dir_all(env_path).unwrap();
        fs::write(env_path.join(Metadata::FILE_NAME), json).unwrap();
    }

    #[test]
    #[serial]
    fn test_read_metadata_result_distinguishes_missing_from_corrupt() {
        with_temp_scoop_home(|temp_dir| {
            let service = require_uv!();
            let env_dir = temp_dir.path().join("virtualenvs").join("subject");
            fs::create_dir_all(&env_dir).unwrap();

            // No file → Ok(None). Distinct from "corrupt" because we
            // shouldn't warn or refuse anything for a missing file.
            assert!(service.read_metadata_result(&env_dir).unwrap().is_none());

            // Corrupt JSON → Err(Json(..)). Caller needs this signal to
            // decide whether to overwrite (no) or warn (yes).
            fs::write(env_dir.join(Metadata::FILE_NAME), "{ not json").unwrap();
            let err = service.read_metadata_result(&env_dir).unwrap_err();
            assert!(
                matches!(err, ScoopError::Json(_)),
                "corrupt JSON must yield ScoopError::Json, got: {err:?}"
            );
        });
    }

    #[test]
    #[serial]
    fn test_touch_metadata_best_effort_updates_only_last_used() {
        with_temp_scoop_home(|temp_dir| {
            let service = require_uv!();
            let env_path = temp_dir.path().join("virtualenvs").join("touched");
            let seed = r#"{
                "name": "touched",
                "python_version": "3.12.1",
                "created_at": "2024-01-15T10:30:00Z",
                "created_by": "scoop 0.5.0",
                "uv_version": "0.4.0"
            }"#;
            seed_metadata_file(&env_path, seed);

            let now = "2026-06-02T12:00:00Z".parse::<DateTime<Utc>>().unwrap();
            service.touch_metadata_best_effort("touched", now);

            let after = service
                .read_metadata_result(&env_path)
                .unwrap()
                .expect("file present after touch");
            assert_eq!(after.last_used, Some(now));
            // Provenance fields must survive a touch — they describe how
            // the env was created and a touch is not a re-creation.
            assert_eq!(after.name, "touched");
            assert_eq!(after.python_version, "3.12.1");
            assert_eq!(after.created_by, "scoop 0.5.0");
            assert_eq!(after.uv_version, Some("0.4.0".to_string()));
            let expected_created_at = "2024-01-15T10:30:00Z".parse::<DateTime<Utc>>().unwrap();
            assert_eq!(after.created_at, expected_created_at);
        });
    }

    #[test]
    #[serial]
    fn test_touch_metadata_best_effort_preserves_corrupt_file() {
        with_temp_scoop_home(|temp_dir| {
            let service = require_uv!();
            let env_path = temp_dir.path().join("virtualenvs").join("broken");
            let garbage = "{ this is not valid json";
            seed_metadata_file(&env_path, garbage);

            // Must not panic / propagate the error. Must not overwrite.
            service.touch_metadata_best_effort("broken", "2026-06-02T12:00:00Z".parse().unwrap());

            let on_disk = fs::read_to_string(env_path.join(Metadata::FILE_NAME)).unwrap();
            assert_eq!(
                on_disk, garbage,
                "corrupt metadata must be preserved verbatim — overwriting it \
                 would destroy the user's only forensic trace of the corruption"
            );
        });
    }

    #[test]
    #[serial]
    fn test_touch_metadata_best_effort_noop_on_missing_metadata() {
        with_temp_scoop_home(|temp_dir| {
            let service = require_uv!();
            let env_path = temp_dir.path().join("virtualenvs").join("nofile");
            fs::create_dir_all(&env_path).unwrap();

            // Legacy env with no metadata file. Must NOT synthesize one
            // (that would lie about created_at) and must NOT error.
            service.touch_metadata_best_effort("nofile", "2026-06-02T12:00:00Z".parse().unwrap());
            assert!(!env_path.join(Metadata::FILE_NAME).exists());
        });
    }

    #[test]
    #[serial]
    fn test_write_metadata_atomic_round_trips() {
        with_temp_scoop_home(|temp_dir| {
            let service = require_uv!();
            let env_path = temp_dir.path().join("virtualenvs").join("atomic");
            fs::create_dir_all(&env_path).unwrap();

            let mut meta = Metadata::new("atomic".to_string(), "3.12".to_string(), None);
            let now = "2026-06-02T12:00:00Z".parse::<DateTime<Utc>>().unwrap();
            meta.touch(now);

            service.write_metadata_atomic(&env_path, &meta).unwrap();

            let restored = service
                .read_metadata_result(&env_path)
                .unwrap()
                .expect("file exists after atomic write");
            assert_eq!(restored.last_used, Some(now));

            // No tempfile left behind. tempfile::NamedTempFile::persist
            // promises this but we assert it explicitly so any future
            // change to the impl that breaks cleanup gets caught.
            let leftover: Vec<_> = fs::read_dir(&env_path)
                .unwrap()
                .filter_map(|e| e.ok())
                .filter(|e| {
                    let n = e.file_name();
                    let s = n.to_string_lossy();
                    s.starts_with(".tmp") || s.contains("scoop-metadata.json.tmp")
                })
                .collect();
            assert!(leftover.is_empty(), "tempfile residue: {leftover:?}");
        });
    }

    #[test]
    #[serial]
    fn test_write_metadata_atomic_overwrites_existing() {
        with_temp_scoop_home(|temp_dir| {
            let service = require_uv!();
            let env_path = temp_dir.path().join("virtualenvs").join("overwrite");
            fs::create_dir_all(&env_path).unwrap();

            // First write
            let m1 = Metadata::new("overwrite".to_string(), "3.11".to_string(), None);
            service.write_metadata_atomic(&env_path, &m1).unwrap();

            // Second write replaces it atomically.
            let mut m2 = Metadata::new("overwrite".to_string(), "3.12".to_string(), None);
            m2.touch("2026-06-02T12:00:00Z".parse().unwrap());
            service.write_metadata_atomic(&env_path, &m2).unwrap();

            let on_disk = service.read_metadata_result(&env_path).unwrap().unwrap();
            assert_eq!(on_disk.python_version, "3.12");
            assert!(on_disk.last_used.is_some());
        });
    }

    // C2 regression — symlinks under virtualenvs/ must NOT be enumerated.
    // Otherwise downstream commands (gc, verify, ...) would treat the
    // symlink target as a real env and end up scanning / exec'ing files
    // outside the venvs dir.
    #[cfg(unix)]
    #[test]
    #[serial]
    fn test_list_skips_symlink_entries() {
        with_temp_scoop_home(|temp_dir| {
            let venvs_dir = temp_dir.path().join("virtualenvs");
            fs::create_dir_all(&venvs_dir).unwrap();

            // Real env so the list isn't empty (controls for "filter is
            // entirely broken" vs "filter caught the symlink").
            create_mock_venv(temp_dir, "real", None);

            // Plant a symlink → some other (existing) directory. Without
            // the symlink filter this would be enumerated as an env.
            let other = tempfile::TempDir::new().unwrap();
            std::os::unix::fs::symlink(other.path(), venvs_dir.join("symlinked")).unwrap();

            let service = require_uv!();
            let envs = service.list().unwrap();
            assert_eq!(envs.len(), 1, "symlink entries must be skipped: {envs:?}");
            assert_eq!(envs[0].name, "real");
        });
    }
}
