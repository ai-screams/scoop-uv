//! Validation utilities for scoop

use once_cell::sync::Lazy;
use regex::Regex;

use crate::error::{Result, ScoopError};

/// Regex for valid environment names
/// - Must start with a letter
/// - Can contain letters, numbers, hyphens, and underscores
static ENV_NAME_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[a-zA-Z][a-zA-Z0-9_-]*$").unwrap());

/// Regex for Python version strings
/// Supports: 3, 3.12, 3.12.0, 3.12.0a1, 3.12.0b2, 3.12.0rc1
static VERSION_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\d+(\.\d+)*((a|b|rc)\d+)?$").unwrap());

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

/// Check if a string is a valid environment name.
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

/// Validate an environment name, returning an error if invalid.
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

/// Parsed Python version components.
///
/// # Examples
///
/// ```
/// use scoop_uv::validate::PythonVersion;
///
/// let v = PythonVersion::parse("3.12.0").unwrap();
/// assert_eq!(v.major, 3);
/// assert_eq!(v.minor, Some(12));
/// assert_eq!(v.patch, Some(0));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PythonVersion {
    /// Major version number (e.g., 3 in "3.12.0")
    pub major: u32,
    /// Minor version number (e.g., 12 in "3.12.0")
    pub minor: Option<u32>,
    /// Patch version number (e.g., 0 in "3.12.0")
    pub patch: Option<u32>,
    /// Pre-release suffix (e.g., "a1", "rc1")
    pub suffix: Option<String>,
}

impl PythonVersion {
    /// Parse a version string into components.
    ///
    /// # Examples
    ///
    /// ```
    /// use scoop_uv::validate::PythonVersion;
    ///
    /// // Parse major.minor.patch
    /// let v = PythonVersion::parse("3.12.0").unwrap();
    /// assert_eq!(v.major, 3);
    /// assert_eq!(v.minor, Some(12));
    ///
    /// // Parse with pre-release suffix (suffix in minor position)
    /// let v = PythonVersion::parse("3.13a1").unwrap();
    /// assert_eq!(v.suffix, Some("a1".to_string()));
    ///
    /// // Returns None for invalid versions
    /// assert!(PythonVersion::parse("").is_none());
    /// assert!(PythonVersion::parse("latest").is_none());
    /// ```
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

/// Check if a version string is a special alias (latest, stable).
///
/// # Examples
///
/// ```
/// use scoop_uv::validate::is_version_alias;
///
/// assert!(is_version_alias("latest"));
/// assert!(is_version_alias("stable"));
/// assert!(is_version_alias("LATEST"));  // case-insensitive
///
/// assert!(!is_version_alias("3.12"));
/// assert!(!is_version_alias("beta"));
/// ```
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

    // ==========================================================================
    // Boundary Value Tests
    // ==========================================================================

    #[test]
    fn test_env_name_boundary_values() {
        // Minimum valid length (1 character)
        assert!(is_valid_env_name("a"));
        assert!(is_valid_env_name("Z"));

        // Just below max length (63 characters)
        let just_under = "a".repeat(63);
        assert!(is_valid_env_name(&just_under));

        // Exactly at max length (64 characters)
        let at_max = "a".repeat(64);
        assert!(is_valid_env_name(&at_max));

        // Just above max length (65 characters)
        let just_over = "a".repeat(65);
        assert!(!is_valid_env_name(&just_over));

        // Way over max length
        let way_over = "a".repeat(1000);
        assert!(!is_valid_env_name(&way_over));
    }

    #[test]
    fn test_env_name_whitespace_handling() {
        // Whitespace-only names must be rejected
        assert!(!is_valid_env_name(" "));
        assert!(!is_valid_env_name("  "));
        assert!(!is_valid_env_name("\t"));
        assert!(!is_valid_env_name("\n"));
        assert!(!is_valid_env_name(" \t\n "));

        // Names with leading/trailing whitespace
        assert!(!is_valid_env_name(" myenv"));
        assert!(!is_valid_env_name("myenv "));
        assert!(!is_valid_env_name(" myenv "));
    }

