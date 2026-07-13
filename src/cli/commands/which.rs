//! Handler for the `scuv which` command.
//!
//! Resolves the absolute path of an executable within a scuv environment's
//! `bin/` directory — the pyenv-which equivalent for scuv.

use crate::core::{VersionService, VirtualenvService, get_active_env};
use crate::error::{Result, ScoopError};
use crate::output::{Output, WhichData};
use crate::{paths, validate};

/// Execute the `which` command.
pub fn execute(output: &Output, exe: &str, env: Option<&str>) -> Result<()> {
    // Resolve the target env first so error messages can name *where* we
    // would have looked even when the exe is rejected as path-like.
    let env_name = resolve_target_env(env)?;

    // Reject path-like executable names to avoid escaping the bin directory.
    if exe.is_empty() || exe.contains('/') || exe.contains('\\') {
        return Err(ScoopError::ExecutableNotFound {
            exe: exe.to_string(),
            env: env_name,
        });
    }

    let service = VirtualenvService::auto()?;
    if !service.exists(&env_name)? {
        return Err(ScoopError::VirtualenvNotFound { name: env_name });
    }

    let bin_dir = paths::virtualenv_bin(&env_name)?;
    let resolved =
        paths::find_executable_in(&bin_dir, exe).ok_or_else(|| ScoopError::ExecutableNotFound {
            exe: exe.to_string(),
            env: env_name.clone(),
        })?;

    if output.is_json() {
        output.json_success(
            "which",
            WhichData {
                exe: exe.to_string(),
                env: env_name,
                path: resolved.display().to_string(),
            },
        );
        return Ok(());
    }

    // Stdout regardless of `--quiet`: the path is the command's only output.
    println!("{}", resolved.display());
    Ok(())
}

/// Resolve the environment to look in: explicit `--env`, otherwise the active
/// (`SCUV_ACTIVE`) env, otherwise the version-file resolution.
fn resolve_target_env(explicit: Option<&str>) -> Result<String> {
    if let Some(name) = explicit {
        validate::validate_env_name(name)?;
        return Ok(name.to_string());
    }
    get_active_env()
        .filter(|n| !n.is_empty())
        .or_else(VersionService::resolve_current)
        .filter(|n| n != "system")
        .ok_or(ScoopError::NoActiveEnvironment)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::with_temp_scoop_home;
    use serial_test::serial;
    use tempfile::TempDir;

    #[test]
    fn execute_rejects_path_separator_in_exe() {
        let output = Output::new(0, true, true, false);
        let err = execute(&output, "../python", Some("myenv")).unwrap_err();
        assert!(matches!(err, ScoopError::ExecutableNotFound { .. }));
    }

    #[test]
    fn execute_rejects_empty_exe() {
        let output = Output::new(0, true, true, false);
        let err = execute(&output, "", Some("myenv")).unwrap_err();
        assert!(matches!(err, ScoopError::ExecutableNotFound { .. }));
    }

    #[test]
    #[serial]
    fn execute_returns_not_found_for_missing_env() {
        with_temp_scoop_home(|temp_dir| {
            std::fs::create_dir_all(temp_dir.path().join("virtualenvs")).unwrap();
            let output = Output::new(0, true, true, false);
            let err = execute(&output, "python", Some("nonexistent")).unwrap_err();
            assert!(matches!(err, ScoopError::VirtualenvNotFound { .. }));
        });
    }

    #[test]
    #[serial]
    fn execute_returns_no_active_when_unset() {
        with_temp_scoop_home(|temp_dir| {
            std::fs::create_dir_all(temp_dir.path().join("virtualenvs")).unwrap();
            // SAFETY: serial test; no concurrent env access.
            unsafe {
                std::env::remove_var("SCUV_ACTIVE");
            }
            let output = Output::new(0, true, true, false);
            // Use a tempdir as CWD that has no .scuv-version file.
            let workdir = TempDir::new().unwrap();
            let prev = std::env::current_dir().ok();
            std::env::set_current_dir(workdir.path()).unwrap();

            let err = execute(&output, "python", None).unwrap_err();
            assert!(matches!(err, ScoopError::NoActiveEnvironment));

            if let Some(p) = prev {
                std::env::set_current_dir(p).unwrap();
            }
        });
    }

    #[test]
    #[serial]
    fn execute_finds_executable_in_env() {
        with_temp_scoop_home(|temp_dir| {
            let bin = temp_dir
                .path()
                .join("virtualenvs")
                .join("myenv")
                .join("bin");
            std::fs::create_dir_all(&bin).unwrap();
            std::fs::write(bin.join("pytest"), b"").unwrap();

            let output = Output::new(0, true, true, false);
            // The handler prints to stdout; we just need the Result to be Ok.
            assert!(execute(&output, "pytest", Some("myenv")).is_ok());
        });
    }
}
