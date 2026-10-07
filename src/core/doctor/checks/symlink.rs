//! Check for symbolic link validity.

use std::path::{Path, PathBuf};

use crate::core::metadata::Metadata;
use crate::paths;
use crate::uv::UvClient;

use super::super::types::{Check, CheckResult, CheckStatus};

/// Check for symbolic link validity.
pub(super) struct SymlinkCheck;

impl Check for SymlinkCheck {
    fn id(&self) -> &'static str {
        "symlink"
    }

    fn name(&self) -> &'static str {
        "symbolic links"
    }

    fn run(&self) -> Vec<CheckResult> {
        let venvs_dir = match paths::virtualenvs_dir() {
            Ok(dir) if dir.exists() => dir,
            _ => return vec![],
        };

        let mut results = Vec::new();
        let mut valid = 0;
        let mut broken_names = Vec::new();

        if let Ok(entries) = std::fs::read_dir(&venvs_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let python_path = crate::paths::virtualenv_python_exe(&path);

                    if python_path.is_symlink() {
                        match std::fs::read_link(&python_path) {
                            Ok(target) if target.exists() => {
                                valid += 1;
                            }
                            Ok(_) => {
                                let name = path
                                    .file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or("unknown")
                                    .to_string();
                                broken_names.push(name);
                            }
                            Err(_) => {
                                // Not a symlink or error reading
                            }
                        }
                    }
                }
            }
        }

        // Report broken symlinks
        for name in &broken_names {
            results.push(
                CheckResult::error(
                    "symlink",
                    "broken symlink",
                    format!("Python symlink in '{}' is broken", name),
                )
                .with_suggestion(format!(
                    "scuv remove {} && scuv create {} <python-version>",
                    name, name
                )),
            );
        }

        // Summary
        if broken_names.is_empty() && valid > 0 {
            results.push(
                CheckResult::ok(self.id(), self.name())
                    .with_details(format!("{} symlinks valid", valid)),
            );
        }

        results
    }

    fn fix(&self, result: &CheckResult, output: &crate::output::Output) -> Option<CheckResult> {
        // Extract environment name from error message: "Python symlink in 'name' is broken"
        let venv_name = if let CheckStatus::Error(msg) = &result.status {
            msg.split('\'').nth(1).map(|s| s.to_string())
        } else {
            None
        }?;

        output.info(&format!("Attempting to fix symlink for '{}'...", venv_name));

        let venv_path = paths::virtualenvs_dir().ok()?.join(&venv_name);
        Some(match relink(output, &venv_path, &venv_name) {
            Ok(()) => {
                output.success(&format!("Fixed symlink for '{}'", venv_name));
                CheckResult::ok("symlink", "broken symlink")
                    .with_details(format!("fixed symlink for '{}'", venv_name))
            }
            Err(failed) => failed,
        })
    }
}

/// A failed fix: the error and what the user can do about it.
fn failure(message: impl Into<String>, suggestion: impl Into<String>) -> CheckResult {
    CheckResult::error("symlink", "broken symlink", message).with_suggestion(suggestion)
}

/// Points the env's `python` at its installed interpreter again: find the
/// env's Python version, find that Python via uv, replace the link.
fn relink(
    output: &crate::output::Output,
    venv_path: &Path,
    venv_name: &str,
) -> Result<(), CheckResult> {
    if !venv_path.exists() {
        return Err(failure(
            format!("environment '{}' not found", venv_name),
            format!("scuv create {} <python-version>", venv_name),
        ));
    }

    // Read the venv's Python version (scuv metadata, then pyvenv.cfg fallback).
    let python_version = read_python_version(venv_path).ok_or_else(|| {
        failure(
            format!("could not determine Python version for '{}'", venv_name),
            format!(
                "scuv remove {} && scuv create {} <python-version>",
                venv_name, venv_name
            ),
        )
    })?;
    output.info(&format!("Found Python version: {}", python_version));

    let python_path = installed_python_path(&python_version)?;
    replace_symlink(
        &python_path,
        &crate::paths::virtualenv_python_exe(venv_path),
    )
}

