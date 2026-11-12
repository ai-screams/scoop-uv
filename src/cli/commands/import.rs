//! Handler for the `scuv import` command.
//!
//! Reads a `scuv export` JSON file (or stdin via `-`), validates the schema,
//! checks that every package can be put back where it came from, creates the
//! env (with implicit lazy Python install — same ergonomic call as
//! `scuv sync`), then installs the packages via uv pip. A failed install
//! removes the env this call created.

use std::io::Read;
use std::path::{Path, PathBuf};

use rust_i18n::t;

use crate::core::packages::prepare_install;
use crate::core::{ExportSchema, VirtualenvService};
use crate::error::{Result, ScoopError};
use crate::output::{ImportData, Output};

/// Execute the `import` command. `source` may be a file path or `-` for stdin.
pub fn execute(
    output: &Output,
    source: &str,
    name_override: Option<&str>,
    force: bool,
) -> Result<()> {
    let (raw_json, source_path) = load_source(source)?;
    let mut schema = ExportSchema::from_json(&raw_json, &source_path)?;

    if let Some(new_name) = name_override {
        schema.environment.name = new_name.to_string();
    }
    // The schema check accepts any name an env may already have, so a file
    // exported from an env named before its word was reserved still loads.
    // The env this creates needs a name a new env may take; check it before
    // `--force` removes anything or a Python is installed.
    crate::validate::validate_env_name(&schema.environment.name)?;

    let service = VirtualenvService::auto()?;
    let env_name = schema.environment.name.clone();
    let python = schema.environment.python.clone();

    // Local sources and hash support are checked before `--force` deletes
    // anything or a Python is installed.
    let plan = prepare_install(
        &schema.installed_packages(),
        &source_path.display().to_string(),
        &crate::paths::virtualenv_path(&env_name)?,
    )?;

    if service.exists(&env_name)? {
        if force {
            service.delete(&env_name)?;
        } else {
            return Err(ScoopError::VirtualenvExists { name: env_name });
        }
    }

    output.info(&t!("import.creating", name = env_name, python = python));

    // Match `scuv sync`'s ergonomic default: declarative entry points should
    // bootstrap missing Python rather than fail.
    if !service.is_python_installed(&python)? {
        output.info(&t!("create.installing_python", version = python));
        service.install_python(&python)?;
    }
    let venv_path = service.create(&env_name, &python)?;
    let created = crate::paths::dir_identity(&venv_path).ok();

    if plan.packages() > 0 {
        output.info(&t!("import.installing", count = plan.packages()));
        if let Err(e) = service.pip_install(&venv_path, plan.args()) {
            // Best effort, like `clone`: this call made the env, nothing
            // else depends on it, and the install error is the one to
            // report. Only that directory goes (an env another command put
            // under the name meanwhile stays), and an env `--force` removed
            // earlier is not restored.
            if let Some(created) = created {
                let _ = crate::paths::remove_dir_if_unchanged(&venv_path, created);
            }
            return Err(e);
        }
    }

    if output.is_json() {
        output.json_success(
            "import",
            ImportData {
                name: env_name.clone(),
                python: python.clone(),
                packages_installed: plan.packages(),
                source: source_path.display().to_string(),
            },
        );
        return Ok(());
    }

    output.success(&t!(
        "import.success",
        name = env_name,
        count = plan.packages()
    ));
    Ok(())
}

