//! Validation utilities for scuv

use std::path::Path;
use std::process::Command;

use once_cell::sync::Lazy;
use regex::Regex;

use crate::error::{Result, ScoopError};

/// Kept here so `scoop_uv::validate::PythonVersion` still resolves; the
/// type lives in [`crate::python_version`].
pub use crate::python_version::PythonVersion;

/// Regex for valid environment names
/// - Must start with a letter
/// - Can contain letters, numbers, hyphens, and underscores
static ENV_NAME_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[a-zA-Z][a-zA-Z0-9_-]*$").expect("ENV_NAME_REGEX is a valid pattern")
});

/// Regex for Python version strings
/// Supports: 3, 3.12, 3.12.0, 3.12.0a1, 3.12.0b2, 3.12.0rc1
static VERSION_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^\d+(\.\d+)*((a|b|rc)\d+)?$").expect("VERSION_REGEX is a valid pattern")
});

/// Names a new environment cannot take: every top-level command and alias
/// (enforced by `reserved_names_cover_every_top_level_command`) plus words
/// that mean something else in a version file or to other tools.
///
/// Only creation checks this list ([`validate_env_name`]). An env that
/// already has one of these names, made before the word was reserved, is
/// still found, used and removed ([`validate_existing_env_name`]).
const RESERVED_NAMES: &[&str] = &[
    "activate",
    "base",
    "clone",
    "completions",
    "create",
    "deactivate",
    "default",
    "delete",
    "diff",
    "doctor",
    "export",
    "gc",
    "global",
    "help",
    "import",
    "info",
    "init",
    "install",
    "lang",
    "list",
    "local",
    "ls",
    "man",
    "migrate",
    "prune",
    "remove",
    "resolve",
    "rm",
    "root",
    "run",
    "self",
    "shell",
    "status",
    "sync",
    "system",
    "uninstall",
    "use",
    "verify",
    "version",
    "versions",
    "which",
];

/// Maximum length for environment names
const MAX_ENV_NAME_LENGTH: usize = 64;

/// Check if a string is a valid name for a new environment.
///
/// Valid names must:
/// - Start with a letter (a-z, A-Z)
/// - Contain only letters, numbers, hyphens, and underscores
/// - Not be a reserved name (e.g., "activate", "list")
/// - Not look like a version string (e.g., "3.12")
/// - Be at most 64 characters long
///
/// # Examples
///
/// ```
/// use scoop_uv::validate::is_valid_env_name;
///
/// assert!(is_valid_env_name("myenv"));
/// assert!(is_valid_env_name("my-project_v2"));
/// assert!(!is_valid_env_name("3.12"));      // looks like version
/// assert!(!is_valid_env_name("activate"));  // reserved name
/// assert!(!is_valid_env_name("123env"));    // starts with digit
/// ```
pub fn is_valid_env_name(name: &str) -> bool {
    validate_env_name(name).is_ok()
}

/// Check if a string is a valid name for an environment that may already
/// exist: the rules of [`is_valid_env_name`] without the reserved list.
///
/// # Examples
///
/// ```
/// use scoop_uv::validate::is_valid_existing_env_name;
///
/// assert!(is_valid_existing_env_name("myenv"));
/// assert!(is_valid_existing_env_name("list"));       // reserved, but may exist
/// assert!(!is_valid_existing_env_name("../escape"));
/// assert!(!is_valid_existing_env_name("3.12"));
/// ```
pub fn is_valid_existing_env_name(name: &str) -> bool {
    validate_existing_env_name(name).is_ok()
}

/// Validate the name a new environment will get, returning an error if
/// invalid. To look up, use or remove an env that may already exist, use
/// [`validate_existing_env_name`].
///
/// # Examples
///
/// ```
/// use scoop_uv::validate::validate_env_name;
///
/// assert!(validate_env_name("myenv").is_ok());
/// assert!(validate_env_name("test-project").is_ok());
///
/// // Invalid names return errors
/// assert!(validate_env_name("").is_err());
/// assert!(validate_env_name("123").is_err());
/// assert!(validate_env_name("list").is_err());
/// ```
///
/// # Errors
///
/// Returns [`ScoopError::InvalidEnvName`] if the name is invalid.
pub fn validate_env_name(name: &str) -> Result<()> {
    check_env_name(name, true)
}

/// Validate the name of an environment that may already exist, for looking
/// it up, using it or removing it.
///
/// Applies every rule of [`validate_env_name`] except the reserved list:
/// the name stays a single path component inside the virtualenvs
/// directory, but an env named before its word was reserved stays
/// reachable. Use [`validate_env_name`] for a name a new env will get.
///
/// # Examples
///
/// ```
/// use scoop_uv::validate::validate_existing_env_name;
///
/// assert!(validate_existing_env_name("myenv").is_ok());
/// assert!(validate_existing_env_name("list").is_ok());
/// assert!(validate_existing_env_name("../escape").is_err());
/// ```
///
/// # Errors
///
/// Returns [`ScoopError::InvalidEnvName`] if the name is invalid.
pub fn validate_existing_env_name(name: &str) -> Result<()> {
    check_env_name(name, false)
}

