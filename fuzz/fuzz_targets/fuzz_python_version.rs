#![no_main]
//! Fuzz Python-version validation/parsing. The contract is "never panic on
//! arbitrary input"; both entry points are exercised.

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(version) = std::str::from_utf8(data) else {
        return;
    };

    let _ = scoop_uv::validate::is_valid_python_version(version);
    let _ = scoop_uv::validate::validate_python_version(version);
    let _ = scoop_uv::validate::PythonVersion::parse(version);
});
