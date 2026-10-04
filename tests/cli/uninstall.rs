//! `uninstall --cascade` as a process: what each output mode prints and the
//! exit status when an env cannot be removed.

use crate::support::*;

const P14: &str = "cpython-3.12.14-macos-aarch64-none";
const RC: &str = "cpython-3.12.14rc1-macos-aarch64-none";

/// A home whose envs `(name, install)` each sit on that uv install, and a
/// fake `uv` reporting `uv_version` whose uninstall deletes the installs
/// named by the request (`cpython-<request>-*`, so not its pre-releases).
#[cfg(unix)]
fn home_with_envs(uv_version: &str, envs: &[(&str, &str)]) -> (TestFixture, TempDir) {
    use std::os::unix::fs::PermissionsExt;

    let fixture = TestFixture::new();
    let uv = TempDir::new().unwrap();
    let py = uv.path().join("py");
    for (name, install) in envs {
        std::fs::create_dir_all(py.join(install).join("bin")).unwrap();
        let env = fixture.scoop_home.join("virtualenvs").join(name);
        std::fs::create_dir_all(env.join("bin")).unwrap();
        std::fs::write(
            env.join("pyvenv.cfg"),
            format!("home = {}\n", py.join(install).join("bin").display()),
        )
        .unwrap();
    }
    let d = uv.path().display();
    std::fs::write(
        uv.path().join("uv"),
        format!(
            "#!/bin/sh\ncase \"$1 $2\" in\n  \"--version \"*) echo \"{uv_version}\" ;;\n  \"python dir\") echo \"{d}/py\" ;;\n  \"python uninstall\") /bin/rm -rf \"{d}/py/cpython-$3-\"* ;;\n  *) echo \"fake uv: unsupported: $*\" >&2; exit 2 ;;\nesac\n"
        ),
    )
    .unwrap();
    std::fs::set_permissions(uv.path().join("uv"), std::fs::Permissions::from_mode(0o755)).unwrap();
    (fixture, uv)
}

/// Two envs on uv's 3.12.14 install: `web`, which the cascade removes, and
/// `1bad`, whose name the removal refuses.
#[cfg(unix)]
fn run_cascade(extra: &[&str]) -> (std::process::Output, TestFixture) {
    let (fixture, uv) = home_with_envs("uv 0.12.22", &[("web", P14), ("1bad", P14)]);
    let out = scoop_cmd(&fixture.scoop_home)
        .env("PATH", uv.path())
        .args(["uninstall", "3.12.14", "--cascade", "--force"])
        .args(extra)
        .output()
        .unwrap();
    (out, fixture)
}

/// The Python is gone but `1bad` is left: exit 1 with one summary line, and
/// no generic `error:` line after it. Fails if the command reports success,
/// prints nothing, or prints the error twice.
#[cfg(unix)]
#[test]
fn cascade_with_an_env_left_behind_exits_one_and_says_so() {
    let (out, fixture) = run_cascade(&[]);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(1), "{stderr}");
    assert!(stderr.contains("Could not remove '1bad'"), "{stderr}");
    assert_eq!(
        stderr
            .matches("1 environment(s) could not be removed")
            .count(),
        1,
        "{stderr}"
    );
    assert!(
        !stderr.contains("uninstalled\n"),
        "no success line: {stderr}"
    );
    let envs = fixture.scoop_home.join("virtualenvs");
    assert!(!envs.join("web").exists());
    assert!(envs.join("1bad").exists());
}

/// `--quiet` still says why the exit is 1. Fails if the summary is dropped.
#[cfg(unix)]
#[test]
fn cascade_with_an_env_left_behind_is_reported_under_quiet() {
    let (out, _fixture) = run_cascade(&["--quiet"]);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(1), "{stderr}");
    assert!(
        stderr.contains("1 environment(s) could not be removed"),
        "{stderr}"
    );
}

/// `--json` prints one error envelope that still carries the data. Fails if
/// stdout is empty, says success, or loses `removed_envs`/`failed_envs`.
#[cfg(unix)]
#[test]
fn cascade_with_an_env_left_behind_emits_a_json_error() {
    let (out, _fixture) = run_cascade(&["--json"]);
    assert_eq!(out.status.code(), Some(1));
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["status"], "error");
    assert_eq!(json["error"]["code"], "UNINSTALL_CASCADE_INCOMPLETE");
    assert_eq!(json["data"]["removed_envs"], serde_json::json!(["web"]));
    assert_eq!(json["data"]["failed_envs"][0]["name"], "1bad");
}

/// Whether a patch request plans to take that patch's pre-releases follows
/// the installed uv: from 0.9.1 it does not, so an env on 3.12.14rc1 is
/// not listed and the run goes ahead without asking; before that it is
/// listed, and with no terminal to confirm on the cascade aborts. Fails if
/// the uv version stops reaching the plan.
#[cfg(unix)]
#[rstest::rstest]
#[case("uv 0.12.22", true)]
#[case("uv 0.9.0", false)]
fn cascade_plan_for_a_patch_follows_uv_on_pre_releases(
    #[case] uv_version: &str,
    #[case] proceeds: bool,
) {
    let (fixture, uv) = home_with_envs(uv_version, &[("rc", RC)]);
    let out = scoop_cmd(&fixture.scoop_home)
        .env("PATH", uv.path())
        .args(["uninstall", "3.12.14", "--cascade"])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.success(), proceeds, "{stderr}");
    assert!(
        fixture.scoop_home.join("virtualenvs").join("rc").exists(),
        "the pre-release survives, and so does its env"
    );
}
