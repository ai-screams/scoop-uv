//! Core business logic

pub mod doctor;
mod metadata;
mod version;
mod virtualenv;

pub use metadata::Metadata;
pub use version::VersionService;
pub use virtualenv::VirtualenvService;

/// Environment variable for currently active virtualenv
pub const SCOOP_ACTIVE_ENV: &str = "SCOOP_ACTIVE";

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
