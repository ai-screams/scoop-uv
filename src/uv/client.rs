//! uv CLI client

use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Deserialize;

use crate::error::{Result, ScoopError};
use crate::validate::PythonVersion;

/// One entry from `uv pip list --format=json`.
///
/// uv emits more fields per release (`location`, `requires`,
/// `requires_python`, ...); we deserialize only what callers consume
/// and let serde drop the rest, matching the [`UvPythonEntry`]
/// pattern. A `Some` `editable_project_location` marks a package
/// installed via `pip install -e <path>`.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct UvPipListEntry {
    pub name: String,
    pub version: String,
    pub editable_project_location: Option<PathBuf>,
}

/// Information about an installed Python version
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PythonInfo {
    /// The version string (e.g., "3.12.0")
    pub version: String,
    /// Path to the Python executable
    pub path: Option<PathBuf>,
    /// Whether this version is installed locally by uv
    pub installed: bool,
    /// Implementation (cpython, pypy, etc.)
    pub implementation: String,
}

/// One entry from `uv python list --output-format=json`.
///
/// uv emits more fields (`key`, `version_parts`, `os`, `arch`, ...); we only
/// deserialize what `PythonInfo` needs and let serde drop the rest. A null
/// `path` marks a version that is available for download but not installed.
#[derive(Debug, Deserialize)]
struct UvPythonEntry {
    version: String,
    path: Option<PathBuf>,
    implementation: String,
}

impl From<UvPythonEntry> for PythonInfo {
    fn from(entry: UvPythonEntry) -> Self {
        let installed = entry.path.is_some();
        Self {
            version: entry.version,
            path: entry.path,
            installed,
            implementation: entry.implementation,
        }
    }
}

/// Client for interacting with the uv CLI
pub struct UvClient {
    /// Path to the uv executable
    path: PathBuf,
}

impl UvClient {
    /// Create a new UvClient, finding uv in PATH
    pub fn new() -> Result<Self> {
        let path = which::which("uv").map_err(|_| ScoopError::UvNotFound)?;
        Ok(Self { path })
    }

    /// Create a new UvClient with a specific path
    pub fn with_path(path: PathBuf) -> Self {
        Self { path }
    }

    /// Get the uv version
    pub fn version(&self) -> Result<String> {
        let mut cmd = Command::new(&self.path);
        cmd.arg("--version");
        let stdout = run_uv(cmd, |message| ScoopError::UvCommandFailed {
            command: "uv --version".to_string(),
            message,
        })?;
        Ok(String::from_utf8_lossy(&stdout).trim().to_string())
    }

    /// Create a virtual environment
    pub fn create_venv(&self, path: &Path, python_version: &str) -> Result<()> {
        let mut cmd = Command::new(&self.path);
        cmd.arg("venv")
            .arg(path)
            .arg("--python")
            .arg(python_version);
        let display = format!("uv venv {} --python {}", path.display(), python_version);
        run_uv(cmd, |message| ScoopError::UvCommandFailed {
            command: display.clone(),
            message,
        })?;
        Ok(())
    }

    /// Install a Python version
    pub fn install_python(&self, version: &str) -> Result<()> {
        let mut cmd = Command::new(&self.path);
        cmd.arg("python").arg("install").arg(version);
        let display = format!("uv python install {version}");
        run_uv(cmd, |message| ScoopError::UvCommandFailed {
            command: display.clone(),
            message,
        })?;
        Ok(())
    }

    /// List all Python versions known to uv (installed and downloadable).
    pub fn list_pythons(&self) -> Result<Vec<PythonInfo>> {
        self.run_python_list(false)
    }

    /// List only the Python versions installed on this machine.
    pub fn list_installed_pythons(&self) -> Result<Vec<PythonInfo>> {
        self.run_python_list(true)
    }

