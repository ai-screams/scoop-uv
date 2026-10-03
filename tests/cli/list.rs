//! `scuv list` without uv-backed environments.

use crate::support::*;

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
fn test_list_pythons_bare() {
    let fixture = TestFixture::new();

    scoop_cmd(&fixture.scoop_home)
        .args(["list", "--pythons", "--bare"])
        .assert()
        .success();
}

#[test]
fn test_list_shows_system_python() {
    let fixture = TestFixture::new();

    let output = scoop_cmd(&fixture.scoop_home).arg("list").output().unwrap();

    // System Python should be shown at the bottom of the list
    // At minimum, the command should succeed
    assert!(output.status.success());

    // Verify system Python is shown in output (if Python is installed)
    let stdout = String::from_utf8_lossy(&output.stdout);
    // System should appear in the output when system Python is available
    assert!(
        stdout.contains("system"),
        "List should show system Python when available"
    );
}

#[test]
fn test_list_bare_includes_system() {
    let fixture = TestFixture::new();

    // --bare mode SHOULD include system Python for tab completion
    // (since `scoop use system` is a valid command)
    let output = scoop_cmd(&fixture.scoop_home)
        .args(["list", "--bare"])
        .output()
        .unwrap();

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    // System Python should be included in bare output for completion
    assert!(
        stdout.contains("system"),
        "Bare mode should include system Python for completion"
    );
}
