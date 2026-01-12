//! Shared test utilities for scoop
//!
//! This module provides common test helpers to avoid code duplication
//! across test modules.

use std::sync::Mutex;
use tempfile::TempDir;

use crate::paths::SCOOP_HOME_ENV;

/// Global mutex to synchronize tests that manipulate SCOOP_HOME environment variable.
///
/// Environment variables are process-global state, so concurrent access causes race conditions.
/// All tests that modify SCOOP_HOME must acquire this lock first.
pub static ENV_LOCK: Mutex<()> = Mutex::new(());

/// Execute a test function with an isolated temporary SCOOP_HOME.
///
/// This helper:
/// 1. Acquires the global ENV_LOCK to prevent race conditions
/// 2. Creates a temporary directory for SCOOP_HOME
/// 3. Sets the SCOOP_HOME environment variable
/// 4. Runs the provided test function
/// 5. Cleans up the environment variable (even on panic)
///
/// # Examples
///
/// ```ignore
/// use scoop::test_utils::with_temp_scoop_home;
///
/// #[test]
/// fn test_something() {
///     with_temp_scoop_home(|temp_dir| {
///         // temp_dir.path() is the temporary SCOOP_HOME
///         assert!(temp_dir.path().exists());
///     });
/// }
/// ```
///
/// # Panics
///
/// Panics if the temporary directory cannot be created.
pub fn with_temp_scoop_home<F, T>(f: F) -> T
where
    F: FnOnce(&TempDir) -> T,
{
    // Recover from poisoned mutex if a previous test panicked
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let temp_dir = TempDir::new().expect("Failed to create temp dir for SCOOP_HOME");

    // SAFETY: Protected by ENV_LOCK mutex - only one test modifies this at a time
    unsafe { std::env::set_var(SCOOP_HOME_ENV, temp_dir.path()) };

    // Use catch_unwind to ensure cleanup even on panic
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f(&temp_dir)));

    // SAFETY: Protected by ENV_LOCK mutex - always cleanup
    unsafe { std::env::remove_var(SCOOP_HOME_ENV) };

    match result {
        Ok(val) => val,
        Err(e) => std::panic::resume_unwind(e),
    }
}

/// Create a mock virtual environment directory structure for testing.
///
/// This creates the necessary directory structure without actually
/// calling uv, useful for testing list/exists/delete operations.
///
/// # Arguments
///
/// * `temp_dir` - The temporary SCOOP_HOME directory
/// * `name` - Name of the virtual environment to create
/// * `python_version` - Optional Python version to write in metadata
///
/// # Examples
///
/// ```ignore
/// use scoop::test_utils::{with_temp_scoop_home, create_mock_venv};
///
/// #[test]
/// fn test_list() {
///     with_temp_scoop_home(|temp_dir| {
///         create_mock_venv(temp_dir, "myenv", Some("3.12"));
///         // Now "myenv" appears as a valid virtualenv
///     });
/// }
/// ```
pub fn create_mock_venv(temp_dir: &TempDir, name: &str, python_version: Option<&str>) {
    use crate::core::Metadata;
    use std::fs;

    let venvs_dir = temp_dir.path().join("virtualenvs");
    let venv_path = venvs_dir.join(name);
    fs::create_dir_all(&venv_path).expect("Failed to create mock venv directory");

    if let Some(version) = python_version {
        let meta = Metadata::new(name.to_string(), version.to_string(), None);
        let meta_path = venv_path.join(Metadata::FILE_NAME);
        fs::write(meta_path, serde_json::to_string(&meta).unwrap())
            .expect("Failed to write mock metadata");
    }
}

// =============================================================================
// Test Helper Macros
// =============================================================================

/// Assert that an environment name is valid.
///
/// # Examples
///
/// ```ignore
/// assert_valid_env!("myenv");
/// assert_valid_env!("my-project");
/// ```
#[macro_export]
macro_rules! assert_valid_env {
    ($name:expr) => {
        assert!(
            $crate::validate::is_valid_env_name($name),
            "Expected '{}' to be a valid env name",
            $name
        );
    };
}

/// Assert that an environment name is invalid.
///
/// # Examples
///
/// ```ignore
/// assert_invalid_env!("123");
/// assert_invalid_env!("activate");
/// ```
#[macro_export]
macro_rules! assert_invalid_env {
    ($name:expr) => {
        assert!(
            !$crate::validate::is_valid_env_name($name),
            "Expected '{}' to be an invalid env name",
            $name
        );
    };
}

/// Assert that a Python version is valid.
///
/// # Examples
///
/// ```ignore
/// assert_valid_version!("3.12");
/// assert_valid_version!("3.12.0");
/// ```
#[macro_export]
macro_rules! assert_valid_version {
    ($version:expr) => {
        assert!(
            $crate::validate::is_valid_python_version($version),
            "Expected '{}' to be a valid Python version",
            $version
        );
    };
}

