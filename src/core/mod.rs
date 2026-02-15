//! Core business logic

pub mod doctor;
mod metadata;
pub mod migrate;
mod version;
mod virtualenv;

pub use metadata::Metadata;
pub use version::VersionService;
pub use virtualenv::VirtualenvService;

/// Environment variable for currently active virtualenv
pub const SCOOP_ACTIVE_ENV: &str = "SCOOP_ACTIVE";

/// Parse `pyvenv.cfg` to extract the resolved Python version.
///
/// Reads the `version` key from a venv's `pyvenv.cfg` file.
/// This is used to resolve the actual Python version after environment creation,
/// regardless of the specifier used (e.g., `cpython@3.12` resolves to `3.12.0`).
///
/// # Examples
///
/// ```no_run
/// # use std::path::Path;
/// use scoop_uv::core::parse_pyvenv_version;
/// let version = parse_pyvenv_version(Path::new("/path/to/venv"));
/// ```
pub fn parse_pyvenv_version(venv_path: &std::path::Path) -> Option<String> {
    let cfg_path = venv_path.join("pyvenv.cfg");
    let content = std::fs::read_to_string(&cfg_path).ok()?;

    for line in content.lines() {
        let line = line.trim();
        if let Some(value) = line.strip_prefix("version") {
            let value = value.trim_start_matches([' ', '=']);
            return Some(value.trim().to_string());
        }
    }

    None
}

/// Get the currently active environment name from $SCOOP_ACTIVE
///
/// # Examples
///
/// ```
/// use scoop_uv::core::get_active_env;
/// // Returns None if SCOOP_ACTIVE is not set
/// // SAFETY: This doctest runs in isolation
/// unsafe { std::env::remove_var("SCOOP_ACTIVE") };
/// assert_eq!(get_active_env(), None);
/// ```
pub fn get_active_env() -> Option<String> {
    std::env::var(SCOOP_ACTIVE_ENV).ok()
}
