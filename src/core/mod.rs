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
pub use version::VersionService;
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

/// List `(name, version)` pairs of packages installed in `venv_path` by asking
/// the venv's own `pip list --format=json`.
///
/// Best-effort: missing pip, non-zero exit, and unparseable JSON all collapse
/// to an empty vector so callers (`scoop info`, `scoop status`) can degrade
/// gracefully when the env is broken or pip simply isn't present.
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
    let pip_path = crate::paths::virtualenv_pip_exe(venv_path);
    if !pip_path.exists() {
        return Vec::new();
    }
    let output = std::process::Command::new(&pip_path)
        .args(["list", "--format=json"])
        .output();
    match output {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            serde_json::from_str::<Vec<serde_json::Value>>(&stdout)
                .unwrap_or_default()
                .into_iter()
                .filter_map(|p| {
                    Some((
                        p.get("name")?.as_str()?.to_string(),
                        p.get("version")?.as_str()?.to_string(),
                    ))
                })
                .collect()
        }
        _ => Vec::new(),
    }
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
    fn list_installed_packages_nonexistent_path_returns_empty() {
        let pkgs = list_installed_packages(std::path::Path::new("/nonexistent/path/to/venv"));
        assert!(pkgs.is_empty());
    }

    #[test]
    fn list_installed_packages_no_pip_returns_empty() {
        // Bin dir exists but pip is missing — must not panic, must return [].
        let temp = tempfile::TempDir::new().unwrap();
        std::fs::create_dir_all(temp.path().join("bin")).unwrap();
        assert!(list_installed_packages(temp.path()).is_empty());
    }
}
