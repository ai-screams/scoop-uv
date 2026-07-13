//! Shared test utilities for scuv
//!
//! This module provides common test helpers to avoid code duplication
//! across test modules.

use std::sync::{Mutex, MutexGuard};
use tempfile::TempDir;

use crate::paths::{LEGACY_HOME_ENV, SCUV_HOME_ENV};

/// Global mutex to synchronize tests that manipulate SCOOP_HOME environment variable.
///
/// Environment variables are process-global state, so concurrent access causes race conditions.
/// All tests that modify SCOOP_HOME must acquire this lock first.
pub static ENV_LOCK: Mutex<()> = Mutex::new(());

/// RAII guard that captures the current i18n locale and restores it on drop.
///
/// rust-i18n stores the active locale as process-global state, so a test that
/// calls `set_locale` (directly or via the `lang` command) would otherwise
/// leak its locale into later tests that assume the default. [`with_temp_scoop_home`]
/// holds one for the duration of its closure, so the many command tests that
/// mutate the locale need no per-test bookkeeping. Standalone locale tests can
/// construct one directly. Pair with `#[serial]` to also exclude concurrent
/// observers of the global locale.
pub struct LocaleGuard {
    previous: String,
}

impl LocaleGuard {
    /// Capture the current locale; it is restored when the guard drops.
    pub fn capture() -> Self {
        Self {
            previous: rust_i18n::locale().to_string(),
        }
    }
}

impl Drop for LocaleGuard {
    fn drop(&mut self) {
        rust_i18n::set_locale(&self.previous);
    }
}

/// Execute a test function with SCUV_HOME and legacy SCOOP_HOME both unset.
///
/// This helper ensures safe testing of default behavior when neither the
/// current (`SCUV_HOME`) nor the legacy (`SCOOP_HOME`) env var is set —
/// unsetting only one would let the other leak through `scoop_home()`'s
/// fallback and mask the default-path behavior under test. Uses
/// catch_unwind to guarantee cleanup even if the test panics.
///
/// # Examples
///
/// ```ignore
/// use scoop_uv::test_utils::with_no_scoop_home;
///
/// #[test]
/// fn test_default_home() {
///     with_no_scoop_home(|| {
///         // SCUV_HOME and SCOOP_HOME are guaranteed to be unset here
///         let home = scoop_home().unwrap();
///         assert!(home.ends_with(".scuv") || home.ends_with(".scoop"));
///     });
/// }
/// ```
pub fn with_no_scoop_home<F, T>(f: F) -> T
where
    F: FnOnce() -> T,
{
    // Recover from poisoned mutex if a previous test panicked
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let backup_new = std::env::var(SCUV_HOME_ENV).ok();
    let backup_legacy = std::env::var(LEGACY_HOME_ENV).ok();

    // SAFETY: Protected by ENV_LOCK mutex - only one test modifies this at a time
    unsafe {
        std::env::remove_var(SCUV_HOME_ENV);
        std::env::remove_var(LEGACY_HOME_ENV);
    }

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));

    // Restore original values if they existed
    // SAFETY: Protected by ENV_LOCK mutex
    unsafe {
        match backup_new {
            Some(val) => std::env::set_var(SCUV_HOME_ENV, val),
            None => std::env::remove_var(SCUV_HOME_ENV),
        }
        match backup_legacy {
            Some(val) => std::env::set_var(LEGACY_HOME_ENV, val),
            None => std::env::remove_var(LEGACY_HOME_ENV),
        }
    }

    match result {
        Ok(val) => val,
        Err(e) => std::panic::resume_unwind(e),
    }
}

/// Execute a test function with an isolated temporary SCUV_HOME.
///
/// This helper:
/// 1. Acquires the global ENV_LOCK to prevent race conditions
/// 2. Creates a temporary directory for SCUV_HOME
/// 3. Sets the SCUV_HOME environment variable
/// 4. Runs the provided test function
/// 5. Cleans up the environment variable (even on panic)
///
/// Sets `SCUV_HOME` (the current-priority var read by `scoop_home()`), not
/// the legacy `SCOOP_HOME` — since `SCUV_HOME` wins whenever both are set,
/// this is the isolation callers actually want by default.
///
/// # Examples
///
/// ```ignore
/// use scoop_uv::test_utils::with_temp_scoop_home;
///
/// #[test]
/// fn test_something() {
///     with_temp_scoop_home(|temp_dir| {
///         // temp_dir.path() is the temporary SCUV_HOME
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
    // Restore the global i18n locale on exit so locale-mutating command tests
    // (e.g. `scuv lang ko`) don't leak into later tests.
    let _locale = LocaleGuard::capture();
    let temp_dir = TempDir::new().expect("Failed to create temp dir for SCUV_HOME");

    // SAFETY: Protected by ENV_LOCK mutex - only one test modifies this at a time
    unsafe { std::env::set_var(SCUV_HOME_ENV, temp_dir.path()) };

    // Use catch_unwind to ensure cleanup even on panic
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f(&temp_dir)));

    // SAFETY: Protected by ENV_LOCK mutex - always cleanup
    unsafe { std::env::remove_var(SCUV_HOME_ENV) };

    match result {
        Ok(val) => val,
        Err(e) => std::panic::resume_unwind(e),
    }
}

