//! Property-based tests for `validate`.

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

    // Property: the strict check is the existing-name check plus the
    // reserved list, nothing else.
    #[test]
    fn prop_strict_is_existing_minus_reserved(name in "[ -~]{0,70}") {
        let reserved = super::RESERVED_NAMES.contains(&name.to_lowercase().as_str());
        prop_assert_eq!(
            is_valid_env_name(&name),
            is_valid_existing_env_name(&name) && !reserved
        );
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
