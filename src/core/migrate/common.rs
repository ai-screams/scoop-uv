//! Common utilities for environment discovery
//!
//! This module contains shared functions used by all discovery implementations
//! to avoid code duplication (DRY principle).

use std::path::{Path, PathBuf};

use crate::paths;

use super::source::EnvironmentStatus;

/// EOL Python minor version threshold (3.8 and below are EOL as of 2024)
const EOL_PYTHON_MINOR: u32 = 8;

/// Calculate directory size in bytes.
///
/// # Performance Note
///
/// This function traverses the entire directory tree and calls `stat()` on every file.
/// For large environments with thousands of files, this can be expensive.
/// Consider using `Option<u64>` in `SourceEnvironment` and calculating size lazily.
///
/// # Examples
///
/// ```no_run
/// use std::path::Path;
/// use crate::core::migrate::common::dir_size;
///
/// let venv_path = Path::new("/home/user/.pyenv/versions/3.12.0/envs/myproject");
/// let size = dir_size(venv_path);
/// let size_mb = size as f64 / 1_048_576.0;
/// println!("Environment size: {:.1} MB", size_mb);
/// ```
#[allow(dead_code)]
pub fn dir_size(path: &Path) -> u64 {
    walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter_map(|e| e.metadata().ok())
        .filter(|m| m.is_file())
        .map(|m| m.len())
        .sum()
}

/// Check if environment name conflicts with existing scoop environment.
///
/// Returns `Some(path)` if a scoop environment with the same name exists,
/// `None` otherwise.
///
/// # Examples
///
/// ```no_run
/// use crate::core::migrate::common::check_name_conflict;
///
/// if let Some(existing_path) = check_name_conflict("myproject") {
///     println!("Conflict: {} already exists", existing_path.display());
/// } else {
///     println!("Name 'myproject' is available");
/// }
/// ```
pub fn check_name_conflict(name: &str) -> Option<PathBuf> {
    if let Ok(venvs_dir) = paths::virtualenvs_dir() {
        let scoop_path = venvs_dir.join(name);
        if scoop_path.exists() {
            return Some(scoop_path);
        }
    }
    None
}

/// Determine environment status based on name conflicts and Python version.
///
/// # Status Priority
///
/// 1. Name conflict (existing scoop environment)
/// 2. Python EOL (3.8 and below, or Python 2.x)
/// 3. Ready to migrate
///
/// # Examples
///
/// ```
/// use crate::core::migrate::common::determine_status;
/// use crate::core::migrate::EnvironmentStatus;
///
/// // Modern Python, no conflict
/// let status = determine_status("new_env", "3.12.0");
/// assert!(matches!(status, EnvironmentStatus::Ready));
///
/// // EOL Python version
/// let status = determine_status("old_env", "3.7.0");
/// assert!(matches!(status, EnvironmentStatus::PythonEol { .. }));
///
/// // Python 2.x is definitely EOL
/// let status = determine_status("ancient_env", "2.7.18");
/// assert!(matches!(status, EnvironmentStatus::PythonEol { .. }));
/// ```
pub fn determine_status(name: &str, python_version: &str) -> EnvironmentStatus {
    // Check for name conflict first
    if let Some(existing) = check_name_conflict(name) {
        return EnvironmentStatus::NameConflict { existing };
    }

    // Check for EOL Python versions
    let major_minor: Vec<&str> = python_version.split('.').collect();
    if major_minor.len() >= 2 {
        if let (Ok(major), Ok(minor)) =
            (major_minor[0].parse::<u32>(), major_minor[1].parse::<u32>())
        {
            // Python 3.8 and earlier are EOL (as of 2024)
            if major == 3 && minor <= EOL_PYTHON_MINOR {
                return EnvironmentStatus::PythonEol {
                    version: python_version.to_string(),
                };
            }
            // Python 2.x is definitely EOL
            if major == 2 {
                return EnvironmentStatus::PythonEol {
                    version: python_version.to_string(),
                };
            }
        }
    }

    EnvironmentStatus::Ready
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determine_status_eol_python27() {
        let status = determine_status("nonexistent_test_env_xyz", "2.7.18");
        assert!(matches!(status, EnvironmentStatus::PythonEol { .. }));
    }

    #[test]
    fn test_determine_status_eol_python38() {
        let status = determine_status("nonexistent_test_env_xyz", "3.8.0");
        assert!(matches!(status, EnvironmentStatus::PythonEol { .. }));
    }

    #[test]
    fn test_determine_status_ready_python312() {
        let status = determine_status("nonexistent_test_env_xyz", "3.12.0");
        assert!(matches!(status, EnvironmentStatus::Ready));
    }

    #[test]
    fn test_check_name_conflict_nonexistent() {
        let result = check_name_conflict("definitely_nonexistent_env_name_12345");
        assert!(result.is_none());
    }
}
