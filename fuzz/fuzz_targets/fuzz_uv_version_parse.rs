#![no_main]
//! Fuzz `uv --version` parsing. Must never panic, and a parsed triple must
//! round-trip through formatting.

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(raw) = std::str::from_utf8(data) else {
        return;
    };

    if let Some((major, minor, patch)) = scoop_uv::uv::version::parse(raw) {
        // Round-trip: a canonical "uv X.Y.Z" string must parse back identically.
        let formatted = format!("uv {major}.{minor}.{patch}");
        assert_eq!(
            scoop_uv::uv::version::parse(&formatted),
            Some((major, minor, patch))
        );
    }
});
