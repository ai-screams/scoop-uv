//! Common test utilities

use std::path::PathBuf;
use tempfile::TempDir;

/// Test fixture for uvenv tests
pub struct TestFixture {
    /// Temporary directory
    pub temp_dir: TempDir,
    /// UVENV_HOME path
    pub uvenv_home: PathBuf,
}

impl TestFixture {
    /// Create a new test fixture
    pub fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let uvenv_home = temp_dir.path().join(".uvenv");

        // Set UVENV_HOME for tests
        std::env::set_var("UVENV_HOME", &uvenv_home);

        Self {
            temp_dir,
            uvenv_home,
        }
    }
}

impl Drop for TestFixture {
    fn drop(&mut self) {
        // Clean up environment
        std::env::remove_var("UVENV_HOME");
    }
}

impl Default for TestFixture {
    fn default() -> Self {
        Self::new()
    }
}