/// The name rules; `reject_reserved` adds the reserved list.
fn check_env_name(name: &str, reject_reserved: bool) -> Result<()> {
    if name.is_empty() {
        return Err(ScoopError::InvalidEnvName {
            name: name.to_string(),
            reason: "name cannot be empty".to_string(),
        });
    }

    if name.len() > MAX_ENV_NAME_LENGTH {
        return Err(ScoopError::InvalidEnvName {
            name: name.to_string(),
            reason: format!("name exceeds maximum length of {MAX_ENV_NAME_LENGTH} characters"),
        });
    }

    if reject_reserved && RESERVED_NAMES.contains(&name.to_lowercase().as_str()) {
        return Err(ScoopError::InvalidEnvName {
            name: name.to_string(),
            reason: "name is reserved".to_string(),
        });
    }

    if VERSION_REGEX.is_match(name) {
        return Err(ScoopError::InvalidEnvName {
            name: name.to_string(),
            reason: "name looks like a version string (must start with a letter)".to_string(),
        });
    }

    if !ENV_NAME_REGEX.is_match(name) {
        return Err(ScoopError::InvalidEnvName {
            name: name.to_string(),
            reason: "name must start with a letter and contain only letters, numbers, hyphens, and underscores".to_string(),
        });
    }

    Ok(())
}

/// Check if a string is a valid Python version.
///
/// Accepts formats like: 3, 3.12, 3.12.0, 3.12.0a1, 3.12.0rc1
///
/// # Examples
///
/// ```
/// use scoop_uv::validate::is_valid_python_version;
///
/// assert!(is_valid_python_version("3"));
/// assert!(is_valid_python_version("3.12"));
/// assert!(is_valid_python_version("3.12.0"));
/// assert!(is_valid_python_version("3.12.0a1"));
/// assert!(is_valid_python_version("3.12.0rc1"));
///
/// assert!(!is_valid_python_version(""));
/// assert!(!is_valid_python_version("abc"));
/// assert!(!is_valid_python_version("v3.12"));
/// ```
pub fn is_valid_python_version(version: &str) -> bool {
    if version.is_empty() {
        return false;
    }
    // Accept formats like: 3, 3.12, 3.12.0, 3.12.0a1, 3.12.0rc1
    VERSION_REGEX.is_match(version) || version.chars().all(|c| c.is_ascii_digit() || c == '.')
}

/// Validate a Python version string, returning an error if invalid
pub fn validate_python_version(version: &str) -> Result<()> {
    let trimmed = version.trim();

    if trimmed.is_empty() {
        return Err(ScoopError::InvalidPythonVersion {
            version: version.to_string(),
        });
    }

    // Allow special version aliases
    if matches!(trimmed.to_lowercase().as_str(), "latest" | "stable") {
        return Ok(());
    }

    if !is_valid_python_version(trimmed) {
        return Err(ScoopError::InvalidPythonVersion {
            version: version.to_string(),
        });
    }

    Ok(())
}

/// Validate that a path points to a valid Python executable.
///
/// Checks that the path:
/// 1. Exists on the filesystem
/// 2. Is a file (not a directory)
/// 3. Is executable (Unix only: checks mode bits)
///
/// # Examples
///
/// ```no_run
/// use std::path::Path;
/// use scoop_uv::validate::validate_python_path;
///
/// // Valid Python path
/// assert!(validate_python_path(Path::new("/usr/bin/python3")).is_ok());
///
/// // Non-existent path
/// assert!(validate_python_path(Path::new("/nonexistent/python")).is_err());
/// ```
///
/// # Errors
///
/// Returns [`ScoopError::InvalidPythonPath`] if the path is invalid.
pub fn validate_python_path(path: &Path) -> Result<()> {
    if !path.exists() {
        return Err(ScoopError::InvalidPythonPath {
            path: path.to_path_buf(),
            reason: "file not found".to_string(),
        });
    }

    if !path.is_file() {
        return Err(ScoopError::InvalidPythonPath {
            path: path.to_path_buf(),
            reason: "not a file".to_string(),
        });
    }

    // Check executable permission (Unix only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = std::fs::metadata(path)?;
        if metadata.permissions().mode() & 0o111 == 0 {
            return Err(ScoopError::InvalidPythonPath {
                path: path.to_path_buf(),
                reason: "not executable".to_string(),
            });
        }
    }

    Ok(())
}

/// Detect the Python version from a Python executable path.
///
/// Runs `<path> --version` and parses the output to extract the version string.
/// Returns `None` if the version cannot be detected.
///
/// # Examples
///
/// ```no_run
/// use std::path::Path;
/// use scoop_uv::validate::detect_python_version;
///
/// let version = detect_python_version(Path::new("/usr/bin/python3"));
/// // Returns Some("3.12.1") or None
/// ```
pub fn detect_python_version(path: &Path) -> Option<String> {
    let output = Command::new(path).arg("--version").output().ok()?;

    if !output.status.success() {
        return None;
    }

    parse_python_version_output(
        &String::from_utf8_lossy(&output.stdout),
        &String::from_utf8_lossy(&output.stderr),
    )
}

/// Parse `python --version` output into a bare version string.
///
/// Python 3 prints `Python X.Y.Z` to stdout; Python 2 prints it to stderr.
/// Returns `None` if neither stream carries a `Python ` prefix. Pure (no I/O)
/// so it is deterministically testable without spawning a subprocess.
fn parse_python_version_output(stdout: &str, stderr: &str) -> Option<String> {
    let version_text = if stdout.starts_with("Python ") {
        stdout
    } else if stderr.starts_with("Python ") {
        stderr
    } else {
        return None;
    };

    version_text
        .trim()
        .strip_prefix("Python ")
        .map(|v| v.trim().to_string())
}

#[cfg(test)]
mod tests;

// =============================================================================
// Property-based Tests
// =============================================================================

#[cfg(test)]
mod proptests;
