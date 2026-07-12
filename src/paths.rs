//! Path utilities for scoop

use std::path::{Path, PathBuf};

use once_cell::sync::Lazy;
use regex::Regex;
use rust_i18n::t;

use crate::error::{Result, ScoopError};

/// Environment variable for the scuv home directory.
pub const SCUV_HOME_ENV: &str = "SCUV_HOME";
/// DEPRECATION(0.16.0): remove legacy env fallback.
pub const LEGACY_HOME_ENV: &str = "SCOOP_HOME";

/// Default scuv home directory name.
const SCUV_HOME_DIR: &str = ".scuv";
/// DEPRECATION(0.16.0): remove legacy dir fallback.
const LEGACY_HOME_DIR: &str = ".scoop";

/// Version file name.
/// TASK-3 FLIPS THIS to ".scuv-version" together with the dual-walk read logic.
pub const VERSION_FILE: &str = ".scoop-version";
/// DEPRECATION(0.16.0): remove legacy version-file fallback.
pub const LEGACY_VERSION_FILE: &str = ".scoop-version";

/// Get the scuv home directory.
///
/// Resolution order: `$SCUV_HOME` > legacy `$SCOOP_HOME` > `~/.scuv` >
/// legacy `~/.scoop` (only when `~/.scuv` doesn't exist yet). Reading either
/// legacy fallback emits a one-shot deprecation warning on stderr.
pub fn scoop_home() -> Result<PathBuf> {
    if let Ok(home) = std::env::var(SCUV_HOME_ENV) {
        return Ok(PathBuf::from(home));
    }

    // DEPRECATION(0.16.0): remove legacy env fallback.
    if let Ok(home) = std::env::var(LEGACY_HOME_ENV) {
        crate::output::deprecation::warn_once(&t!(
            "deprecation.env_var",
            old = LEGACY_HOME_ENV,
            new = SCUV_HOME_ENV
        ));
        return Ok(PathBuf::from(home));
    }

    let base = dirs::home_dir().ok_or(ScoopError::HomeNotFound)?;
    let new = base.join(SCUV_HOME_DIR);
    // DEPRECATION(0.16.0): remove legacy dir fallback.
    let legacy = base.join(LEGACY_HOME_DIR);
    if !new.exists() && legacy.exists() {
        crate::output::deprecation::warn_once(&t!("deprecation.home_dir"));
        return Ok(legacy);
    }
    Ok(new)
}

/// Get the virtualenvs directory (~/.scuv/virtualenvs)
pub fn virtualenvs_dir() -> Result<PathBuf> {
    Ok(scoop_home()?.join("virtualenvs"))
}

/// Get the pythons directory (~/.scuv/pythons)
pub fn pythons_dir() -> Result<PathBuf> {
    Ok(scoop_home()?.join("pythons"))
}

/// Get the global version file path (~/.scuv/version)
pub fn global_version_file() -> Result<PathBuf> {
    Ok(scoop_home()?.join("version"))
}

/// Get the local version file path in the given directory
pub fn local_version_file(dir: &std::path::Path) -> PathBuf {
    dir.join(VERSION_FILE)
}

/// Get the path to a specific virtualenv
pub fn virtualenv_path(name: &str) -> Result<PathBuf> {
    Ok(virtualenvs_dir()?.join(name))
}

/// Get the bin directory of a virtualenv (`name`-based wrapper).
///
/// Thin name-resolving wrapper over [`virtualenv_bin_dir`]; existing
/// callers continue to pass the env name.
pub fn virtualenv_bin(name: &str) -> Result<PathBuf> {
    Ok(virtualenv_bin_dir(&virtualenv_path(name)?))
}

/// Get the python executable in a virtualenv (`name`-based wrapper).
pub fn virtualenv_python(name: &str) -> Result<PathBuf> {
    Ok(virtualenv_python_exe(&virtualenv_path(name)?))
}

/// Returns the bin/Scripts directory of a virtualenv given its root path.
///
/// On Unix this is `<root>/bin`; on Windows `<root>/Scripts`. Use this
/// when you already hold a resolved venv root (`&Path`) and want to
/// avoid going through [`virtualenv_bin`]'s name → path lookup.
#[cfg(unix)]
pub fn virtualenv_bin_dir(venv_root: &Path) -> PathBuf {
    venv_root.join("bin")
}