/// The interpreter uv has installed for `version`.
fn installed_python_path(version: &str) -> Result<PathBuf, CheckResult> {
    let install_hint = format!("scuv install {}", version);
    let uv = UvClient::new().map_err(|_| failure("uv not available", "Install uv first"))?;
    match uv.find_python(version) {
        Ok(Some(info)) => info
            .path
            .ok_or_else(|| failure(format!("Python {} path not found", version), install_hint)),
        Ok(None) => Err(failure(
            format!("Python {} not installed", version),
            install_hint,
        )),
        Err(_) => Err(failure("failed to find Python installation", install_hint)),
    }
}

/// Replaces `link` (if anything is there) with a symlink to `target`.
fn replace_symlink(target: &Path, link: &Path) -> Result<(), CheckResult> {
    if (link.exists() || link.is_symlink())
        && let Err(e) = std::fs::remove_file(link)
    {
        return Err(failure(
            format!("failed to remove old symlink: {}", e),
            "Check file permissions",
        ));
    }

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(target, link).map_err(|e| {
            failure(
                format!("failed to create symlink: {}", e),
                "Check file permissions",
            )
        })
    }

    #[cfg(not(unix))]
    {
        let _ = target;
        Err(CheckResult::warn(
            "symlink",
            "broken symlink",
            "symlink fix not supported on this platform",
        )
        .with_suggestion("Manually recreate the symlink"))
    }
}

