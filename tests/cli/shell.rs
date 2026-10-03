//! Shell integration: init scripts, completions, activate/deactivate,
//! resolve, and the commands the shell wrappers eval.

use crate::support::*;

/// Runs the real fish integration end to end when a `fish` binary is
/// installed (skipped otherwise; see `find_shell`). Every path here sources
/// a multi-line fish script, which only works if the wrapper pipes it to
/// `source` with an explicit `--shell fish`:
///
/// - sourcing `scuv init fish` runs the auto-activate hook once; with a
///   stale activation in the environment it must deactivate it;
/// - `scuv shell system` goes through the wrapper's
///   activate/deactivate/shell arm (its output always carries the
///   deactivation block);
/// - `scuv shell --shell fish system` must not get a second `--shell`;
/// - `scuv use SYSTEM` must deactivate instead of activating the reserved
///   name (`use` takes it in any case);
/// - a failed `scuv activate` must keep its exit status.
///
/// Fails if a call site evals the output (fish rejoins the lines with
/// spaces and errors out), lets scuv guess the shell (fish does not export
/// FISH_VERSION, so the guess is bash), duplicates `--shell`, routes
/// `use system` to `activate`, or loses scuv's exit status.
#[test]
fn fish_wrapper_and_hook_source_multiline_scripts() {
    let Some(fish) = find_shell("fish") else {
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
        "scuv use SYSTEM; ",
        "echo \"use-system SCUV_ACTIVE=[$SCUV_ACTIVE] VIRTUAL_ENV=[$VIRTUAL_ENV]\"; ",
        "scuv activate --shell fish no-such-env 2>/dev/null; ",
        "echo \"missing-status=$status\"; ",
        // A usage error exits 2; the wrapper's own `string match` before it
        // leaves 1, so only scuv's status can make this read 2.
        "scuv activate --no-such-flag 2>/dev/null; ",
        "echo \"usage-status=$status\"",
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
        "missing-status=1",
        "usage-status=2",
    ] {
        assert!(
            stdout.contains(expected),
            "missing {expected:?}.\nstdout:\n{stdout}\nstderr:\n{stderr}"
        );
    }
}

/// A home with one env, `data-hub`: its name contains `-h`, which the
/// wrappers once took for a help flag. A fake uv lets `activate` find it.
#[cfg(unix)]
fn fixture_with_data_hub() -> (TestFixture, TempDir) {
    let fixture = TestFixture::new();
    let env = fixture.scoop_home.join("virtualenvs").join("data-hub");
    std::fs::create_dir_all(env.join("bin")).unwrap();
    std::fs::write(env.join("pyvenv.cfg"), "version = 3.12.1\n").unwrap();
    (fixture, fake_uv_dir())
}

/// PATH with the built scuv first, then the fake uv's directory holding a
/// second `scuv` (a link to the same binary): two installs on PATH, as a
/// user with both a cargo and a package-manager copy has.
#[cfg(unix)]
fn shell_test_path(fake_uv: &TempDir) -> String {
    let bin = assert_cmd::cargo::cargo_bin("scuv");
    let second = fake_uv.path().join("scuv");
    if !second.exists() {
        std::os::unix::fs::symlink(&bin, &second).unwrap();
    }
    format!(
        "{}:{}:{}",
        bin.parent().unwrap().display(),
        fake_uv.path().display(),
        std::env::var("PATH").unwrap_or_default()
    )
}

