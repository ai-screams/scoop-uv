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

/// A fake uv whose `python list` succeeds with one Python and warns on
/// stderr, as uv 0.12.11+ does for an interpreter it cannot query.
#[cfg(unix)]
fn warning_uv_dir() -> tempfile::TempDir {
    use std::os::unix::fs::PermissionsExt;

    let dir = tempfile::TempDir::new().unwrap();
    let uv = dir.path().join("uv");
    std::fs::write(
        &uv,
        "#!/bin/sh\ncase \"$1 $2\" in\n  \"--version \"*) echo \"uv 0.13.7\" ;;\n  \"python list\")\n    case \"$*\" in *\"--color never\"*) ;; *) echo 'fake uv: --color never missing' >&2; exit 3 ;; esac\n    echo 'warning: Failed to query /opt/broken/bin/python3' >&2\n    echo '[{\"version\":\"3.12.14\",\"path\":\"/opt/py/bin/python3.12\",\"implementation\":\"cpython\"}]' ;;\n  *) echo \"fake uv: unsupported: $*\" >&2; exit 2 ;;\nesac\n",
    )
    .unwrap();
    std::fs::set_permissions(&uv, std::fs::Permissions::from_mode(0o755)).unwrap();
    dir
}

/// uv's warning about a left-out interpreter reaches the user, except
/// under --quiet and --json, where Output::warn stays silent too.
#[cfg(unix)]
#[rstest::rstest]
#[case::normal(&["list", "--pythons"], true)]
#[case::quiet(&["--quiet", "list", "--pythons"], false)]
#[case::json(&["list", "--pythons", "--json"], false)]
fn list_pythons_passes_on_uv_warnings(#[case] args: &[&str], #[case] shown: bool) {
    let fixture = TestFixture::new();
    let uv = warning_uv_dir();
    let out = scoop_cmd(&fixture.scoop_home)
        .env("PATH", format!("{}:/usr/bin:/bin", uv.path().display()))
        .args(args)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);

    assert!(out.status.success(), "{stderr}");
    assert!(
        stdout.contains("3.12.14"),
        "the listed Python is kept: {stdout}"
    );
    assert_eq!(
        stderr.contains("warning: Failed to query /opt/broken/bin/python3"),
        shown,
        "{stderr}"
    );
    if args.contains(&"--json") {
        serde_json::from_str::<serde_json::Value>(&stdout).expect("stdout stays one JSON document");
    }
}
