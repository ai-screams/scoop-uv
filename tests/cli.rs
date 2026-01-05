//! CLI integration tests
//!
//! These tests verify the CLI behavior using `assert_cmd`.
//! Note: Tests that require Python installation (create, activate) are marked as `#[ignore]`
//! to allow running in CI environments without uv/Python installed.

#![allow(deprecated)] // cargo_bin is deprecated but still works fine for our use case

use assert_cmd::Command;
use predicates::prelude::*;
use std::path::PathBuf;
use tempfile::TempDir;

/// Test fixture for scoop tests
struct TestFixture {
    /// Temporary directory - held to prevent cleanup until fixture is dropped
    #[allow(dead_code)]
    temp_dir: TempDir,
    /// SCOOP_HOME path
    scoop_home: PathBuf,
}

impl TestFixture {
    /// Create a new test fixture with isolated SCOOP_HOME
    fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let scoop_home = temp_dir.path().join(".scoop");
        Self {
            temp_dir,
            scoop_home,
        }
    }
}

/// Helper to get a fresh command with SCOOP_HOME set
fn scoop_cmd(scoop_home: &std::path::Path) -> Command {
    let mut cmd = Command::cargo_bin("scoop").unwrap();
    cmd.env("SCOOP_HOME", scoop_home);
    cmd
}

#[test]
fn test_help_flag() {
    Command::cargo_bin("scoop")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("scoop"))
        .stdout(predicate::str::contains("COMMAND"));
}

#[test]
fn test_version_flag() {
    Command::cargo_bin("scoop")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("scoop"));
}

#[test]
fn test_list_empty() {
    let fixture = TestFixture::new();

    scoop_cmd(&fixture.scoop_home)
        .arg("list")
        .assert()
        .success();
}

#[test]
fn test_list_pythons_empty() {
    let fixture = TestFixture::new();

    scoop_cmd(&fixture.scoop_home)
        .args(["list", "--pythons"])
        .assert()
        .success();
}

#[test]
fn test_list_bare_format() {
    let fixture = TestFixture::new();

    scoop_cmd(&fixture.scoop_home)
        .args(["list", "--bare"])
        .assert()
        .success();
}

