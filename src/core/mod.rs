//! Core business logic

pub mod doctor;
pub mod export_schema;
pub mod manifest;
mod metadata;
pub mod migrate;
mod version;
mod virtualenv;

pub use export_schema::{EXPORT_SCHEMA_VERSION, ExportSchema};
pub use manifest::ScoopManifest;
pub use metadata::Metadata;
pub use version::{VersionService, VersionSource};
pub use virtualenv::{VirtualenvInfo, VirtualenvService};

/// Environment variable for currently active virtualenv
pub const SCUV_ACTIVE_ENV: &str = "SCUV_ACTIVE";

/// Parse `pyvenv.cfg` to extract the resolved Python version.
///
/// Reads the `version` key (written by the stdlib `venv`) or `version_info`
/// (written by `uv`) and normalizes the value to `MAJOR.MINOR.PATCH`. This
/// resolves the actual Python version after environment creation, regardless
/// of the specifier used (e.g. `cpython@3.12` -> `3.12.0`).
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
    content.lines().find_map(pyvenv_version_from_line)
}

/// Extract a normalized Python version from a single `pyvenv.cfg` line.
///
/// Returns `Some` only when the line's key is exactly `version` (written by
/// the stdlib `venv`) or `version_info` (written by `uv`). Matching the key
/// precisely avoids the prefix bug where `strip_prefix("version")` turned
/// `version_info = 3.14.3.final.0` into the literal `_info = 3.14.3.final.0`.
/// The value is normalized to `MAJOR.MINOR.PATCH`.
///
/// Shared by the migrate discovery parsers so every `pyvenv.cfg` reader in the
/// codebase handles both keys identically.
pub(crate) fn pyvenv_version_from_line(line: &str) -> Option<String> {
    let (key, value) = line.split_once('=')?;
    matches!(key.trim(), "version" | "version_info").then(|| normalize_pyvenv_version(value.trim()))
}

/// Keep only the leading `MAJOR.MINOR.PATCH` numeric dotted components.
///
/// uv's `version_info` can be `3.14.3.final.0`; this normalizes it to
/// `3.14.3`. A clean `MAJOR.MINOR.PATCH` (or shorter) passes through
/// unchanged. Falls back to the original string if there is no leading
/// numeric segment.
fn normalize_pyvenv_version(raw: &str) -> String {
    let numeric: Vec<&str> = raw
        .split('.')
        .take_while(|seg| !seg.is_empty() && seg.bytes().all(|b| b.is_ascii_digit()))
        .take(3)
        .collect();
    if numeric.is_empty() {
        raw.to_string()
    } else {
        numeric.join(".")
    }
}

/// List `(name, version)` pairs of packages installed in `venv_path`, via
/// `uv pip list --python <venv>`.
///
/// Not the venv's own pip: `scuv create` makes envs with uv, which installs
/// no pip into them, so asking pip found nothing and `clone` copied no
/// packages. uv reads the env's site-packages without needing pip in it.
///
/// Best-effort: no uv, a non-zero exit and unparseable JSON all collapse to
/// an empty vector so callers (`scuv info`, `scuv status`) can degrade
/// gracefully when the env is broken.
///
/// # Examples
///
/// ```no_run
/// use scoop_uv::core::list_installed_packages;
/// use std::path::Path;
///
/// let pkgs = list_installed_packages(Path::new("/path/to/venv"));
/// println!("{} packages installed", pkgs.len());
/// ```
pub fn list_installed_packages(venv_path: &std::path::Path) -> Vec<(String, String)> {
    crate::uv::UvClient::new()
        .and_then(|uv| uv.pip_list(venv_path))
        .map(|entries| entries.into_iter().map(|e| (e.name, e.version)).collect())
        .unwrap_or_default()
}

