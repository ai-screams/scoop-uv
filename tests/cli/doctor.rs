//! `doctor --fix` and installing uv: every way of reaching it without a person
//! at a terminal must leave the machine alone.
//!
//! The tests run without a terminal, so the real prompt is out of reach; the
//! prompted path is covered in `src/uv/install/tests.rs` by injecting the answer.

use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use predicates::prelude::*;
use tempfile::TempDir;

use crate::support::{TestFixture, scoop_cmd};

/// A `PATH` with no uv, and a `curl` and `sh` that record being run.
struct NoUv {
    fixture: TestFixture,
    bin: TempDir,
}

impl NoUv {
    fn new() -> Self {
        let bin = TempDir::new().unwrap();
        for tool in ["curl", "sh"] {
            write_script(
                &bin.path().join(tool),
                &format!("echo ran > '{}/ran'\n", bin.path().display()),
            );
        }
        Self {
            fixture: TestFixture::new(),
            bin,
        }
    }

    fn doctor(&self, args: &[&str]) -> assert_cmd::Command {
        let mut cmd = scoop_cmd(&self.fixture.scoop_home);
        cmd.arg("doctor")
            .args(args)
            .env("PATH", self.bin.path())
            .env("HOME", self.fixture.temp_dir.path())
            .env_remove("UV_OFFLINE");
        cmd
    }

    /// Nothing ran and nothing landed where uv would go.
    fn assert_untouched(&self) {
        assert!(!self.bin.path().join("ran").exists(), "a tool was run");
        let local = self.fixture.temp_dir.path().join(".local");
        assert!(!local.exists(), "{} was created", local.display());
    }
}

fn write_script(path: &Path, body: &str) {
    std::fs::write(path, format!("#!/bin/sh\n{body}")).unwrap();
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
}

#[test]
fn fix_without_a_terminal_says_so_and_installs_nothing() {
    let env = NoUv::new();
    env.doctor(&["--fix"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("confirmation needs a terminal"));
    env.assert_untouched();
}

#[test]
fn fix_honours_uv_offline() {
    let env = NoUv::new();
    env.doctor(&["--fix"])
        .env("UV_OFFLINE", "1")
        .assert()
        .code(2)
        .stderr(predicate::str::contains("UV_OFFLINE"));
    env.assert_untouched();
}

#[test]
fn fix_json_stays_one_document_and_installs_nothing() {
    let env = NoUv::new();
    let assert = env.doctor(&["--fix", "--json"]).assert().code(2);
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    let report: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert!(report["checks"].is_array(), "{report}");
    env.assert_untouched();
}

#[test]
fn fix_quiet_does_not_mention_installing() {
    let env = NoUv::new();
    let mut cmd = env.doctor(&["--fix"]);
    cmd.arg("--quiet");
    cmd.assert()
        .code(2)
        .stderr(predicate::str::contains("terminal").not());
    env.assert_untouched();
}

#[test]
fn plain_doctor_never_reaches_the_install_path() {
    let env = NoUv::new();
    env.doctor(&[]).assert().code(2).stderr(
        predicate::str::contains("confirmation")
            .not()
            .and(predicate::str::contains("scuv doctor --fix")),
    );
    env.assert_untouched();
}

#[test]
fn fix_leaves_a_present_uv_alone() {
    let env = NoUv::new();
    write_script(&env.bin.path().join("uv"), "echo uv 0.13.6\n");
    env.doctor(&["--fix"])
        .assert()
        .stderr(predicate::str::contains("confirmation").not());
    env.assert_untouched();
}

#[test]
fn fix_does_not_offer_or_suggest_itself_for_a_uv_that_cannot_start() {
    let env = NoUv::new();
    let uv = env.bin.path().join("uv");
    std::fs::write(&uv, "#!/nonexistent/python\n").unwrap();
    std::fs::set_permissions(&uv, std::fs::Permissions::from_mode(0o755)).unwrap();
    env.doctor(&["--fix"]).assert().code(2).stderr(
        predicate::str::contains("could not run")
            .and(predicate::str::contains("scuv doctor --fix").not())
            .and(predicate::str::contains("confirmation").not()),
    );
    env.assert_untouched();
}

#[test]
fn ordinary_commands_point_at_doctor_fix_and_install_nothing() {
    let env = NoUv::new();
    scoop_cmd(&env.fixture.scoop_home)
        .args(["list", "--pythons"])
        .env("PATH", env.bin.path())
        .env("HOME", env.fixture.temp_dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("scuv doctor --fix"));
    env.assert_untouched();
}

/// The report follows the display language, in text and in `--json`, while
/// the ids scripts key on stay the same and commands stay literal. What
/// fails this: a check name, message or summary left in English, or an id
/// that gets translated.
#[test]
fn report_follows_the_display_language_and_ids_stay_stable() {
    let env = NoUv::new();
    let broken = env.fixture.scoop_home.join("virtualenvs").join("broken");
    std::fs::create_dir_all(&broken).unwrap();

    env.doctor(&[])
        .env("SCUV_LANG", "ko")
        .assert()
        .code(2)
        .stderr(
            predicate::str::contains("설치 상태를 확인하는 중...")
                .and(predicate::str::contains(
                    "✗ 깨진 가상 환경: 'broken' 환경이 손상됐어요",
                ))
                .and(predicate::str::contains(
                    "→ scuv remove broken && scuv create broken <python-version>",
                ))
                .and(predicate::str::contains("PATH에서 uv를 찾을 수 없어요"))
                .and(predicate::str::contains("찾았어요."))
                .and(predicate::str::contains("Checking installation").not())
                .and(predicate::str::contains("broken virtualenv").not()),
        );

    let out = env
        .doctor(&["--json"])
        .env("SCUV_LANG", "ko")
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let checks = json["checks"].as_array().unwrap();
    let uv = checks.iter().find(|c| c["id"] == "uv").unwrap();
    assert_eq!(uv["name"], "uv 설치", "{json}");
    assert_eq!(uv["message"], "PATH에서 uv를 찾을 수 없어요", "{json}");
    let venv = checks
        .iter()
        .find(|c| c["id"] == "venv" && c["status"] == "error")
        .unwrap();
    assert_eq!(venv["name"], "깨진 가상 환경", "{json}");
}
