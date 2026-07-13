//! Handler for the `scuv run` command.
//!
//! Spawns a program inside an environment without activating it in the parent
//! shell. The child sees the same `VIRTUAL_ENV` / `PATH` prefix / `SCUV_ACTIVE`
//! that `scuv activate` would set, and `scuv run` propagates the child's
//! exit code as its own.

use std::path::{Path, PathBuf};
use std::process::Command;

use rust_i18n::t;

use crate::core::VirtualenvService;
use crate::error::{Result, ScoopError};
use crate::output::Output;
use crate::{paths, validate};

/// Execute the `run` command.
pub fn execute(_output: &Output, env_name: &str, command: &[String]) -> Result<()> {
    validate::validate_env_name(env_name)?;

    let service = VirtualenvService::auto()?;
    if !service.exists(env_name)? {
        return Err(ScoopError::VirtualenvNotFound {
            name: env_name.to_string(),
        });
    }

    if command.is_empty() {
        return Err(ScoopError::InvalidArgument {
            message: t!("run.empty_command").to_string(),
        });
    }

    let venv_path = service.get_path(env_name)?;
    let bin_dir = paths::virtualenv_bin(env_name)?;

    // Touch BEFORE spawn: long-running commands shouldn't leave the env
    // looking idle, and a child crash shouldn't lose the "we used it"
    // signal. Best-effort — never blocks the run on metadata I/O.
    service.touch_metadata_best_effort(env_name);

    let mut cmd = build_command(&venv_path, &bin_dir, env_name, command);
    let status = match cmd.status() {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err(ScoopError::ExecutableNotFound {
                exe: command[0].clone(),
                env: env_name.to_string(),
            });
        }
        Err(e) => return Err(ScoopError::Io(e)),
    };

    // 128+signal is the conventional placeholder when no exit code is reported
    // (e.g. the child was killed by a signal). Matches what bash exposes via
    // `$?`, so users can detect the failure mode without a separate channel.
    let code = status
        .code()
        .or_else(|| signal_exit_code(&status))
        .unwrap_or(1);
    std::process::exit(code);
}

/// Build the subprocess `Command` with env-activation semantics. Kept pure
/// (no spawn) so the env wiring can be unit-tested without touching the OS.
fn build_command(venv_path: &Path, bin_dir: &Path, env_name: &str, command: &[String]) -> Command {
    let program = resolve_program(bin_dir, &command[0]);
    let mut cmd = Command::new(program);
    cmd.args(&command[1..])
        .env("VIRTUAL_ENV", venv_path)
        .env("SCUV_ACTIVE", env_name)
        .env_remove("PYTHONHOME");

    // Mirror activation: prepend the env's bin dir to PATH so any indirect
    // executions (a wrapper script that exec's `python`, etc.) also see the
    // env-local binaries first.
    let mut path_parts: Vec<PathBuf> = vec![bin_dir.to_path_buf()];
    if let Some(existing) = std::env::var_os("PATH") {
        path_parts.extend(std::env::split_paths(&existing));
    }
    if let Ok(joined) = std::env::join_paths(path_parts) {
        cmd.env("PATH", joined);
    }

    cmd
}

/// Pick the program path to invoke: an explicit relative/absolute path is used
/// verbatim, but a bare name is first looked up inside the env's bin dir so
/// `scuv run env -- python` runs the env's interpreter (not a system one).
fn resolve_program(bin_dir: &Path, program: &str) -> PathBuf {
    if program.contains('/') || program.contains('\\') {
        return PathBuf::from(program);
    }
    paths::find_executable_in(bin_dir, program).unwrap_or_else(|| PathBuf::from(program))
}

#[cfg(unix)]
fn signal_exit_code(status: &std::process::ExitStatus) -> Option<i32> {
    use std::os::unix::process::ExitStatusExt;
    status.signal().map(|s| 128 + s)
}