/// Returns the Scripts directory of a virtualenv on Windows.
#[cfg(windows)]
pub fn virtualenv_bin_dir(venv_root: &Path) -> PathBuf {
    venv_root.join("Scripts")
}

/// Returns the python executable path inside a virtualenv root.
///
/// On Unix this is `<root>/bin/python`; on Windows
/// `<root>/Scripts/python.exe`.
#[cfg(unix)]
pub fn virtualenv_python_exe(venv_root: &Path) -> PathBuf {
    virtualenv_bin_dir(venv_root).join("python")
}

#[cfg(windows)]
pub fn virtualenv_python_exe(venv_root: &Path) -> PathBuf {
    virtualenv_bin_dir(venv_root).join("python.exe")
}

/// Returns the pip executable path inside a virtualenv root.
///
/// On Unix this is `<root>/bin/pip`; on Windows `<root>/Scripts/pip.exe`.
#[cfg(unix)]
pub fn virtualenv_pip_exe(venv_root: &Path) -> PathBuf {
    virtualenv_bin_dir(venv_root).join("pip")
}

#[cfg(windows)]
pub fn virtualenv_pip_exe(venv_root: &Path) -> PathBuf {
    virtualenv_bin_dir(venv_root).join("pip.exe")
}

/// Returns the activation script path inside a virtualenv root.
///
/// On Unix this is `<root>/bin/activate` (POSIX shell). On Windows it is
/// `<root>/Scripts/Activate.ps1` (PowerShell).
///
/// Distinct from [`virtualenv_activate`], which returns the cmd.exe
/// `.bat` variant on Windows. PowerShell users are surfaced through
/// this helper (used by `scoop verify`).
#[cfg(unix)]
pub fn virtualenv_activate_script(venv_root: &Path) -> PathBuf {
    virtualenv_bin_dir(venv_root).join("activate")
}

#[cfg(windows)]
pub fn virtualenv_activate_script(venv_root: &Path) -> PathBuf {
    virtualenv_bin_dir(venv_root).join("Activate.ps1")
}

/// Strict glob pattern for Python lib directories inside a venv.
///
/// Requires `MAJOR.MINOR` (e.g. `python3.12`) — `python312` (no dot) is
/// rejected because uv never produces that form, and matching it would
/// risk false-positives on user-created directories like `pythonpath`.
/// The optional `t` suffix accepts free-threaded Python (`python3.13t`).
static PYTHON_LIB_DIR_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^python\d+\.\d+t?$").expect("PYTHON_LIB_DIR_RE is a valid pattern"));

/// Returns the `site-packages` directory for a virtualenv root.
///
/// # Resolution order (Unix)
///
/// 1. `pyvenv.cfg` `version_info` → `lib/python<MAJOR.MINOR>/site-packages`
/// 2. Glob `lib/` for a single `python<MAJOR.MINOR>[t]/site-packages` dir
/// 3. Subprocess fallback: `python -c "import sysconfig; print(sysconfig.get_path('purelib'))"`
///
/// On Windows the path is always `<root>/Lib/site-packages` (no Python
/// version in the path).
///
/// # Errors
///
/// Returns [`ScoopError::SitePackagesNotFound`] when every strategy
/// fails (malformed/missing `pyvenv.cfg`, ambiguous or absent glob
/// matches, and either no python executable or sysconfig produced no
/// usable path). The Err carries the venv path so callers can surface
/// it to the user.
#[cfg(windows)]
pub fn virtualenv_site_packages(venv_root: &Path) -> Result<PathBuf> {
    let candidate = venv_root.join("Lib").join("site-packages");
    if candidate.is_dir() {
        Ok(candidate)
    } else {
        Err(ScoopError::SitePackagesNotFound {
            venv: venv_root.display().to_string(),
        })
    }
}

