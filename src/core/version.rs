//! Version file service
//!
//! # Precedence contract
//!
//! Resolution order, highest priority first:
//!
//! 1. `SCUV_VERSION` environment variable.
//! 2. Nearest `.scuv-version` walking up from the target directory.
//! 3. The global version file (`~/.scuv/version`).
//!
//! The scoop-era names (`SCOOP_VERSION`, `.scoop-version`) stopped being
//! read in 0.16.0.

use std::fs;
use std::path::{Path, PathBuf};

use crate::error::Result;
use crate::paths;

/// Service for managing version files
pub struct VersionService;

/// Which input supplied the resolved environment name.
///
/// Only the distinction `status` needs: an environment variable is not a
/// version file, and reporting it as one misleads anything consuming
/// `status --json`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VersionSource {
    /// `SCUV_VERSION`.
    EnvVar,
    /// `.scuv-version` (local or parent) or `~/.scuv/version`.
    VersionFile,
}

impl VersionService {
    /// Set the local version for a directory
    pub fn set_local(dir: &Path, env_name: &str) -> Result<()> {
        let version_file = paths::local_version_file(dir);
        fs::write(&version_file, format!("{env_name}\n"))?;
        Ok(())
    }

    /// Set the global version
    pub fn set_global(env_name: &str) -> Result<()> {
        let version_file = paths::global_version_file()?;
        if let Some(parent) = version_file.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&version_file, format!("{env_name}\n"))?;
        Ok(())
    }

    /// Get the local version for a directory
    pub fn get_local(dir: &Path) -> Option<String> {
        Self::read_version_file(&paths::local_version_file(dir))
    }

    /// Get the global version
    pub fn get_global() -> Option<String> {
        let version_file = paths::global_version_file().ok()?;
        Self::read_version_file(&version_file)
    }

    /// Resolve the version for a directory (env var -> local -> parent -> global)
    ///
    /// See the module-level precedence contract for the full ordering.
    ///
    /// # Environment Variables
    ///
    /// - `SCUV_VERSION`: overrides file-based resolution entirely when set
    ///   to a valid environment name or `system`.
    /// - `SCUV_RESOLVE_MAX_DEPTH`: Limits parent directory traversal depth.
    ///   Useful for slow network filesystems (NFS, SSHFS, etc).
    ///   - `0` = current directory only
    ///   - `3` = current + up to 3 parent directories
    ///   - unset = unlimited (default behavior)
    pub fn resolve(dir: &Path) -> Option<String> {
        // Priority 1: SCUV_VERSION environment variable.
        if let Some(name) = Self::resolve_env_version() {
            return Some(name);
        }

        // Get max depth from environment variable (None = unlimited).
        let max_depth = std::env::var("SCUV_RESOLVE_MAX_DEPTH")
            .ok()
            .and_then(|s| s.parse::<usize>().ok());

        // Check current and parent directories for local version
        let mut current = dir.to_path_buf();
        let mut depth = 0;

        loop {
            if let Some(version) = Self::get_local(&current) {
                return Some(version);
            }

            // Check depth limit for network filesystem optimization
            if let Some(max) = max_depth {
                depth += 1;
                if depth > max {
                    break;
                }
            }

            if !current.pop() {
                break;
            }
        }

        // Fall back to global
        Self::get_global()
    }

    /// Resolve from current directory
    pub fn resolve_current() -> Option<String> {
        let cwd = std::env::current_dir().ok()?;
        Self::resolve(&cwd)
    }

    /// Resolve from the current directory, reporting which source supplied
    /// the answer.
    ///
    /// `status --json` reports a `source` field, and reconstructing the
    /// priority order at the call site would let the two drift apart. The
    /// order lives in [`Self::resolve`]; this only labels the result, so the
    /// env-var branch is consulted exactly once here and nowhere else.
    pub fn resolve_current_with_source() -> Option<(String, VersionSource)> {
        if let Some(name) = Self::resolve_env_version() {
            return Some((name, VersionSource::EnvVar));
        }
        let cwd = std::env::current_dir().ok()?;
        // `resolve` re-checks the env var first, but that branch cannot fire
        // here — it just returned None above.
        Self::resolve(&cwd).map(|name| (name, VersionSource::VersionFile))
    }

    /// Read a version file
    ///
    /// Returns `None` if:
    /// - File doesn't exist or can't be read
    /// - Content is empty after trimming
    /// - Content is not a valid environment name or "system" (security: prevents command injection)
    fn read_version_file(path: &PathBuf) -> Option<String> {
        fs::read_to_string(path)
            .ok()
            .and_then(|s| Self::normalize_version_value(&s))
    }

    /// Validate and normalize a raw version value (from a file or an
    /// environment variable).
    ///
    /// Returns `None` if the trimmed value is empty or is not a valid
    /// environment name / `system` (security: prevents command injection).
    /// `system` is normalized to lowercase for consistent shell hook
    /// comparison, regardless of source-value casing.
    fn normalize_version_value(raw: &str) -> Option<String> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return None;
        }
        if trimmed.eq_ignore_ascii_case("system") {
            return Some("system".to_string());
        }
        crate::validate::is_valid_env_name(trimmed).then(|| trimmed.to_string())
    }

    /// Priority-1 environment-variable override: `SCUV_VERSION`. Falls
    /// through to `None` (i.e. file-based resolution) when it is not set to
    /// a valid value.
    fn resolve_env_version() -> Option<String> {
        if let Ok(raw) = std::env::var("SCUV_VERSION")
            && let Some(name) = Self::normalize_version_value(&raw)
        {
            return Some(name);
        }
        None
    }

    /// Unset local version
    ///
    /// Removes the `.scuv-version` file in `dir`, if present.
    pub fn unset_local(dir: &Path) -> Result<()> {
        let version_file = paths::local_version_file(dir);
        if version_file.exists() {
            fs::remove_file(&version_file)?;
        }
        Ok(())
    }

    /// Unset global version
    pub fn unset_global() -> Result<()> {
        let version_file = paths::global_version_file()?;
        if version_file.exists() {
            fs::remove_file(&version_file)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
