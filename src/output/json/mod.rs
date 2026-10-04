//! JSON output types for CLI commands

use serde::{Deserialize, Serialize};

/// Success response wrapper
#[derive(Serialize)]
pub struct JsonResponse<T: Serialize> {
    /// Response status (always "success")
    pub status: &'static str,
    /// Command that was executed
    pub command: &'static str,
    /// Response data
    pub data: T,
}

impl<T: Serialize> JsonResponse<T> {
    /// Create a new success response
    pub fn success(command: &'static str, data: T) -> Self {
        Self {
            status: "success",
            command,
            data,
        }
    }
}

/// Error response wrapper
#[derive(Serialize)]
pub struct JsonErrorResponse {
    /// Response status (always "error")
    pub status: &'static str,
    /// Command that was executed
    pub command: &'static str,
    /// Error details
    pub error: JsonError,
}

/// Error details
#[derive(Serialize)]
pub struct JsonError {
    /// Error code (e.g., "ENV_NOT_FOUND")
    pub code: &'static str,
    /// Human-readable error message
    pub message: String,
    /// Suggested fix (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
}

impl JsonErrorResponse {
    /// Create a new error response
    pub fn error(command: &'static str, code: &'static str, message: String) -> Self {
        Self {
            status: "error",
            command,
            error: JsonError {
                code,
                message,
                suggestion: None,
            },
        }
    }

    /// Add a suggestion to the error
    pub fn with_suggestion(mut self, suggestion: String) -> Self {
        self.error.suggestion = Some(suggestion);
        self
    }
}

// ============================================================================
// Command-specific data types
// ============================================================================

/// List virtualenvs response data
#[derive(Serialize)]
pub struct ListEnvsData {
    pub virtualenvs: Vec<VirtualenvInfo>,
    pub total: usize,
}

/// Virtualenv info for JSON output
#[derive(Serialize)]
pub struct VirtualenvInfo {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub python: Option<String>,
    pub path: String,
    pub active: bool,
    /// Creation timestamp (RFC 3339). Omitted when metadata is missing
    /// or doesn't carry `created_at` yet (legacy envs).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    /// Last-use timestamp (RFC 3339). Omitted for envs that haven't
    /// been activated since the field landed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_used: Option<String>,
}

/// List pythons response data
#[derive(Serialize)]
pub struct ListPythonsData {
    pub pythons: Vec<PythonInfo>,
    pub total: usize,
}

/// Python info for JSON output
#[derive(Serialize)]
pub struct PythonInfo {
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub implementation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

/// Create response data
#[derive(Serialize)]
pub struct CreateData {
    pub name: String,
    pub python: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub python_path: Option<String>,
}

/// Use response data
#[derive(Serialize)]
pub struct UseData {
    pub name: String,
    pub mode: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symlink: Option<String>,
}

/// Remove response data
#[derive(Serialize)]
pub struct RemoveData {
    pub name: String,
    pub path: String,
    /// The project `.venv` symlink removed along with the env, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unlinked: Option<String>,
    /// Why that link could not be removed; the env itself is gone either way.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unlink_error: Option<String>,
}

/// Install response data
#[derive(Serialize)]
pub struct InstallData {
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

/// Uninstall response data
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct UninstallData {
    pub version: String,
    /// Environments removed by cascade (only present if --cascade used)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub removed_envs: Option<Vec<String>>,
    /// Environments the cascade meant to remove but could not (absent when
    /// none failed)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub failed_envs: Vec<CascadeFailure>,
}

/// An environment a cascade could not remove, and why
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CascadeFailure {
    pub name: String,
    pub error: String,
}

/// Package info for JSON output
#[derive(Serialize)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
}

/// Packages summary for JSON output
#[derive(Serialize)]
pub struct PackagesInfo {
    pub total: usize,
    pub items: Vec<PackageInfo>,
    pub truncated: bool,
}

impl PackagesInfo {
    /// Creates a new PackagesInfo with truncation applied.
    ///
    /// # Arguments
    /// * `packages` - Full list of (name, version) tuples
    /// * `limit` - Maximum number of packages to include in items
    ///
    /// # Examples
    /// ```
    /// # use scoop_uv::output::PackagesInfo;
    /// let packages = vec![
    ///     ("pkg1".to_string(), "1.0".to_string()),
    ///     ("pkg2".to_string(), "2.0".to_string()),
    /// ];
    /// let info = PackagesInfo::new(&packages, 1);
    /// assert_eq!(info.total, 2);
    /// assert_eq!(info.items.len(), 1);
    /// assert!(info.truncated);
    /// ```
    pub fn new(packages: &[(String, String)], limit: usize) -> Self {
        let total = packages.len();
        let truncated = total > limit;
        Self {
            total,
            items: packages
                .iter()
                .take(limit)
                .map(|(n, v)| PackageInfo {
                    name: n.clone(),
                    version: v.clone(),
                })
                .collect(),
            truncated,
        }
    }

    /// Returns the number of packages not included due to truncation.
    #[inline]
    pub fn remaining(&self) -> usize {
        self.total.saturating_sub(self.items.len())
    }
}

/// `scuv which` response data
#[derive(Serialize)]
pub struct WhichData {
    pub exe: String,
    pub env: String,
    pub path: String,
}

/// `scuv clone` response data
#[derive(Serialize)]
pub struct CloneData {
    pub src: String,
    pub dst: String,
    pub python: String,
    pub path: String,
    pub packages_copied: usize,
    /// `true` when `--no-packages` was passed.
    pub packages_skipped: bool,
}

/// `scuv import` response data
#[derive(Serialize)]
pub struct ImportData {
    pub name: String,
    pub python: String,
    pub packages_installed: usize,
    pub source: String,
}

/// `scuv sync` response data
#[derive(Serialize)]
pub struct SyncData {
    /// Absolute path of the resolved `.scuv.toml`.
    pub manifest_path: String,
    /// Environment name the manifest targets.
    pub environment: String,
    /// Python version specifier from the manifest.
    pub python: String,
    /// All resolved group names (always includes `"default"`).
    pub groups: Vec<String>,
    /// Packages that would be / were installed (deduped, ordered).
    pub packages: Vec<String>,
    /// `true` if `scuv sync` created the env; `false` if it already existed.
    pub env_created: bool,
    /// `true` when `--dry-run` was passed (no side effects performed).
    pub dry_run: bool,
}

/// `scuv status` response data
#[derive(Serialize)]
pub struct StatusData {
    /// `"active"` (shell-activated), `"configured"` (version-file), `"system"`, or `"none"`.
    pub state: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Where the env name came from: `"scuv_active_env"` or `"version_file"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub python: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    /// RFC 3339 timestamp of the last `scuv activate` / `run` / `shell`
    /// against this env. `None` for legacy envs whose metadata predates
    /// the field, and for freshly created envs that have never been
    /// activated. Omitted from JSON when absent so old consumers don't
    /// have to learn a new key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_used: Option<String>,
    /// Best-effort installed-package count via the venv's own `pip list`.
    /// `None` when the env has no pip (broken / not yet bootstrapped).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub packages: Option<usize>,
}

/// Detailed environment info for JSON output
#[derive(Serialize)]
pub struct EnvInfoData {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub python: Option<String>,
    pub path: String,
    pub active: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    /// RFC 3339 last-use timestamp, omitted when unknown. See
    /// `StatusData::last_used` for the full contract.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_used: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_display: Option<String>,
    pub packages: PackagesInfo,
}

#[cfg(test)]
mod tests;