    /// Run `uv python list --output-format=json` and parse the result.
    ///
    /// uv stabilized the JSON output in 0.5.14 (our [`MIN_VERSION`] floor), so
    /// we rely on the structured schema instead of scraping the human-readable
    /// table, which changes format between releases.
    ///
    /// [`MIN_VERSION`]: crate::uv::version::MIN_VERSION
    fn run_python_list(&self, only_installed: bool) -> Result<Vec<PythonInfo>> {
        let mut cmd = Command::new(&self.path);
        cmd.arg("python").arg("list").arg("--output-format=json");
        if only_installed {
            cmd.arg("--only-installed");
        }

        let display = if only_installed {
            "uv python list --only-installed --output-format=json"
        } else {
            "uv python list --output-format=json"
        };

        let stdout = run_uv(cmd, |message| ScoopError::UvCommandFailed {
            command: display.to_string(),
            message,
        })?;
        parse_python_list_json(&String::from_utf8_lossy(&stdout))
    }

    /// Prune the uv cache.
    ///
    /// Removes unused download archives, wheels, and source artifacts from
    /// `~/.cache/uv/` (or wherever uv stores its cache on this platform).
    /// Returns uv's stdout for surfacing to the user.
    ///
    /// # Errors
    ///
    /// Returns [`ScoopError::UvCommandFailed`] if `uv cache prune` exits non-zero.
    pub fn cache_prune(&self) -> Result<String> {
        let mut cmd = Command::new(&self.path);
        cmd.arg("cache").arg("prune");
        let stdout = run_uv(cmd, |message| ScoopError::UvCommandFailed {
            command: "uv cache prune".to_string(),
            message,
        })?;
        Ok(String::from_utf8_lossy(&stdout).into_owned())
    }

    /// Uninstall a Python version
    pub fn uninstall_python(&self, version: &str) -> Result<()> {
        let mut cmd = Command::new(&self.path);
        cmd.arg("python").arg("uninstall").arg(version);
        run_uv(cmd, |message| ScoopError::PythonUninstallFailed {
            version: version.to_string(),
            message,
        })?;
        Ok(())
    }

    /// Find an installed Python matching the version pattern
    pub fn find_python(&self, version_pattern: &str) -> Result<Option<PythonInfo>> {
        let installed = self.list_installed_pythons()?;

        if let Some(pattern) = PythonVersion::parse(version_pattern) {
            for info in installed {
                if let Some(ver) = PythonVersion::parse(&info.version) {
                    if pattern.matches(&ver) {
                        return Ok(Some(info));
                    }
                }
            }
        }

        Ok(None)
    }

    /// Install packages into a virtual environment.
    ///
    /// # Arguments
    ///
    /// * `venv_path` - Path to the virtual environment
    /// * `packages` - List of package specifications (e.g., "requests==2.31.0")
    ///
    /// # Errors
    ///
    /// Returns [`ScoopError::UvCommandFailed`] if installation fails.
    pub fn pip_install(&self, venv_path: &Path, packages: &[String]) -> Result<()> {
        if packages.is_empty() {
            return Ok(());
        }

        let mut cmd = Command::new(&self.path);
        cmd.arg("pip")
            .arg("install")
            .arg("--python")
            .arg(crate::paths::virtualenv_python_exe(venv_path));

        for package in packages {
            cmd.arg(package);
        }

        let display = format!("uv pip install (into {})", venv_path.display());
        run_uv(cmd, |message| ScoopError::UvCommandFailed {
            command: display.clone(),
            message,
        })?;
        Ok(())
    }

    /// Install packages from a requirements file into a virtual environment.
    ///
    /// # Errors
    ///
    /// Returns [`ScoopError::UvCommandFailed`] if installation fails.
    pub fn pip_install_requirements(
        &self,
        venv_path: &Path,
        requirements_path: &Path,
    ) -> Result<()> {
        let mut cmd = Command::new(&self.path);
        cmd.arg("pip")
            .arg("install")
            .arg("--python")
            .arg(crate::paths::virtualenv_python_exe(venv_path))
            .arg("-r")
            .arg(requirements_path);
        let display = format!("uv pip install -r {}", requirements_path.display());
        run_uv(cmd, |message| ScoopError::UvCommandFailed {
            command: display.clone(),
            message,
        })?;
        Ok(())
    }

