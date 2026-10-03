//! Tests that need a real uv and Python; `#[ignore]`d by default.

use crate::support::*;

// Tests requiring uv and Python - mark as #[ignore]

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
