//! Handler for the `scuv export` command.
//!
//! Writes a portable, versioned JSON document describing the env so another
//! machine can recreate it with `scuv import`. Stdout *is* the schema, so
//! status messages always go to stderr — keeps the command pipe-friendly:
//! `scuv export myenv > myenv.json`.

use std::path::Path;

use rust_i18n::t;

use crate::core::{ExportSchema, VirtualenvService, list_installed_packages};
use crate::error::{Result, ScoopError};
use crate::output::Output;

/// Execute the `export` command.
pub fn execute(output: &Output, name: &str, dest: Option<&Path>) -> Result<()> {
    let service = VirtualenvService::auto()?;
    if !service.exists(name)? {
        return Err(ScoopError::VirtualenvNotFound {
            name: name.to_string(),
        });
    }

    let env_path = service.get_path(name)?;
    let metadata = service.read_metadata(&env_path);

    let python = metadata
        .as_ref()
        .map(|m| m.python_version.clone())
        .unwrap_or_default();
    let created_at = metadata.as_ref().map(|m| m.created_at.to_rfc3339());
    let packages = list_installed_packages(&env_path);

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
            let output = Output::new(0, true, true, false);
            let err = execute(&output, "ghost", None).unwrap_err();
            assert!(matches!(err, ScoopError::VirtualenvNotFound { .. }));
        });
    }

    #[test]
    #[serial]
    fn execute_writes_file_when_dest_given() {
        with_temp_scoop_home(|temp_dir| {
            // Build a minimal-but-real env layout: dir + metadata file. We
            // can't run uv from a unit test so list_installed_packages will
            // return an empty list — that's fine, we just need the export
            // file to materialise with valid JSON.
            let env_dir = temp_dir.path().join("virtualenvs").join("snap");
            std::fs::create_dir_all(env_dir.join("bin")).unwrap();
            let meta = crate::core::Metadata {
                name: "snap".to_string(),
                python_version: "3.12.0".to_string(),
                created_at: chrono::Utc::now(),
                created_by: "test".to_string(),
                uv_version: None,
                python_path: None,
                last_used: None,
            };
            let meta_json = serde_json::to_string(&meta).unwrap();
            std::fs::write(env_dir.join(".scoop-metadata.json"), meta_json).unwrap();

            let out_file = temp_dir.path().join("snap.json");
            let output = Output::new(0, true, true, false);
            execute(&output, "snap", Some(&out_file)).expect("export should succeed");

            let contents = std::fs::read_to_string(&out_file).unwrap();
            let parsed = ExportSchema::from_json(&contents, &out_file).unwrap();
            assert_eq!(parsed.environment.name, "snap");
            assert_eq!(parsed.environment.python, "3.12.0");
        });
    }
}