#[cfg(unix)]
pub fn virtualenv_site_packages(venv_root: &Path) -> Result<PathBuf> {
    // Strategy 1: pyvenv.cfg version → lib/python{MAJOR.MINOR}/site-packages.
    // parse_pyvenv_version normalizes "3.14.3.final.0" to "3.14.3".
    if let Some(ver) = crate::core::parse_pyvenv_version(venv_root) {
        let major_minor: String = ver.splitn(3, '.').take(2).collect::<Vec<_>>().join(".");
        let candidate = venv_root
            .join("lib")
            .join(format!("python{major_minor}"))
            .join("site-packages");
        if candidate.is_dir() {
            return Ok(candidate);
        }
    }

    // Strategy 2: glob lib/ for a single python* directory matching the
    // strict regex. Multiple matches → ambiguity; fall through to sysconfig
    // rather than guessing.
    let lib_dir = venv_root.join("lib");
    if lib_dir.is_dir() {
        let mut matches: Vec<PathBuf> = std::fs::read_dir(&lib_dir)
            .into_iter()
            .flatten()
            .flatten()
            .filter_map(|entry| {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                if entry.path().is_dir() && PYTHON_LIB_DIR_RE.is_match(&name_str) {
                    let sp = entry.path().join("site-packages");
                    if sp.is_dir() { Some(sp) } else { None }
                } else {
                    None
                }
            })
            .collect();
        if matches.len() == 1 {
            return Ok(matches.remove(0));
        }
    }

    // Strategy 3: sysconfig subprocess fallback. Pure-python and platform
    // libs resolve to the same dir inside a venv; `purelib` is the
    // canonical site-packages.
    let python_exe = virtualenv_python_exe(venv_root);
    if python_exe.is_file() {
        let output = std::process::Command::new(&python_exe)
            .args([
                "-c",
                "import sysconfig; print(sysconfig.get_path('purelib'))",
            ])
            .output();
        if let Ok(out) = output {
            if out.status.success() {
                let line = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if !line.is_empty() {
                    let path = PathBuf::from(line);
                    if path.is_dir() {
                        return Ok(path);
                    }
                }
            }
        }
    }

    Err(ScoopError::SitePackagesNotFound {
        venv: venv_root.display().to_string(),
    })
}

/// Ensure all scoop directories exist
///
/// Creates the following directory structure:
/// - ~/.scuv/
/// - ~/.scuv/virtualenvs/
/// - ~/.scuv/pythons/
pub fn ensure_scoop_dirs() -> Result<()> {
    let home = scoop_home()?;
    std::fs::create_dir_all(&home)?;
    std::fs::create_dir_all(home.join("virtualenvs"))?;
    std::fs::create_dir_all(home.join("pythons"))?;
    Ok(())
}

/// Check if a virtualenv exists
pub fn virtualenv_exists(name: &str) -> Result<bool> {
    let path = virtualenv_path(name)?;
    Ok(path.exists() && path.is_dir())
}

/// Get the activate script path for a virtualenv
#[cfg(unix)]
pub fn virtualenv_activate(name: &str) -> Result<PathBuf> {
    Ok(virtualenv_bin(name)?.join("activate"))
}

/// Get the activate script path for a virtualenv (Windows)
#[cfg(windows)]
pub fn virtualenv_activate(name: &str) -> Result<PathBuf> {
    Ok(virtualenv_path(name)?.join("Scripts").join("activate.bat"))
}

/// Calculate directory size recursively
///
/// Symlinks are skipped to prevent infinite loops.
///
/// # Errors
///
/// Returns `std::io::Error` if:
/// - Directory cannot be read (permission denied)
/// - File metadata cannot be accessed
///
/// # Examples
///
/// ```no_run
/// use std::path::Path;
/// use scoop_uv::paths::calculate_dir_size;
///
/// let size = calculate_dir_size(Path::new("/tmp/mydir"))?;
/// println!("Directory size: {} bytes", size);
/// # Ok::<(), std::io::Error>(())
/// ```
pub fn calculate_dir_size(path: &std::path::Path) -> std::io::Result<u64> {
    let mut total: u64 = 0;
    // Skip symlinks to prevent infinite loops
    if path.is_dir() && !path.is_symlink() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path(); // Fixed: avoid variable shadowing
            // Skip symlinks in size calculation
            if entry_path.is_symlink() {
                continue;
            }
            if entry_path.is_dir() {
                total += calculate_dir_size(&entry_path)?;
            } else {
                total += entry.metadata()?.len();
            }
        }
    }
    Ok(total)
}

