//! Shell integration: init scripts, completions, activate/deactivate,
//! resolve, and the commands the shell wrappers eval.

use crate::support::*;

/// Runs the real fish integration end to end when a `fish` binary is
/// installed (skipped otherwise; see `find_fish`). Every path here sources
/// a multi-line fish script, which only works if the wrapper pipes it to
/// `source` with an explicit `--shell fish`:
///
/// - sourcing `scuv init fish` runs the auto-activate hook once; with a
///   stale activation in the environment it must deactivate it;
/// - `scuv shell system` goes through the wrapper's
///   activate/deactivate/shell arm (its output always carries the
///   deactivation block);
/// - `scuv shell --shell fish system` must not get a second `--shell`;
/// - `scuv use system` must deactivate instead of activating the reserved
///   name.
///
/// Fails if a call site evals the output (fish rejoins the lines with
/// spaces and errors out), lets scuv guess the shell (fish does not export
/// FISH_VERSION, so the guess is bash), duplicates `--shell`, or routes
/// `use system` to `activate`.
#[test]
fn fish_wrapper_and_hook_source_multiline_scripts() {
    let Some(fish) = find_fish() else {
        eprintln!("skipping: no fish binary found");
        return;
    };
    let fixture = TestFixture::new();
    let bin = assert_cmd::cargo::cargo_bin("scuv");
    let bin_dir = bin.parent().unwrap();
    let path = format!(
        "{}:{}",
        bin_dir.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    // An empty config dir isolates fish from the developer's config.fish on
    // every fish version (`--no-config` only exists from 3.4).
    let config_home = fixture.temp_dir.path().join("xdg-config");
    std::fs::create_dir_all(&config_home).unwrap();
    let script = concat!(
        "command scuv init fish | source; ",
        "echo \"after-init SCUV_ACTIVE=[$SCUV_ACTIVE] VIRTUAL_ENV=[$VIRTUAL_ENV]\"; ",
        "scuv shell system; ",
        "echo \"shell-status=$status SCUV_VERSION=[$SCUV_VERSION]\"; ",
        "set -e SCUV_VERSION; ",
        "scuv shell --shell fish system; ",
        "echo \"explicit-status=$status SCUV_VERSION=[$SCUV_VERSION]\"; ",
        "set -gx SCUV_ACTIVE stale2; set -gx VIRTUAL_ENV /stale2; ",
        "scuv use system; ",
        "echo \"use-system SCUV_ACTIVE=[$SCUV_ACTIVE] VIRTUAL_ENV=[$VIRTUAL_ENV]\"",
    );
    let output = std::process::Command::new(fish)
        .args(["-c", script])
        .env("XDG_CONFIG_HOME", &config_home)
        .env("SCUV_HOME", &fixture.scoop_home)
        .env("SCUV_LANG", "en")
        .env("PATH", path)
        .env_remove("SCUV_VERSION")
        .env_remove("SCUV_NO_AUTO")
        .env_remove("_SCUV_OLD_PATH")
        .env_remove("_SCUV_OLD_PYTHONHOME")
        // A stale activation the startup hook must clear.
        .env("SCUV_ACTIVE", "stale")
        .env("VIRTUAL_ENV", "/stale")
        .current_dir(fixture.temp_dir.path())
        .output()
        .expect("fish must run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    for expected in [
        "after-init SCUV_ACTIVE=[] VIRTUAL_ENV=[]",
        "shell-status=0 SCUV_VERSION=[system]",
        "explicit-status=0 SCUV_VERSION=[system]",
        "use-system SCUV_ACTIVE=[] VIRTUAL_ENV=[]",
    ] {
        assert!(
            stdout.contains(expected),
            "missing {expected:?}.\nstdout:\n{stdout}\nstderr:\n{stderr}"
        );
    }
}

#[test]
fn test_init_bash() {
    Command::cargo_bin("scuv")
        .unwrap()
        .args(["init", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("scuv()"))
        .stdout(predicate::str::contains("_scuv_hook"));
}

#[test]
fn test_init_zsh() {
    Command::cargo_bin("scuv")
        .unwrap()
        .args(["init", "zsh"])
        .assert()
        .success()
        .stdout(predicate::str::contains("scuv()"))
        .stdout(predicate::str::contains("add-zsh-hook"));
}

// test_init_unsupported_shell removed: fish shell is now fully supported

#[test]
fn test_completions_bash() {
    Command::cargo_bin("scuv")
        .unwrap()
        .args(["completions", "bash"])
        .assert()
        .success();
}

#[test]
fn test_completions_zsh() {
    Command::cargo_bin("scuv")
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
        .stderr(predicate::str::contains("Can't find"));
}

/// The shell wrappers eval `activate`'s stdout, so a log line there runs as
/// a command. Corrupt metadata makes activation log a warning. Fails if the
/// tracing subscriber writes to stdout (its default) or ignores the color
/// decision.
#[test]
fn test_activate_keeps_log_warnings_off_stdout() {
    let fixture = fixture_with_active_env();
    let env = fixture.scoop_home.join("virtualenvs").join("demo");
    std::fs::write(env.join(".scoop-metadata.json"), "{ broken").unwrap();

    let out = scoop_cmd(&fixture.scoop_home)
        .env_remove("SCUV_ACTIVE")
        .args(["activate", "demo", "--shell", "bash"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stdout.contains("export VIRTUAL_ENV="), "{stdout}");
    assert!(
        !stdout.contains("WARN"),
        "log line on eval'd stdout: {stdout}"
    );
    assert!(
        stderr.contains("WARN"),
        "the warning should still be shown: {stderr}"
    );
    // stderr is a pipe here, so `auto` means no color, for the log line too.
    // Fails if tracing keeps its own ANSI default (`with_ansi` dropped).
    assert!(!stderr.contains(ESC), "tracing ignored --color: {stderr:?}");
}

/// Runs the real bash completion function over one command line and
/// returns what it offers.
fn bash_complete(fixture: &TestFixture, words: &[&str]) -> Vec<String> {
    let script = scoop_cmd(&fixture.scoop_home)
        .args(["init", "bash"])
        .output()
        .unwrap()
        .stdout;
    let quoted: Vec<String> = words.iter().map(|w| format!("'{w}'")).collect();
    let driver = format!(
        "COMP_WORDS=({}); COMP_CWORD=$((${{#COMP_WORDS[@]}}-1)); \
         cur=\"${{COMP_WORDS[COMP_CWORD]}}\"; _scuv_complete; \
         printf '%s\\n' \"${{COMPREPLY[@]}}\"",
        quoted.join(" ")
    );
    let out = std::process::Command::new("bash")
        .args(["--norc", "-c"])
        .arg(format!("{}\n{driver}", String::from_utf8(script).unwrap()))
        .env("SCUV_HOME", &fixture.scoop_home)
        .env("SCUV_NO_AUTO", "1")
        .output()
        .expect("bash runs");
    String::from_utf8(out.stdout)
        .unwrap()
        .lines()
        .map(str::to_string)
        .collect()
}

/// The global options come from one list, so every subcommand offers them,
/// and options already on the line (with their exclusive partners) drop
/// out. Fails if `_scuv_offer` loses the global list, the `-q/--quiet` or
/// subcommand groups, or the `--opt=value` form.
#[test]
fn test_bash_completion_offers_global_options_once() {
    let fixture = TestFixture::new();
    let has = |words: &[&str], opt: &str| bash_complete(&fixture, words).iter().any(|o| o == opt);

    for cmd in ["list", "lang", "migrate", "status"] {
        assert!(has(&["scuv", cmd, "--"], "--color"), "{cmd} lacks --color");
    }
    assert!(
        !has(&["scuv", "list", "-q", "--"], "--quiet"),
        "-q drops --quiet"
    );
    assert!(
        !has(&["scuv", "use", "--link", "--"], "--no-link"),
        "--link drops --no-link"
    );
    assert!(
        !has(&["scuv", "list", "--sort=name", "--"], "--sort"),
        "--sort=x drops --sort"
    );
    assert!(has(&["scuv", "list", "--sort=name", "--"], "--json"));
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

/// `scuv resolve` reads `.scuv-version` end-to-end through the real binary.
#[test]
fn test_resolve_with_version_file() {
    let fixture = TestFixture::new();

    std::fs::write(fixture.temp_dir.path().join(".scuv-version"), "testenv").unwrap();

    scoop_cmd(&fixture.scoop_home)
        .arg("resolve")
        .current_dir(fixture.temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("testenv"));
}

/// The scoop-era `.scoop-version` name is not a version file any more: with
/// only that file present, `scuv resolve` finds nothing in the directory.
/// Fails if a `.scoop-version` fallback is reintroduced.
#[test]
fn test_resolve_ignores_legacy_version_file() {
    let fixture = TestFixture::new();

    std::fs::write(fixture.temp_dir.path().join(".scoop-version"), "testenv").unwrap();

    scoop_cmd(&fixture.scoop_home)
        .arg("resolve")
        .current_dir(fixture.temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("testenv").not());
}

#[test]
#[ignore = "requires uv to be installed"]
fn test_shell_outputs_activation_script() {
    let fixture = TestFixture::new();

    // First create an environment
    scoop_cmd(&fixture.scoop_home)
        .args(["create", "shelltest", "3.12"])
        .assert()
        .success();

    // Test shell command outputs activation script
    scoop_cmd(&fixture.scoop_home)
        .args(["shell", "shelltest"])
        .assert()
        .success()
        .stdout(predicate::str::contains("SCUV_VERSION"))
        .stdout(predicate::str::contains("VIRTUAL_ENV"));
}

#[test]
fn test_shell_system_outputs_deactivation() {
    let fixture = TestFixture::new();

    // Explicitly specify bash to avoid CI environment detecting PowerShell
    scoop_cmd(&fixture.scoop_home)
        .args(["shell", "--shell", "bash", "system"])
        .assert()
        .success()
        // Security: verify quotes are present to prevent shell injection
        .stdout(predicate::str::contains(r#"export SCUV_VERSION="system""#))
        .stdout(predicate::str::contains("unset VIRTUAL_ENV"));
}

#[test]
fn test_shell_bash_exports_quoted_version() {
    let fixture = TestFixture::new();

    // Explicitly test bash shell output format with quotes
    scoop_cmd(&fixture.scoop_home)
        .args(["shell", "--shell", "bash", "system"])
        .assert()
        .success()
        // Security: double quotes prevent shell injection
        .stdout(predicate::str::contains(r#"export SCUV_VERSION="system""#))
        // The scoop-era name is no longer exported alongside.
        // Fails if `print_export_scoop_version` emits SCOOP_VERSION again.
        .stdout(predicate::str::contains("SCOOP_VERSION").not());
}

#[test]
fn test_shell_fish_uses_single_quotes() {
    let fixture = TestFixture::new();

    // Fish shell should use single quotes for SCUV_VERSION
    scoop_cmd(&fixture.scoop_home)
        .args(["shell", "--shell", "fish", "system"])
        .assert()
        .success()
        // Fish uses single quotes which also prevent injection
        .stdout(predicate::str::contains("set -gx SCUV_VERSION 'system'"));
}

#[test]
fn test_shell_unset_clears_version() {
    let fixture = TestFixture::new();

    // Explicitly specify bash to avoid CI environment detecting PowerShell
    scoop_cmd(&fixture.scoop_home)
        .args(["shell", "--shell", "bash", "--unset"])
        .assert()
        .success()
        .stdout(predicate::str::contains("unset SCUV_VERSION"))
        // Fails if `print_unset_scoop_version` clears SCOOP_VERSION again.
        .stdout(predicate::str::contains("SCOOP_VERSION").not());
}

#[test]
fn test_use_system_creates_version_file() {
    let fixture = TestFixture::new();
    let project_dir = fixture.temp_dir.path().join("project");
    std::fs::create_dir_all(&project_dir).unwrap();

    scoop_cmd(&fixture.scoop_home)
        .current_dir(&project_dir)
        .args(["use", "system"])
        .assert()
        .success();

    let version_file = project_dir.join(".scuv-version");
    assert!(
        version_file.exists(),
        ".scuv-version file should be created"
    );
    let content = std::fs::read_to_string(&version_file).unwrap();
    assert_eq!(content.trim(), "system");
}

#[test]
#[ignore = "requires uv to be installed"]
fn test_shell_fish_output_format() {
    let fixture = TestFixture::new();

    // First create an environment
    scoop_cmd(&fixture.scoop_home)
        .args(["create", "fishenv", "3.12"])
        .assert()
        .success();

    // Test fish-specific output format
    scoop_cmd(&fixture.scoop_home)
        .args(["shell", "--shell", "fish", "fishenv"])
        .assert()
        .success()
        .stdout(predicate::str::contains("set -gx SCUV_VERSION"))
        .stdout(predicate::str::contains("set -gx VIRTUAL_ENV"));
}
