//! Unit tests for `python_version`.

use super::*;

/// Fuzz regression (crash-dbaf50a5, found 2026-07-14): suffix extraction
/// took a CHAR index from `chars().position()` and sliced the string with
/// it as a BYTE index — any multi-byte character before the alphabetic
/// suffix panicked on a non-char-boundary slice.
#[test]
fn test_python_version_parse_multibyte_before_suffix_does_not_panic() {
    // Exact fuzzer input: "0.\u{0678}la'0+"
    let input = String::from_utf8(vec![48, 46, 217, 184, 108, 97, 39, 48, 43]).unwrap();
    let v = PythonVersion::parse(&input).expect("major digit parses");
    assert_eq!(v.major, 0);
    assert_eq!(v.minor, None);
    // suffix starts at the first ASCII-alphabetic char, at a byte boundary
    assert_eq!(v.suffix.as_deref(), Some("la'0+"));

    // Multi-byte char before a DIGIT then suffix: slicing by char index
    // would silently return the wrong suffix here instead of panicking.
    let v = PythonVersion::parse("3.β1a").expect("major digit parses");
    assert_eq!(v.suffix.as_deref(), Some("a"));
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
fn test_python_version_ordering() {
    let v3_9 = PythonVersion::parse("3.9").unwrap();
    let v3_10 = PythonVersion::parse("3.10").unwrap();
    let v3_11 = PythonVersion::parse("3.11").unwrap();
    let v3_12 = PythonVersion::parse("3.12").unwrap();
    let v3_12_0 = PythonVersion::parse("3.12.0").unwrap();
    let v3_12_1 = PythonVersion::parse("3.12.1").unwrap();
    let v3_13a1 = PythonVersion::parse("3.13a1").unwrap();
    let v3_13rc1 = PythonVersion::parse("3.13rc1").unwrap();
    let v3_13 = PythonVersion::parse("3.13").unwrap();

    // Major.minor ordering
    assert!(v3_9 < v3_10);
    assert!(v3_10 < v3_11);
    assert!(v3_11 < v3_12);

    // Patch ordering
    assert!(v3_12_0 < v3_12_1);

    // Pre-release < stable
    assert!(v3_13a1 < v3_13rc1);
    assert!(v3_13rc1 < v3_13);
    assert!(v3_13a1 < v3_13);

    // Sorted collection test
    use std::collections::BTreeSet;
    let mut versions = BTreeSet::new();
    versions.insert(v3_12.clone());
    versions.insert(v3_10.clone());
    versions.insert(v3_11.clone());
    versions.insert(v3_9.clone());

    let sorted: Vec<_> = versions.iter().collect();
    assert_eq!(sorted, vec![&v3_9, &v3_10, &v3_11, &v3_12]);
}
