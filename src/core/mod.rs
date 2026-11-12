//! Core business logic

pub mod doctor;
pub mod export_schema;
pub mod manifest;
mod metadata;
pub mod migrate;
pub mod packages;
mod version;
mod virtualenv;

pub use export_schema::{EXPORT_SCHEMA_VERSION, ExportSchema};
pub use manifest::ScoopManifest;
pub use metadata::Metadata;
pub use version::{VersionService, VersionSource};
pub(crate) use virtualenv::{
    MetadataState, create_venv_dir, env_dirs, env_dirs_complete, inspect_env, read_metadata_file,
    write_metadata_file,
};
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

/// The `home` value of a `pyvenv.cfg`: the directory its interpreter lives
/// in. The first line whose key is exactly `home` with a non-empty value.
pub(crate) fn pyvenv_home(content: &str) -> Option<&str> {
    content.lines().find_map(|line| {
        let (key, value) = line.split_once('=')?;
        let value = value.trim();
        (key.trim() == "home" && !value.is_empty()).then_some(value)
    })
}

/// A Python version read off a `home` path, by whole path components:
/// `python@3.12` (Homebrew), `Versions/3.11` (macOS framework) or
/// `versions/3.11.0` (pyenv). The token is kept as written (`3.13.0rc1`
/// stays), unlike the `version` keys, which are normalized.
pub(crate) fn version_from_home(home: &str) -> Option<String> {
    let parts: Vec<&str> = home.split(['/', '\\']).collect();
    for (i, part) in parts.iter().enumerate() {
        if let Some(version) = part.strip_prefix("python@")
            && !version.is_empty()
        {
            return Some(version.to_string());
        }
        if matches!(*part, "Versions" | "versions")
            && let Some(next) = parts.get(i + 1)
            && !next.is_empty()
        {
            return Some((*next).to_string());
        }
    }
    None
}

/// The Python version of the env at `path` from its `pyvenv.cfg`: the
/// `version` / `version_info` key, else the version in its `home` path.
/// What the migrate discoverers record for a source env.
pub(crate) fn pyvenv_python_version(path: &std::path::Path) -> Option<String> {
    let content = std::fs::read_to_string(path.join("pyvenv.cfg")).ok()?;
    content
        .lines()
        .find_map(pyvenv_version_from_line)
        .or_else(|| version_from_home(pyvenv_home(&content)?))
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
    try_list_installed_packages(venv_path).unwrap_or_default()
}

/// [`list_installed_packages`] that reports a failure instead of an empty
/// list, for callers that copy what they read (`scuv clone`, `scuv export`):
/// an empty list there would silently produce an env or file without the
/// source's packages.
///
/// # Examples
///
/// ```no_run
/// use scoop_uv::core::try_list_installed_packages;
/// use std::path::Path;
///
/// # fn main() -> scoop_uv::error::Result<()> {
/// let pkgs = try_list_installed_packages(Path::new("/path/to/venv"))?;
/// println!("{} packages installed", pkgs.len());
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns [`ScoopError::UvNotFound`](crate::error::ScoopError::UvNotFound)
/// without uv, and the `uv pip list` error when it fails or prints
/// unparseable JSON.
pub fn try_list_installed_packages(
    venv_path: &std::path::Path,
) -> crate::error::Result<Vec<(String, String)>> {
    let entries = crate::uv::UvClient::new()?.pip_list(venv_path)?;
    Ok(entries.into_iter().map(|e| (e.name, e.version)).collect())
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
    use super::{pyvenv_home, version_from_home};

    /// Whole path components only: a directory merely ending in `versions`
    /// or containing `python@` mid-name does not count.
    #[rstest::rstest]
    #[case("/opt/homebrew/opt/python@3.12/bin", Some("3.12"))]
    #[case("/usr/local/Cellar/python@3.11", Some("3.11"))]
    #[case("/Library/Frameworks/Python.framework/Versions/3.11/bin", Some("3.11"))]
    #[case("/Users/u/.pyenv/versions/3.13.0rc1/bin", Some("3.13.0rc1"))]
    #[case("/Users/u/.pyenv/versions/3.9.7", Some("3.9.7"))]
    #[case(r"C:\pyenv\versions\3.12.1\bin", Some("3.12.1"))]
    #[case("/usr/bin", None)]
    #[case("/srv/myversions/3.12/bin", None)]
    #[case("/opt/notpython@3.12/bin", None)]
    #[case("/opt/python@/bin", None)]
    #[case("/x/versions/", None)]
    #[case("/x/versions", None)]
    fn version_from_home_reads_whole_components(#[case] home: &str, #[case] want: Option<&str>) {
        assert_eq!(version_from_home(home).as_deref(), want);
    }

    /// The exact `home` key, first non-empty one; `homebrew = ...` is not it.
    #[test]
    fn pyvenv_home_matches_the_key_exactly() {
        assert_eq!(
            pyvenv_home("home = /a/bin\nhome = /b/bin\n"),
            Some("/a/bin")
        );
        assert_eq!(pyvenv_home("home=/a/bin"), Some("/a/bin"));
        assert_eq!(pyvenv_home("homebrew = /a\nhome =\nhome = /b"), Some("/b"));
        assert_eq!(pyvenv_home("version = 3.12"), None);
    }

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
