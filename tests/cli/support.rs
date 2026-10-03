//! Shared fixtures and helpers for the CLI tests.

pub use assert_cmd::Command;
pub use predicates::prelude::*;
pub use std::path::PathBuf;
pub use tempfile::TempDir;

/// Test fixture for scoop tests
pub(crate) struct TestFixture {
    /// Temporary directory: holds `scoop_home` (and any project directory a
    /// test creates) until the fixture is dropped
    pub(crate) temp_dir: TempDir,
    /// SCUV_HOME path
    pub(crate) scoop_home: PathBuf,
}

impl TestFixture {
    /// Create a new test fixture with isolated SCUV_HOME
    pub(crate) fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let scoop_home = temp_dir.path().join(".scuv");
        Self {
            temp_dir,
            scoop_home,
        }
    }
}

/// Helper to get a fresh command with SCUV_HOME set
pub(crate) fn scoop_cmd(scoop_home: &std::path::Path) -> Command {
    let mut cmd = Command::cargo_bin("scuv").unwrap();
    cmd.env("SCUV_HOME", scoop_home);
    // Force English locale for consistent test assertions
    cmd.env("SCUV_LANG", "en");
    cmd
}

/// Locate a fish binary: PATH first, then the usual install prefixes. When
/// `SCUV_REQUIRE_FISH` is set a missing fish is a failure, not a skip: the CI
/// Test and MSRV jobs install fish and set that variable, so a silently
/// skipped test cannot look like a pass there. Jobs that run the suite
/// without fish (coverage, mutants) leave it unset and skip.
pub(crate) fn find_fish() -> Option<std::path::PathBuf> {
    let from_path = std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths)
            .map(|dir| dir.join("fish"))
            .find(|p| p.is_file())
    });
    let found = from_path.or_else(|| {
        [
            "/opt/homebrew/bin/fish",
            "/usr/local/bin/fish",
            "/usr/bin/fish",
        ]
        .iter()
        .map(std::path::PathBuf::from)
        .find(|p| p.is_file())
    });
    if found.is_none() && std::env::var_os("SCUV_REQUIRE_FISH").is_some() {
        panic!("SCUV_REQUIRE_FISH is set but no fish binary was found");
    }
    found
}

pub(crate) const ESC: &str = "\x1b[";

/// A home with one env, `demo`, that `SCUV_ACTIVE` marks active so `list`
/// highlights it (the only colored stdout).
pub(crate) fn fixture_with_active_env() -> TestFixture {
    let fixture = TestFixture::new();
    let env = fixture.scoop_home.join("virtualenvs").join("demo");
    std::fs::create_dir_all(env.join("bin")).unwrap();
    std::fs::write(env.join("pyvenv.cfg"), "version = 3.12.1\n").unwrap();
    fixture
}
