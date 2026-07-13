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

/// Information about a virtual environment.
///
/// `#[non_exhaustive]` so future field additions (the v2 plan calls
/// for at least one more metadata-derived field down the line) aren't
/// a breaking change for external `scoop-uv` library consumers. All
/// in-tree construction sites use struct-literal syntax and live in
/// `cfg(test)` or this crate's command handlers, so the attribute
/// only affects downstream consumers — which currently must build
/// `VirtualenvInfo` via field-by-field literal anyway.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct VirtualenvInfo {
    /// Name of the environment
    pub name: String,
    /// Path to the environment
    pub path: PathBuf,
    /// Python version (if metadata exists)
    pub python_version: Option<String>,
    /// Creation timestamp from metadata, if present. Populated by
    /// [`VirtualenvService::list`] so callers (like `list --sort=created`)
    /// don't need a second `read_metadata` round trip per env.
    pub created_at: Option<DateTime<Utc>>,
    /// Last-used timestamp from metadata, if present. Same rationale:
    /// list-time sort can use it without re-reading the JSON file.
    pub last_used: Option<DateTime<Utc>>,
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
            // and downstream commands like `scuv verify` would exec the
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
                // Single metadata read per entry: we extract every field
                // the new VirtualenvInfo wants (python_version + the two
                // timestamps) in one shot, so callers that sort by
                // created_at / last_used don't drive a second pass over
                // every metadata file. Cheap when corrupt — `None` is a
                // valid bucket-end value for sort, and the legacy passive
                // contract (display "-"/"never") is preserved.
                let metadata = self.read_metadata(&path);
                let (python_version, created_at, last_used) = match metadata {
                    Some(m) => (Some(m.python_version), Some(m.created_at), m.last_used),
                    None => (None, None, None),
                };
                envs.push(VirtualenvInfo {
                    name: name.to_string(),
                    path: path.clone(),
                    python_version,
                    created_at,
                    last_used,
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

        self.write_metadata_atomic(&path, &metadata)?;

        Ok(path)
    }

    /// Delete a virtual environment.
    ///
    /// Validates `name` internally before touching the filesystem.
    /// `PathBuf::join` does not block `..` and silently replaces the
    /// base when the right side is absolute, so `delete("/tmp/x")`
    /// against an unvalidated name would happily walk into and remove
    /// the target directory under the user's UID. The validation guard
    /// here is the trust boundary — CLI handlers no longer have to
    /// remember to call `validate_env_name` themselves for safety.
    pub fn delete(&self, name: &str) -> Result<()> {
        validate::validate_env_name(name)?;
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

    /// Check if a virtual environment exists.
    ///
    /// Validates `name` internally — see [`Self::delete`] for the path
    /// traversal rationale. A `false` return for an invalid name would
    /// hide the bug instead of surfacing it, so we error out.
    pub fn exists(&self, name: &str) -> Result<bool> {
        validate::validate_env_name(name)?;
        let path = paths::virtualenv_path(name)?;
        Ok(path.exists())
    }

    /// Get the path to a virtual environment.
    ///
    /// Validates `name` internally — see [`Self::delete`] for the path
    /// traversal rationale.
    pub fn get_path(&self, name: &str) -> Result<PathBuf> {
        validate::validate_env_name(name)?;
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

    /// Write metadata using an atomic replace: write to a sibling tempfile
    /// in the same directory, then rename over the target.
    ///
    /// Same-directory tempfile is required so the rename stays on one
    /// filesystem — cross-device rename via `fs::rename` / tempfile fails
    /// with `EXDEV` rather than degrading to copy+delete, so a sibling
    /// tempfile is what makes the rename viable at all.
    ///
    /// The rename itself is atomic in the visible-state sense: on Unix
    /// it's `rename(2)`; on Windows tempfile uses `MoveFileExW` with
    /// `MOVEFILE_REPLACE_EXISTING`. A *normal process crash* mid-write
    /// therefore leaves either the old file intact or the new file
    /// in place — readers never observe a half-written file.
    ///
    /// This is NOT a full power-loss durability promise. We don't `fsync`
    /// the file or the parent directory: this is best-effort metadata
    /// (timestamps for display + gc heuristics), and `sync_all` on every
    /// auto-activation would put a disk flush on the `cd` hot path. If a
    /// power loss hits between the rename and the cache flush, the
    /// metadata may roll back to its previous state. We accept that.
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
        tmp.persist(&metadata_path)
            .map_err(|e| ScoopError::Io(e.error))?;
        Ok(())
    }

    /// Touch an env's `last_used` to *now* (wall clock at write time),
    /// best-effort.
    ///
    /// Production entry point. The timestamp is sampled inside this
    /// function so two racing callers (e.g. `scuv activate myenv` from
    /// two shells) write *current* values instead of stale "now at
    /// caller-time" values. This narrows — but does not eliminate — the
    /// regression window between racing touches.
    ///
    /// Concurrency contract: **last-writer-wins**. The read→mutate→write
    /// sequence is not locked, so under contention the file's final
    /// timestamp reflects whichever process committed the rename last,
    /// not necessarily the most recent wall-clock instant. Acceptable for
    /// display ("Last used: 2 hours ago") and gc heuristics at the
    /// day/week granularity Step 5 will offer; not acceptable for
    /// anything that needs strict ordering.
    ///
    /// Never returns an error: activation must not be blocked by metadata
    /// I/O failure. Three documented behaviors:
    ///
    /// 1. **Missing metadata** — skipped silently (legacy env that was
    ///    never `scuv create`d via this binary). No file is created.
    /// 2. **Corrupt metadata** — `warn!` logged and the file left
    ///    untouched. Overwriting would destroy the user's only on-disk
    ///    evidence of the corruption. (Warning is observability sugar,
    ///    not a tested contract.)
    /// 3. **Healthy metadata** — `last_used` updated via atomic replace.
    pub fn touch_metadata_best_effort(&self, env_name: &str) {
        self.touch_metadata_at(env_name, Utc::now());
    }

    /// Test seam: same behavior as [`Self::touch_metadata_best_effort`]
    /// but with an explicit timestamp so tests can pin a deterministic
    /// `last_used` value. Not for production callers — using it from
    /// activate/run/shell would re-introduce the caller-stale timestamp
    /// race that the public entry point exists to narrow.
    pub(crate) fn touch_metadata_at(&self, env_name: &str, now: DateTime<Utc>) {
        // Validation guard — see Self::delete for the path traversal
        // rationale. Touch is best-effort, so we warn instead of
        // returning; the public callers (activate/run/shell) already
        // validate first so this is purely defense-in-depth against
        // future internal callers.
        if let Err(e) = validate::validate_env_name(env_name) {
            warn!("touch_metadata: rejecting invalid env name {env_name:?}: {e}");
            return;
        }
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
mod tests;