/// Get the currently active environment name from $SCUV_ACTIVE
///
/// # Examples
///
/// ```
/// use scoop_uv::core::get_active_env;
/// // Returns None if SCUV_ACTIVE is not set
/// // SAFETY: This doctest runs in isolation
/// unsafe { std::env::remove_var("SCUV_ACTIVE") };
/// assert_eq!(get_active_env(), None);
/// ```
pub fn get_active_env() -> Option<String> {
    std::env::var(SCUV_ACTIVE_ENV).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn venv_with_cfg(contents: &str) -> tempfile::TempDir {
        let dir = tempfile::TempDir::new().unwrap();
        let mut f = std::fs::File::create(dir.path().join("pyvenv.cfg")).unwrap();
        f.write_all(contents.as_bytes()).unwrap();
        dir
    }

    #[test]
    fn parse_pyvenv_version_reads_stdlib_version_key() {
        let dir = venv_with_cfg("home = /usr/bin\nversion = 3.12.12\n");
        assert_eq!(
            parse_pyvenv_version(dir.path()),
            Some("3.12.12".to_string())
        );
    }

    #[test]
    fn parse_pyvenv_version_reads_uv_version_info_key() {
        // Regression: uv writes `version_info`. The old prefix match produced
        // "_info = 3.14.3.final.0"; we must read the value and normalize it.
        let dir =
            venv_with_cfg("home = /x\nimplementation = CPython\nversion_info = 3.14.3.final.0\n");
        assert_eq!(parse_pyvenv_version(dir.path()), Some("3.14.3".to_string()));
    }

    #[test]
    fn parse_pyvenv_version_missing_returns_none() {
        let dir = venv_with_cfg("home = /x\nimplementation = CPython\n");
        assert_eq!(parse_pyvenv_version(dir.path()), None);
    }

    #[test]
    fn normalize_pyvenv_version_truncates_suffix() {
        assert_eq!(normalize_pyvenv_version("3.14.3.final.0"), "3.14.3");
        assert_eq!(normalize_pyvenv_version("3.12.12"), "3.12.12");
        assert_eq!(normalize_pyvenv_version("3.12"), "3.12");
        assert_eq!(normalize_pyvenv_version("3"), "3");
    }

    #[test]
    fn normalize_pyvenv_version_non_numeric_falls_back() {
        assert_eq!(normalize_pyvenv_version("garbage"), "garbage");
        assert_eq!(normalize_pyvenv_version(""), "");
        // Stops at the first empty/non-numeric segment; a leading `v` has no
        // numeric prefix at all, so the original string is returned.
        assert_eq!(normalize_pyvenv_version("3..4"), "3");
        assert_eq!(normalize_pyvenv_version("v3.12"), "v3.12");
    }

    // ==========================================================================
    // list_installed_packages — graceful-degradation pins
    // ==========================================================================

    #[test]
    #[serial_test::serial]
    fn list_installed_packages_nonexistent_path_returns_empty() {
        let pkgs = list_installed_packages(std::path::Path::new("/nonexistent/path/to/venv"));
        assert!(pkgs.is_empty());
    }

    /// `scuv create` makes envs with uv, which puts no pip in them; the
    /// packages come from `uv pip list` (here a fake uv that reports six).
    /// Fails if the lookup goes back to the env's own pip, which finds
    /// nothing, so clone copies nothing and export writes an empty list.
    #[cfg(unix)]
    #[test]
    #[serial_test::serial]
    fn list_installed_packages_reads_an_env_without_pip() {
        let uv = crate::test_utils::FakeUv::new(&[]);
        let path = uv.path_var();
        let _env = crate::test_utils::env_guard(&[("PATH", Some(path.as_str()))]);
        let temp = tempfile::TempDir::new().unwrap();
        std::fs::create_dir_all(temp.path().join("bin")).unwrap();
        assert_eq!(
            list_installed_packages(temp.path()),
            [("six".to_string(), "1.16.0".to_string())]
        );
        // It asks about this env's interpreter, in JSON.
        let python = crate::paths::virtualenv_python_exe(temp.path());
        assert_eq!(
            uv.pip_calls(),
            [format!(
                "pip list --format=json --python {}",
                python.display()
            )]
        );
    }
}