/// Assert that a Python version is invalid.
///
/// # Examples
///
/// ```ignore
/// assert_invalid_version!("abc");
/// assert_invalid_version!("");
/// ```
#[macro_export]
macro_rules! assert_invalid_version {
    ($version:expr) => {
        assert!(
            !$crate::validate::is_valid_python_version($version),
            "Expected '{}' to be an invalid Python version",
            $version
        );
    };
}

/// Assert that an error matches a specific variant.
///
/// # Examples
///
/// ```ignore
/// let err = ScoopError::VirtualenvNotFound { name: "test".to_string() };
/// assert_error_variant!(err, ScoopError::VirtualenvNotFound { .. });
/// ```
#[macro_export]
macro_rules! assert_error_variant {
    ($err:expr, $variant:pat) => {
        assert!(
            matches!($err, $variant),
            "Expected error variant {}, got {:?}",
            stringify!($variant),
            $err
        );
    };
}

// =============================================================================
// Migrate Test Helpers
// =============================================================================

/// Environment variables backed up during isolated migrate tests.
const MIGRATE_ENV_VARS: &[&str] = &[
    "PYENV_ROOT",
    "PYENV_VIRTUALENV_INIT",
    "WORKON_HOME",
    "VIRTUALENVWRAPPER_HOOK_DIR",
    "CONDA_PREFIX",
    "CONDA_EXE",
];

/// Global mutex for migrate environment tests.
///
/// Prevents race conditions when tests modify PYENV_ROOT, WORKON_HOME, etc.
pub static MIGRATE_ENV_LOCK: Mutex<()> = Mutex::new(());

/// Execute a test function with isolated migrate environment variables.
///
/// This helper:
/// 1. Acquires the MIGRATE_ENV_LOCK to prevent race conditions
/// 2. Backs up and unsets all migrate-related environment variables
/// 3. Runs the provided test function
/// 4. Restores original environment variables (even on panic)
///
/// Use this for testing `find_environment_by_name` error cases where
/// no real pyenv/conda/virtualenvwrapper installation should be found.
///
/// # Examples
///
/// ```ignore
/// use scoop::test_utils::with_isolated_migrate_env;
///
/// #[test]
/// fn test_find_returns_error_when_not_found() {
///     with_isolated_migrate_env(|| {
///         // PYENV_ROOT, WORKON_HOME, CONDA_PREFIX are all unset
///         let result = find_environment_by_name("nonexistent", None);
///         assert!(result.is_err());
///     });
/// }
/// ```
pub fn with_isolated_migrate_env<F, T>(f: F) -> T
where
    F: FnOnce() -> T,
{
    use std::collections::HashMap;

    // Recover from poisoned mutex
    let _guard = MIGRATE_ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    // Backup and unset environment variables
    let mut backup: HashMap<&str, Option<String>> = HashMap::new();
    for var in MIGRATE_ENV_VARS {
        backup.insert(var, std::env::var(var).ok());
        // SAFETY: Protected by MIGRATE_ENV_LOCK mutex
        unsafe { std::env::remove_var(var) };
    }

    // Run test with catch_unwind to ensure cleanup
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));

    // Restore environment variables
    for (var, value) in backup {
        // SAFETY: Protected by MIGRATE_ENV_LOCK mutex
        unsafe {
            match value {
                Some(v) => std::env::set_var(var, v),
                None => std::env::remove_var(var),
            }
        }
    }

    match result {
        Ok(val) => val,
        Err(e) => std::panic::resume_unwind(e),
    }
}

/// Create a mock pyenv-virtualenv environment for testing.
///
/// This creates the correct pyenv-virtualenv directory structure:
/// - `$PYENV_ROOT/versions/<python_version>/envs/<name>/` directory
/// - `pyvenv.cfg` with Python version
/// - `bin/python` (empty file, just needs to exist)
///
/// # Arguments
///
/// * `pyenv_root` - The PYENV_ROOT directory
/// * `name` - Name of the environment
/// * `python_version` - Python version string (e.g., "3.12.0")
///
/// # Examples
///
/// ```ignore
/// use tempfile::TempDir;
/// use scoop::test_utils::create_mock_pyenv_env;
///
/// let temp = TempDir::new().unwrap();
/// create_mock_pyenv_env(temp.path(), "myenv", "3.12.0");
/// // Creates: temp/versions/3.12.0/envs/myenv/
/// ```
pub fn create_mock_pyenv_env(pyenv_root: &std::path::Path, name: &str, python_version: &str) {
    use std::fs;

    // pyenv-virtualenv structure: versions/<python_version>/envs/<env_name>/
    let versions_dir = pyenv_root.join("versions");
    let python_dir = versions_dir.join(python_version);
    let envs_dir = python_dir.join("envs");
    let env_dir = envs_dir.join(name);
    let bin_dir = env_dir.join("bin");

    fs::create_dir_all(&bin_dir).expect("Failed to create mock pyenv bin directory");

    // Create pyvenv.cfg
    let pyvenv_cfg = format!(
        "home = {}/versions/{}/bin\nversion = {}\n",
        pyenv_root.display(),
        python_version,
        python_version
    );
    fs::write(env_dir.join("pyvenv.cfg"), pyvenv_cfg).expect("Failed to write pyvenv.cfg");

    // Create bin/python (empty file)
    fs::write(bin_dir.join("python"), "").expect("Failed to write mock python binary");
}