    #[test]
    fn test_python_version_boundary_values() {
        // Single digit major version
        assert!(is_valid_python_version("3"));
        assert!(is_valid_python_version("2"));

        // Two digit major version (future-proofing)
        assert!(is_valid_python_version("12"));
        assert!(is_valid_python_version("99"));

        // Very long version string
        assert!(is_valid_python_version("3.12.0"));
        assert!(is_valid_python_version("3.12.999"));

        // Pre-release versions
        assert!(is_valid_python_version("3.12.0a1"));
        assert!(is_valid_python_version("3.12.0b99"));
        assert!(is_valid_python_version("3.12.0rc1"));
    }

    #[test]
    fn test_reserved_names_case_insensitivity() {
        // All reserved names should be rejected regardless of case
        for reserved in ["activate", "deactivate", "list", "create", "remove", "use"] {
            assert!(
                !is_valid_env_name(reserved),
                "lowercase '{}' should be rejected",
                reserved
            );
            assert!(
                !is_valid_env_name(&reserved.to_uppercase()),
                "uppercase '{}' should be rejected",
                reserved
            );

            // Mixed case
            let mixed: String = reserved
                .chars()
                .enumerate()
                .map(|(i, c)| {
                    if i % 2 == 0 {
                        c.to_uppercase().next().unwrap()
                    } else {
                        c
                    }
                })
                .collect();
            assert!(
                !is_valid_env_name(&mixed),
                "mixed case '{}' should be rejected",
                mixed
            );
        }
    }

    // ==========================================================================
    // Security Tests: Path Traversal and Injection Prevention
    // ==========================================================================

    #[test]
    fn test_env_name_path_traversal_rejected() {
        // Path traversal attempts must be rejected
        assert!(!is_valid_env_name("../"));
        assert!(!is_valid_env_name(".."));
        assert!(!is_valid_env_name("../etc"));
        assert!(!is_valid_env_name("../../../etc/passwd"));
        assert!(!is_valid_env_name("foo/../bar"));
        assert!(!is_valid_env_name("foo/.."));
    }

    #[test]
    fn test_env_name_hidden_file_rejected() {
        // Hidden file/directory names (starting with .) must be rejected
        assert!(!is_valid_env_name(".hidden"));
        assert!(!is_valid_env_name("."));
        assert!(!is_valid_env_name(".."));
        assert!(!is_valid_env_name(".gitignore"));
        assert!(!is_valid_env_name(".env"));
    }

    #[test]
    fn test_env_name_absolute_path_rejected() {
        // Absolute paths must be rejected
        assert!(!is_valid_env_name("/etc/passwd"));
        assert!(!is_valid_env_name("/tmp/evil"));
        assert!(!is_valid_env_name("/"));
    }

    #[test]
    fn test_env_name_special_chars_rejected() {
        // Special shell characters must be rejected
        assert!(!is_valid_env_name("env;ls"));
        assert!(!is_valid_env_name("env|cat"));
        assert!(!is_valid_env_name("env&bg"));
        assert!(!is_valid_env_name("env`cmd`"));
        assert!(!is_valid_env_name("env$(cmd)"));
        assert!(!is_valid_env_name("env>file"));
        assert!(!is_valid_env_name("env<file"));
        assert!(!is_valid_env_name("env\nwhoami"));
        assert!(!is_valid_env_name("env\twhoami"));
    }

    #[test]
    fn test_env_name_null_byte_rejected() {
        // Null bytes must be rejected (C string terminator attack)
        assert!(!is_valid_env_name("env\0hidden"));
        assert!(!is_valid_env_name("\0"));
    }

    #[test]
    fn test_env_name_unicode_rejected() {
        // Unicode homoglyphs and non-ASCII must be rejected
        assert!(!is_valid_env_name("еnv")); // Cyrillic 'е' (U+0435) looks like 'e'
        assert!(!is_valid_env_name("환경")); // Korean
        assert!(!is_valid_env_name("環境")); // Chinese
        assert!(!is_valid_env_name("env™"));
        assert!(!is_valid_env_name("env🐍")); // Emoji
    }

    #[test]
    fn test_validate_env_name_path_traversal_error_message() {
        // Verify error message is helpful for security-related rejections
        let result = validate_env_name("../etc");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("must start with a letter"));
    }
}

