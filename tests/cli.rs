//! CLI integration tests
//!
//! These tests verify the CLI behavior using `assert_cmd`.
//! Note: Tests that require Python installation (create, activate) are marked as `#[ignore]`
//! to allow running in CI environments without uv/Python installed.

// `Command::cargo_bin()` is deprecated since assert_cmd 2.1.0 due to
// incompatibility with custom cargo build directories. The recommended
// replacement is `escargot` crate for more flexible binary building.
// For now, we allow deprecated usage as it works correctly for standard
// cargo layouts. See: https://docs.rs/assert_cmd/latest/assert_cmd/cargo/
// TODO: Consider migrating to escargot if custom build-dir support is needed.
#![allow(deprecated)]

use assert_cmd::Command;
use predicates::prelude::*;
use std::path::PathBuf;
use tempfile::TempDir;

/// Test fixture for scoop tests
struct TestFixture {
    /// Temporary directory - held to prevent cleanup until fixture is dropped
    #[allow(dead_code)]
    temp_dir: TempDir,
    /// SCUV_HOME path
    scoop_home: PathBuf,
}

impl TestFixture {
    /// Create a new test fixture with isolated SCUV_HOME
    fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let scoop_home = temp_dir.path().join(".scuv");
        Self {
            temp_dir,
            scoop_home,
        }
    }
}

/// Helper to get a fresh command with SCUV_HOME set
fn scoop_cmd(scoop_home: &std::path::Path) -> Command {
    let mut cmd = Command::cargo_bin("scuv").unwrap();
    cmd.env("SCUV_HOME", scoop_home);
    // Force English locale for consistent test assertions
    cmd.env("SCUV_LANG", "en");
    cmd
}

/// Locate a fish binary: PATH first, then the usual install prefixes. When
/// `SCUV_REQUIRE_FISH` is set a missing fish is a failure, not a skip: the CI
/// Test and MSRV jobs install fish and set that variable, so a silently
/// skipped test cannot look like a pass there. Jobs that run the suite
/// without fish (coverage, mutants) leave it unset and skip.
fn find_fish() -> Option<std::path::PathBuf> {
    let from_path = std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths)
            .map(|dir| dir.join("fish"))
            .find(|p| p.is_file())
    });
    let found = from_path.or_else(|| {
        [
            "/opt/homebrew/bin/fish",
            "/usr/local/bin/fish",
            "/usr/bin/fish",
        ]
        .iter()
        .map(std::path::PathBuf::from)
        .find(|p| p.is_file())
    });
    if found.is_none() && std::env::var_os("SCUV_REQUIRE_FISH").is_some() {
        panic!("SCUV_REQUIRE_FISH is set but no fish binary was found");
    }
    found
}

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

const ESC: &str = "\x1b[";

