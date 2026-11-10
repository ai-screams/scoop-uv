//! Property-based tests for `python_version`.

use super::*;
use proptest::prelude::*;

proptest! {
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
}
