//! `scuv migrate` against a fake pyenv tree.

use crate::support::*;

/// A pyenv tree under `root` holding one env, `name`, with a stub
/// interpreter: enough for discovery to report it without running Python.
fn fake_pyenv_env(root: &std::path::Path, name: &str) {
    fake_pyenv_env_with(root, name, "3.12.14");
}

/// [`fake_pyenv_env`] on Python `version`.
fn fake_pyenv_env_with(root: &std::path::Path, name: &str, version: &str) {
    let env = root.join(format!("versions/{version}/envs")).join(name);
    std::fs::create_dir_all(env.join("bin")).unwrap();
    std::fs::write(
        env.join("pyvenv.cfg"),
        format!("home = /usr/bin\nversion_info = {version}\n"),
    )
    .unwrap();
    let python = env.join("bin/python");
    std::fs::write(&python, format!("#!/bin/sh\necho Python {version}\n")).unwrap();
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

/// An EOL env without a flag is refused (exit 2) and the hint names
/// `--allow-eol`, not `--force`, which would also overwrite.
#[cfg(unix)]
#[test]
fn migrate_env_eol_hint_names_allow_eol() {
    let fixture = TestFixture::new();
    let pyenv = fixture.temp_dir.path().join("pyenv");
    fake_pyenv_env_with(&pyenv, "old", "3.10.14");

    scoop_cmd(&fixture.scoop_home)
        .args(["migrate", "@env", "old"])
        .env("PYENV_ROOT", &pyenv)
        .env("HOME", fixture.temp_dir.path())
        .assert()
        .code(2)
        .stderr(predicate::str::contains("--allow-eol"));
}

/// `--allow-eol` does not overwrite: an EOL env whose name is taken still
/// stops as a name conflict (exit 2) and the existing env keeps its files.
/// Fails if `--allow-eol` is handed on as `--force`.
#[cfg(unix)]
#[test]
fn migrate_env_allow_eol_does_not_overwrite_a_conflict() {
    let fixture = TestFixture::new();
    let pyenv = fixture.temp_dir.path().join("pyenv");
    fake_pyenv_env_with(&pyenv, "old", "3.10.14");
    let existing = fixture.scoop_home.join("virtualenvs/old");
    std::fs::create_dir_all(&existing).unwrap();
    std::fs::write(existing.join("keep"), "x").unwrap();

    scoop_cmd(&fixture.scoop_home)
        .args(["migrate", "@env", "old", "--allow-eol"])
        .env("PYENV_ROOT", &pyenv)
        .env("HOME", fixture.temp_dir.path())
        .assert()
        .code(2)
        .stderr(predicate::str::contains("'old' already exists"));
    assert!(existing.join("keep").exists());
}

/// `migrate all` with no terminal to ask on and no `--yes` is an error that
/// names `--yes` (exit 1), not "Dialog error: not a terminal", and nothing
/// is migrated. assert_cmd's piped stderr is never a terminal.
#[cfg(unix)]
#[test]
fn migrate_all_without_a_terminal_names_yes() {
    let fixture = TestFixture::new();
    let pyenv = fixture.temp_dir.path().join("pyenv");
    fake_pyenv_env(&pyenv, "web");

    scoop_cmd(&fixture.scoop_home)
        .args(["migrate", "all", "--source", "pyenv"])
        .env("PYENV_ROOT", &pyenv)
        .env("HOME", fixture.temp_dir.path())
        .assert()
        .code(1)
        .stderr(predicate::str::contains("Pass --yes"))
        .stderr(predicate::str::contains("Dialog error").not());
    assert!(!fixture.scoop_home.join("virtualenvs/web").exists());
}

/// Runs `scuv` with `args` against `pyenv` in language `lang` and returns
/// (exit code, stdout, stderr). A fake `uv` comes first on `PATH`: its
/// `pip list` reports no packages, so a dry run gets past reading the
/// source without a real interpreter.
#[cfg(unix)]
fn migrate_in(
    fixture: &TestFixture,
    pyenv: &std::path::Path,
    lang: &str,
    args: &[&str],
) -> (Option<i32>, String, String) {
    let uv = fake_uv_dir();
    let path = format!(
        "{}:{}",
        uv.path().display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let out = scoop_cmd(&fixture.scoop_home)
        .args(args)
        .env("PATH", path)
        .env("PYENV_ROOT", pyenv)
        .env("HOME", fixture.temp_dir.path())
        .env("SCUV_LANG", lang)
        .output()
        .unwrap();
    (
        out.status.code(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// `migrate @env` on an EOL env: the source line and the refusal follow
/// the language, the English stays word for word, and under `--json` the
/// error code does not change with the language. Fails if either message
/// goes back to a hard-coded string.
#[cfg(unix)]
#[test]
fn migrate_env_source_line_and_eol_refusal_follow_the_language() {
    let fixture = TestFixture::new();
    let pyenv = fixture.temp_dir.path().join("pyenv");
    fake_pyenv_env_with(&pyenv, "old", "3.10.14");

    let (code, _, err) = migrate_in(&fixture, &pyenv, "en", &["migrate", "@env", "old"]);
    assert_eq!(code, Some(2));
    assert!(
        err.contains("Source: old (pyenv, Python 3.10.14)\n"),
        "{err}"
    );
    assert!(
        err.contains("Migration failed: Python 3.10.14 is EOL"),
        "{err}"
    );

    let (code, _, err) = migrate_in(&fixture, &pyenv, "ko", &["migrate", "@env", "old"]);
    assert_eq!(code, Some(2));
    assert!(err.contains("원본: old (pyenv, Python 3.10.14)\n"), "{err}");
    assert!(err.contains("Python 3.10.14 지원 종료됨"), "{err}");
    assert!(!err.contains("is EOL"), "{err}");

    let (_, out, _) = migrate_in(
        &fixture,
        &pyenv,
        "ko",
        &["migrate", "@env", "old", "--json"],
    );
    let json: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(json["error"]["code"], "MIGRATE_FAILED", "{json}");
    let message = json["error"]["message"].as_str().unwrap();
    assert!(message.contains("Python 3.10.14 지원 종료됨"), "{json}");
}

/// `migrate list` ends an EOL env's line with a hint in the user's language.
/// Fails if the hint is hard-coded again.
#[cfg(unix)]
#[test]
fn migrate_list_eol_hint_follows_the_language() {
    let fixture = TestFixture::new();
    let pyenv = fixture.temp_dir.path().join("pyenv");
    fake_pyenv_env_with(&pyenv, "old", "3.10.14");

    let args = ["migrate", "list", "--source", "pyenv"];
    let (_, out, _) = migrate_in(&fixture, &pyenv, "en", &args);
    assert!(out.contains("MB (Python 3.10.14 is EOL)\n"), "{out}");
    let (_, out, _) = migrate_in(&fixture, &pyenv, "ko", &args);
    assert!(out.contains("MB (Python 3.10.14 지원 종료)\n"), "{out}");
}

/// The `migrate all` preview (plan header, skipped count, dry-run target)
/// follows the language, the English stays word for word, and the JSON
/// `skipped[].reason` stays English for scripts. Fails if any of the three
/// lines is hard-coded again or the JSON reason gets translated.
#[cfg(unix)]
#[test]
fn migrate_all_preview_follows_the_language_and_json_reasons_do_not() {
    let fixture = TestFixture::new();
    let pyenv = fixture.temp_dir.path().join("pyenv");
    fake_pyenv_env_with(&pyenv, "web", "3.12.14");
    fake_pyenv_env_with(&pyenv, "old", "3.10.14");
    let args = ["migrate", "all", "--source", "pyenv", "--dry-run", "--yes"];

    let (code, _, err) = migrate_in(&fixture, &pyenv, "en", &args);
    assert_eq!(code, Some(0), "{err}");
    assert!(
        err.contains("Found 1 environment(s) to migrate:\n"),
        "{err}"
    );
    assert!(
        err.contains("1 environment(s) will be skipped (see summary)\n"),
        "{err}"
    );
    assert!(err.contains("  [DRY-RUN] Would create: "), "{err}");

    let (code, _, err) = migrate_in(&fixture, &pyenv, "ko", &args);
    assert_eq!(code, Some(0), "{err}");
    assert!(err.contains("마이그레이션할 환경 1개 발견:\n"), "{err}");
    assert!(
        err.contains("환경 1개는 건너뛸 예정 (요약 참고)\n"),
        "{err}"
    );
    assert!(err.contains("  [DRY-RUN] 생성 예정: "), "{err}");

    let mut json_args = args.to_vec();
    json_args.push("--json");
    let (_, out, _) = migrate_in(&fixture, &pyenv, "ko", &json_args);
    let json: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(
        json["data"]["skipped"][0]["reason"], "Python 3.10.14 is EOL (use --allow-eol)",
        "{json}"
    );
}