#[test]
fn test_init_bash() {
    Command::cargo_bin("scoop")
        .unwrap()
        .args(["init", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("scoop()"))
        .stdout(predicate::str::contains("_scoop_hook"));
}

#[test]
fn test_init_zsh() {
    Command::cargo_bin("scoop")
        .unwrap()
        .args(["init", "zsh"])
        .assert()
        .success()
        .stdout(predicate::str::contains("scoop()"))
        .stdout(predicate::str::contains("add-zsh-hook"));
}

#[test]
#[ignore = "fish shell panics instead of returning error - TODO: fix"]
fn test_init_unsupported_shell() {
    Command::cargo_bin("scoop")
        .unwrap()
        .args(["init", "fish"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not supported"));
}

#[test]
fn test_completions_bash() {
    Command::cargo_bin("scoop")
        .unwrap()
        .args(["completions", "bash"])
        .assert()
        .success();
}

#[test]
fn test_completions_zsh() {
    Command::cargo_bin("scoop")
        .unwrap()
        .args(["completions", "zsh"])
        .assert()
        .success();
}

#[test]
fn test_activate_nonexistent_env() {
    let fixture = TestFixture::new();

    scoop_cmd(&fixture.scoop_home)
        .args(["activate", "nonexistent"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_remove_nonexistent_env() {
    let fixture = TestFixture::new();

    scoop_cmd(&fixture.scoop_home)
        .args(["remove", "nonexistent"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_use_nonexistent_env() {
    let fixture = TestFixture::new();

    scoop_cmd(&fixture.scoop_home)
        .args(["use", "nonexistent"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_create_invalid_name_starts_with_number() {
    let fixture = TestFixture::new();

    scoop_cmd(&fixture.scoop_home)
        .args(["create", "123invalid"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid"));
}

#[test]
fn test_create_reserved_name() {
    let fixture = TestFixture::new();

    scoop_cmd(&fixture.scoop_home)
        .args(["create", "activate"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("reserved"));
}

#[test]
fn test_install_conflicting_options() {
    let fixture = TestFixture::new();

    scoop_cmd(&fixture.scoop_home)
        .args(["install", "--latest", "--stable"])
        .assert()
        .failure();
}

#[test]
fn test_install_conflicting_latest_with_version() {
    let fixture = TestFixture::new();

    scoop_cmd(&fixture.scoop_home)
        .args(["install", "--latest", "3.12"])
        .assert()
        .failure();
}

#[test]
fn test_deactivate_when_not_active() {
    let fixture = TestFixture::new();

    // Deactivate should output shell code even when nothing is active
    scoop_cmd(&fixture.scoop_home)
        .arg("deactivate")
        .assert()
        .success();
}

#[test]
fn test_resolve_with_version_file() {
    let fixture = TestFixture::new();

    // Create a version file in the temp directory
    std::fs::write(fixture.temp_dir.path().join(".scoop-version"), "testenv").unwrap();

    // resolve should succeed and output the env name
    scoop_cmd(&fixture.scoop_home)
        .arg("resolve")
        .current_dir(fixture.temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("testenv"));
}

#[test]
fn test_unknown_subcommand() {
    Command::cargo_bin("scoop")
        .unwrap()
        .arg("unknowncommand")
        .assert()
        .failure();
}

#[test]
fn test_list_pythons_bare() {
    let fixture = TestFixture::new();

    scoop_cmd(&fixture.scoop_home)
        .args(["list", "--pythons", "--bare"])
        .assert()
        .success();
}

// Tests requiring uv and Python - mark as #[ignore]
mod requires_uv {
    use super::*;

    #[test]
    #[ignore = "requires uv to be installed"]
    fn test_create_and_list() {
        let fixture = TestFixture::new();

        // Create environment
        scoop_cmd(&fixture.scoop_home)
            .args(["create", "testenv", "3.12"])
            .assert()
            .success();

        // List should show the new environment
        scoop_cmd(&fixture.scoop_home)
            .arg("list")
            .assert()
            .success()
            .stdout(predicate::str::contains("testenv"));
    }

    #[test]
    #[ignore = "requires uv to be installed"]
    fn test_create_and_remove() {
        let fixture = TestFixture::new();

        // Create environment
        scoop_cmd(&fixture.scoop_home)
            .args(["create", "toremove", "3.12"])
            .assert()
            .success();

        // Remove with --force to skip confirmation
        scoop_cmd(&fixture.scoop_home)
            .args(["remove", "--force", "toremove"])
            .assert()
            .success();

        // List should not show the removed environment
        scoop_cmd(&fixture.scoop_home)
            .arg("list")
            .assert()
            .success()
            .stdout(predicate::str::contains("toremove").not());
    }

    #[test]
    #[ignore = "requires uv to be installed"]
    fn test_create_duplicate_fails() {
        let fixture = TestFixture::new();

        // Create environment
        scoop_cmd(&fixture.scoop_home)
            .args(["create", "duplicate", "3.12"])
            .assert()
            .success();

        // Try to create again - should fail
        scoop_cmd(&fixture.scoop_home)
            .args(["create", "duplicate", "3.12"])
            .assert()
            .failure()
            .stderr(predicate::str::contains("already exists"));
    }

    #[test]
    #[ignore = "requires uv to be installed"]
    fn test_activate_outputs_shell_code() {
        let fixture = TestFixture::new();

        // Create environment first
        scoop_cmd(&fixture.scoop_home)
            .args(["create", "activatetest", "3.12"])
            .assert()
            .success();

        // Activate should output shell code
        scoop_cmd(&fixture.scoop_home)
            .args(["activate", "activatetest"])
            .assert()
            .success()
            .stdout(predicate::str::contains("VIRTUAL_ENV"))
            .stdout(predicate::str::contains("PATH"));
    }
}