/// Locate `exe` inside `dir`, returning the full path if a matching file
/// exists. On Windows the standard executable extensions are probed in turn.
///
/// Shared by `scoop which` (display the resolved path) and `scoop run`
/// (preflight a program lookup against the env's `bin/` before spawning).
///
/// # Examples
///
/// ```
/// # use std::path::PathBuf;
/// use scoop_uv::paths::find_executable_in;
///
/// let dir = tempfile::tempdir().unwrap();
/// std::fs::write(dir.path().join("python"), b"").unwrap();
/// assert_eq!(
///     find_executable_in(dir.path(), "python"),
///     Some(dir.path().join("python")),
/// );
/// assert!(find_executable_in(dir.path(), "missing").is_none());
/// ```
pub fn find_executable_in(dir: &std::path::Path, exe: &str) -> Option<PathBuf> {
    executable_candidates(exe)
        .into_iter()
        .map(|name| dir.join(name))
        .find(|p| p.is_file())
}

#[cfg(windows)]
fn executable_candidates(exe: &str) -> Vec<String> {
    if exe.contains('.') {
        vec![exe.to_string()]
    } else {
        vec![
            exe.to_string(),
            format!("{exe}.exe"),
            format!("{exe}.bat"),
            format!("{exe}.cmd"),
        ]
    }
}

#[cfg(not(windows))]
fn executable_candidates(exe: &str) -> Vec<String> {
    vec![exe.to_string()]
}