/// Load JSON content from a file path or stdin (`-`). Returns the raw text
/// plus a `Path` to attach to error messages.
fn load_source(source: &str) -> Result<(String, PathBuf)> {
    if source == "-" {
        let mut buf = String::new();
        std::io::stdin()
            .read_to_string(&mut buf)
            .map_err(|e| ScoopError::InvalidExportFile {
                path: PathBuf::from("<stdin>"),
                reason: e.to_string(),
            })?;
        return Ok((buf, PathBuf::from("<stdin>")));
    }
    let path = Path::new(source);
    let content = std::fs::read_to_string(path)?;
    Ok((content, path.to_path_buf()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::with_temp_scoop_home;
    use serial_test::serial;

    fn write_export_file(dir: &Path, body: &str) -> PathBuf {
        let p = dir.join("export.json");
        std::fs::write(&p, body).unwrap();
        p
    }

    #[test]
    #[serial]
    fn execute_rejects_invalid_json() {
        with_temp_scoop_home(|temp_dir| {
            let file = write_export_file(temp_dir.path(), "not json");
            let output = Output::new(0, true, crate::output::Colors::NONE, false);
            let err = execute(&output, file.to_str().unwrap(), None, false).unwrap_err();
            assert!(matches!(err, ScoopError::InvalidExportFile { .. }));
        });
    }

    #[test]
    #[serial]
    fn execute_rejects_future_schema_version() {
        with_temp_scoop_home(|temp_dir| {
            let payload = r#"{
                "scoop_export_version": "99",
                "environment": { "name": "x", "python": "3.12" },
                "packages": []
            }"#;
            let file = write_export_file(temp_dir.path(), payload);
            let output = Output::new(0, true, crate::output::Colors::NONE, false);
            let err = execute(&output, file.to_str().unwrap(), None, false).unwrap_err();
            assert!(matches!(err, ScoopError::UnsupportedExportVersion { .. }));
        });
    }

    #[test]
    #[serial]
    fn execute_rejects_invalid_name_override() {
        with_temp_scoop_home(|temp_dir| {
            // Valid schema but the user overrides with a reserved word.
            let payload = r#"{
                "scoop_export_version": "1",
                "environment": { "name": "ok-env", "python": "3.12" },
                "packages": []
            }"#;
            let file = write_export_file(temp_dir.path(), payload);
            let output = Output::new(0, true, crate::output::Colors::NONE, false);
            let err = execute(
                &output,
                file.to_str().unwrap(),
                Some("list"), /* reserved */
                false,
            )
            .unwrap_err();
            assert!(matches!(err, ScoopError::InvalidEnvName { .. }));
        });
    }

    #[test]
    #[serial]
    fn execute_rejects_existing_env_without_force() {
        with_temp_scoop_home(|temp_dir| {
            // Pre-create a directory so `service.exists` returns true. We
            // never reach the uv-touching code path because we expect an
            // early VirtualenvExists error.
            std::fs::create_dir_all(temp_dir.path().join("virtualenvs").join("dupe")).unwrap();

            let payload = r#"{
                "scoop_export_version": "1",
                "environment": { "name": "dupe", "python": "3.12" },
                "packages": []
            }"#;
            let file = write_export_file(temp_dir.path(), payload);
            let output = Output::new(0, true, crate::output::Colors::NONE, false);
            let err = execute(
                &output,
                file.to_str().unwrap(),
                None,
                false, /* no force */
            )
            .unwrap_err();
            assert!(matches!(err, ScoopError::VirtualenvExists { .. }));
        });
    }

    /// Import against the fake uv, with Python 3.12 installed.
    #[cfg(unix)]
    mod with_fake_uv {
        use super::*;
        use crate::test_utils::{FakeUv, env_guard};

        fn run(extra: &[(&'static str, Option<&str>)], f: impl FnOnce(&Path, &FakeUv)) {
            let uv = FakeUv::new(&["3.12.0"]);
            let home = tempfile::TempDir::new().unwrap();
            let path = uv.path_var();
            let home_str = home.path().display().to_string();
            let mut vars: Vec<(&'static str, Option<&str>)> = vec![
                ("PATH", Some(path.as_str())),
                ("SCUV_HOME", Some(home_str.as_str())),
            ];
            vars.extend_from_slice(extra);
            let _env = env_guard(&vars);
            std::fs::create_dir_all(home.path().join("virtualenvs")).unwrap();
            f(home.path(), &uv);
        }

        fn quiet() -> Output {
            Output::new(0, true, crate::output::Colors::NONE, false)
        }

        fn export(home: &Path, version: &str, packages: &str) -> PathBuf {
            write_export_file(
                home,
                &format!(
                    r#"{{"scoop_export_version":"{version}","environment":{{"name":"imp","python":"3.12"}},"packages":{packages}}}"#
                ),
            )
        }

        /// A failed install removes the env this import created and
        /// reports the install error. What fails this: `?` on the install.
        #[test]
        #[serial]
        fn install_failure_removes_the_new_env() {
            run(&[("FAKE_UV_PIP_INSTALL_FAILS", Some("all"))], |home, _| {
                let file = export(
                    home,
                    "2",
                    r#"[{"name":"six","version":"1.16.0","source":null}]"#,
                );
                let err = execute(&quiet(), file.to_str().unwrap(), None, false).unwrap_err();
                assert!(matches!(err, ScoopError::UvCommandFailed { .. }), "{err:?}");
                assert!(
                    !home.join("virtualenvs/imp").exists(),
                    "half-built env left behind"
                );
            });
        }

        /// A version 2 editable package is installed from its path.
        #[test]
        #[serial]
        fn installs_an_editable_package_from_its_path() {
            let work = tempfile::TempDir::new().unwrap();
            let proj = std::fs::canonicalize(work.path()).unwrap();
            run(&[], |home, uv| {
                let packages = format!(
                    r#"[{{"name":"six","version":"1.16.0","source":null}},{{"name":"p","version":"1","source":{{"url":"file://{}","dir_info":{{"editable":true}}}}}}]"#,
                    proj.display()
                );
                let file = export(home, "2", &packages);
                execute(&quiet(), file.to_str().unwrap(), None, false).unwrap();
                let installs = uv.install_args();
                let last = installs.last().expect("an install call");
                let tail: Vec<&str> = last
                    .iter()
                    .rev()
                    .take(3)
                    .rev()
                    .map(String::as_str)
                    .collect();
                assert_eq!(
                    tail,
                    ["six==1.16.0", "--editable", proj.to_str().unwrap()],
                    "{installs:?}"
                );
            });
        }

        /// A local source missing on this machine stops the import before
        /// `--force` deletes the existing env.
        #[test]
        #[serial]
        fn force_keeps_the_env_when_a_local_source_is_missing() {
            run(&[], |home, _| {
                let existing = home.join("virtualenvs/imp");
                std::fs::create_dir_all(&existing).unwrap();
                std::fs::write(existing.join("keep"), "").unwrap();
                let file = export(
                    home,
                    "2",
                    r#"[{"name":"p","version":"1","source":{"url":"file:///nonexistent/scuv/p","dir_info":{}}}]"#,
                );
                let err = execute(&quiet(), file.to_str().unwrap(), None, true).unwrap_err();
                assert!(
                    matches!(err, ScoopError::PackageSourceMissing { .. }),
                    "{err:?}"
                );
                assert!(
                    existing.join("keep").exists(),
                    "--force removed the env before failing"
                );
            });
        }

        /// A version 1 file still imports, as `name==version`.
        #[test]
        #[serial]
        fn imports_a_version_1_file() {
            run(&[], |home, uv| {
                let file = export(home, "1", r#"[{"name":"six","version":"1.16.0"}]"#);
                execute(&quiet(), file.to_str().unwrap(), None, false).unwrap();
                let installs = uv.install_args();
                assert!(
                    installs.iter().flatten().any(|a| a == "six==1.16.0"),
                    "{installs:?}"
                );
                assert!(home.join("virtualenvs/imp").exists());
            });
        }
    }
}
