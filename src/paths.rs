//! Path utilities for scuv

use std::path::{Path, PathBuf};

use crate::error::{Result, ScoopError};

/// Environment variable for the scuv home directory.
pub const SCUV_HOME_ENV: &str = "SCUV_HOME";

/// Default scuv home directory name.
const SCUV_HOME_DIR: &str = ".scuv";

/// Version file name.
pub const VERSION_FILE: &str = ".scuv-version";

/// Get the scuv home directory.
///
/// Resolution order: `$SCUV_HOME` > `~/.scuv`. The scoop-era names
/// (`$SCOOP_HOME`, `~/.scoop`) stopped being read in 0.16.0.
pub fn scoop_home() -> Result<PathBuf> {
    if let Ok(home) = std::env::var(SCUV_HOME_ENV) {
        return Ok(PathBuf::from(home));
    }

    let base = dirs::home_dir().ok_or(ScoopError::HomeNotFound)?;
    Ok(base.join(SCUV_HOME_DIR))
}

/// Get the virtualenvs directory (~/.scuv/virtualenvs)
pub fn virtualenvs_dir() -> Result<PathBuf> {
    Ok(scoop_home()?.join("virtualenvs"))
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

/// Returns the activation script path inside a virtualenv root.
///
/// On Unix this is `<root>/bin/activate` (POSIX shell). On Windows it is
/// `<root>/Scripts/Activate.ps1` (PowerShell).
///
/// Distinct from [`virtualenv_activate`], which returns the cmd.exe
/// `.bat` variant on Windows. PowerShell users are surfaced through
/// this helper (used by `scuv verify`).
#[cfg(unix)]
pub fn virtualenv_activate_script(venv_root: &Path) -> PathBuf {
    virtualenv_bin_dir(venv_root).join("activate")
}

#[cfg(windows)]
pub fn virtualenv_activate_script(venv_root: &Path) -> PathBuf {
    virtualenv_bin_dir(venv_root).join("Activate.ps1")
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
/// Shared by `scuv which` (display the resolved path) and `scuv run`
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

/// Best-effort: removes `link` if it is still a symlink whose raw target is
/// `expected`, the value the caller read when it decided to remove it.
///
/// Returns `Ok(true)` when removed. A path that is gone, is no longer a
/// symlink, or now points elsewhere is left alone and gives `Ok(false)`.
///
/// This narrows, but does not close, the gap between deciding and
/// deleting: it shrinks from the caller's own work in between (seconds for
/// an env deletion) to the two syscalls here. Closing it needs a
/// directory-handle-relative unlink.
///
/// # Errors
///
/// Returns the I/O error if the link cannot be read for another reason
/// (e.g. a permission error), or if the unlink itself fails.
///
/// # Examples
///
/// ```no_run
/// use std::path::Path;
/// use scoop_uv::paths::remove_symlink_if_unchanged;
///
/// let link = Path::new(".venv");
/// let recorded = std::fs::read_link(link)?;
/// // ... other work ...
/// remove_symlink_if_unchanged(link, &recorded)?;
/// # Ok::<(), std::io::Error>(())
/// ```
pub fn remove_symlink_if_unchanged(
    link: &std::path::Path,
    expected: &std::path::Path,
) -> std::io::Result<bool> {
    match std::fs::read_link(link) {
        Ok(dest) if dest == expected => std::fs::remove_file(link).map(|()| true),
        Ok(_) => Ok(false),
        // Gone, or not a symlink any more: changed, not ours.
        Err(e)
            if matches!(
                e.kind(),
                std::io::ErrorKind::NotFound | std::io::ErrorKind::InvalidInput
            ) =>
        {
            Ok(false)
        }
        Err(e) => Err(e),
    }
}

/// Identifies a directory beyond its path, so an env removed and created
/// again under the same name is told apart: device and inode on unix, the
/// creation time elsewhere (Windows). Neither is unique for all time: a
/// filesystem may reuse an inode once the old directory is gone, and a copy
/// can keep the original's creation time; with the same `home` such a
/// replacement passes. (The crate does not build for Windows yet, so that
/// branch is not exercised.) Reading it fails, with `NotFound`, when the
/// directory is gone.
///
/// # Errors
///
/// Reading the directory's metadata.
pub type DirIdentity = (u64, u64);

pub fn dir_identity(path: &Path) -> std::io::Result<DirIdentity> {
    let meta = std::fs::symlink_metadata(path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        Ok((meta.dev(), meta.ino()))
    }
    #[cfg(not(unix))]
    created_identity(&meta)
}

/// The creation time as a [`DirIdentity`]. Not compiled on Unix, so its
/// mutants are excluded there (`.cargo/mutants.toml`).
#[cfg(not(unix))]
fn created_identity(meta: &std::fs::Metadata) -> std::io::Result<DirIdentity> {
    let since = meta
        .created()?
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(std::io::Error::other)?;
    Ok((since.as_secs(), u64::from(since.subsec_nanos())))
}

/// Removes the directory at `path` only if it is still the one `created`
/// identifies, and reports whether it did. A failed `clone` or `import`
/// cleans up with this, so an env another command put under the same name
/// meanwhile is left alone. A replacement between the check and the
/// removal is not caught (that would need a lock across commands).
///
/// # Errors
///
/// Reading the directory's metadata (other than `NotFound`), or removing
/// it.
pub fn remove_dir_if_unchanged(path: &Path, created: DirIdentity) -> std::io::Result<bool> {
    match dir_identity(path) {
        Ok(now) if now == created => std::fs::remove_dir_all(path).map(|()| true),
        Ok(_) => Ok(false),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(e),
    }
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
/// let path = home.join(".scuv/virtualenvs/myenv");
/// let abbreviated = abbreviate_home(&path);
/// assert!(abbreviated.starts_with("~/"));
/// ```
pub fn abbreviate_home(path: &std::path::Path) -> String {
    if let Some(home) = dirs::home_dir()
        && let Ok(stripped) = path.strip_prefix(&home)
    {
        return format!("~/{}", stripped.display());
    }
    path.display().to_string()
}

#[cfg(test)]
mod tests;
