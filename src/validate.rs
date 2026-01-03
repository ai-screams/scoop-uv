//! Validation utilities for scoop

use once_cell::sync::Lazy;
use regex::Regex;

use crate::error::{Result, ScoopError};

/// Regex for valid environment names
/// - Must start with a letter
/// - Can contain letters, numbers, hyphens, and underscores
static ENV_NAME_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[a-zA-Z][a-zA-Z0-9_-]*$").unwrap());

/// Regex for Python version strings
static VERSION_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\d+(\.\d+)*([a-z]\d+)?$").unwrap());

/// Reserved names that cannot be used as environment names
const RESERVED_NAMES: &[&str] = &[
    "activate",
    "base",
    "completions",
    "create",
    "deactivate",
    "default",
    "delete",
    "global",
    "help",
    "init",
    "install",
    "list",
    "local",
    "remove",
    "resolve",
    "root",
    "system",
    "uninstall",
    "use",
    "version",
    "versions",
];

/// Maximum length for environment names
const MAX_ENV_NAME_LENGTH: usize = 64;

/// Check if a string is a valid environment name
pub fn is_valid_env_name(name: &str) -> bool {
    if name.is_empty() || name.len() > MAX_ENV_NAME_LENGTH {
        return false;
    }

    if RESERVED_NAMES.contains(&name.to_lowercase().as_str()) {
        return false;
    }

    if VERSION_REGEX.is_match(name) {
        return false;
    }

    ENV_NAME_REGEX.is_match(name)
}

/// Validate an environment name, returning an error if invalid
pub fn validate_env_name(name: &str) -> Result<()> {
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

    if RESERVED_NAMES.contains(&name.to_lowercase().as_str()) {
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

/// Check if a string is a valid Python version
pub fn is_valid_python_version(version: &str) -> bool {
    // Accept formats like: 3, 3.12, 3.12.0, 3.12.0a1, 3.12.0rc1
    VERSION_REGEX.is_match(version)
        || version.chars().all(|c| c.is_ascii_digit() || c == '.')
}

/// Normalize a Python version string
/// e.g., "3" -> "3", "3.12" -> "3.12", "3.12.0" -> "3.12.0"
pub fn normalize_python_version(version: &str) -> String {
    version.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_env_names() {
        assert!(is_valid_env_name("myenv"));
        assert!(is_valid_env_name("MyEnv"));
        assert!(is_valid_env_name("my-env"));
        assert!(is_valid_env_name("my_env"));
        assert!(is_valid_env_name("env123"));
        assert!(is_valid_env_name("a"));
    }

    #[test]
    fn test_invalid_env_names() {
        assert!(!is_valid_env_name(""));
        assert!(!is_valid_env_name("123"));
        assert!(!is_valid_env_name("3.12"));
        assert!(!is_valid_env_name("-env"));
        assert!(!is_valid_env_name("_env"));
        assert!(!is_valid_env_name("my env"));
        assert!(!is_valid_env_name("my.env"));
    }

    #[test]
    fn test_reserved_names() {
        assert!(!is_valid_env_name("activate"));
        assert!(!is_valid_env_name("ACTIVATE"));
        assert!(!is_valid_env_name("list"));
        assert!(!is_valid_env_name("version"));
    }

    #[test]
    fn test_valid_python_versions() {
        assert!(is_valid_python_version("3"));
        assert!(is_valid_python_version("3.12"));
        assert!(is_valid_python_version("3.12.0"));
        assert!(is_valid_python_version("3.12.0a1"));
    }
}