/// Runs the real bash and zsh integration end to end (each skipped when the
/// shell is missing; see `find_shell`). `PSModulePath` is set, as Windows
/// sets it for every process (Git Bash included): shell detection reads it
/// before `ZSH_VERSION`, so any call that lets scuv guess the shell gets
/// PowerShell syntax.
///
/// Fails if a wrapper or hook call omits `--shell` (the stale activation
/// survives init, or eval chokes on PowerShell), duplicates an explicit
/// `--shell`, routes `use system` (in any case) to `activate`, loses scuv's
/// exit status behind `eval`, or treats an env name
/// containing `-h` as a help flag (`activate data-hub` then prints instead
/// of activating). On bash older than 4.4 (macOS's 3.2) it also fails if
/// completion registration depends on `complete -o nosort`.
#[cfg(unix)]
#[test]
fn posix_wrappers_and_hook_name_their_shell() {
    let shells: [(&str, &[&str]); 2] = [("bash", &["--norc", "--noprofile"]), ("zsh", &["-f"])];
    for (shell, flags) in shells {
        let Some(binary) = find_shell(shell) else {
            eprintln!("skipping: no {shell} binary found");
            continue;
        };
        let (fixture, fake_uv) = fixture_with_data_hub();
        let script = format!(
            concat!(
                "eval \"$(command scuv init {sh})\"\n",
                "echo \"after-init SCUV_ACTIVE=[$SCUV_ACTIVE] VIRTUAL_ENV=[$VIRTUAL_ENV]\"\n",
                "scuv activate data-hub\n",
                "echo \"activate SCUV_ACTIVE=[$SCUV_ACTIVE]\"\n",
                "scuv deactivate\n",
                "echo \"deactivate SCUV_ACTIVE=[$SCUV_ACTIVE] VIRTUAL_ENV=[$VIRTUAL_ENV]\"\n",
                "scuv shell --shell {sh} system\n",
                "echo \"explicit-status=$? SCUV_VERSION=[$SCUV_VERSION]\"\n",
                "unset SCUV_VERSION\n",
                "export SCUV_ACTIVE=stale2 VIRTUAL_ENV=/stale2\n",
                "scuv use SYSTEM\n",
                "echo \"use-system SCUV_ACTIVE=[$SCUV_ACTIVE] VIRTUAL_ENV=[$VIRTUAL_ENV]\"\n",
                "scuv activate no-such-env 2>/dev/null\n",
                "echo \"missing-status=$?\"\n",
                "{completion}",
            ),
            sh = shell,
            // bash < 4.4 (macOS ships 3.2) rejects `complete -o nosort`
            // outright; the init script must still register completion.
            completion = if shell == "bash" {
                "complete -p scuv\n"
            } else {
                ""
            }
        );
        let output = std::process::Command::new(&binary)
            .args(flags)
            .args(["-c", &script])
            .env("HOME", fixture.temp_dir.path())
            .env("SCUV_HOME", &fixture.scoop_home)
            .env("SCUV_LANG", "en")
            .env("PATH", shell_test_path(&fake_uv))
            .env("PSModulePath", "C:\\Program Files\\PowerShell\\Modules")
            .env_remove("SCUV_VERSION")
            .env_remove("SCUV_NO_AUTO")
            .env_remove("_SCUV_OLD_PATH")
            .env_remove("_SCUV_OLD_PYTHONHOME")
            .env_remove("FISH_VERSION")
            .env_remove("ZSH_VERSION")
            // A stale activation the startup hook must clear.
            .env("SCUV_ACTIVE", "stale")
            .env("VIRTUAL_ENV", "/stale")
            .current_dir(fixture.temp_dir.path())
            .output()
            .unwrap_or_else(|e| panic!("{shell} must run: {e}"));
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        for expected in [
            "after-init SCUV_ACTIVE=[] VIRTUAL_ENV=[]",
            "activate SCUV_ACTIVE=[data-hub]",
            "deactivate SCUV_ACTIVE=[] VIRTUAL_ENV=[]",
            "explicit-status=0 SCUV_VERSION=[system]",
            "use-system SCUV_ACTIVE=[] VIRTUAL_ENV=[]",
            "missing-status=1",
            if shell == "bash" {
                "-F _scuv_complete scuv"
            } else {
                ""
            },
        ] {
            assert!(
                stdout.contains(expected),
                "{shell}: missing {expected:?}.\nstdout:\n{stdout}\nstderr:\n{stderr}"
            );
        }
    }
}