    /// Get the latest installed Python version.
    pub fn latest_installed_python(&self) -> Result<Option<PythonInfo>> {
        Ok(pick_latest_python(self.list_installed_pythons()?))
    }

    /// List packages installed in `venv_path` via `uv pip list --format=json`.
    ///
    /// Used by `scoop diff` to enumerate packages on each side. Calls uv
    /// directly (rather than the venv's own pip) so the command works on
    /// envs whose pip is absent or broken — uv only needs the interpreter
    /// path to inspect a venv's `site-packages`.
    ///
    /// The parsing step lives in [`parse_pip_list_json`] so it is unit-
    /// testable with fixtures (no process spawn). This matches the split
    /// already used for [`parse_python_list_json`].
    ///
    /// # Errors
    ///
    /// - [`ScoopError::UvCommandFailed`] if `uv` exits non-zero (env
    ///   missing, interpreter broken, etc).
    /// - [`ScoopError::Json`] if uv's JSON output cannot be parsed.
    pub fn pip_list(&self, venv_path: &Path) -> Result<Vec<UvPipListEntry>> {
        let python = crate::paths::virtualenv_python_exe(venv_path);
        let mut cmd = Command::new(&self.path);
        cmd.arg("pip")
            .arg("list")
            .arg("--format=json")
            .arg("--python")
            .arg(&python);
        let display = format!("uv pip list --format=json --python {}", python.display());
        let stdout = run_uv(cmd, |message| ScoopError::UvCommandFailed {
            command: display.clone(),
            message,
        })?;
        parse_pip_list_json(&stdout)
    }
}

