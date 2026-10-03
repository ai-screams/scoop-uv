//! Every subcommand reaches its handler.
//!
//! `main.rs`'s `dispatch` is one arm per command; coverage showed most arms
//! never ran in tests, so a command routed to the wrong handler, or an
//! argument passed to the wrong parameter, would go unnoticed. Each test
//! runs one command against an empty home and a fake `uv`, and checks for
//! something only that command's handler produces.

use crate::support::*;

/// Runs `scuv <args>` in an empty project dir with only the fake uv (and
/// the system bin dirs) on PATH. Returns (stdout, stderr).
#[cfg(unix)]
fn run(args: &[&str]) -> (String, String) {
    let fixture = TestFixture::new();
    let uv = fake_uv_dir();
    let project = fixture.temp_dir.path().join("project");
    std::fs::create_dir_all(&project).unwrap();
    let out = scoop_cmd(&fixture.scoop_home)
        .current_dir(&project)
        .env("PATH", format!("{}:/usr/bin:/bin", uv.path().display()))
        .env_remove("SCUV_ACTIVE")
        .env_remove("VIRTUAL_ENV")
        .args(args)
        .output()
        .unwrap();
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// `--json` commands answer with their own command name.
#[cfg(unix)]
#[test]
fn json_commands_reach_their_handlers() {
    for (args, command) in [
        (&["migrate", "list", "--json"][..], "migrate list"),
        (&["status", "--json"][..], "status"),
        (&["prune", "--json"][..], "prune"),
        (&["gc", "--json"][..], "gc"),
        (&["verify", "--json"][..], "verify"),
    ] {
        let (stdout, stderr) = run(args);
        let json: serde_json::Value = serde_json::from_str(&stdout)
            .unwrap_or_else(|e| panic!("{args:?}: not JSON ({e}): {stdout}{stderr}"));
        assert_eq!(json["command"], command, "{args:?}");
    }
    let (stdout, _) = run(&["doctor", "--json"]);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert!(json["checks"].is_array(), "doctor --json: {stdout}");
}

/// Commands that take an env name pass *that* argument to their handler.
/// For clone the source is looked up first, so swapping source and
/// destination would name the wrong env.
#[cfg(unix)]
#[test]
fn env_name_commands_reach_their_handlers() {
    for args in [
        &["info", "src-env"][..],
        &["run", "src-env", "--", "echo", "hi"][..],
        &["export", "src-env"][..],
        &["clone", "src-env", "dst-env"][..],
        &["diff", "src-env", "dst-env"][..],
    ] {
        let (_, stderr) = run(args);
        assert!(
            stderr.contains("Can't find 'src-env'"),
            "{args:?}: {stderr}"
        );
    }
}

/// The rest, each by a message only its handler prints.
#[cfg(unix)]
#[test]
fn other_commands_reach_their_handlers() {
    let cases: &[(&[&str], &str)] = &[
        (&["lang", "--list"], "Supported languages"),
        (&["uninstall", "3.99"], "Couldn't uninstall Python 3.99"),
        (&["sync", "--dry-run"], "No .scuv.toml found"),
        (&["which", "python"], "No active environment"),
        (&["import", "/nonexistent/export.json"], "No such file"),
        (&["man"], ".TH scuv 1"),
    ];
    for (args, needle) in cases {
        let (stdout, stderr) = run(args);
        assert!(
            stdout.contains(needle) || stderr.contains(needle),
            "{args:?}: expected {needle:?} in\n{stdout}{stderr}"
        );
    }
}

/// `self update` with no cargo on PATH stops at its first step. PATH holds
/// only the fake uv, so this can never install anything.
#[cfg(unix)]
#[test]
fn self_update_reaches_its_handler() {
    let fixture = TestFixture::new();
    let uv = fake_uv_dir();
    let out = scoop_cmd(&fixture.scoop_home)
        .env("PATH", uv.path())
        .args(["self", "update", "--no-verify"])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success());
    assert!(stderr.contains("cargo search"), "{stderr}");
}