/// Runs the real PowerShell integration end to end when `pwsh` is installed
/// (skipped otherwise; see `find_shell`), starting from the documented
/// profile line. Every scuv output it evaluates spans several lines.
///
/// Fails if a call hands `Invoke-Expression` the raw output (an array of
/// lines it refuses to bind), the deactivation script names variables as
/// `Env:\\X` (removes nothing, silently), `use system` activates instead of
/// deactivating, an env name containing `-h` counts as a help flag, a
/// failed scuv call reports success, or the
/// binary lookup breaks with more than one `scuv` on PATH.
#[cfg(unix)]
#[test]
fn powershell_wrapper_and_hook_evaluate_multiline_scripts() {
    let Some(pwsh) = find_shell("pwsh") else {
        eprintln!("skipping: no pwsh binary found");
        return;
    };
    let (fixture, fake_uv) = fixture_with_data_hub();
    let script = concat!(
        "Invoke-Expression (& scuv init powershell | Out-String)\n",
        "\"after-init SCUV_ACTIVE=[$env:SCUV_ACTIVE] VIRTUAL_ENV=[$env:VIRTUAL_ENV]\"\n",
        "scuv activate data-hub\n",
        "\"activate SCUV_ACTIVE=[$env:SCUV_ACTIVE]\"\n",
        "scuv deactivate\n",
        "\"deactivate SCUV_ACTIVE=[$env:SCUV_ACTIVE] VIRTUAL_ENV=[$env:VIRTUAL_ENV]\"\n",
        "scuv shell system\n",
        "\"shell SCUV_VERSION=[$env:SCUV_VERSION]\"\n",
        "scuv shell --unset\n",
        "\"unset SCUV_VERSION=[$env:SCUV_VERSION]\"\n",
        "$env:SCUV_ACTIVE = 'stale2'; $env:VIRTUAL_ENV = '/stale2'\n",
        "scuv use SYSTEM\n",
        "\"use-system SCUV_ACTIVE=[$env:SCUV_ACTIVE] VIRTUAL_ENV=[$env:VIRTUAL_ENV]\"\n",
        "scuv activate no-such-env 2>$null\n",
        "\"missing-status=$LASTEXITCODE\"\n",
    );
    let output = std::process::Command::new(pwsh)
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .env("HOME", fixture.temp_dir.path())
        .env("SCUV_HOME", &fixture.scoop_home)
        .env("SCUV_LANG", "en")
        .env("PATH", shell_test_path(&fake_uv))
        .env_remove("SCUV_VERSION")
        .env_remove("SCUV_NO_AUTO")
        .env_remove("_SCUV_OLD_PATH")
        .env_remove("_SCUV_OLD_PYTHONHOME")
        .env("SCUV_ACTIVE", "stale")
        .env("VIRTUAL_ENV", "/stale")
        .current_dir(fixture.temp_dir.path())
        .output()
        .expect("pwsh must run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    for expected in [
        "after-init SCUV_ACTIVE=[] VIRTUAL_ENV=[]",
        "activate SCUV_ACTIVE=[data-hub]",
        "deactivate SCUV_ACTIVE=[] VIRTUAL_ENV=[]",
        "shell SCUV_VERSION=[system]",
        "unset SCUV_VERSION=[]",
        "use-system SCUV_ACTIVE=[] VIRTUAL_ENV=[]",
        "missing-status=1",
    ] {
        assert!(
            stdout.contains(expected),
            "missing {expected:?}.\nstdout:\n{stdout}\nstderr:\n{stderr}"
        );
    }
}

/// A home with two envs, `alpha` and `beta`, plus a fake uv, for the
/// completion tests: completing an env name lists them via `scuv list --bare`.
#[cfg(unix)]
fn completion_fixture() -> (TestFixture, TempDir, String) {
    let fixture = TestFixture::new();
    for env in ["alpha", "beta"] {
        let dir = fixture.scoop_home.join("virtualenvs").join(env);
        std::fs::create_dir_all(dir.join("bin")).unwrap();
        std::fs::write(dir.join("pyvenv.cfg"), "version = 3.12.1\n").unwrap();
    }
    let fake_uv = fake_uv_dir();
    let bin = assert_cmd::cargo::cargo_bin("scuv");
    let path = format!(
        "{}:{}:{}",
        bin.parent().unwrap().display(),
        fake_uv.path().display(),
        std::env::var("PATH").unwrap_or_default()
    );
    (fixture, fake_uv, path)
}

/// Each line is `<command line> => <candidates>`; asserts the shell's
/// output holds every line.
#[cfg(unix)]
fn assert_completions(shell: &str, stdout: &str, stderr: &str, expected: &[&str]) {
    for line in expected {
        assert!(
            stdout.lines().any(|l| l.trim_end() == *line),
            "{shell}: missing {line:?}.\nstdout:\n{stdout}\nstderr:\n{stderr}"
        );
    }
}

