//! Handler for the `scuv export` command.
//!
//! Writes a portable, versioned JSON document describing the env so another
//! machine can recreate it with `scuv import`. Stdout *is* the schema, so
//! status messages always go to stderr — keeps the command pipe-friendly:
//! `scuv export myenv > myenv.json`.

use std::path::Path;

use rust_i18n::t;

use crate::core::{ExportSchema, VirtualenvService, try_list_installed_packages};
use crate::error::{Result, ScoopError};
use crate::output::Output;

/// Execute the `export` command.
pub fn execute(output: &Output, name: &str, dest: Option<&Path>) -> Result<()> {
    let service = VirtualenvService::auto()?;
    let env_path = service.get_path(name)?;
    let metadata = service.read_metadata(&env_path);

    // `scuv import` needs the Python version, so an env without metadata
    // cannot be exported (the same refusal as `scuv clone`).
    let metadata = metadata.ok_or_else(|| ScoopError::CorruptedEnvironment {
        name: name.to_string(),
        reason: "missing metadata".to_string(),
    })?;
    let python = metadata.python_version.clone();
    let created_at = Some(metadata.created_at.to_rfc3339());
    let packages = try_list_installed_packages(&env_path)?;

    let schema = ExportSchema::new(name.to_string(), python, created_at, packages);
    let json = schema.to_json_pretty();

    match dest {
        Some(path) => {
            std::fs::write(path, &json)?;
            output.success(&t!("export.written", name = name, path = path.display()));
        }
        None => {
            // Schema → stdout; status stays on stderr (already the case for
            // Output::success). Pipe-safe.
            println!("{json}");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::with_temp_scoop_home;
    use serial_test::serial;

    #[test]
    #[serial]
    fn execute_returns_not_found_for_missing_env() {
        with_temp_scoop_home(|temp_dir| {
            std::fs::create_dir_all(temp_dir.path().join("virtualenvs")).unwrap();
            let output = Output::new(0, true, crate::output::Colors::NONE, false);
            let err = execute(&output, "ghost", None).unwrap_err();
            assert!(matches!(err, ScoopError::VirtualenvNotFound { .. }));
        });
    }

    /// Export against the fake uv: `SCUV_HOME` is a fresh directory and the
    /// fake is first on `PATH`.
    #[cfg(unix)]
    fn with_fake_uv(extra: &[(&'static str, Option<&str>)], f: impl FnOnce(&std::path::Path)) {
        let uv = crate::test_utils::FakeUv::new(&[]);
        let home = tempfile::TempDir::new().unwrap();
        let path = uv.path_var();
        let home_str = home.path().display().to_string();
        let mut vars: Vec<(&'static str, Option<&str>)> = vec![
            ("PATH", Some(path.as_str())),
            ("SCUV_HOME", Some(home_str.as_str())),
        ];
        vars.extend_from_slice(extra);
        let _env = crate::test_utils::env_guard(&vars);
        f(home.path());
    }

    /// An env directory under `home`, with metadata if asked.
    #[cfg(unix)]
    fn env(home: &std::path::Path, name: &str, with_metadata: bool) {
        let dir = home.join("virtualenvs").join(name);
        std::fs::create_dir_all(dir.join("bin")).unwrap();
        if with_metadata {
            let meta = crate::core::Metadata::new(name.to_string(), "3.12.0".to_string(), None);
            std::fs::write(
                dir.join(".scoop-metadata.json"),
                serde_json::to_string(&meta).unwrap(),
            )
            .unwrap();
        }
    }

    #[test]
    #[serial]
    #[cfg(unix)]
    fn execute_writes_file_when_dest_given() {
        with_fake_uv(&[], |home| {
            env(home, "snap", true);
            let out_file = home.join("snap.json");
            let output = Output::new(0, true, crate::output::Colors::NONE, false);
            execute(&output, "snap", Some(&out_file)).expect("export should succeed");

            let contents = std::fs::read_to_string(&out_file).unwrap();
            let parsed = ExportSchema::from_json(&contents, &out_file).unwrap();
            assert_eq!(parsed.environment.name, "snap");
            assert_eq!(parsed.environment.python, "3.12.0");
            assert_eq!(parsed.packages.len(), 1, "{contents}");
        });
    }

    /// Without metadata there is no Python version to record; an export
    /// with `"python": ""` cannot be imported.
    #[test]
    #[serial]
    #[cfg(unix)]
    fn execute_refuses_an_env_without_metadata() {
        with_fake_uv(&[], |home| {
            env(home, "bare", false);
            let out_file = home.join("bare.json");
            let output = Output::new(0, true, crate::output::Colors::NONE, false);
            let err = execute(&output, "bare", Some(&out_file)).unwrap_err();
            assert!(
                matches!(err, ScoopError::CorruptedEnvironment { .. }),
                "{err}"
            );
            assert!(!out_file.exists());
        });
    }

    /// A failed package listing is an error, not an export with no packages.
    #[test]
    #[serial]
    #[cfg(unix)]
    fn execute_fails_when_package_listing_fails() {
        with_fake_uv(&[("FAKE_UV_PIP_LIST_FAILS", Some("1"))], |home| {
            env(home, "snap", true);
            let out_file = home.join("snap.json");
            let output = Output::new(0, true, crate::output::Colors::NONE, false);
            let err = execute(&output, "snap", Some(&out_file)).unwrap_err();
            assert!(matches!(err, ScoopError::UvCommandFailed { .. }), "{err}");
            assert!(!out_file.exists());
        });
    }
}