/// Reads the venv's Python version from its scuv metadata, falling back to
/// parsing `pyvenv.cfg`. Returns `None` if neither source yields a version.
fn read_python_version(venv_path: &std::path::Path) -> Option<String> {
    let metadata_path = venv_path.join(Metadata::FILE_NAME);
    if metadata_path.exists() {
        match std::fs::read_to_string(&metadata_path) {
            Ok(content) => match serde_json::from_str::<Metadata>(&content) {
                Ok(meta) => Some(meta.python_version),
                Err(_) => None,
            },
            Err(_) => None,
        }
    } else {
        // Try to extract from pyvenv.cfg as fallback.
        let pyvenv_cfg = venv_path.join("pyvenv.cfg");
        if pyvenv_cfg.exists() {
            std::fs::read_to_string(&pyvenv_cfg)
                .ok()
                .and_then(|content| {
                    for line in content.lines() {
                        if line.starts_with("version") {
                            return line.split('=').nth(1).map(|v| v.trim().to_string());
                        }
                    }
                    None
                })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::test_utils::with_temp_scoop_home;
    use serial_test::serial;

    #[test]
    fn symlink_check_reports_identity() {
        // Pins id()/name() directly; run()'s healthy summary (which also uses
        // them) is excluded from mutants for its equivalent dir-guard mutation,
        // so this is the killing test for the id/name mutants.
        let check = SymlinkCheck;
        assert_eq!(check.id(), "symlink");
        assert_eq!(check.name(), "symbolic links");
    }

    #[test]
    fn read_python_version_prefers_scuv_metadata() {
        // Metadata present -> the version comes from it.
        let tmp = tempfile::tempdir().unwrap();
        let meta = Metadata::new("env".to_string(), "3.12.5".to_string(), None);
        std::fs::write(
            tmp.path().join(Metadata::FILE_NAME),
            serde_json::to_string(&meta).unwrap(),
        )
        .unwrap();
        assert_eq!(read_python_version(tmp.path()).as_deref(), Some("3.12.5"));
    }

    #[test]
    fn read_python_version_falls_back_to_pyvenv_cfg() {
        // No metadata -> parse the `version = X` line from pyvenv.cfg.
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(
            tmp.path().join("pyvenv.cfg"),
            "home = /usr\nversion = 3.11.9\n",
        )
        .unwrap();
        assert_eq!(read_python_version(tmp.path()).as_deref(), Some("3.11.9"));
    }

    #[test]
    fn read_python_version_none_when_no_sources() {
        let tmp = tempfile::tempdir().unwrap();
        assert_eq!(read_python_version(tmp.path()), None);
    }

    #[test]
    fn read_python_version_none_on_corrupt_metadata() {
        // Corrupt metadata does NOT fall through to pyvenv.cfg (original behavior).
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join(Metadata::FILE_NAME), "{ not json").unwrap();
        std::fs::write(tmp.path().join("pyvenv.cfg"), "version = 9.9.9\n").unwrap();
        assert_eq!(read_python_version(tmp.path()), None);
    }

    /// On Unix we can deterministically create a symlink whose target
    /// doesn't exist, which is exactly the failure SymlinkCheck::run
    /// surfaces. cfg-gated to Unix because the symlink primitive
    /// differs on Windows (and the CI matrix that exercises mutants is
    /// Linux only — same rationale as the existing
    /// test_virtualenv_exists_with_broken_symlink in paths.rs).
    #[cfg(unix)]
    #[test]
    #[serial]
    fn symlink_check_emits_error_for_broken_python_symlink() {
        with_temp_scoop_home(|temp| {
            use std::os::unix::fs::symlink;

            let env = temp.path().join("virtualenvs").join("brokenlink");
            std::fs::create_dir_all(env.join("bin")).unwrap();
            symlink("/nonexistent/python", env.join("bin").join("python")).unwrap();

            let results = SymlinkCheck.run();
            assert!(
                !results.is_empty(),
                "broken symlink env must produce at least one result"
            );
            assert!(
                results.iter().any(|r| r.is_error()),
                "expected at least one error result, got {results:#?}"
            );
        });
    }

    #[test]
    #[serial]
    fn fix_symlink_returns_some_for_parseable_error_name() {
        with_temp_scoop_home(|temp| {
            // SymlinkCheck::fix parses the env name out of a "Python symlink
            // in 'name' is broken" message. If the parse succeeds and
            // the env doesn't exist, it returns Some(error suggesting
            // scuv create). The cargo-mutants -> None replacement
            // would silently drop that guidance.
            let broken = temp.path().join("virtualenvs");
            std::fs::create_dir_all(&broken).unwrap();

            let probe = CheckResult::error(
                "symlink",
                "broken symlink",
                "Python symlink in 'fix-target' is broken".to_string(),
            );
            let output = crate::output::Output::new(0, true, crate::output::Colors::NONE, false);

            let fixed = SymlinkCheck.fix(&probe, &output);
            assert!(fixed.is_some(), "fix_symlink must return Some");
            let r = fixed.unwrap();
            assert!(r.is_error() || r.is_warning());
            // Suggestion text should point the user at `scuv create`.
            assert!(
                r.suggestion
                    .as_deref()
                    .is_some_and(|s| s.contains("scuv create"))
                    || matches!(&r.status, CheckStatus::Error(msg) if msg.contains("fix-target"))
            );
        });
    }

    /// Fails if `relink` stops refusing a missing env (`!` deleted).
    #[test]
    fn relink_reports_a_missing_env() {
        let tmp = tempfile::tempdir().unwrap();
        let out = crate::output::Output::new(0, true, crate::output::Colors::NONE, false);
        let err = relink(&out, &tmp.path().join("gone"), "gone").unwrap_err();
        assert!(err.is_error());
        assert!(format!("{err:?}").contains("not found"), "{err:?}");
    }

    /// A dangling `python` link (target gone) is replaced, and so is one
    /// pointing elsewhere. Fails if `replace_symlink` does nothing, or if the
    /// `exists() || is_symlink()` test becomes `&&` (a dangling link then
    /// survives and creating the new one fails).
    #[cfg(unix)]
    #[test]
    fn replace_symlink_replaces_dangling_and_stale_links() {
        use std::os::unix::fs::symlink;

        let tmp = tempfile::tempdir().unwrap();
        let target = tmp.path().join("python3.12");
        std::fs::write(&target, b"").unwrap();
        let link = tmp.path().join("python");

        symlink(tmp.path().join("gone"), &link).unwrap();
        replace_symlink(&target, &link).unwrap();
        assert_eq!(std::fs::read_link(&link).unwrap(), target);

        let other = tmp.path().join("other");
        std::fs::write(&other, b"").unwrap();
        std::fs::remove_file(&link).unwrap();
        symlink(&other, &link).unwrap();
        replace_symlink(&target, &link).unwrap();
        assert_eq!(std::fs::read_link(&link).unwrap(), target);
    }

    /// No uv, or uv without that Python: either way an error, never a
    /// made-up path. Fails if `installed_python_path` returns `Ok` blindly.
    #[test]
    #[serial]
    fn installed_python_path_errors_for_a_version_nobody_has() {
        assert!(installed_python_path("0.0.1").is_err());
    }
}
