//! Argument validation and error messages.

use crate::support::*;

#[test]
fn test_use_nonexistent_env() {
    let fixture = TestFixture::new();

    scoop_cmd(&fixture.scoop_home)
        .args(["use", "nonexistent"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Can't find"));
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
fn test_create_empty_name() {
    let fixture = TestFixture::new();

    // Empty name should fail with clear error
    scoop_cmd(&fixture.scoop_home)
        .args(["create", ""])
        .assert()
        .failure();
}

#[test]
fn test_create_name_with_spaces() {
    let fixture = TestFixture::new();

    scoop_cmd(&fixture.scoop_home)
        .args(["create", "my env"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid"));
}

#[test]
fn test_create_name_with_special_chars() {
    let fixture = TestFixture::new();

    scoop_cmd(&fixture.scoop_home)
        .args(["create", "my@env"])
        .assert()
        .failure();
}

#[test]
fn test_create_path_traversal_attempt() {
    let fixture = TestFixture::new();

    scoop_cmd(&fixture.scoop_home)
        .args(["create", "../etc/passwd"])
        .assert()
        .failure();
}

#[test]
fn test_use_without_env_name() {
    let fixture = TestFixture::new();

    scoop_cmd(&fixture.scoop_home).arg("use").assert().failure();
}

#[test]
fn test_activate_without_env_name() {
    let fixture = TestFixture::new();

    scoop_cmd(&fixture.scoop_home)
        .arg("activate")
        .assert()
        .failure();
}

#[test]
fn test_remove_without_env_name() {
    let fixture = TestFixture::new();

    scoop_cmd(&fixture.scoop_home)
        .arg("remove")
        .assert()
        .failure();
}

#[test]
fn test_init_without_shell() {
    Command::cargo_bin("scuv")
        .unwrap()
        .arg("init")
        .assert()
        .failure();
}

#[test]
fn test_completions_without_shell() {
    Command::cargo_bin("scuv")
        .unwrap()
        .arg("completions")
        .assert()
        .failure();
}

#[test]
fn test_uninstall_without_version() {
    let fixture = TestFixture::new();

    scoop_cmd(&fixture.scoop_home)
        .arg("uninstall")
        .assert()
        .failure();
}

#[test]
fn test_invalid_subcommand_suggestion() {
    // Test that invalid subcommand gives helpful error
    Command::cargo_bin("scuv")
        .unwrap()
        .arg("craete") // typo
        .assert()
        .failure();
}

#[test]
fn test_help_for_subcommand() {
    // Each subcommand should support --help
    for subcmd in ["list", "create", "remove", "use", "install"] {
        Command::cargo_bin("scuv")
            .unwrap()
            .args([subcmd, "--help"])
            .assert()
            .success();
    }
}