/// RAII guard that sets/unsets a list of environment variables for its
/// lifetime, restoring their previous values on drop.
///
/// Holds [`ENV_LOCK`] for the guard's lifetime, so callers are serialized
/// against [`with_temp_scoop_home`], [`with_no_scoop_home`], and each other —
/// unlike the single-variable helpers above, this covers an arbitrary set of
/// vars in one call, which is useful for tests that need precise control
/// over more than one variable at once (e.g. asserting priority between a
/// current and a legacy env var). `Some(value)` sets/overwrites the
/// variable; `None` removes it. Restoration happens even if the caller's
/// closure panics, since `Drop` still runs during unwind — but only within
/// the same thread; a panic that unwinds past the guard on another thread
/// will not restore it.
///
/// # Examples
///
/// ```ignore
/// use scoop_uv::test_utils::env_guard;
///
/// let _g = env_guard(&[
///     ("SCOOP_UV_TEST_ENV_GUARD_DOCTEST_A", Some("value")),
///     ("SCOOP_UV_TEST_ENV_GUARD_DOCTEST_B", None),
/// ]);
/// assert_eq!(
///     std::env::var("SCOOP_UV_TEST_ENV_GUARD_DOCTEST_A").as_deref(),
///     Ok("value")
/// );
/// assert!(std::env::var("SCOOP_UV_TEST_ENV_GUARD_DOCTEST_B").is_err());
/// ```
pub struct EnvGuard {
    _lock: MutexGuard<'static, ()>,
    backup: Vec<(&'static str, Option<String>)>,
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        for (name, value) in self.backup.drain(..) {
            // SAFETY: Protected by ENV_LOCK, held by `self._lock` until this
            // guard drops.
            unsafe {
                match value {
                    Some(v) => std::env::set_var(name, v),
                    None => std::env::remove_var(name),
                }
            }
        }
    }
}

