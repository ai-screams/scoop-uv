//! Handler for the `scuv clone` command.
//!
//! Composes existing service primitives (`create` + `try_list_installed_packages` +
//! `pip_install`) rather than adding a clone method on `VirtualenvService`:
//! the operation is "create a new env at the same python and optionally
//! re-install the source's pinned packages", which is more cleanly expressed
//! at the handler layer than as a bespoke service API.

use rust_i18n::t;

use crate::core::{VirtualenvService, try_list_installed_packages};
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

    let src_path = service.get_path(src)?;
    let dst_exists = service.exists(dst)?;
    if dst_exists {
        if !force {
            return Err(ScoopError::VirtualenvExists {
                name: dst.to_string(),
            });
        }
        // Two names can reach one directory (`Web` and `web` on a
        // case-insensitive filesystem); `--force` would then delete the source.
        if std::fs::canonicalize(&src_path)? == std::fs::canonicalize(service.get_path(dst)?)? {
            return Err(ScoopError::InvalidArgument {
                message: t!("clone.self_clone_error").to_string(),
            });
        }
    }

    // Everything read from the source comes before `--force` removes the
    // destination, so a source that cannot be cloned leaves it in place.
    let metadata = service.read_metadata(&src_path);
    let python = metadata
        .as_ref()
        .map(|m| m.python_version.clone())
        .ok_or_else(|| ScoopError::CorruptedEnvironment {
            name: src.to_string(),
            reason: "missing metadata".to_string(),
        })?;
    let packages = if no_packages {
        Vec::new()
    } else {
        try_list_installed_packages(&src_path)?
    };

    if dst_exists {
        service.delete(dst)?;
    }

    output.info(&t!("clone.cloning", src = src, dst = dst, python = python));

    let dst_path = service.create(dst, &python)?;

    if no_packages {
        output.info(&t!("clone.no_packages_skipped"));
    } else if !packages.is_empty() {
        output.info(&t!("clone.copying_packages", count = packages.len()));
        let specs: Vec<String> = packages.iter().map(|(n, v)| format!("{n}=={v}")).collect();
        if let Err(e) = service.pip_install(&dst_path, &specs) {
            // Best effort: the env was made by this call, so nothing else
            // depends on it. The install error is the one to report.
            let _ = service.delete(dst);
            return Err(e);
        }
    }

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
            let output = Output::new(0, true, crate::output::Colors::NONE, false);
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
            let output = Output::new(0, true, crate::output::Colors::NONE, false);
            let err = execute(&output, "same", "same", false, false).unwrap_err();
            assert!(matches!(err, ScoopError::InvalidArgument { .. }));
        });
    }

    #[test]
    #[serial]
    fn execute_returns_not_found_when_src_missing() {
        with_temp_scoop_home(|temp_dir| {
            std::fs::create_dir_all(temp_dir.path().join("virtualenvs")).unwrap();
            let output = Output::new(0, true, crate::output::Colors::NONE, false);
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

            let output = Output::new(0, true, crate::output::Colors::NONE, false);
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
            let output = Output::new(0, true, crate::output::Colors::NONE, false);
            let err = execute(&output, "nometa", "newone", false, false).unwrap_err();
            assert!(matches!(err, ScoopError::CorruptedEnvironment { .. }));
        });
    }

    /// Clone against the fake uv: `SCUV_HOME` is a fresh directory and the
    /// fake is first on `PATH`, so `create`, `pip list` and `pip install`
    /// run without a real uv or Python.
    #[cfg(unix)]
    mod with_fake_uv {
        use super::*;
        use crate::test_utils::{FakeUv, env_guard};
        use std::path::{Path, PathBuf};

        fn run<T>(
            extra: &[(&'static str, Option<&str>)],
            f: impl FnOnce(&Path, &FakeUv) -> T,
        ) -> T {
            let uv = FakeUv::new(&[]);
            let home = tempfile::TempDir::new().unwrap();
            let path = uv.path_var();
            let home_str = home.path().display().to_string();
            let mut vars: Vec<(&'static str, Option<&str>)> = vec![
                ("PATH", Some(path.as_str())),
                ("SCUV_HOME", Some(home_str.as_str())),
            ];
            vars.extend_from_slice(extra);
            let _env = env_guard(&vars);
            f(&home.path().join("virtualenvs"), &uv)
        }

        /// An env directory with a `keep` marker file, and metadata if asked.
        fn env(envs: &Path, name: &str, with_metadata: bool) -> PathBuf {
            let dir = envs.join(name);
            std::fs::create_dir_all(dir.join("bin")).unwrap();
            std::fs::write(dir.join("keep"), "").unwrap();
            if with_metadata {
                let meta = crate::core::Metadata::new(name.to_string(), "3.12".to_string(), None);
                std::fs::write(
                    dir.join(".scoop-metadata.json"),
                    serde_json::to_string(&meta).unwrap(),
                )
                .unwrap();
            }
            dir
        }

        fn quiet() -> Output {
            Output::new(0, true, crate::output::Colors::NONE, false)
        }

        #[test]
        #[serial]
        fn copies_the_source_packages() {
            run(&[], |envs, uv| {
                env(envs, "src", true);
                execute(&quiet(), "src", "dst", false, false).unwrap();
                assert!(envs.join("dst").join("pyvenv.cfg").exists());
                let calls = uv.pip_calls();
                assert!(
                    calls
                        .iter()
                        .any(|c| c.starts_with("pip install") && c.ends_with("six==1.16.0")),
                    "{calls:?}"
                );
            });
        }

        /// `--force` replaces an existing destination once the source checks
        /// pass. Fails if the deletion is dropped: `create` then refuses the
        /// existing directory.
        #[test]
        #[serial]
        fn force_replaces_an_existing_destination() {
            run(&[], |envs, _| {
                env(envs, "src", true);
                let dst = env(envs, "dst", true);
                execute(&quiet(), "src", "dst", false, true).unwrap();
                assert!(!dst.join("keep").exists(), "old destination still there");
                assert!(dst.join("pyvenv.cfg").exists());
            });
        }

        /// The source check comes before `--force` removes the destination.
        #[test]
        #[serial]
        fn force_keeps_the_destination_when_source_metadata_is_missing() {
            run(&[], |envs, _| {
                env(envs, "src", false);
                let dst = env(envs, "dst", true);
                let err = execute(&quiet(), "src", "dst", false, true).unwrap_err();
                assert!(
                    matches!(err, ScoopError::CorruptedEnvironment { .. }),
                    "{err}"
                );
                assert!(
                    dst.join("keep").exists(),
                    "--force removed the destination before failing"
                );
            });
        }

        /// A failed package listing is an error, not an empty package list,
        /// and it comes before `--force` removes the destination.
        #[test]
        #[serial]
        fn force_keeps_the_destination_when_package_listing_fails() {
            run(&[("FAKE_UV_PIP_LIST_FAILS", Some("1"))], |envs, _| {
                env(envs, "src", true);
                let dst = env(envs, "dst", true);
                let err = execute(&quiet(), "src", "dst", false, true).unwrap_err();
                assert!(matches!(err, ScoopError::UvCommandFailed { .. }), "{err}");
                assert!(
                    dst.join("keep").exists(),
                    "--force removed the destination before failing"
                );
            });
        }

        #[test]
        #[serial]
        fn package_listing_failure_creates_no_destination() {
            run(&[("FAKE_UV_PIP_LIST_FAILS", Some("1"))], |envs, _| {
                env(envs, "src", true);
                assert!(execute(&quiet(), "src", "dst", false, false).is_err());
                assert!(!envs.join("dst").exists());
            });
        }

        /// `--no-packages` does not list packages, so a failing listing
        /// does not stop it.
        #[test]
        #[serial]
        fn no_packages_skips_the_listing() {
            run(&[("FAKE_UV_PIP_LIST_FAILS", Some("1"))], |envs, uv| {
                env(envs, "src", true);
                execute(&quiet(), "src", "dst", true, false).unwrap();
                assert!(envs.join("dst").exists());
                assert!(uv.pip_calls().is_empty(), "{:?}", uv.pip_calls());
            });
        }

        /// A clone whose install fails removes the env it just created.
        #[test]
        #[serial]
        fn install_failure_removes_the_new_destination() {
            run(&[("FAKE_UV_PIP_INSTALL_FAILS", Some("all"))], |envs, _| {
                env(envs, "src", true);
                assert!(execute(&quiet(), "src", "dst", false, false).is_err());
                assert!(
                    !envs.join("dst").exists(),
                    "half-built destination left behind"
                );
            });
        }

        /// Two names for one directory (a case-insensitive filesystem's
        /// `Web`/`web`, here a symlink) are a self-clone: `--force` would
        /// otherwise delete the source.
        #[test]
        #[serial]
        fn rejects_a_destination_that_is_the_source_directory() {
            run(&[], |envs, _| {
                let src = env(envs, "src", true);
                std::os::unix::fs::symlink(&src, envs.join("alias")).unwrap();
                let err = execute(&quiet(), "src", "alias", false, true).unwrap_err();
                assert!(matches!(err, ScoopError::InvalidArgument { .. }), "{err}");
                assert!(src.join("keep").exists());
            });
        }
    }
}
