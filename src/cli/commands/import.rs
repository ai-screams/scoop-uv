//! Handler for the `scoop import` command.
//!
//! Reads a `scoop export` JSON file (or stdin via `-`), validates the schema,
//! creates the env (with implicit lazy Python install — same ergonomic call as
//! `scoop sync`), then installs every pinned package via uv pip.

use std::io::Read;
use std::path::{Path, PathBuf};

use rust_i18n::t;

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
        crate::validate::validate_env_name(new_name)?;
        schema.environment.name = new_name.to_string();
    }

    let service = VirtualenvService::auto()?;
    let env_name = schema.environment.name.clone();
    let python = schema.environment.python.clone();

    if service.exists(&env_name)? {
        if force {
            service.delete(&env_name)?;
        } else {
            return Err(ScoopError::VirtualenvExists { name: env_name });
        }
    }

    output.info(&t!("import.creating", name = env_name, python = python));

    // Match `scoop sync`'s ergonomic default: declarative entry points should
    // bootstrap missing Python rather than fail.
    if !service.is_python_installed(&python)? {
        output.info(&t!("create.installing_python", version = python));
        service.install_python(&python)?;
    }
    service.create(&env_name, &python)?;

    let pip_specs = schema.pip_specs();
    if !pip_specs.is_empty() {
        output.info(&t!("import.installing", count = pip_specs.len()));
        let venv_path = service.get_path(&env_name)?;
        service.pip_install(&venv_path, &pip_specs)?;
    }

    if output.is_json() {
        output.json_success(
            "import",
            ImportData {
                name: env_name.clone(),
                python: python.clone(),
                packages_installed: pip_specs.len(),
                source: source_path.display().to_string(),
            },
        );
        return Ok(());
    }

    output.success(&t!(
        "import.success",
        name = env_name,
        count = pip_specs.len()
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
            let output = Output::new(0, true, true, false);
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
            let output = Output::new(0, true, true, false);
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
            let output = Output::new(0, true, true, false);
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
            let output = Output::new(0, true, true, false);
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
}
