//! Tests that need a real uv and Python; `#[ignore]`d by default.

use crate::support::*;

// Tests requiring uv and Python - mark as #[ignore]

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

// ============================================================================
// Package origins: clone, export/import and diff against a real uv
// ============================================================================
//
// Every test here has `origin` in its name: the uv-floor CI job runs them
// with the oldest supported uv as well.

/// A buildable project `name` at `dir` (hatchling backend, fetched by uv).
fn origin_project(dir: &std::path::Path, name: &str) {
    std::fs::create_dir_all(dir.join("src").join(name)).unwrap();
    std::fs::write(
        dir.join("pyproject.toml"),
        format!(
            "[project]\nname = \"{name}\"\nversion = \"0.1.0\"\n\n[build-system]\nrequires = [\"hatchling\"]\nbuild-backend = \"hatchling.build\"\n"
        ),
    )
    .unwrap();
    std::fs::write(dir.join("src").join(name).join("__init__.py"), "X = 1\n").unwrap();
}

/// Panics with the output of `what` unless it succeeded.
fn origin_ok(what: &str, out: std::process::Output) -> std::process::Output {
    assert!(
        out.status.success(),
        "{what}\n{}\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    out
}

/// Runs a non-scuv command; panics unless it succeeds.
fn origin_run(cmd: &mut std::process::Command) -> std::process::Output {
    let what = format!("{cmd:?}");
    origin_ok(&what, cmd.output().unwrap())
}

/// Runs `scuv <args>` against the fixture; panics unless it succeeds.
fn origin_scuv<I, S>(fixture: &TestFixture, args: I) -> std::process::Output
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    let args: Vec<std::ffi::OsString> = args
        .into_iter()
        .map(|a| a.as_ref().to_os_string())
        .collect();
    let what = format!("scuv {args:?}");
    origin_ok(
        &what,
        scoop_cmd(&fixture.scoop_home).args(&args).output().unwrap(),
    )
}

fn origin_python(fixture: &TestFixture, env: &str) -> std::path::PathBuf {
    fixture
        .scoop_home
        .join("virtualenvs")
        .join(env)
        .join("bin")
        .join("python")
}

/// `uv pip install` into scuv env `env`.
fn origin_install(fixture: &TestFixture, env: &str, args: &[&str]) {
    let mut cmd = std::process::Command::new("uv");
    cmd.args(["pip", "install", "--python"])
        .arg(origin_python(fixture, env))
        .args(args);
    origin_run(&mut cmd);
}

/// The `source` of each package in `scuv export <env>`, by name.
fn origin_sources(
    fixture: &TestFixture,
    env: &str,
) -> std::collections::BTreeMap<String, serde_json::Value> {
    let out = origin_scuv(fixture, ["export", env]);
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["scoop_export_version"], "2", "{json}");
    json["packages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| (p["name"].as_str().unwrap().to_string(), p["source"].clone()))
        .collect()
}

/// An env holding an editable project, an editable project in a
/// subdirectory, a plain directory install and a git checkout. Returns the
/// canonical work directory.
fn origin_source_env(fixture: &TestFixture) -> std::path::PathBuf {
    let work = std::fs::canonicalize(fixture.temp_dir.path())
        .unwrap()
        .join("work");
    origin_project(&work.join("edit"), "originedit");
    origin_project(&work.join("mono").join("pkgs").join("inner"), "origininner");
    origin_project(&work.join("plain"), "originplain");
    let git = work.join("git");
    origin_project(&git, "origingit");
    for args in [
        vec!["init", "-q"],
        vec!["add", "-A"],
        vec![
            "-c",
            "user.email=t@t",
            "-c",
            "user.name=t",
            "commit",
            "-qm",
            "init",
        ],
    ] {
        origin_run(
            std::process::Command::new("git")
                .current_dir(&git)
                .args(args),
        );
    }
    origin_scuv(fixture, ["create", "src", "3.12"]);
    let edit = work.join("edit");
    let inner = work.join("mono/pkgs/inner");
    let plain = work.join("plain");
    let git_req = format!("origingit @ git+file://{}", git.display());
    origin_install(
        fixture,
        "src",
        &[
            "--editable",
            edit.to_str().unwrap(),
            "--editable",
            inner.to_str().unwrap(),
            plain.to_str().unwrap(),
            &git_req,
        ],
    );
    work
}

