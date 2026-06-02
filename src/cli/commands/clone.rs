//! Handler for the `scoop clone` command.
//!
//! Composes existing service primitives (`create` + `list_installed_packages` +
//! `pip_install`) rather than adding a clone method on `VirtualenvService`:
//! the operation is "create a new env at the same python and optionally
//! re-install the source's pinned packages", which is more cleanly expressed
//! at the handler layer than as a bespoke service API.

use rust_i18n::t;

use crate::core::{VirtualenvService, list_installed_packages};
use crate::error::{Result, ScoopError};
use crate::output::{CloneData, Output};
use crate::paths::abbreviate_home;
use crate::validate;

/// Execute the `clone` command.
pub fn execute(
    output: &Output,
    src: &str,
    dst: &str,
    no_packages: bool,
    force: bool,
) -> Result<()> {
    // Validate destination name *before* touching the source env so users get
    // the most actionable error first.
    validate::validate_env_name(dst)?;

    if src == dst {
        return Err(ScoopError::InvalidArgument {
            message: t!("clone.self_clone_error").to_string(),
        });
    }

    let service = VirtualenvService::auto()?;

    if !service.exists(src)? {
        return Err(ScoopError::VirtualenvNotFound {
            name: src.to_string(),
        });
    }
    if service.exists(dst)? {
        if force {
            service.delete(dst)?;
        } else {
            return Err(ScoopError::VirtualenvExists {
                name: dst.to_string(),
            });
        }
    }

    let src_path = service.get_path(src)?;
    let metadata = service.read_metadata(&src_path);
    let python = metadata
        .as_ref()
        .map(|m| m.python_version.clone())
        .ok_or_else(|| ScoopError::CorruptedEnvironment {
            name: src.to_string(),
            reason: "missing metadata".to_string(),
        })?;

    output.info(&t!("clone.cloning", src = src, dst = dst, python = python));

    let dst_path = service.create(dst, &python)?;

    let packages = if no_packages {
        output.info(&t!("clone.no_packages_skipped"));
        Vec::new()
    } else {
        let pkgs = list_installed_packages(&src_path);
        if pkgs.is_empty() {
            Vec::new()
        } else {
            output.info(&t!("clone.copying_packages", count = pkgs.len()));
            let specs: Vec<String> = pkgs.iter().map(|(n, v)| format!("{n}=={v}")).collect();
            service.pip_install(&dst_path, &specs)?;
            pkgs
        }
    };

    if output.is_json() {
        output.json_success(
            "clone",
            CloneData {
                src: src.to_string(),
                dst: dst.to_string(),
                python: python.clone(),
                path: dst_path.display().to_string(),
                packages_copied: packages.len(),
                packages_skipped: no_packages,
            },
        );
        return Ok(());
    }

    output.success(&t!("clone.success", src = src, dst = dst));
    output.info(&format!("  Path: {}", abbreviate_home(&dst_path)));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::with_temp_scoop_home;
    use serial_test::serial;

    #[test]
    #[serial]
    fn execute_rejects_invalid_dst_name() {
        with_temp_scoop_home(|temp_dir| {
            std::fs::create_dir_all(temp_dir.path().join("virtualenvs")).unwrap();
            let output = Output::new(0, true, true, false);
            // "list" is reserved.
            let err = execute(&output, "src", "list", false, false).unwrap_err();
            assert!(matches!(err, ScoopError::InvalidEnvName { .. }));
        });
    }

    #[test]
    #[serial]
    fn execute_rejects_self_clone() {
        with_temp_scoop_home(|temp_dir| {
            std::fs::create_dir_all(temp_dir.path().join("virtualenvs")).unwrap();
            let output = Output::new(0, true, true, false);
            let err = execute(&output, "same", "same", false, false).unwrap_err();
            assert!(matches!(err, ScoopError::InvalidArgument { .. }));
        });
    }

    #[test]
    #[serial]
    fn execute_returns_not_found_when_src_missing() {
        with_temp_scoop_home(|temp_dir| {
            std::fs::create_dir_all(temp_dir.path().join("virtualenvs")).unwrap();
            let output = Output::new(0, true, true, false);
            let err = execute(&output, "ghost", "newcopy", false, false).unwrap_err();
            assert!(matches!(err, ScoopError::VirtualenvNotFound { .. }));
        });
    }

    #[test]
    #[serial]
    fn execute_rejects_existing_dst_without_force() {
        with_temp_scoop_home(|temp_dir| {
            // Set up a valid src with metadata + a colliding dst.
            let src = temp_dir.path().join("virtualenvs").join("source");
            std::fs::create_dir_all(src.join("bin")).unwrap();
            let meta = crate::core::Metadata {
                name: "source".to_string(),
                python_version: "3.12.0".to_string(),
                created_at: chrono::Utc::now(),
                created_by: "test".to_string(),
                uv_version: None,
                python_path: None,
                last_used: None,
            };
            std::fs::write(
                src.join(".scoop-metadata.json"),
                serde_json::to_string(&meta).unwrap(),
            )
            .unwrap();

            std::fs::create_dir_all(temp_dir.path().join("virtualenvs").join("dupe")).unwrap();

            let output = Output::new(0, true, true, false);
            let err = execute(&output, "source", "dupe", false, false /* no force */).unwrap_err();
            assert!(matches!(err, ScoopError::VirtualenvExists { .. }));
        });
    }

    #[test]
    #[serial]
    fn execute_surfaces_corrupted_when_metadata_missing() {
        with_temp_scoop_home(|temp_dir| {
            // src exists as a directory but lacks the metadata file.
            std::fs::create_dir_all(
                temp_dir
                    .path()
                    .join("virtualenvs")
                    .join("nometa")
                    .join("bin"),
            )
            .unwrap();
            let output = Output::new(0, true, true, false);
            let err = execute(&output, "nometa", "newone", false, false).unwrap_err();
            assert!(matches!(err, ScoopError::CorruptedEnvironment { .. }));
        });
    }
}