#[cfg(not(unix))]
fn signal_exit_code(_status: &std::process::ExitStatus) -> Option<i32> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::with_temp_scoop_home;
    use serial_test::serial;
    use std::path::PathBuf;

    fn fake_command(name: &str, args: &[&str]) -> Vec<String> {
        std::iter::once(name.to_string())
            .chain(args.iter().map(|s| s.to_string()))
            .collect()
    }

    #[test]
    fn build_command_sets_env_vars() {
        let venv = PathBuf::from("/envs/myenv");
        let bin = PathBuf::from("/envs/myenv/bin");
        let cmd = build_command(&venv, &bin, "myenv", &fake_command("python", &["x.py"]));

        let envs: Vec<_> = cmd.get_envs().collect();
        let lookup = |key: &str| -> Option<Option<&std::ffi::OsStr>> {
            envs.iter()
                .find(|(k, _)| k.to_str() == Some(key))
                .map(|(_, v)| *v)
        };

        assert_eq!(
            lookup("VIRTUAL_ENV")
                .and_then(|v| v)
                .and_then(|v| v.to_str()),
            Some("/envs/myenv")
        );
        assert_eq!(
            lookup("SCUV_ACTIVE")
                .and_then(|v| v)
                .and_then(|v| v.to_str()),
            Some("myenv")
        );
        // Explicit removal of PYTHONHOME shows up as a `None` value entry.
        assert_eq!(lookup("PYTHONHOME"), Some(None));
        // PATH must be set with bin dir somewhere in it.
        let path_val = lookup("PATH")
            .and_then(|v| v)
            .and_then(|v| v.to_str().map(str::to_string))
            .expect("PATH set");
        assert!(path_val.starts_with("/envs/myenv/bin"));
    }

    #[test]
    fn build_command_passes_through_args() {
        let venv = PathBuf::from("/envs/myenv");
        let bin = PathBuf::from("/envs/myenv/bin");
        let cmd = build_command(
            &venv,
            &bin,
            "myenv",
            &fake_command("pytest", &["-vv", "--tb=short"]),
        );

        let args: Vec<_> = cmd
            .get_args()
            .map(|a| a.to_string_lossy().to_string())
            .collect();
        assert_eq!(args, vec!["-vv", "--tb=short"]);
    }

    #[test]
    fn resolve_program_uses_bin_dir_for_bare_name() {
        let dir = tempfile::tempdir().unwrap();
        let exe = dir.path().join("python");
        std::fs::write(&exe, b"").unwrap();
        assert_eq!(resolve_program(dir.path(), "python"), exe);
    }

    #[test]
    fn resolve_program_passes_through_path_unchanged() {
        // Explicit path must be kept literal: do not probe the bin dir.
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(
            resolve_program(dir.path(), "/usr/bin/python3"),
            PathBuf::from("/usr/bin/python3")
        );
    }

    #[test]
    fn resolve_program_falls_back_to_bare_name_when_missing() {
        // If the env doesn't have the program, hand the bare name to the OS so
        // it can resolve it via `PATH` — matches what activation would do.
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(resolve_program(dir.path(), "ls"), PathBuf::from("ls"));
    }

    #[test]
    #[serial]
    fn execute_returns_invalid_env_name() {
        with_temp_scoop_home(|temp_dir| {
            std::fs::create_dir_all(temp_dir.path().join("virtualenvs")).unwrap();
            let output = Output::new(0, true, true, false);
            let err = execute(&output, "../bad", &["python".to_string()]).unwrap_err();
            assert!(matches!(err, ScoopError::InvalidEnvName { .. }));
        });
    }

    #[test]
    #[serial]
    fn execute_returns_not_found_for_missing_env() {
        with_temp_scoop_home(|temp_dir| {
            std::fs::create_dir_all(temp_dir.path().join("virtualenvs")).unwrap();
            let output = Output::new(0, true, true, false);
            let err = execute(&output, "ghost", &["python".to_string()]).unwrap_err();
            assert!(matches!(err, ScoopError::VirtualenvNotFound { .. }));
        });
    }

    #[test]
    #[serial]
    fn execute_rejects_empty_command() {
        with_temp_scoop_home(|temp_dir| {
            let bin = temp_dir
                .path()
                .join("virtualenvs")
                .join("myenv")
                .join("bin");
            std::fs::create_dir_all(&bin).unwrap();
            let output = Output::new(0, true, true, false);
            let err = execute(&output, "myenv", &[]).unwrap_err();
            assert!(matches!(err, ScoopError::InvalidArgument { .. }));
        });
    }
}