/// clone puts every package back from where it came, and diff (origins
/// included) finds no package difference. What fails this: reinstalling
/// `name==version`, which the registry cannot satisfy for these names.
#[test]
#[ignore = "requires uv to be installed"]
fn clone_reproduces_package_origins() {
    let fixture = TestFixture::new();
    origin_source_env(&fixture);
    origin_scuv(&fixture, ["clone", "src", "dst"]);

    let (src, dst) = (
        origin_sources(&fixture, "src"),
        origin_sources(&fixture, "dst"),
    );
    assert_eq!(dst["originedit"]["dir_info"]["editable"], true, "{dst:?}");
    assert!(
        dst["origingit"]["vcs_info"]["commit_id"].is_string(),
        "{dst:?}"
    );
    assert_eq!(
        src["origingit"]["vcs_info"]["commit_id"],
        dst["origingit"]["vcs_info"]["commit_id"]
    );
    assert!(dst["originplain"]["dir_info"].is_object(), "{dst:?}");
    origin_scuv(
        &fixture,
        ["diff", "src", "dst", "--packages-only", "--strict"],
    );
}

/// `migrate @env` on a real virtualenvwrapper env puts every package back
/// from where it came (editable, plain directory, git), leaves pip out, and
/// under `--delete-source` removes the source once all of it moved. What
/// fails this: reading the source with `pip freeze` again, which skipped
/// editables and reinstalled the rest as `name==version`.
#[test]
#[ignore = "requires uv to be installed"]
fn migrate_reproduces_package_origins() {
    let fixture = TestFixture::new();
    let work = std::fs::canonicalize(fixture.temp_dir.path())
        .unwrap()
        .join("work");
    origin_project(&work.join("edit"), "originedit");
    origin_project(&work.join("plain"), "originplain");
    let git = work.join("git");
    origin_project(&git, "origingit");
    for args in [
        vec!["init", "-q"],
        vec!["add", "-A"],
        vec![
            "-c",
            "user.email=t@t",
            "-c",
            "user.name=t",
            "commit",
            "-qm",
            "init",
        ],
    ] {
        origin_run(
            std::process::Command::new("git")
                .current_dir(&git)
                .args(args),
        );
    }
    let workon = work.join("workon");
    let source = workon.join("wsrc");
    origin_run(
        std::process::Command::new("uv")
            .args(["venv", "-q", "-p", "3.12"])
            .arg(&source),
    );
    let git_req = format!("origingit @ git+file://{}", git.display());
    origin_run(
        std::process::Command::new("uv")
            .args(["pip", "install", "--python"])
            .arg(source.join("bin/python"))
            .args(["--editable", work.join("edit").to_str().unwrap()])
            .arg(work.join("plain"))
            .arg(&git_req)
            .arg("pip"),
    );

    let out = origin_ok(
        "scuv migrate",
        scoop_cmd(&fixture.scoop_home)
            .env("HOME", fixture.temp_dir.path())
            .env("WORKON_HOME", &workon)
            .args([
                "migrate",
                "@env",
                "wsrc",
                "--source",
                "virtualenvwrapper",
                "--delete-source",
                "--json",
            ])
            .output()
            .unwrap(),
    );
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let data = &json["data"];
    assert_eq!(data["packages_failed"], serde_json::json!([]), "{json}");
    assert_eq!(data["packages_migrated"], 3, "{json}");
    assert_eq!(data["packages_excluded"][0]["name"], "pip", "{json}");
    assert_eq!(data["source_deleted"], true, "{json}");
    assert!(!source.exists());

    let got = origin_sources(&fixture, "wsrc");
    assert_eq!(got["originedit"]["dir_info"]["editable"], true, "{got:?}");
    assert!(got["originplain"]["dir_info"].is_object(), "{got:?}");
    assert!(
        got["origingit"]["vcs_info"]["commit_id"].is_string(),
        "{got:?}"
    );
    assert!(!got.contains_key("pip"), "{got:?}");
}

/// export → import keeps every origin.
#[test]
#[ignore = "requires uv to be installed"]
fn export_import_round_trips_package_origins() {
    let fixture = TestFixture::new();
    origin_source_env(&fixture);
    let file = fixture.temp_dir.path().join("src.json");
    let file = file.to_str().unwrap();
    origin_scuv(&fixture, ["export", "src", "-o", file]);
    origin_scuv(&fixture, ["import", file, "--name", "imp"]);
    origin_scuv(
        &fixture,
        ["diff", "src", "imp", "--packages-only", "--strict"],
    );
}