/// Env names are offered only where an env name goes: after an option's
/// value (`-o out.json`, `--color auto`), at diff's second slot but not its
/// third, and not at clone's destination, run's command, a second `use`
/// argument or an option's own value (`-o <TAB>`; `--shell <TAB>` gets the
/// shells). Fails if the
/// position count includes option values or loses its limits.
#[cfg(unix)]
#[test]
fn bash_completion_offers_env_names_only_where_they_go() {
    let Some(bash) = find_shell("bash") else {
        eprintln!("skipping: no bash binary found");
        return;
    };
    let (fixture, _fake_uv, path) = completion_fixture();
    let script = concat!(
        "eval \"$(command scuv init bash)\" 2>/dev/null\n",
        "t() { COMP_WORDS=(\"$@\"); COMP_CWORD=$((${#COMP_WORDS[@]}-1)); COMPREPLY=();",
        " _scuv_complete; echo \"${COMP_WORDS[*]} => ${COMPREPLY[*]}\"; }\n",
        "t scuv export -o out.json ''\n",
        "t scuv verify --color auto ''\n",
        "t scuv clone alpha ''\n",
        "t scuv diff alpha ''\n",
        "t scuv diff alpha beta ''\n",
        "t scuv self ''\n",
        "t scuv self update ''\n",
        "t scuv use alpha ''\n",
        "t scuv activate --shell ''\n",
        "t scuv export -o ''\n",
    );
    let output = std::process::Command::new(bash)
        .args(["--norc", "--noprofile", "-c", script])
        .env("HOME", fixture.temp_dir.path())
        .env("SCUV_HOME", &fixture.scoop_home)
        .env("PATH", path)
        .current_dir(fixture.temp_dir.path())
        .output()
        .expect("bash must run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_completions(
        "bash",
        &stdout,
        &stderr,
        &[
            "scuv export -o out.json  => alpha beta system",
            "scuv verify --color auto  => alpha beta system",
            "scuv clone alpha  =>",
            "scuv diff alpha  => alpha beta system",
            "scuv diff alpha beta  =>",
            "scuv self  => update",
            "scuv self update  =>",
            "scuv use alpha  =>",
            "scuv activate --shell  => bash zsh fish powershell",
        ],
    );
    // The value of -o is a path: no env names there.
    let line = stdout.lines().find(|l| l.starts_with("scuv export -o  =>"));
    assert!(
        line.is_some_and(|l| !l.contains("alpha")),
        "bash: {line:?}\n{stdout}"
    );
}

/// zsh's position helpers, called directly with `words`/`CURRENT` set as
/// the completion system sets them (driving `compadd` itself needs a
/// terminal). Fails if `_scuv_positionals` counts an option's value or
/// `_scuv_at_option_value` misses one.
#[cfg(unix)]
#[test]
fn zsh_completion_counts_positions_past_option_values() {
    let Some(zsh) = find_shell("zsh") else {
        eprintln!("skipping: no zsh binary found");
        return;
    };
    let (fixture, _fake_uv, path) = completion_fixture();
    let script = concat!(
        "eval \"$(command scuv init zsh)\" 2>/dev/null\n",
        "t() { words=(\"$@\"); CURRENT=${#words}; _scuv_positionals;",
        " if _scuv_at_option_value; then v=yes; else v=no; fi;",
        " echo \"${words[*]} => $REPLY $v\"; }\n",
        "t scuv export -o out.json ''\n",
        "t scuv verify --color auto ''\n",
        "t scuv diff alpha ''\n",
        "t scuv use alpha ''\n",
        "t scuv export -o ''\n",
        "t scuv activate --shell ''\n",
    );
    let output = std::process::Command::new(zsh)
        .args(["-f", "-c", script])
        .env("HOME", fixture.temp_dir.path())
        .env("SCUV_HOME", &fixture.scoop_home)
        .env("PATH", path)
        .current_dir(fixture.temp_dir.path())
        .output()
        .expect("zsh must run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_completions(
        "zsh",
        &stdout,
        &stderr,
        &[
            "scuv export -o out.json  => 0 no",
            "scuv verify --color auto  => 0 no",
            "scuv diff alpha  => 1 no",
            "scuv use alpha  => 1 no",
            "scuv export -o  => 0 yes",
            "scuv activate --shell  => 0 yes",
        ],
    );
}

