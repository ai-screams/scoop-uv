//! `scuv migrate` against a fake pyenv tree.

use crate::support::*;

/// A pyenv tree under `root` holding one env, `name`, with a stub
/// interpreter: enough for discovery to report it without running Python.
fn fake_pyenv_env(root: &std::path::Path, name: &str) {
    let env = root.join("versions/3.12.14/envs").join(name);
    std::fs::create_dir_all(env.join("bin")).unwrap();
    std::fs::write(
        env.join("pyvenv.cfg"),
        "home = /usr/bin\nversion_info = 3.12.14\n",
    )
    .unwrap();
    let python = env.join("bin/python");
    std::fs::write(&python, "#!/bin/sh\necho Python 3.12.14\n").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&python, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
}

/// A name conflict with no flag and no terminal to prompt on is a name
/// conflict (exit 2), not "Dialog error: not a terminal" (exit 1).
/// assert_cmd gives the child piped stdio, so it never has a terminal.
#[cfg(unix)]
#[test]
fn migrate_env_name_conflict_without_a_terminal_exits_2() {
    let fixture = TestFixture::new();
    let pyenv = fixture.temp_dir.path().join("pyenv");
    fake_pyenv_env(&pyenv, "dup");
    std::fs::create_dir_all(fixture.scoop_home.join("virtualenvs/dup")).unwrap();

    scoop_cmd(&fixture.scoop_home)
        .args(["migrate", "@env", "dup"])
        .env("PYENV_ROOT", &pyenv)
        .env("HOME", fixture.temp_dir.path())
        .assert()
        .code(2)
        .stderr(predicate::str::contains("'dup' already exists"))
        .stderr(predicate::str::contains("Dialog error").not());
}

/// `migrate list` reports each env's size; it printed "- MB" and
/// `"size_bytes": null` for every env because nothing ever measured it.
#[cfg(unix)]
#[test]
fn migrate_list_reports_each_env_size() {
    let fixture = TestFixture::new();
    let pyenv = fixture.temp_dir.path().join("pyenv");
    fake_pyenv_env(&pyenv, "sized");
    let env = pyenv.join("versions/3.12.14/envs/sized");
    std::fs::write(env.join("payload"), vec![0u8; 4096]).unwrap();

    let out = scoop_cmd(&fixture.scoop_home)
        .args(["migrate", "list", "--source", "pyenv", "--json"])
        .env("PYENV_ROOT", &pyenv)
        .env("HOME", fixture.temp_dir.path())
        .output()
        .unwrap();
    assert!(out.status.success());
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let size = json["data"]["environments"][0]["size_bytes"].as_u64();
    // The three files written above: pyvenv.cfg, bin/python and payload.
    let expected = ["pyvenv.cfg", "bin/python", "payload"]
        .iter()
        .map(|f| std::fs::metadata(env.join(f)).unwrap().len())
        .sum::<u64>();
    assert_eq!(size, Some(expected), "{json}");
}
