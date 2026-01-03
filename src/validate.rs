//! Validation utilities for scoop

use once_cell::sync::Lazy;
use regex::Regex;

use crate::error::{Result, ScoopError};

/// Regex for valid environment names
/// - Must start with a letter
/// - Can contain letters, numbers, hyphens, and underscores
static ENV_NAME_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[a-zA-Z][a-zA-Z0-9_-]*$").unwrap());

/// Regex for Python version strings
static VERSION_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\d+(\.\d+)*([a-z]\d+)?$").unwrap());

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
    if version.is_empty() {
        return false;
    }
    // Accept formats like: 3, 3.12, 3.12.0, 3.12.0a1, 3.12.0rc1
    VERSION_REGEX.is_match(version) || version.chars().all(|c| c.is_ascii_digit() || c == '.')
}

/// Normalize a Python version string
/// e.g., "3" -> "3", "3.12" -> "3.12", "3.12.0" -> "3.12.0"
pub fn normalize_python_version(version: &str) -> String {
    version.trim().to_string()
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

/// Parsed Python version components
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PythonVersion {
    pub major: u32,
    pub minor: Option<u32>,
    pub patch: Option<u32>,
    pub suffix: Option<String>,
}

impl PythonVersion {
    /// Parse a version string into components
    pub fn parse(version: &str) -> Option<Self> {
        let trimmed = version.trim();
        if trimmed.is_empty() {
            return None;
        }

        // Handle special aliases
        if matches!(trimmed.to_lowercase().as_str(), "latest" | "stable") {
            return None;
        }

        // Split by dots and parse
        let mut parts = trimmed.split('.');
        let major_str = parts.next()?;
        let major: u32 = major_str.parse().ok()?;

        let minor = parts.next().and_then(|s| {
            // Minor might have suffix like "12a1"
            let digits: String = s.chars().take_while(|c| c.is_ascii_digit()).collect();
            digits.parse().ok()
        });

        let patch = parts.next().and_then(|s| {
            let digits: String = s.chars().take_while(|c| c.is_ascii_digit()).collect();
            digits.parse().ok()
        });

        // Extract suffix (e.g., "a1", "b2", "rc1")
        let suffix = if let Some(minor_str) = trimmed.split('.').nth(1) {
            let suffix_start = minor_str.chars().position(|c| c.is_ascii_alphabetic());
            suffix_start.map(|pos| minor_str[pos..].to_string())
        } else {
            None
        };

        Some(Self {
            major,
            minor,
            patch,
            suffix,
        })
    }

    /// Check if this version matches another (for version resolution)
    /// e.g., "3.12" matches "3.12.0", "3.12.1", etc.
    pub fn matches(&self, other: &Self) -> bool {
        if self.major != other.major {
            return false;
        }

        if let Some(minor) = self.minor {
            if other.minor != Some(minor) {
                return false;
            }
        }

        if let Some(patch) = self.patch {
            if other.patch != Some(patch) {
                return false;
            }
        }

        true
    }
}

impl std::fmt::Display for PythonVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.major)?;
        if let Some(minor) = self.minor {
            write!(f, ".{minor}")?;
        }
        if let Some(patch) = self.patch {
            write!(f, ".{patch}")?;
        }
        if let Some(ref suffix) = self.suffix {
            write!(f, "{suffix}")?;
        }
        Ok(())
    }
}

/// Check if a version string is a special alias (latest, stable)
pub fn is_version_alias(version: &str) -> bool {
    matches!(version.trim().to_lowercase().as_str(), "latest" | "stable")
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

    #[test]
    fn test_invalid_python_versions() {
        assert!(!is_valid_python_version(""));
        assert!(!is_valid_python_version("abc"));
        assert!(!is_valid_python_version("v3.12")); // "v" prefix is invalid
        assert!(!is_valid_python_version("3.12-beta")); // Hyphen is invalid
    }

    #[test]
    fn test_validate_python_version() {
        assert!(validate_python_version("3.12").is_ok());
        assert!(validate_python_version("3.12.0").is_ok());
        assert!(validate_python_version("latest").is_ok());
        assert!(validate_python_version("LATEST").is_ok());
        assert!(validate_python_version("stable").is_ok());
        assert!(validate_python_version("STABLE").is_ok());
        assert!(validate_python_version("").is_err());
    }

    #[test]
    fn test_python_version_parse() {
        let v = PythonVersion::parse("3").unwrap();
        assert_eq!(v.major, 3);
        assert_eq!(v.minor, None);
        assert_eq!(v.patch, None);

        let v = PythonVersion::parse("3.12").unwrap();
        assert_eq!(v.major, 3);
        assert_eq!(v.minor, Some(12));
        assert_eq!(v.patch, None);

        let v = PythonVersion::parse("3.12.0").unwrap();
        assert_eq!(v.major, 3);
        assert_eq!(v.minor, Some(12));
        assert_eq!(v.patch, Some(0));

        assert!(PythonVersion::parse("latest").is_none());
        assert!(PythonVersion::parse("").is_none());
    }

    #[test]
    fn test_python_version_parse_with_suffix() {
        let v = PythonVersion::parse("3.12.0a1").unwrap();
        assert_eq!(v.major, 3);
        assert_eq!(v.minor, Some(12));
        assert_eq!(v.patch, Some(0));
        // Suffix is parsed from minor string position, so it won't include patch suffix

        let v = PythonVersion::parse("3.13a1").unwrap();
        assert_eq!(v.major, 3);
        assert_eq!(v.minor, Some(13));
        assert!(v.suffix.is_some());
    }

    #[test]
    fn test_python_version_matches() {
        let v312 = PythonVersion::parse("3.12").unwrap();
        let v3120 = PythonVersion::parse("3.12.0").unwrap();
        let v3121 = PythonVersion::parse("3.12.1").unwrap();
        let v311 = PythonVersion::parse("3.11").unwrap();

        // 3.12 matches 3.12.0 and 3.12.1
        assert!(v312.matches(&v3120));
        assert!(v312.matches(&v3121));

        // 3.12 does not match 3.11
        assert!(!v312.matches(&v311));

        // 3.12.0 does not match 3.12.1
        assert!(!v3120.matches(&v3121));
    }

    #[test]
    fn test_python_version_display() {
        assert_eq!(PythonVersion::parse("3").unwrap().to_string(), "3");
        assert_eq!(PythonVersion::parse("3.12").unwrap().to_string(), "3.12");
        assert_eq!(
            PythonVersion::parse("3.12.0").unwrap().to_string(),
            "3.12.0"
        );
    }

    #[test]
    fn test_is_version_alias() {
        assert!(is_version_alias("latest"));
        assert!(is_version_alias("LATEST"));
        assert!(is_version_alias("stable"));
        assert!(is_version_alias("  stable  "));
        assert!(!is_version_alias("3.12"));
        assert!(!is_version_alias(""));
    }

    #[test]
    fn test_validate_env_name_errors() {
        let err = validate_env_name("").unwrap_err();
        assert!(err.to_string().contains("empty"));

        let err = validate_env_name("activate").unwrap_err();
        assert!(err.to_string().contains("reserved"));

        let err = validate_env_name("3.12").unwrap_err();
        assert!(err.to_string().contains("version"));

        let err = validate_env_name("-invalid").unwrap_err();
        assert!(err.to_string().contains("must start with a letter"));
    }

    #[test]
    fn test_env_name_max_length() {
        let long_name = "a".repeat(64);
        assert!(is_valid_env_name(&long_name));

        let too_long = "a".repeat(65);
        assert!(!is_valid_env_name(&too_long));
    }
}