/// The same positions in fish, read back with `complete -C`. Fails if the
/// env-name or `update` rules lose their `__fish_is_nth_token` limits.
#[cfg(unix)]
#[test]
fn fish_completion_offers_env_names_only_where_they_go() {
    let Some(fish) = find_shell("fish") else {
        eprintln!("skipping: no fish binary found");
        return;
    };
    let (fixture, _fake_uv, path) = completion_fixture();
    let config_home = fixture.temp_dir.path().join("xdg-config");
    std::fs::create_dir_all(&config_home).unwrap();
    let script = concat!(
        "command scuv init fish | source\n",
        "for l in 'scuv clone ' 'scuv clone alpha ' 'scuv run alpha ' 'scuv diff alpha ' ",
        "'scuv diff alpha beta ' 'scuv self ' 'scuv self update ' 'scuv use alpha ' ",
        "'scuv export -o out.json ' 'scuv export -o ' 'scuv activate --shell ' 'scuv import --name '\n",
        "    set -l c (complete -C $l | string split -f1 \\t | string join ' ')\n",
        "    echo \"$l=> $c\"\n",
        "end\n",
    );
    let output = std::process::Command::new(fish)
        .args(["-c", script])
        .env("XDG_CONFIG_HOME", &config_home)
        .env("HOME", fixture.temp_dir.path())
        .env("SCUV_HOME", &fixture.scoop_home)
        .env("PATH", path)
        .current_dir(fixture.temp_dir.path())
        .output()
        .expect("fish must run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_completions(
        "fish",
        &stdout,
        &stderr,
        &[
            "scuv clone => alpha beta system",
            "scuv clone alpha =>",
            "scuv run alpha =>",
            "scuv diff alpha => alpha beta system",
            "scuv diff alpha beta =>",
            "scuv self => update",
            "scuv self update =>",
            "scuv use alpha =>",
            "scuv export -o out.json => alpha beta system",
            "scuv activate --shell => bash fish powershell zsh", // fish sorts
            "scuv import --name =>",
        ],
    );
    // The value of -o is a path: files, never env names.
    let line = stdout.lines().find(|l| l.starts_with("scuv export -o =>"));
    assert!(
        line.is_some_and(|l| !l.contains("alpha") && l.len() > "scuv export -o =>".len()),
        "fish: {line:?}\n{stdout}"
    );
}

/// The same positions in PowerShell, through `TabExpansion2`. Fails if
/// `scuv use <TAB>` is taken for the subcommand slot again (the AST text
/// drops the trailing space), or the position count includes option values
/// or loses its limits. Paths are PowerShell's fallback when the completer
/// returns nothing, so a slot with no candidates is checked by the absence
/// of env names.
#[cfg(unix)]
#[test]
fn powershell_completion_offers_env_names_only_where_they_go() {
    let Some(pwsh) = find_shell("pwsh") else {
        eprintln!("skipping: no pwsh binary found");
        return;
    };
    let (fixture, _fake_uv, path) = completion_fixture();
    let script = concat!(
        "Invoke-Expression (& scuv init powershell | Out-String)\n",
        "foreach ($l in 'scuv use ', 'scuv clone ', 'scuv clone alpha ', 'scuv diff alpha ',\n",
        "             'scuv diff alpha beta ', 'scuv export -o out.json ', 'scuv self ',\n",
        "             'scuv self update ', 'scuv use alpha ', 'scuv export -o \"out file.json\" ',\n",
        "             'scuv activate --shell ', 'scuv export -o ') {\n",
        "    $r = TabExpansion2 -inputScript $l -cursorColumn $l.Length\n",
        "    $c = @($r.CompletionMatches | ForEach-Object CompletionText |\n",
        "          Where-Object { $_ -in 'alpha', 'beta', 'system', 'update', 'list', 'bash' })\n",
        "    $l + '=> ' + ($c -join ' ')\n",
        "}\n",
    );
    let output = std::process::Command::new(pwsh)
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .env("HOME", fixture.temp_dir.path())
        .env("SCUV_HOME", &fixture.scoop_home)
        .env("PATH", path)
        .current_dir(fixture.temp_dir.path())
        .output()
        .expect("pwsh must run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_completions(
        "pwsh",
        &stdout,
        &stderr,
        &[
            "scuv use => alpha beta system",
            "scuv clone => alpha beta system",
            "scuv clone alpha =>",
            "scuv diff alpha => alpha beta system",
            "scuv diff alpha beta =>",
            "scuv export -o out.json => alpha beta system",
            "scuv self => update",
            "scuv self update =>",
            "scuv use alpha =>",
            "scuv export -o \"out file.json\" => alpha beta system",
            "scuv activate --shell => bash",
            "scuv export -o =>",
        ],
    );
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
