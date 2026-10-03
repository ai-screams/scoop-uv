//! `scuv remove`, including the project `.venv` link cleanup (#202).

use crate::support::*;

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

/// Without --force, `remove` asks first. With no terminal to ask on (a
/// script), that is an error and nothing is deleted; it used to print
/// "Cancelled" and exit 0. Fails if the prompt error turns into "no" again.
#[test]
fn test_remove_without_force_and_without_terminal_fails() {
    let fixture = TestFixture::new();
    let env_path = fixture.scoop_home.join("virtualenvs").join("keepme");
    std::fs::create_dir_all(&env_path).unwrap();

    scoop_cmd(&fixture.scoop_home)
        .args(["remove", "keepme"])
        .assert()
        .failure();
    assert!(
        env_path.exists(),
        "nothing may be removed without an answer"
    );
}