/// Run a built uv `Command`, returning captured stdout on success.
///
/// Centralizes the spawn + non-zero-exit handling every uv call repeats.
/// `make_err` builds the error from the failure message, letting each caller
/// pick its own variant (most use [`ScoopError::UvCommandFailed`]; uninstall
/// uses [`ScoopError::PythonUninstallFailed`]). It is invoked at most once.
fn run_uv(mut cmd: Command, make_err: impl Fn(String) -> ScoopError) -> Result<Vec<u8>> {
    let output = cmd.output().map_err(|e| make_err(e.to_string()))?;
    if !output.status.success() {
        return Err(make_err(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }
    Ok(output.stdout)
}

/// Pick the highest-versioned entry using [`PythonVersion`]'s full `Ord`
/// (major.minor.patch.suffix).
///
/// The previous hand-rolled comparator only compared major then minor, so
/// ties on minor (e.g. `3.12.1` vs `3.12.9`) were left in input order and
/// could return an older patch as "latest". Entries whose version string
/// doesn't parse are skipped; returns `None` if nothing parses.
fn pick_latest_python(pythons: Vec<PythonInfo>) -> Option<PythonInfo> {
    pythons
        .into_iter()
        .filter_map(|info| PythonVersion::parse(&info.version).map(|v| (v, info)))
        .max_by(|(a, _), (b, _)| a.cmp(b))
        .map(|(_, info)| info)
}

/// Parse `uv python list --output-format=json` stdout into structured info.
///
/// # Errors
///
/// Returns [`ScoopError::Json`] if the output is not the expected JSON array.
fn parse_python_list_json(stdout: &str) -> Result<Vec<PythonInfo>> {
    let entries: Vec<UvPythonEntry> = serde_json::from_str(stdout)?;
    Ok(entries.into_iter().map(PythonInfo::from).collect())
}

/// Parse `uv pip list --format=json` stdout into structured entries.
///
/// Extracted from [`UvClient::pip_list`] for testability — the parsing
/// logic is the part that benefits from fixture coverage, while the
/// process spawn is exercised by integration tests. uv emits additional
/// fields (`location`, `requires`, `requires_python`, ...) that serde
/// drops silently.
///
/// # Errors
///
/// Returns [`ScoopError::Json`] if the output is not the expected JSON array.
fn parse_pip_list_json(stdout: &[u8]) -> Result<Vec<UvPipListEntry>> {
    serde_json::from_slice(stdout).map_err(ScoopError::Json)
}

impl Default for UvClient {
    fn default() -> Self {
        Self::new().expect("uv not found in PATH")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uv_client_creation() {
        // This test will only pass if uv is installed
        if which::which("uv").is_ok() {
            let client = UvClient::new();
            assert!(client.is_ok());
        }
    }

    // Note: `UvClient::default()`'s `.expect("uv not found in PATH")` is the
    // only production panic, but it can only be triggered by emptying the
    // process-global `PATH`, which races with any concurrent test that resolves
    // uv. A `#[should_panic]` test for it was intentionally dropped — a flaky
    // test is worse than none, and the contract is obvious from the one-line
    // impl. See `.docs/dev/testing-strategy.md` (should_panic policy).

    #[test]
    fn test_parse_python_list_with_paths() {
        let json = r#"[
            {"key":"cpython-3.12.0-macos-aarch64-none","version":"3.12.0","version_parts":{"major":3,"minor":12,"patch":0},"path":"/Users/test/.local/share/uv/python/cpython-3.12.0/bin/python3","symlink":null,"url":null,"os":"macos","variant":"default","implementation":"cpython","arch":"aarch64","libc":"none"},
            {"key":"cpython-3.11.8-macos-aarch64-none","version":"3.11.8","version_parts":{"major":3,"minor":11,"patch":8},"path":"/Users/test/.local/share/uv/python/cpython-3.11.8/bin/python3","symlink":null,"url":null,"os":"macos","variant":"default","implementation":"cpython","arch":"aarch64","libc":"none"}
        ]"#;
        let pythons = parse_python_list_json(json).expect("valid json");
        assert_eq!(pythons.len(), 2);

        assert_eq!(pythons[0].version, "3.12.0");
        assert_eq!(pythons[0].implementation, "cpython");
        assert!(pythons[0].installed);
        assert!(pythons[0].path.is_some());

        assert_eq!(pythons[1].version, "3.11.8");
        assert_eq!(pythons[1].implementation, "cpython");
    }

    #[test]
    fn test_parse_python_list_without_paths() {
        let json = r#"[
            {"key":"cpython-3.13.0-macos-aarch64-none","version":"3.13.0","path":null,"implementation":"cpython"},
            {"key":"cpython-3.12.0-macos-aarch64-none","version":"3.12.0","path":null,"implementation":"cpython"}
        ]"#;
        let pythons = parse_python_list_json(json).expect("valid json");
        assert_eq!(pythons.len(), 2);

        assert_eq!(pythons[0].version, "3.13.0");
        assert!(!pythons[0].installed);
        assert!(pythons[0].path.is_none());
    }

    #[test]
    fn test_parse_python_list_mixed() {
        let json = r#"[
            {"version":"3.12.0","path":"/path/to/python3","implementation":"cpython"},
            {"version":"3.11.0","path":null,"implementation":"cpython"},
            {"version":"3.10.0","path":"/path/to/pypy","implementation":"pypy"}
        ]"#;
        let pythons = parse_python_list_json(json).expect("valid json");
        assert_eq!(pythons.len(), 3);

        assert_eq!(pythons[0].implementation, "cpython");
        assert!(pythons[0].installed);

        assert_eq!(pythons[1].implementation, "cpython");
        assert!(!pythons[1].installed);

        assert_eq!(pythons[2].implementation, "pypy");
        assert!(pythons[2].installed);
    }

    #[test]
    fn test_parse_python_list_empty() {
        let pythons = parse_python_list_json("[]").expect("valid json");
        assert!(pythons.is_empty());
    }

    #[test]
    fn test_python_info_equality() {
        let info1 = PythonInfo {
            version: "3.12.0".to_string(),
            path: Some(PathBuf::from("/path/to/python")),
            installed: true,
            implementation: "cpython".to_string(),
        };

        let info2 = PythonInfo {
            version: "3.12.0".to_string(),
            path: Some(PathBuf::from("/path/to/python")),
            installed: true,
            implementation: "cpython".to_string(),
        };

        assert_eq!(info1, info2);
    }

    #[test]
    fn test_uv_client_with_path() {
        let client = UvClient::with_path(PathBuf::from("/usr/bin/uv"));
        assert_eq!(client.path, PathBuf::from("/usr/bin/uv"));
    }

    // =========================================================================
    // parse_python_list_json Security & Edge Case Tests
    // =========================================================================

    /// Malformed JSON surfaces as an error rather than silently parsing nothing.
    #[test]
    fn test_parse_python_list_json_malformed_is_error() {
        assert!(parse_python_list_json("{ not json }").is_err());
    }

    /// A JSON object (not the expected array) is rejected.
    #[test]
    fn test_parse_python_list_json_non_array_is_error() {
        assert!(parse_python_list_json(r#"{"version":"3.12.0"}"#).is_err());
    }

    /// Path is stored as-is - validation happens elsewhere, not in the parser.
    #[test]
    fn test_parse_python_list_path_traversal_attempt() {
        let json =
            r#"[{"version":"3.12.0","path":"../../../etc/passwd","implementation":"cpython"}]"#;
        let pythons = parse_python_list_json(json).expect("valid json");

        assert_eq!(pythons.len(), 1);
        assert_eq!(pythons[0].version, "3.12.0");
        assert_eq!(pythons[0].path, Some(PathBuf::from("../../../etc/passwd")));
    }

    /// Unicode in paths - should round-trip intact.
    #[test]
    fn test_parse_python_list_unicode_path() {
        let json =
            r#"[{"version":"3.12.0","path":"/Users/한글/python","implementation":"cpython"}]"#;
        let pythons = parse_python_list_json(json).expect("valid json");

        assert_eq!(pythons.len(), 1);
        assert_eq!(pythons[0].path, Some(PathBuf::from("/Users/한글/python")));
    }

    /// A missing `path` key deserializes to None (not installed).
    #[test]
    fn test_parse_python_list_missing_path_is_not_installed() {
        let json = r#"[{"version":"3.12.0","implementation":"cpython"}]"#;
        let pythons = parse_python_list_json(json).expect("valid json");

        assert_eq!(pythons.len(), 1);
        assert!(!pythons[0].installed);
        assert!(pythons[0].path.is_none());
    }

    /// Pre-release versions (e.g. 3.15.0b1) are preserved exactly as uv reports.
    #[test]
    fn test_parse_python_list_prerelease_version() {
        let json = r#"[{"version":"3.15.0b1","path":null,"implementation":"cpython"}]"#;
        let pythons = parse_python_list_json(json).expect("valid json");

        assert_eq!(pythons.len(), 1);
        assert_eq!(pythons[0].version, "3.15.0b1");
        assert!(!pythons[0].installed);
    }

    /// Different Python implementations are preserved.
    #[test]
    fn test_parse_python_list_various_implementations() {
        let json = r#"[
            {"version":"3.12.0","path":"/path/cpython","implementation":"cpython"},
            {"version":"3.10.0","path":"/path/pypy","implementation":"pypy"},
            {"version":"3.11.0","path":"/path/graalpy","implementation":"graalpy"}
        ]"#;
        let pythons = parse_python_list_json(json).expect("valid json");

        assert_eq!(pythons.len(), 3);
        assert_eq!(pythons[0].implementation, "cpython");
        assert_eq!(pythons[1].implementation, "pypy");
        assert_eq!(pythons[2].implementation, "graalpy");
    }

    // =========================================================================
    // PythonInfo Tests
    // =========================================================================

    /// PythonInfo with None path
    #[test]
    fn test_python_info_none_path() {
        let info = PythonInfo {
            version: "3.12.0".to_string(),
            path: None,
            installed: false,
            implementation: "cpython".to_string(),
        };

        assert!(info.path.is_none());
        assert!(!info.installed);
    }

    // =========================================================================
    // pick_latest_python Tests
    // =========================================================================

    fn py_info(version: &str) -> PythonInfo {
        PythonInfo {
            version: version.to_string(),
            path: None,
            installed: true,
            implementation: "cpython".to_string(),
        }
    }

    /// Regression: ties on minor must compare patch (was returning input order).
    #[test]
    fn pick_latest_python_picks_highest_patch() {
        let got = pick_latest_python(vec![
            py_info("3.12.1"),
            py_info("3.12.9"),
            py_info("3.12.3"),
        ]);
        assert_eq!(got.unwrap().version, "3.12.9");
    }

    #[test]
    fn pick_latest_python_compares_major_then_minor() {
        let got = pick_latest_python(vec![
            py_info("3.9.18"),
            py_info("3.13.0"),
            py_info("3.12.12"),
        ]);
        assert_eq!(got.unwrap().version, "3.13.0");
    }

    #[test]
    fn pick_latest_python_skips_unparseable() {
        let got = pick_latest_python(vec![py_info("garbage"), py_info("3.10.1")]);
        assert_eq!(got.unwrap().version, "3.10.1");
    }

    #[test]
    fn pick_latest_python_empty_is_none() {
        assert!(pick_latest_python(vec![]).is_none());
    }

    // ====== parse_pip_list_json fixtures ======
    // The pip_list flow is tested without spawning uv: parse_pip_list_json
    // takes raw bytes, so a fixture verifies every shape contract we care
    // about (entry shape, editable installs, empty list, extra fields,
    // invalid input). The live spawn is covered by integration tests of
    // `scoop diff` against a real env.

    #[test]
    fn parse_pip_list_canonical_entry() {
        let json = br#"[{"name":"requests","version":"2.31.0"}]"#;
        let entries = parse_pip_list_json(json).expect("valid json");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "requests");
        assert_eq!(entries[0].version, "2.31.0");
        assert!(entries[0].editable_project_location.is_none());
    }

    #[test]
    fn parse_pip_list_editable_entry() {
        let json = br#"[{"name":"myproj","version":"0.1.0","editable_project_location":"/home/u/myproj"}]"#;
        let entries = parse_pip_list_json(json).expect("valid json");
        assert_eq!(entries.len(), 1);
        assert_eq!(
            entries[0].editable_project_location.as_deref(),
            Some(std::path::Path::new("/home/u/myproj"))
        );
    }

    #[test]
    fn parse_pip_list_empty_array() {
        let entries = parse_pip_list_json(b"[]").expect("empty is valid");
        assert!(entries.is_empty());
    }

    #[test]
    fn parse_pip_list_ignores_extra_fields() {
        // uv emits more fields (location, requires, requires_python, ...).
        // serde drops them silently — same contract as UvPythonEntry.
        let json = br#"[{"name":"x","version":"1.0","location":"/y","requires":[],"requires_python":">=3.8"}]"#;
        let entries = parse_pip_list_json(json).expect("should parse with extras");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "x");
    }

    #[test]
    fn parse_pip_list_rejects_invalid_json() {
        assert!(parse_pip_list_json(b"not json").is_err());
        // Also reject non-array shapes — uv only ever emits arrays here.
        assert!(parse_pip_list_json(br#"{"name":"x"}"#).is_err());
    }
}