/// An import whose local source is gone on this machine fails before
/// creating anything and names the package.
#[test]
#[ignore = "requires uv to be installed"]
fn import_refuses_a_missing_origin_path() {
    let fixture = TestFixture::new();
    let file = fixture.temp_dir.path().join("gone.json");
    std::fs::write(
        &file,
        r#"{"scoop_export_version":"2","environment":{"name":"gone","python":"3.12"},"packages":[{"name":"originedit","version":"0.1.0","source":{"url":"file:///nonexistent/scuv/originedit","dir_info":{"editable":true}}}]}"#,
    )
    .unwrap();
    let out = scoop_cmd(&fixture.scoop_home)
        .arg("import")
        .arg(&file)
        .arg("--json")
        .output()
        .unwrap();
    assert!(!out.status.success());
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["error"]["code"], "PACKAGE_SOURCE_MISSING", "{json}");
    assert!(!fixture.scoop_home.join("virtualenvs/gone").exists());
}

/// A hashed archive whose hash is wrong never ends up installed: a uv that
/// checks hashes rejects it (and import removes the env), an older one is
/// refused by scuv before the env is created.
#[test]
#[ignore = "requires uv to be installed"]
fn import_never_installs_a_hashed_origin_archive_unchecked() {
    let fixture = TestFixture::new();
    let work = std::fs::canonicalize(fixture.temp_dir.path())
        .unwrap()
        .join("work");
    origin_project(&work.join("whl"), "originwhl");
    let dist = work.join("dist");
    origin_run(
        std::process::Command::new("uv")
            .args(["build", "-q", "--wheel"])
            .arg(work.join("whl"))
            .arg("-o")
            .arg(&dist),
    );
    let wheel = std::fs::read_dir(&dist)
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let file = fixture.temp_dir.path().join("whl.json");
    std::fs::write(
        &file,
        format!(
            r#"{{"scoop_export_version":"2","environment":{{"name":"whl","python":"3.12"}},"packages":[{{"name":"originwhl","version":"0.1.0","source":{{"url":"file://{}","archive_info":{{"hashes":{{"sha256":"{}"}}}}}}}}]}}"#,
            wheel.display(),
            "0".repeat(64)
        ),
    )
    .unwrap();
    let out = scoop_cmd(&fixture.scoop_home)
        .arg("import")
        .arg(&file)
        .arg("--json")
        .output()
        .unwrap();
    assert!(!out.status.success(), "a wrong hash was installed");
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    // 0.11.29 is the first uv that checks a `#sha256=` URL hash.
    let uv =
        String::from_utf8(origin_run(std::process::Command::new("uv").arg("--version")).stdout)
            .unwrap();
    let v: Vec<u32> = uv
        .split_whitespace()
        .nth(1)
        .unwrap()
        .split('.')
        .map(|n| n.parse().unwrap())
        .collect();
    let checks_hashes = (v[0], v[1], v[2]) >= (0, 11, 29);
    let want = if checks_hashes {
        "UV_COMMAND_FAILED"
    } else {
        "PACKAGE_HASH_UNENFORCED"
    };
    assert_eq!(json["error"]["code"], want, "{uv}: {json}");
    assert!(
        !fixture.scoop_home.join("virtualenvs/whl").exists(),
        "env left behind"
    );
    if !checks_hashes {
        return;
    }
    // The failure is the hash check itself, and it names the right digest.
    // With it, the form uv itself records (`#sha256=` in the URL, empty
    // `archive_info`) imports, and the env it makes clones in turn.
    let message = json["error"]["message"].as_str().unwrap_or_default();
    let digest = message
        .split("sha256:")
        .nth(2)
        .and_then(|rest| rest.get(..64))
        .unwrap_or_else(|| panic!("no computed sha256 in: {message}"));
    assert!(message.contains("mismatch"), "{message}");
    std::fs::write(
        &file,
        format!(
            r#"{{"scoop_export_version":"2","environment":{{"name":"whl","python":"3.12"}},"packages":[{{"name":"originwhl","version":"0.1.0","source":{{"url":"file://{}#sha256={digest}","archive_info":{{}}}}}}]}}"#,
            wheel.display()
        ),
    )
    .unwrap();
    origin_scuv(&fixture, [std::ffi::OsStr::new("import"), file.as_os_str()]);
    origin_scuv(&fixture, ["clone", "whl", "whl2"]);
}
