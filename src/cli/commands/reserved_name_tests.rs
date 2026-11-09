//! An env named with a word that was reserved after the env was made.
//!
//! Reserved words only stop a new env from taking the name
//! (`validate::validate_env_name`). Every path that finds, uses or removes
//! an env takes the existing-name check, so such an env stays reachable,
//! and no `--force` path removes it on the way to refusing the name.

#![cfg(unix)]

use std::path::{Path, PathBuf};

use serial_test::serial;

use crate::core::{VersionService, VirtualenvService};
use crate::error::ScoopError;
use crate::output::Output;
use crate::test_utils::{FakeUv, env_guard};

/// Reserved since 0.21.0, so an env may already have it.
const NAME: &str = "gc";

/// Runs `f` with a fake uv (Python 3.12.0 installed) first on `PATH`, a
/// fresh `SCUV_HOME` and a fresh working directory. `f` gets the
/// virtualenvs dir and the working dir.
fn run<T>(extra: &[(&'static str, Option<&str>)], f: impl FnOnce(&Path, &Path) -> T) -> T {
    let uv = FakeUv::new(&["3.12.0"]);
    let home = tempfile::TempDir::new().unwrap();
    let work = tempfile::TempDir::new().unwrap();
    let path = uv.path_var();
    let home_str = home.path().display().to_string();
    let mut vars: Vec<(&'static str, Option<&str>)> = vec![
        ("PATH", Some(path.as_str())),
        ("SCUV_HOME", Some(home_str.as_str())),
        ("SCUV_VERSION", None),
    ];
    vars.extend_from_slice(extra);
    let _env = env_guard(&vars);
    let envs = home.path().join("virtualenvs");
    std::fs::create_dir_all(&envs).unwrap();
    f(&envs, work.path())
}

/// The env `NAME` as an earlier scuv left it: a `keep` marker and metadata.
fn existing_env(envs: &Path) -> PathBuf {
    let dir = envs.join(NAME);
    std::fs::create_dir_all(dir.join("bin")).unwrap();
    std::fs::write(dir.join("keep"), "").unwrap();
    let meta = crate::core::Metadata::new(NAME.to_string(), "3.12".to_string(), None);
    std::fs::write(
        dir.join(".scoop-metadata.json"),
        serde_json::to_string(&meta).unwrap(),
    )
    .unwrap();
    dir
}

/// Runs `f` with `dir` as the working directory, restoring it after, also
/// when `f` panics (the temp dir is gone by then, and later tests would
/// inherit a deleted working directory).
fn in_dir<T>(dir: &Path, f: impl FnOnce() -> T) -> T {
    struct Restore(PathBuf);
    impl Drop for Restore {
        fn drop(&mut self) {
            let _ = std::env::set_current_dir(&self.0);
        }
    }
    let _restore = Restore(std::env::current_dir().unwrap());
    std::env::set_current_dir(dir).unwrap();
    f()
}

fn quiet() -> Output {
    Output::new(0, true, crate::output::Colors::NONE, false)
}

fn assert_invalid_name(result: crate::error::Result<()>) {
    assert!(
        matches!(result, Err(ScoopError::InvalidEnvName { ref name, .. }) if name == NAME),
        "{result:?}"
    );
}

#[test]
#[serial]
fn existing_env_is_found_and_removed() {
    run(&[], |envs, _| {
        let dir = existing_env(envs);
        let service = VirtualenvService::auto().unwrap();
        assert!(service.exists(NAME).unwrap());
        assert_eq!(service.get_path(NAME).unwrap(), dir);

        super::remove::execute(&quiet(), NAME, true).unwrap();
        assert!(!dir.exists());
    });
}

#[test]
#[serial]
fn existing_env_records_its_last_use() {
    run(&[], |envs, _| {
        let dir = existing_env(envs);
        let service = VirtualenvService::auto().unwrap();
        service.touch_metadata_at(NAME, chrono::Utc::now());
        let meta = service.read_metadata(&dir).unwrap();
        assert!(meta.last_used.is_some());
    });
}

#[test]
#[serial]
fn version_file_naming_the_env_resolves() {
    run(&[], |_, work| {
        std::fs::write(work.join(".scuv-version"), NAME).unwrap();
        assert_eq!(VersionService::resolve(work).as_deref(), Some(NAME));
    });
}

#[test]
#[serial]
fn scuv_version_naming_the_env_resolves() {
    run(&[("SCUV_VERSION", Some(NAME))], |_, work| {
        assert_eq!(VersionService::resolve(work).as_deref(), Some(NAME));
    });
}

/// What fails this: the name check in `create::execute` moving after the
/// `--force` removal (the env is gone before the name is refused).
#[test]
#[serial]
fn create_refuses_the_name_without_removing_the_env() {
    run(&[], |envs, _| {
        let dir = existing_env(envs);
        let result = super::create::execute(&quiet(), NAME, "3.12", None, true, false);
        assert_invalid_name(result);
        assert!(dir.join("keep").exists());
    });
}

fn write_export(dir: &Path) -> String {
    let file = dir.join("gc.json");
    std::fs::write(
        &file,
        format!(
            r#"{{"scoop_export_version": "1", "environment": {{"name": "{NAME}", "python": "3.12"}}, "packages": []}}"#
        ),
    )
    .unwrap();
    file.display().to_string()
}

/// What fails this: import skipping the name check of the env it creates,
/// or running it after the `--force` removal.
#[test]
#[serial]
fn import_refuses_the_name_without_removing_the_env() {
    run(&[], |envs, work| {
        let dir = existing_env(envs);
        let file = write_export(work);
        assert_invalid_name(super::import::execute(&quiet(), &file, None, true));
        assert!(dir.join("keep").exists());
    });
}

/// What fails this: the schema check refusing reserved words again.
#[test]
#[serial]
fn import_with_a_new_name_recovers_the_export() {
    run(&[], |envs, work| {
        let file = write_export(work);
        super::import::execute(&quiet(), &file, Some("recovered"), false).unwrap();
        assert!(envs.join("recovered").join("pyvenv.cfg").exists());
    });
}

fn write_manifest(dir: &Path) {
    std::fs::write(
        dir.join(".scuv.toml"),
        format!("[environment]\nname = \"{NAME}\"\npython = \"3.12\"\n"),
    )
    .unwrap();
}

/// What fails this: sync checking the strict name even when the env exists.
#[test]
#[serial]
fn sync_uses_the_existing_env() {
    run(&[], |envs, work| {
        let dir = existing_env(envs);
        write_manifest(work);
        in_dir(work, || {
            super::sync::execute(&quiet(), &[], true).unwrap();
            super::sync::execute(&quiet(), &[], false).unwrap();
        });
        assert!(dir.join("keep").exists());
    });
}

/// What fails this: the absent-env name check moving after the dry-run
/// plan, or dropping it (the fake uv would create the env).
#[test]
#[serial]
fn sync_refuses_to_create_the_env() {
    run(&[], |envs, work| {
        write_manifest(work);
        in_dir(work, || {
            assert_invalid_name(super::sync::execute(&quiet(), &[], true));
            assert_invalid_name(super::sync::execute(&quiet(), &[], false));
        });
        assert!(!envs.join(NAME).exists());
    });
}
