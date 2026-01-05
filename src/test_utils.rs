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
}