/// Abbreviate home directory to `~` for display.
///
/// # Examples
///
/// ```
/// use std::path::Path;
/// use scoop_uv::paths::abbreviate_home;
///
/// // Home directory paths get abbreviated
/// let home = dirs::home_dir().unwrap();
/// let path = home.join(".scoop/virtualenvs/myenv");
/// let abbreviated = abbreviate_home(&path);
/// assert!(abbreviated.starts_with("~/"));
/// ```
pub fn abbreviate_home(path: &std::path::Path) -> String {
    if let Some(home) = dirs::home_dir() {
        if let Ok(stripped) = path.strip_prefix(&home) {
            return format!("~/{}", stripped.display());
        }
    }
    path.display().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{with_no_scoop_home, with_temp_scoop_home};
    use serial_test::serial;

    #[test]
    fn test_scoop_home_default() {
        with_no_scoop_home(|| {
            let home = scoop_home().unwrap();
            // .scoop only when a legacy dir already exists on the machine.
            assert!(home.ends_with(".scuv") || home.ends_with(".scoop"));
        });
    }

    #[test]
    #[serial]
    fn scuv_home_env_wins_over_legacy() {
        let _g = crate::test_utils::env_guard(&[
            (SCUV_HOME_ENV, Some("/tmp/newhome")),
            (LEGACY_HOME_ENV, Some("/tmp/oldhome")),
        ]);
        assert_eq!(scoop_home().unwrap(), PathBuf::from("/tmp/newhome"));
    }

    #[test]
    #[serial]
    fn legacy_home_env_still_read() {
        let _g = crate::test_utils::env_guard(&[
            (SCUV_HOME_ENV, None),
            (LEGACY_HOME_ENV, Some("/tmp/oldhome")),
        ]);
        assert_eq!(scoop_home().unwrap(), PathBuf::from("/tmp/oldhome"));
    }

    #[test]
    #[serial]
    fn default_home_is_dot_scuv() {
        let _g = crate::test_utils::env_guard(&[(SCUV_HOME_ENV, None), (LEGACY_HOME_ENV, None)]);
        let home = scoop_home().unwrap();
        assert!(home.ends_with(".scuv") || home.ends_with(".scoop")); // .scoop only when legacy dir exists on the machine
    }

    #[test]
    fn test_scoop_home_env() {
        with_temp_scoop_home(|temp_dir| {
            let home = scoop_home().unwrap();
            assert_eq!(home, temp_dir.path());
        });
    }

    #[test]
    #[serial]
    fn test_virtualenvs_dir() {
        with_temp_scoop_home(|temp_dir| {
            let venvs = virtualenvs_dir().unwrap();
            assert_eq!(venvs, temp_dir.path().join("virtualenvs"));
        });
    }

    #[test]
    #[serial]
    fn test_pythons_dir() {
        with_temp_scoop_home(|temp_dir| {
            let pythons = pythons_dir().unwrap();
            assert_eq!(pythons, temp_dir.path().join("pythons"));
        });
    }

    #[test]
    #[serial]
    fn test_virtualenv_path() {
        with_temp_scoop_home(|temp_dir| {
            let path = virtualenv_path("myenv").unwrap();
            assert_eq!(path, temp_dir.path().join("virtualenvs").join("myenv"));
        });
    }

    #[test]
    #[serial]
    fn test_virtualenv_bin() {
        with_temp_scoop_home(|temp_dir| {
            let bin = virtualenv_bin("myenv").unwrap();
            assert_eq!(
                bin,
                temp_dir
                    .path()
                    .join("virtualenvs")
                    .join("myenv")
                    .join("bin")
            );
        });
    }

    #[test]
    #[serial]
    fn test_virtualenv_python() {
        with_temp_scoop_home(|temp_dir| {
            let python = virtualenv_python("myenv").unwrap();
            assert_eq!(
                python,
                temp_dir
                    .path()
                    .join("virtualenvs")
                    .join("myenv")
                    .join("bin")
                    .join("python")
            );
        });
    }

    #[test]
    #[serial]
    fn test_ensure_scoop_dirs() {
        with_temp_scoop_home(|temp_dir| {
            ensure_scoop_dirs().unwrap();
            assert!(temp_dir.path().exists());
            assert!(temp_dir.path().join("virtualenvs").exists());
            assert!(temp_dir.path().join("pythons").exists());
        });
    }

    #[test]
    #[serial]
    fn test_virtualenv_exists() {
        with_temp_scoop_home(|temp_dir| {
            // Create the virtualenvs directory
            let venv_path = temp_dir.path().join("virtualenvs").join("existing");
            std::fs::create_dir_all(&venv_path).unwrap();

            assert!(virtualenv_exists("existing").unwrap());
            assert!(!virtualenv_exists("nonexistent").unwrap());
        });
    }

    #[test]
    fn test_local_version_file() {
        // This test doesn't use environment variables
        let dir = PathBuf::from("/some/project");
        let version_file = local_version_file(&dir);
        assert_eq!(version_file, dir.join(".scoop-version"));
    }

    #[test]
    #[serial]
    fn test_global_version_file() {
        with_temp_scoop_home(|temp_dir| {
            let version_file = global_version_file().unwrap();
            assert_eq!(version_file, temp_dir.path().join("version"));
        });
    }

    #[cfg(unix)]
    #[test]
    #[serial]
    fn test_virtualenv_activate_unix() {
        with_temp_scoop_home(|temp_dir| {
            let activate = virtualenv_activate("myenv").unwrap();
            assert_eq!(
                activate,
                temp_dir
                    .path()
                    .join("virtualenvs")
                    .join("myenv")
                    .join("bin")
                    .join("activate")
            );
        });
    }

    // ==========================================================================
    // Symlink and Edge Case Tests
    // ==========================================================================

    #[cfg(unix)]
    #[test]
    #[serial]
    fn test_virtualenv_exists_with_symlink() {
        with_temp_scoop_home(|temp_dir| {
            use std::os::unix::fs::symlink;

            // Create a real directory
            let real_dir = temp_dir.path().join("real_venv");
            std::fs::create_dir_all(&real_dir).unwrap();

            // Create virtualenvs directory and symlink
            let venvs_dir = temp_dir.path().join("virtualenvs");
            std::fs::create_dir_all(&venvs_dir).unwrap();
            let symlink_path = venvs_dir.join("symlinked");
            symlink(&real_dir, &symlink_path).unwrap();

            // Symlinked virtualenv should be detected as existing
            assert!(virtualenv_exists("symlinked").unwrap());
        });
    }

    #[cfg(unix)]
    #[test]
    #[serial]
    fn test_virtualenv_exists_with_broken_symlink() {
        with_temp_scoop_home(|temp_dir| {
            use std::os::unix::fs::symlink;

            // Create virtualenvs directory
            let venvs_dir = temp_dir.path().join("virtualenvs");
            std::fs::create_dir_all(&venvs_dir).unwrap();

            // Create a symlink to non-existent target
            let broken_symlink = venvs_dir.join("broken");
            symlink("/nonexistent/path", &broken_symlink).unwrap();

            // Broken symlink should NOT be detected as existing directory
            assert!(!virtualenv_exists("broken").unwrap());
        });
    }

    #[cfg(unix)]
    #[test]
    #[serial]
    fn test_virtualenv_exists_symlink_to_file() {
        with_temp_scoop_home(|temp_dir| {
            use std::os::unix::fs::symlink;

            // Create a regular file
            let file_path = temp_dir.path().join("regular_file");
            std::fs::write(&file_path, "test").unwrap();

            // Create virtualenvs directory and symlink to file
            let venvs_dir = temp_dir.path().join("virtualenvs");
            std::fs::create_dir_all(&venvs_dir).unwrap();
            let symlink_path = venvs_dir.join("filelink");
            symlink(&file_path, &symlink_path).unwrap();

            // Symlink to file should NOT be detected as existing (needs to be directory)
            assert!(!virtualenv_exists("filelink").unwrap());
        });
    }

    #[test]
    #[serial]
    fn test_path_with_special_characters() {
        with_temp_scoop_home(|temp_dir| {
            // Environment names with allowed special characters
            let venvs_dir = temp_dir.path().join("virtualenvs");
            std::fs::create_dir_all(venvs_dir.join("my-env")).unwrap();
            std::fs::create_dir_all(venvs_dir.join("my_env")).unwrap();
            std::fs::create_dir_all(venvs_dir.join("env123")).unwrap();

            assert!(virtualenv_exists("my-env").unwrap());
            assert!(virtualenv_exists("my_env").unwrap());
            assert!(virtualenv_exists("env123").unwrap());
        });
    }

    // ==========================================================================
    // calculate_dir_size Tests
    // ==========================================================================

    #[test]
    fn test_calculate_dir_size_empty_dir() {
        let dir = tempfile::tempdir().unwrap();
        let size = calculate_dir_size(dir.path()).unwrap();
        assert_eq!(size, 0);
    }

    #[test]
    fn test_calculate_dir_size_with_file() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        std::fs::write(&file_path, b"hello").unwrap();

        let size = calculate_dir_size(dir.path()).unwrap();
        assert_eq!(size, 5);
    }

    #[test]
    fn test_calculate_dir_size_nested_dirs() {
        let dir = tempfile::tempdir().unwrap();
        let subdir = dir.path().join("subdir");
        std::fs::create_dir(&subdir).unwrap();

        std::fs::write(subdir.join("test.txt"), b"hello world").unwrap();

        let size = calculate_dir_size(dir.path()).unwrap();
        assert_eq!(size, 11);
    }

    #[test]
    fn test_calculate_dir_size_nonexistent() {
        // is_dir() returns false for nonexistent, so returns 0
        let result = calculate_dir_size(std::path::Path::new("/nonexistent/path"));
        assert_eq!(result.unwrap(), 0);
    }

    #[cfg(unix)]
    #[test]
    fn test_calculate_dir_size_skips_symlinks() {
        use std::os::unix::fs::symlink;

        let dir = tempfile::tempdir().unwrap();

        // Create a file
        std::fs::write(dir.path().join("file.txt"), b"test").unwrap();

        // Create a symlink to the file (should be skipped)
        symlink(dir.path().join("file.txt"), dir.path().join("link")).unwrap();

        // Size should only include the file, not the symlink
        let size = calculate_dir_size(dir.path()).unwrap();
        assert_eq!(size, 4); // Only "test" (4 bytes)
    }

    #[cfg(unix)]
    #[test]
    fn test_calculate_dir_size_circular_symlink() {
        use std::os::unix::fs::symlink;

        let dir = tempfile::tempdir().unwrap();
        let subdir = dir.path().join("sub");
        std::fs::create_dir(&subdir).unwrap();

        // Create circular symlink: sub/loop -> ..
        symlink(dir.path(), subdir.join("loop")).unwrap();

        // Should not hang or overflow - symlinks are skipped
        let result = calculate_dir_size(dir.path());
        assert!(result.is_ok());
    }

    // ==========================================================================
    // abbreviate_home Tests
    // ==========================================================================

    #[test]
    fn test_abbreviate_home_with_home_path() {
        // Path under home directory should be abbreviated
        if let Some(home) = dirs::home_dir() {
            let path = home.join(".scoop").join("virtualenvs").join("myenv");
            let result = abbreviate_home(&path);
            assert!(result.starts_with("~/"));
            assert!(result.contains(".scoop/virtualenvs/myenv"));
        }
    }

    #[test]
    fn test_abbreviate_home_outside_home() {
        // Path outside home directory should remain unchanged
        let path = PathBuf::from("/tmp/some/path");
        let result = abbreviate_home(&path);
        assert_eq!(result, "/tmp/some/path");
    }

    #[test]
    fn test_abbreviate_home_root_path() {
        // Root path should remain unchanged
        let path = PathBuf::from("/");
        let result = abbreviate_home(&path);
        assert_eq!(result, "/");
    }

    // ==========================================================================
    // find_executable_in / executable_candidates Tests
    // ==========================================================================

    #[test]
    fn find_executable_in_locates_existing_file() {
        let dir = tempfile::tempdir().unwrap();
        let exe = dir.path().join("python");
        std::fs::write(&exe, b"").unwrap();
        assert_eq!(find_executable_in(dir.path(), "python"), Some(exe));
    }

    #[test]
    fn find_executable_in_returns_none_for_missing() {
        let dir = tempfile::tempdir().unwrap();
        assert!(find_executable_in(dir.path(), "python").is_none());
    }

    #[test]
    fn find_executable_in_ignores_directories() {
        // A subdirectory whose name matches the executable must not satisfy
        // the lookup — we want files only.
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join("python")).unwrap();
        assert!(find_executable_in(dir.path(), "python").is_none());
    }

    #[test]
    fn find_executable_in_returns_none_when_dir_missing() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("does-not-exist");
        assert!(find_executable_in(&missing, "python").is_none());
    }

    #[cfg(not(windows))]
    #[test]
    fn executable_candidates_unix_is_single_entry() {
        assert_eq!(executable_candidates("python"), vec!["python".to_string()]);
    }

    #[cfg(windows)]
    #[test]
    fn executable_candidates_windows_probes_standard_extensions() {
        let candidates = executable_candidates("python");
        assert!(candidates.contains(&"python".to_string()));
        assert!(candidates.contains(&"python.exe".to_string()));
        assert!(candidates.contains(&"python.bat".to_string()));
        assert!(candidates.contains(&"python.cmd".to_string()));
    }

    #[cfg(windows)]
    #[test]
    fn executable_candidates_windows_respects_explicit_extension() {
        assert_eq!(executable_candidates("python.exe"), vec!["python.exe"]);
    }

    // ==========================================================================
    // Cross-platform venv path helpers
    // ==========================================================================

    #[cfg(unix)]
    mod cross_platform_unix {
        use super::*;
        use std::fs;
        use tempfile::TempDir;

        fn make_venv_root(pyvenv_cfg: Option<&str>, lib_dirs: &[&str]) -> TempDir {
            let tmp = TempDir::new().unwrap();
            if let Some(cfg) = pyvenv_cfg {
                fs::write(tmp.path().join("pyvenv.cfg"), cfg).unwrap();
            }
            for d in lib_dirs {
                fs::create_dir_all(tmp.path().join("lib").join(d).join("site-packages")).unwrap();
            }
            tmp
        }

        #[test]
        fn virtualenv_bin_dir_returns_bin_on_unix() {
            let root = std::path::PathBuf::from("/tmp/fake");
            assert_eq!(virtualenv_bin_dir(&root), root.join("bin"));
        }

        #[test]
        fn virtualenv_python_exe_returns_bin_python_on_unix() {
            let root = std::path::PathBuf::from("/tmp/fake");
            assert_eq!(
                virtualenv_python_exe(&root),
                root.join("bin").join("python")
            );
        }

        #[test]
        fn virtualenv_pip_exe_returns_bin_pip_on_unix() {
            let root = std::path::PathBuf::from("/tmp/fake");
            assert_eq!(virtualenv_pip_exe(&root), root.join("bin").join("pip"));
        }

        #[test]
        fn virtualenv_activate_script_returns_bin_activate_on_unix() {
            let root = std::path::PathBuf::from("/tmp/fake");
            assert_eq!(
                virtualenv_activate_script(&root),
                root.join("bin").join("activate")
            );
        }

        #[test]
        fn site_packages_resolves_via_pyvenv_cfg_when_dir_exists() {
            let tmp = make_venv_root(
                Some("home = /usr/local/bin\nversion = 3.12.4\n"),
                &["python3.12"],
            );
            let sp = virtualenv_site_packages(tmp.path()).unwrap();
            assert!(sp.ends_with("lib/python3.12/site-packages"), "got {:?}", sp);
        }

        #[test]
        fn site_packages_falls_back_to_glob_when_pyvenv_cfg_absent() {
            let tmp = make_venv_root(None, &["python3.12"]);
            let sp = virtualenv_site_packages(tmp.path()).unwrap();
            assert!(sp.ends_with("lib/python3.12/site-packages"), "got {:?}", sp);
        }

        #[test]
        fn site_packages_glob_ignores_non_python_dirs() {
            // pythonbenchmarks must NOT match — strict regex requires MAJOR.MINOR.
            let tmp = make_venv_root(None, &["pythonbenchmarks", "python3.12"]);
            let sp = virtualenv_site_packages(tmp.path()).unwrap();
            assert!(sp.ends_with("lib/python3.12/site-packages"));
        }

        #[test]
        fn site_packages_glob_ambiguous_falls_through_to_err() {
            // python3.12 AND python3.13 both present, no python exe → Err.
            let tmp = make_venv_root(None, &["python3.12", "python3.13"]);
            let result = virtualenv_site_packages(tmp.path());
            assert!(matches!(
                result,
                Err(ScoopError::SitePackagesNotFound { .. })
            ));
        }

        #[test]
        fn site_packages_returns_err_when_completely_empty() {
            let tmp = TempDir::new().unwrap();
            let result = virtualenv_site_packages(tmp.path());
            assert!(matches!(
                result,
                Err(ScoopError::SitePackagesNotFound { .. })
            ));
        }

        #[test]
        fn site_packages_glob_accepts_free_threaded_t_suffix() {
            let tmp = make_venv_root(None, &["python3.13t"]);
            let sp = virtualenv_site_packages(tmp.path()).unwrap();
            assert!(
                sp.ends_with("lib/python3.13t/site-packages"),
                "got {:?}",
                sp
            );
        }

        #[test]
        fn virtualenv_bin_wrapper_delegates_to_helper() {
            with_temp_scoop_home(|_| {
                let bin = virtualenv_bin("anyname").unwrap();
                assert!(bin.ends_with("bin"));
            });
        }

        #[test]
        fn virtualenv_python_wrapper_delegates_to_helper() {
            with_temp_scoop_home(|_| {
                let py = virtualenv_python("anyname").unwrap();
                assert!(py.ends_with("bin/python"));
            });
        }
    }

    #[cfg(windows)]
    mod cross_platform_windows {
        use super::*;

        #[test]
        fn virtualenv_bin_dir_returns_scripts_on_windows() {
            let root = std::path::PathBuf::from("C:\\fake");
            assert_eq!(virtualenv_bin_dir(&root), root.join("Scripts"));
        }

        #[test]
        fn virtualenv_python_exe_returns_scripts_python_exe_on_windows() {
            let root = std::path::PathBuf::from("C:\\fake");
            assert_eq!(
                virtualenv_python_exe(&root),
                root.join("Scripts").join("python.exe")
            );
        }

        #[test]
        fn virtualenv_pip_exe_returns_scripts_pip_exe_on_windows() {
            let root = std::path::PathBuf::from("C:\\fake");
            assert_eq!(
                virtualenv_pip_exe(&root),
                root.join("Scripts").join("pip.exe")
            );
        }

        #[test]
        fn virtualenv_activate_script_returns_ps1_on_windows() {
            let root = std::path::PathBuf::from("C:\\fake");
            assert_eq!(
                virtualenv_activate_script(&root),
                root.join("Scripts").join("Activate.ps1")
            );
        }
    }
}