/// Create a corrupted mock pyenv-virtualenv environment (missing bin/python).
///
/// Useful for testing error handling when environment is corrupted.
pub fn create_corrupted_pyenv_env(pyenv_root: &std::path::Path, name: &str, python_version: &str) {
    use std::fs;

    // pyenv-virtualenv structure: versions/<python_version>/envs/<env_name>/
    let versions_dir = pyenv_root.join("versions");
    let python_dir = versions_dir.join(python_version);
    let envs_dir = python_dir.join("envs");
    let env_dir = envs_dir.join(name);

    fs::create_dir_all(&env_dir).expect("Failed to create mock pyenv directory");

    // Create pyvenv.cfg but NO bin/python
    let pyvenv_cfg = format!("version = {}\n", python_version);
    fs::write(env_dir.join("pyvenv.cfg"), pyvenv_cfg).expect("Failed to write pyvenv.cfg");
}

/// Execute a test with both SCOOP_HOME and PYENV_ROOT isolated.
///
/// This helper combines `with_temp_scoop_home` and `with_isolated_migrate_env`,
/// setting up a complete isolated environment for migration testing.
///
/// # Returns
///
/// A tuple of (scoop_home TempDir, pyenv_root TempDir)
pub fn with_full_migrate_env<F, T>(f: F) -> T
where
    F: FnOnce(&TempDir, &TempDir) -> T,
{
    use std::collections::HashMap;

    // Recover from poisoned mutex
    let _env_guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let _migrate_guard = MIGRATE_ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    // Create temp directories
    let scoop_home = TempDir::new().expect("Failed to create temp SCOOP_HOME");
    let pyenv_root = TempDir::new().expect("Failed to create temp PYENV_ROOT");

    // Create virtualenvs directory for scoop
    std::fs::create_dir_all(scoop_home.path().join("virtualenvs"))
        .expect("Failed to create virtualenvs dir");

    // Backup and set environment variables
    let mut backup: HashMap<&str, Option<String>> = HashMap::new();
    for var in MIGRATE_ENV_VARS {
        backup.insert(var, std::env::var(var).ok());
        unsafe { std::env::remove_var(var) };
    }
    backup.insert("SCOOP_HOME", std::env::var("SCOOP_HOME").ok());

    // Set isolated environment
    unsafe {
        std::env::set_var(SCOOP_HOME_ENV, scoop_home.path());
        std::env::set_var("PYENV_ROOT", pyenv_root.path());
    }

    // Run test with catch_unwind
    let result =
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f(&scoop_home, &pyenv_root)));

    // Restore environment
    for (var, value) in backup {
        unsafe {
            match value {
                Some(v) => std::env::set_var(var, v),
                None => std::env::remove_var(var),
            }
        }
    }

    match result {
        Ok(val) => val,
        Err(e) => std::panic::resume_unwind(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::path::PathBuf;

    #[test]
    #[serial]
    fn test_with_temp_scoop_home_sets_env() {
        with_temp_scoop_home(|temp_dir| {
            let scoop_home = std::env::var(SCOOP_HOME_ENV).unwrap();
            assert_eq!(PathBuf::from(scoop_home), temp_dir.path());
        });
    }

    #[test]
    #[serial]
    fn test_with_temp_scoop_home_cleans_up() {
        with_temp_scoop_home(|_| {
            // Do nothing
        });
        // After the function, SCOOP_HOME should be unset
        assert!(std::env::var(SCOOP_HOME_ENV).is_err());
    }

    #[test]
    #[serial]
    fn test_create_mock_venv_creates_directory() {
        with_temp_scoop_home(|temp_dir| {
            create_mock_venv(temp_dir, "testenv", None);
            let venv_path = temp_dir.path().join("virtualenvs").join("testenv");
            assert!(venv_path.exists());
            assert!(venv_path.is_dir());
        });
    }

    #[test]
    #[serial]
    fn test_create_mock_venv_with_metadata() {
        with_temp_scoop_home(|temp_dir| {
            create_mock_venv(temp_dir, "withversion", Some("3.12"));
            let meta_path = temp_dir
                .path()
                .join("virtualenvs")
                .join("withversion")
                .join(".scoop-metadata.json");
            assert!(meta_path.exists());

            let content = std::fs::read_to_string(meta_path).unwrap();
            // Check for python_version field (JSON format may vary)
            assert!(content.contains("\"python_version\""));
            assert!(content.contains("\"3.12\""));
        });
    }

    // ==========================================================================
    // Concurrency Tests
    // ==========================================================================

    #[test]
    #[serial]
    fn test_multiple_mock_venvs_sequential() {
        with_temp_scoop_home(|temp_dir| {
            // Create multiple venvs sequentially
            for i in 0..5 {
                let name = format!("env{}", i);
                create_mock_venv(temp_dir, &name, Some("3.12"));
            }

            // Verify all exist
            for i in 0..5 {
                let name = format!("env{}", i);
                let path = temp_dir.path().join("virtualenvs").join(&name);
                assert!(path.exists(), "env{} should exist", i);
            }
        });
    }

    #[test]
    fn test_env_lock_prevents_concurrent_modification() {
        use std::sync::Arc;
        use std::thread;

        let counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let mut handles = vec![];

        // Spawn multiple threads that all try to acquire the lock
        for _ in 0..4 {
            let counter = Arc::clone(&counter);
            let handle = thread::spawn(move || {
                let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
                // Increment counter while holding lock
                let current = counter.load(std::sync::atomic::Ordering::SeqCst);
                // Small delay to increase chance of race condition if lock doesn't work
                thread::yield_now();
                counter.store(current + 1, std::sync::atomic::Ordering::SeqCst);
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().expect("Thread should not panic");
        }

        // All increments should have happened
        assert_eq!(
            counter.load(std::sync::atomic::Ordering::SeqCst),
            4,
            "All threads should have incremented"
        );
    }

    #[test]
    fn test_metadata_serialization_is_deterministic() {
        use crate::core::Metadata;

        // Create same metadata multiple times and verify JSON is consistent
        let mut jsons = Vec::new();
        for _ in 0..3 {
            let meta = Metadata::new(
                "test".to_string(),
                "3.12".to_string(),
                Some("1.0".to_string()),
            );
            // Note: created_at will differ, so we check structure only
            let json = serde_json::to_string(&meta).unwrap();
            assert!(json.contains("\"name\":\"test\""));
            assert!(json.contains("\"python_version\":\"3.12\""));
            jsons.push(json);
        }

        // All should have same structure (excluding timestamps)
        for json in &jsons {
            assert!(json.contains("created_at"));
            assert!(json.contains("created_by"));
        }
    }

    // ==========================================================================
    // Isolated Migrate Env Tests
    // ==========================================================================

    #[test]
    #[serial]
    fn test_with_isolated_migrate_env_unsets_vars() {
        with_isolated_migrate_env(|| {
            // All migrate env vars should be unset
            assert!(std::env::var("PYENV_ROOT").is_err());
            assert!(std::env::var("WORKON_HOME").is_err());
            assert!(std::env::var("CONDA_PREFIX").is_err());
        });
    }

    #[test]
    #[serial]
    fn test_with_isolated_migrate_env_restores_vars() {
        // Set a var before the test
        unsafe { std::env::set_var("PYENV_ROOT", "/original/path") };

        with_isolated_migrate_env(|| {
            // Should be unset inside
            assert!(std::env::var("PYENV_ROOT").is_err());
        });

        // Should be restored after
        assert_eq!(std::env::var("PYENV_ROOT").unwrap(), "/original/path");

        // Cleanup
        unsafe { std::env::remove_var("PYENV_ROOT") };
    }

    // ==========================================================================
    // Macro Tests
    // ==========================================================================

    #[test]
    fn test_assert_valid_env_macro() {
        assert_valid_env!("myenv");
        assert_valid_env!("my-project");
        assert_valid_env!("test_env");
    }

    #[test]
    fn test_assert_invalid_env_macro() {
        assert_invalid_env!("123");
        assert_invalid_env!("activate");
        assert_invalid_env!("");
    }

    #[test]
    fn test_assert_valid_version_macro() {
        assert_valid_version!("3");
        assert_valid_version!("3.12");
        assert_valid_version!("3.12.0");
    }

    #[test]
    fn test_assert_invalid_version_macro() {
        assert_invalid_version!("");
        assert_invalid_version!("abc");
        assert_invalid_version!("v3.12");
    }

    #[test]
    fn test_assert_error_variant_macro() {
        use crate::error::ScoopError;

        let err = ScoopError::VirtualenvNotFound {
            name: "test".to_string(),
        };
        assert_error_variant!(err, ScoopError::VirtualenvNotFound { .. });

        let err = ScoopError::HomeNotFound;
        assert_error_variant!(err, ScoopError::HomeNotFound);
    }
}
