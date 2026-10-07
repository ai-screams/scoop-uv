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

/// An unknown language code is an error: exit 1 with the hint, and the
/// config keeps no language. It printed the error and still exited 0, so a
/// script setting the language could not tell that nothing changed.
#[test]
fn lang_rejects_an_unsupported_code_with_exit_1() {
    let fixture = TestFixture::new();

    scoop_cmd(&fixture.scoop_home)
        .args(["lang", "xx"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("Unsupported language: xx"))
        .stderr(predicate::str::contains("scuv lang --list"));
    assert!(!fixture.scoop_home.join("config.json").exists());
}

/// Writing to a pipe whose reader is gone ends the process by SIGPIPE, as it
/// does `cat`, instead of panicking: `scuv info x | true` printed a crash
/// report and exited 101. The read end is closed before scuv starts, so the
/// first write always hits EPIPE.
#[cfg(unix)]
#[test]
fn closed_stdout_ends_quietly_by_sigpipe() {
    use std::os::fd::{FromRawFd, OwnedFd};
    use std::os::unix::process::ExitStatusExt;

    let fixture = TestFixture::new();
    let mut fds = [0; 2];
    // SAFETY: `fds` has room for the two descriptors pipe(2) writes.
    assert_eq!(unsafe { libc::pipe(fds.as_mut_ptr()) }, 0);
    // SAFETY: pipe(2) succeeded, so both descriptors are open and ours.
    let (read_end, write_end) =
        unsafe { (OwnedFd::from_raw_fd(fds[0]), OwnedFd::from_raw_fd(fds[1])) };
    drop(read_end);

    let out = std::process::Command::new(assert_cmd::cargo::cargo_bin("scuv"))
        .args(["lang", "--list"])
        .env("SCUV_HOME", &fixture.scoop_home)
        .env("SCUV_LANG", "en")
        .stdout(std::process::Stdio::from(write_end))
        .stderr(std::process::Stdio::piped())
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        out.status.signal(),
        Some(libc::SIGPIPE),
        "status {:?}, stderr:\n{stderr}",
        out.status
    );
    assert!(!stderr.contains("panicked"), "stderr:\n{stderr}");
}
