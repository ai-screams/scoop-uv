//! Common test utilities

use std::path::PathBuf;
use tempfile::TempDir;

/// Test fixture for scoop tests
pub struct TestFixture {
    /// Temporary directory
    pub temp_dir: TempDir,
    /// SCOOP_HOME path
    pub scoop_home: PathBuf,
}

impl TestFixture {
    /// Create a new test fixture
    pub fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let scoop_home = temp_dir.path().join(".scoop");

        // Set SCOOP_HOME for tests
        std::env::set_var("SCOOP_HOME", &scoop_home);

        Self {
            temp_dir,
            scoop_home,
        }
    }
}

impl Drop for TestFixture {
    fn drop(&mut self) {
        // Clean up environment
        std::env::remove_var("SCOOP_HOME");
    }
}

impl Default for TestFixture {
    fn default() -> Self {
        Self::new()
    }
}
