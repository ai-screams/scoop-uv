//! Top-level behaviour: help, version, unknown subcommands.

use crate::support::*;

#[test]
fn test_help_flag() {
    Command::cargo_bin("scuv")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("scuv"))
        .stdout(predicate::str::contains("COMMAND"));
}

#[test]
fn test_version_flag() {
    Command::cargo_bin("scuv")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("scuv"));
}

#[test]
fn test_unknown_subcommand() {
    Command::cargo_bin("scuv")
        .unwrap()
        .arg("unknowncommand")
        .assert()
        .failure();
}