/// A home with one env, `demo`, that `SCUV_ACTIVE` marks active so `list`
/// highlights it (the only colored stdout).
fn fixture_with_active_env() -> TestFixture {
    let fixture = TestFixture::new();
    let env = fixture.scoop_home.join("virtualenvs").join("demo");
    std::fs::create_dir_all(env.join("bin")).unwrap();
    std::fs::write(env.join("pyvenv.cfg"), "version = 3.12.1\n").unwrap();
    fixture
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

#[test]
fn test_remove_nonexistent_env() {
    let fixture = TestFixture::new();

    scoop_cmd(&fixture.scoop_home)
        .args(["remove", "nonexistent"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Can't find"));
}

/// Builds an env directory by hand (no uv needed: `remove` only deletes it)
/// and a project directory whose `.venv` symlinks to `link_target`.
#[cfg(unix)]
fn env_and_project_with_link(
    fixture: &TestFixture,
    env: &str,
    link_target: Option<&std::path::Path>,
) -> (PathBuf, PathBuf) {
    let env_path = fixture.scoop_home.join("virtualenvs").join(env);
    std::fs::create_dir_all(&env_path).unwrap();
    let project = fixture.temp_dir.path().join("project");
    std::fs::create_dir_all(&project).unwrap();
    let target = link_target.map_or(env_path.clone(), std::path::Path::to_path_buf);
    std::os::unix::fs::symlink(&target, project.join(".venv")).unwrap();
    (env_path, project)
}

/// #202: removing an env from the project that `scuv use --link`-ed it must
/// not leave a dangling `.venv` behind. Fails if `remove` never calls the
/// link cleanup or drops its result.
#[cfg(unix)]
#[test]
fn test_remove_deletes_venv_link_to_removed_env() {
    let fixture = TestFixture::new();
    let (env_path, project) = env_and_project_with_link(&fixture, "linked", None);

    scoop_cmd(&fixture.scoop_home)
        .current_dir(&project)
        .args(["remove", "--force", "linked"])
        .assert()
        .success()
        .stderr(predicate::str::contains("Removed .venv link"));

    assert!(!env_path.exists());
    assert!(
        !project.join(".venv").is_symlink(),
        ".venv link to the removed env must be gone"
    );
}

/// A `.venv` link to anything other than the removed env is not ours to touch.
/// Fails if the target comparison is dropped.
#[cfg(unix)]
#[test]
fn test_remove_keeps_venv_link_to_other_target() {
    let fixture = TestFixture::new();
    let other = fixture.temp_dir.path().join("other-venv");
    std::fs::create_dir_all(&other).unwrap();
    let (_, project) = env_and_project_with_link(&fixture, "linked", Some(&other));

    scoop_cmd(&fixture.scoop_home)
        .current_dir(&project)
        .args(["remove", "--force", "linked"])
        .assert()
        .success()
        .stderr(predicate::str::contains(".venv link").not());

    assert_eq!(std::fs::read_link(project.join(".venv")).unwrap(), other);
}

/// The link may reach the env through an alias of `SCUV_HOME` (a symlinked
/// home directory). Fails if the comparison is lexical.
#[cfg(unix)]
#[test]
fn test_remove_deletes_venv_link_through_home_alias() {
    let fixture = TestFixture::new();
    let alias = fixture.temp_dir.path().join("home-alias");
    let env_path = fixture.scoop_home.join("virtualenvs").join("linked");
    std::fs::create_dir_all(&env_path).unwrap();
    std::os::unix::fs::symlink(&fixture.scoop_home, &alias).unwrap();
    let via_alias = alias.join("virtualenvs").join("linked");
    let (_, project) = env_and_project_with_link(&fixture, "linked", Some(&via_alias));

    scoop_cmd(&fixture.scoop_home)
        .current_dir(&project)
        .args(["remove", "--force", "linked"])
        .assert()
        .success();

    assert!(!project.join(".venv").is_symlink());
}

/// Makes a directory read-only so unlinking inside it fails, and restores it
/// on drop (a panicking assertion must not leave an undeletable tempdir).
#[cfg(unix)]
struct ReadOnlyDir(PathBuf);

#[cfg(unix)]
impl ReadOnlyDir {
    /// `None` when the lock has no effect: root ignores directory permissions
    /// (the Docker integration jobs run as root), so there is nothing to test.
    fn lock(dir: &std::path::Path) -> Option<Self> {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o555)).unwrap();
        let guard = Self(dir.to_path_buf());
        if std::fs::write(dir.join("probe"), b"").is_ok() {
            eprintln!("skipped: directory permissions are not enforced (root?)");
            return None;
        }
        Some(guard)
    }
}

#[cfg(unix)]
impl Drop for ReadOnlyDir {
    fn drop(&mut self) {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&self.0, std::fs::Permissions::from_mode(0o755));
    }
}

/// The env is deleted before the link, so a link that cannot be removed must
/// not turn the finished `remove` into a failure. Fails if the unlink error
/// is propagated with `?`.
#[cfg(unix)]
#[test]
fn test_remove_warns_when_venv_link_cannot_be_removed() {
    let fixture = TestFixture::new();
    let (env_path, project) = env_and_project_with_link(&fixture, "linked", None);
    let Some(_lock) = ReadOnlyDir::lock(&project) else {
        return;
    };

    scoop_cmd(&fixture.scoop_home)
        .current_dir(&project)
        .args(["remove", "--force", "linked"])
        .assert()
        .success()
        .stderr(predicate::str::contains("Could not remove .venv link"));
    assert!(!env_path.exists());
    assert!(project.join(".venv").is_symlink());
}

