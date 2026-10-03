//! Color decisions: `--color`, `--no-color`, `NO_COLOR` (#204, #205).

use crate::support::*;

/// #204: clap used to parse NO_COLOR's value as a bool, so the usual
/// `NO_COLOR=1` (and an empty value) aborted every subcommand. Fails if
/// `env = "NO_COLOR"` comes back on the `--no-color` argument.
#[test]
fn test_no_color_env_accepts_any_value() {
    let fixture = TestFixture::new();
    for value in ["1", "", "yes"] {
        scoop_cmd(&fixture.scoop_home)
            .env("NO_COLOR", value)
            .arg("list")
            .assert()
            .success();
    }
}

fn list_stdout(fixture: &TestFixture, args: &[&str], no_color_env: Option<&str>) -> String {
    let mut cmd = scoop_cmd(&fixture.scoop_home);
    cmd.env("SCUV_ACTIVE", "demo").env_remove("NO_COLOR");
    if let Some(value) = no_color_env {
        cmd.env("NO_COLOR", value);
    }
    let out = cmd.args(args).arg("list").output().unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

/// #205: test stdout is a pipe, so `auto` must not color it. Fails if `auto`
/// skips the terminal check.
#[test]
fn test_color_auto_leaves_piped_output_plain() {
    let fixture = fixture_with_active_env();
    let stdout = list_stdout(&fixture, &[], None);
    assert!(stdout.contains("demo"), "{stdout}");
    assert!(
        !stdout.contains(ESC),
        "piped list must be plain: {stdout:?}"
    );
}

/// `always` colors a pipe, and an explicit flag beats NO_COLOR. Fails if the
/// flag is ignored or NO_COLOR overrides it.
#[test]
fn test_color_always_colors_pipes_even_with_no_color_env() {
    let fixture = fixture_with_active_env();
    assert!(list_stdout(&fixture, &["--color", "always"], None).contains(ESC));
    assert!(list_stdout(&fixture, &["--color=always"], Some("1")).contains(ESC));
}

/// `--no-color` and `--color never` both mean never, and whichever comes
/// last wins. Fails if `color_choice` ignores `--no-color` or the two flags
/// stop overriding each other.
#[test]
fn test_no_color_and_color_flags_override_each_other() {
    let fixture = fixture_with_active_env();
    assert!(!list_stdout(&fixture, &["--color", "always", "--no-color"], None).contains(ESC));
    assert!(list_stdout(&fixture, &["--no-color", "--color", "always"], None).contains(ESC));
}

/// scuv's own messages (stderr) follow the same choice. Fails if `Output`
/// is built without the startup decision.
#[test]
fn test_color_choice_reaches_stderr_messages() {
    let fixture = TestFixture::new();
    let stderr = |args: &[&str]| {
        let out = scoop_cmd(&fixture.scoop_home)
            .env_remove("NO_COLOR")
            .args(args)
            .args(["remove", "--force", "missing"])
            .output()
            .unwrap();
        String::from_utf8(out.stderr).unwrap()
    };
    assert!(!stderr(&[]).contains(ESC), "auto on a pipe stays plain");
    assert!(stderr(&["--color", "always"]).contains(ESC));
}

/// clap prints parse errors before `Cli` exists, so it only sees the early
/// scan of the raw arguments. Fails if clap is not given that choice.
#[test]
fn test_color_choice_reaches_clap_errors() {
    let fixture = TestFixture::new();
    let stderr = |args: &[&str]| {
        let out = scoop_cmd(&fixture.scoop_home)
            .env_remove("NO_COLOR")
            .args(args)
            .arg("no-such-command")
            .output()
            .unwrap();
        assert!(!out.status.success());
        String::from_utf8(out.stderr).unwrap()
    };
    assert!(stderr(&["--color", "always"]).contains(ESC));
    assert!(!stderr(&["--color", "always", "--no-color"]).contains(ESC));
}
