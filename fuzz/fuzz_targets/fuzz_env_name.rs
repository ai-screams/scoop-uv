#![no_main]
//! Fuzz environment-name validation — the security-critical gate, since names
//! become filesystem paths.

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(name) = std::str::from_utf8(data) else {
        return;
    };

    let is_valid = scoop_uv::validate::is_valid_env_name(name);

    // Invariant 1: the bool and Result validators must always agree.
    assert_eq!(is_valid, scoop_uv::validate::validate_env_name(name).is_ok());

    // Invariant 2: an accepted name must satisfy the security contract — no path
    // separators, parent refs, NUL, and it must look like a plain identifier.
    if is_valid {
        assert!(!name.contains('/'));
        assert!(!name.contains('\\'));
        assert!(!name.contains('\0'));
        assert!(!name.contains(".."));
        assert!(name.chars().next().is_some_and(|c| c.is_ascii_alphabetic()));
        assert!(name.len() <= 64);
    }
});