/// Set/unset `vars` for the returned guard's lifetime; see [`EnvGuard`].
pub fn env_guard(vars: &[(&'static str, Option<&str>)]) -> EnvGuard {
    // Recover from poisoned mutex if a previous test panicked
    let lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut backup = Vec::with_capacity(vars.len());
    for (name, value) in vars {
        backup.push((*name, std::env::var(name).ok()));
        // SAFETY: Protected by ENV_LOCK, held by `lock` above.
        unsafe {
            match value {
                Some(v) => std::env::set_var(name, v),
                None => std::env::remove_var(name),
            }
        }
    }
    EnvGuard {
        _lock: lock,
        backup,
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
/// use scoop_uv::test_utils::{with_temp_scoop_home, create_mock_venv};
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
/// 3. Sets HOME to a temporary directory to prevent fallback to ~/.pyenv, ~/.virtualenvs, etc.
/// 4. Runs the provided test function
/// 5. Restores original environment variables (even on panic)
///
/// Use this for testing `find_environment_by_name` error cases where
/// no real pyenv/conda/virtualenvwrapper installation should be found.
///
/// # Examples
///
/// ```ignore
/// use scoop_uv::test_utils::with_isolated_migrate_env;
///
/// #[test]
/// fn test_find_returns_error_when_not_found() {
///     with_isolated_migrate_env(|| {
///         // PYENV_ROOT, WORKON_HOME, CONDA_PREFIX are all unset
///         // HOME points to an empty temp directory
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

    // Create temp dir for HOME to prevent fallback discovery to ~/.pyenv, ~/.virtualenvs, etc.
    // Discovery functions use dirs::home_dir() which reads HOME env var on Unix.
    let temp_home = TempDir::new().expect("Failed to create temp HOME for migrate isolation");

    // Backup and unset environment variables
    let mut backup: HashMap<&str, Option<String>> = HashMap::new();
    for var in MIGRATE_ENV_VARS {
        backup.insert(var, std::env::var(var).ok());
        // SAFETY: Protected by MIGRATE_ENV_LOCK mutex
        unsafe { std::env::remove_var(var) };
    }

    // Also backup and set HOME to temp directory
    backup.insert("HOME", std::env::var("HOME").ok());
    // SAFETY: Protected by MIGRATE_ENV_LOCK mutex
    unsafe { std::env::set_var("HOME", temp_home.path()) };

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
/// use scoop_uv::test_utils::create_mock_pyenv_env;
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

/// Execute a test with both SCUV_HOME and PYENV_ROOT isolated.
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
    let _env_lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let _migrate_guard = MIGRATE_ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    // Create temp directories
    let scoop_home = TempDir::new().expect("Failed to create temp SCUV_HOME");
    let pyenv_root = TempDir::new().expect("Failed to create temp PYENV_ROOT");

    // Create virtualenvs directory for scuv
    std::fs::create_dir_all(scoop_home.path().join("virtualenvs"))
        .expect("Failed to create virtualenvs dir");

    // Backup and set environment variables
    let mut backup: HashMap<&str, Option<String>> = HashMap::new();
    for var in MIGRATE_ENV_VARS {
        backup.insert(var, std::env::var(var).ok());
        unsafe { std::env::remove_var(var) };
    }
    backup.insert(SCUV_HOME_ENV, std::env::var(SCUV_HOME_ENV).ok());

    // Set isolated environment
    unsafe {
        std::env::set_var(SCUV_HOME_ENV, scoop_home.path());
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
            let scoop_home = std::env::var(SCUV_HOME_ENV).unwrap();
            assert_eq!(PathBuf::from(scoop_home), temp_dir.path());
        });
    }

    #[test]
    #[serial]
    fn test_with_temp_scoop_home_cleans_up() {
        with_temp_scoop_home(|_| {
            // Do nothing
        });
        // After the function, SCUV_HOME should be unset
        assert!(std::env::var(SCUV_HOME_ENV).is_err());
    }

    #[test]
    #[serial]
    fn test_with_no_scoop_home_unsets_both_vars() {
        // SAFETY: serialized via #[serial]; env restored after test.
        unsafe {
            std::env::set_var(SCUV_HOME_ENV, "/tmp/should-not-leak-new");
            std::env::set_var(LEGACY_HOME_ENV, "/tmp/should-not-leak-legacy");
        }
        with_no_scoop_home(|| {
            assert!(std::env::var(SCUV_HOME_ENV).is_err());
            assert!(std::env::var(LEGACY_HOME_ENV).is_err());
        });
        assert_eq!(
            std::env::var(SCUV_HOME_ENV).as_deref(),
            Ok("/tmp/should-not-leak-new")
        );
        assert_eq!(
            std::env::var(LEGACY_HOME_ENV).as_deref(),
            Ok("/tmp/should-not-leak-legacy")
        );
        // SAFETY: serialized via #[serial]; cleanup.
        unsafe {
            std::env::remove_var(SCUV_HOME_ENV);
            std::env::remove_var(LEGACY_HOME_ENV);
        }
    }

    #[test]
    #[serial]
    fn test_env_guard_sets_and_restores() {
        // SAFETY: serialized via #[serial]; env restored after test.
        unsafe {
            std::env::set_var("SCOOP_UV_TEST_ENV_GUARD_PRIOR", "prior-value");
        }
        {
            let _g = env_guard(&[
                ("SCOOP_UV_TEST_ENV_GUARD_PRIOR", Some("overridden")),
                ("SCOOP_UV_TEST_ENV_GUARD_NEW", Some("fresh")),
            ]);
            assert_eq!(
                std::env::var("SCOOP_UV_TEST_ENV_GUARD_PRIOR").as_deref(),
                Ok("overridden")
            );
            assert_eq!(
                std::env::var("SCOOP_UV_TEST_ENV_GUARD_NEW").as_deref(),
                Ok("fresh")
            );
        }
        assert_eq!(
            std::env::var("SCOOP_UV_TEST_ENV_GUARD_PRIOR").as_deref(),
            Ok("prior-value")
        );
        assert!(std::env::var("SCOOP_UV_TEST_ENV_GUARD_NEW").is_err());
        // SAFETY: serialized via #[serial]; cleanup.
        unsafe {
            std::env::remove_var("SCOOP_UV_TEST_ENV_GUARD_PRIOR");
        }
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