// =============================================================================
// Property-based Tests
// =============================================================================

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    // Strategy for generating valid environment names
    fn valid_env_name_strategy() -> impl Strategy<Value = String> {
        // Start with a letter, followed by 0-62 alphanumeric/dash/underscore chars
        "[a-zA-Z][a-zA-Z0-9_-]{0,62}".prop_filter("not reserved", |s| {
            !super::RESERVED_NAMES.contains(&s.to_lowercase().as_str())
        })
    }

    // Strategy for generating valid Python versions
    fn valid_python_version_strategy() -> impl Strategy<Value = String> {
        prop_oneof![
            // Major only: 2, 3, etc.
            (1u32..20).prop_map(|m| m.to_string()),
            // Major.minor: 3.12, 3.13, etc.
            (2u32..4, 0u32..20).prop_map(|(m, n)| format!("{}.{}", m, n)),
            // Major.minor.patch: 3.12.0, 3.12.1, etc.
            (2u32..4, 0u32..20, 0u32..100).prop_map(|(m, n, p)| format!("{}.{}.{}", m, n, p)),
            // With pre-release: 3.12.0a1, 3.12.0b2, 3.12.0rc1
            (
                2u32..4,
                0u32..20,
                0u32..10,
                prop_oneof!["a", "b", "rc"],
                1u32..10
            )
                .prop_map(|(m, n, p, pre, num)| format!("{}.{}.{}{}{}", m, n, p, pre, num)),
        ]
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(500))]

        // Property: Valid env names should always pass validation
        #[test]
        fn prop_valid_env_names_accepted(name in valid_env_name_strategy()) {
            prop_assert!(is_valid_env_name(&name), "Valid name '{}' was rejected", name);
            prop_assert!(validate_env_name(&name).is_ok(), "validate_env_name failed for '{}'", name);
        }

        // Property: Names starting with digits should always be rejected
        #[test]
        fn prop_digit_prefix_rejected(
            digit in "[0-9]",
            rest in "[a-zA-Z0-9_-]{0,10}"
        ) {
            let name = format!("{}{}", digit, rest);
            prop_assert!(!is_valid_env_name(&name), "Digit-prefixed '{}' was accepted", name);
        }

        // Property: Names with path separators should always be rejected
        #[test]
        fn prop_path_separators_rejected(
            prefix in "[a-zA-Z]{1,5}",
            suffix in "[a-zA-Z]{1,5}"
        ) {
            let with_slash = format!("{}/{}", prefix, suffix);
            let with_backslash = format!("{}\\{}", prefix, suffix);
            let with_dots = format!("{}/../{}", prefix, suffix);

            prop_assert!(!is_valid_env_name(&with_slash), "Path '{}' was accepted", with_slash);
            prop_assert!(!is_valid_env_name(&with_backslash), "Path '{}' was accepted", with_backslash);
            prop_assert!(!is_valid_env_name(&with_dots), "Path '{}' was accepted", with_dots);
        }

        // Property: Valid Python versions should always pass
        #[test]
        fn prop_valid_python_versions_accepted(version in valid_python_version_strategy()) {
            prop_assert!(
                is_valid_python_version(&version),
                "Valid version '{}' was rejected", version
            );
        }

        // Property: Python version parsing roundtrip
        #[test]
        fn prop_python_version_parse_roundtrip(
            major in 2u32..4,
            minor in 0u32..20,
            patch in 0u32..100
        ) {
            let version_str = format!("{}.{}.{}", major, minor, patch);
            let parsed = PythonVersion::parse(&version_str);
            prop_assert!(parsed.is_some(), "Failed to parse '{}'", version_str);

            let pv = parsed.unwrap();
            prop_assert_eq!(pv.major, major);
            prop_assert_eq!(pv.minor, Some(minor));
            prop_assert_eq!(pv.patch, Some(patch));
        }

        // Property: Empty strings should always be invalid
        #[test]
        fn prop_empty_always_invalid(spaces in " {0,10}") {
            prop_assert!(!is_valid_env_name(&spaces));
            prop_assert!(!is_valid_python_version(&spaces));
        }

        // Property: Reserved names should always be rejected (case-insensitive)
        #[test]
        fn prop_reserved_names_rejected(
            reserved in prop::sample::select(super::RESERVED_NAMES.to_vec()),
            upper in prop::bool::ANY
        ) {
            let name = if upper { reserved.to_uppercase() } else { reserved.to_string() };
            prop_assert!(!is_valid_env_name(&name), "Reserved '{}' was accepted", name);
        }
    }
}