/// `warn` is silent under `--json`, so the failure must be in the payload:
/// without it a script cannot tell "link left behind" from "no link". Fails
/// if `unlink_error` is not threaded into `RemoveData`.
#[cfg(unix)]
#[test]
fn test_remove_json_reports_unlink_error() {
    let fixture = TestFixture::new();
    let (_, project) = env_and_project_with_link(&fixture, "linked", None);
    let Some(_lock) = ReadOnlyDir::lock(&project) else {
        return;
    };

    let out = scoop_cmd(&fixture.scoop_home)
        .current_dir(&project)
        .args(["remove", "--json", "linked"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(json["data"].get("unlinked").is_none());
    assert!(
        json["data"]["unlink_error"]
            .as_str()
            .is_some_and(|e| !e.is_empty())
    );
}

/// JSON reports the removed link as `unlinked`, and omits the field otherwise.
/// Fails if the removed path is not threaded into `RemoveData`.
#[cfg(unix)]
#[test]
fn test_remove_json_reports_unlinked_path() {
    let fixture = TestFixture::new();
    let (_, project) = env_and_project_with_link(&fixture, "linked", None);

    let out = scoop_cmd(&fixture.scoop_home)
        .current_dir(&project)
        .args(["remove", "--json", "linked"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    // The child resolves its cwd through getcwd, which canonicalizes
    // (/var -> /private/var on macOS).
    let link = project.canonicalize().unwrap().join(".venv");
    assert_eq!(
        json["data"]["unlinked"].as_str(),
        Some(link.to_str().unwrap())
    );

    // Second env, no link this time: the field is absent.
    let plain = fixture.scoop_home.join("virtualenvs").join("plain");
    std::fs::create_dir_all(&plain).unwrap();
    let out = scoop_cmd(&fixture.scoop_home)
        .current_dir(&project)
        .args(["remove", "--json", "plain"])
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(json["data"].get("unlinked").is_none());
}

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
fn test_unknown_subcommand() {
    Command::cargo_bin("scuv")
        .unwrap()
        .arg("unknowncommand")
        .assert()
        .failure();
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

// =============================================================================
// Error Case Tests
// =============================================================================

mod error_cases {
    use super::*;

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
}

// =============================================================================
// Output Format Tests (Real assertions, not fake snapshots)
// =============================================================================

mod output_format {
    use super::*;

    #[test]
    fn test_version_output_format() {
        let output = Command::cargo_bin("scuv")
            .unwrap()
            .arg("--version")
            .output()
            .unwrap();

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Version format should be "scuv X.Y.Z"
        assert!(
            stdout.starts_with("scuv "),
            "Version should start with 'scuv '"
        );

        // Verify semver format (X.Y.Z)
        let version_part = stdout.trim().strip_prefix("scuv ").unwrap();
        let parts: Vec<&str> = version_part.split('.').collect();
        assert_eq!(parts.len(), 3, "Version should be semver format X.Y.Z");
        for (i, part) in parts.iter().enumerate() {
            assert!(
                part.chars().all(|c| c.is_ascii_digit()),
                "Version part {} ('{}') should be numeric",
                i,
                part
            );
        }
    }

    #[test]
    fn test_help_has_required_sections() {
        let output = Command::cargo_bin("scuv")
            .unwrap()
            .arg("--help")
            .output()
            .unwrap();

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Help must have these sections
        assert!(stdout.contains("Usage:"), "Help missing 'Usage:' section");
        assert!(
            stdout.contains("Commands:"),
            "Help missing 'Commands:' section"
        );
        assert!(
            stdout.contains("Options:"),
            "Help missing 'Options:' section"
        );
    }

    #[test]
    fn test_help_lists_all_subcommands() {
        let output = Command::cargo_bin("scuv")
            .unwrap()
            .arg("--help")
            .output()
            .unwrap();

        let stdout = String::from_utf8_lossy(&output.stdout);

        // User-facing subcommands that should be visible in help
        // Note: activate/deactivate are hidden (shell wrapper handles them)
        let visible_subcommands = [
            "list", "create", "remove", "use", "install", "init", "shell",
        ];

        for cmd in visible_subcommands {
            assert!(
                stdout.contains(cmd),
                "Help missing required subcommand: {}",
                cmd
            );
        }
    }

    #[test]
    fn test_error_message_is_helpful() {
        let fixture = TestFixture::new();

        let output = scoop_cmd(&fixture.scoop_home)
            .args(["activate", "nonexistent"])
            .output()
            .unwrap();

        let stderr = String::from_utf8_lossy(&output.stderr);

        // Error messages should be helpful
        assert!(
            stderr.to_lowercase().contains("error") || !output.status.success(),
            "Failed command should indicate error"
        );
        assert!(
            stderr.contains("nonexistent"),
            "Error should mention the problematic env name"
        );
        assert!(
            stderr.contains("Can't find"),
            "Error should explain the problem"
        );
    }
}

// Tests requiring uv and Python - mark as #[ignore]
mod requires_uv {
    use super::*;

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
}

// =============================================================================
// Shell Commands Tests (scoop shell, scoop use system)
// =============================================================================

mod shell_commands {
    use super::*;

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
}
