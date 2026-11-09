//! Check for symbolic link validity.

use std::borrow::Cow;
use std::path::{Path, PathBuf};

use rust_i18n::t;

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

    fn name(&self) -> Cow<'static, str> {
        t!("doctor.name.symlink")
    }

    fn run(&self) -> Vec<CheckResult> {
        let mut results = Vec::new();
        let mut valid = 0;
        let mut broken_names = Vec::new();

        // The same walk as `scuv list`: a symlink planted under virtualenvs/
        // is not an env, so its interpreter is not inspected either. Without
        // a readable virtualenvs/ there is nothing to report here; the
        // virtualenvs check reports that.
        for (name, path) in crate::core::env_dirs().into_iter().flatten() {
            let python_path = crate::paths::virtualenv_python_exe(&path);
            if python_path.is_symlink() {
                match std::fs::read_link(&python_path) {
                    Ok(target) if target.exists() => valid += 1,
                    Ok(_) => broken_names.push(name),
                    // Not a symlink after all, or unreadable.
                    Err(_) => {}
                }
            }
        }

        // Report broken symlinks
        for name in &broken_names {
            results.push(
                CheckResult::error(
                    "symlink",
                    t!("doctor.name.broken_symlink"),
                    t!("doctor.symlink.broken", name = name),
                )
                .with_suggestion(format!(
                    "scuv remove {} && scuv create {} <python-version>",
                    name, name
                ))
                .with_target(name.clone()),
            );
        }

        // Summary
        if broken_names.is_empty() && valid > 0 {
            results.push(
                CheckResult::ok(self.id(), self.name())
                    .with_details(t!("doctor.symlink.valid", count = valid)),
            );
        }

        results
    }

    fn fix(&self, result: &CheckResult, output: &crate::output::Output) -> Option<CheckResult> {
        // `run` names the env in `target` on each broken-link error.
        if !matches!(result.status, CheckStatus::Error(_)) {
            return None;
        }
        let venv_name = result.target.clone()?;

        output.info(&t!("doctor.symlink.fixing", name = venv_name));

        let venv_path = paths::virtualenvs_dir().ok()?.join(&venv_name);
        Some(match relink(output, &venv_path, &venv_name) {
            Ok(()) => {
                output.success(&t!("doctor.symlink.fixed", name = venv_name));
                CheckResult::ok("symlink", t!("doctor.name.broken_symlink"))
                    .with_details(t!("doctor.symlink.fixed_details", name = venv_name))
            }
            Err(failed) => *failed,
        })
    }
}

/// A failed fix: the error and what the user can do about it. Boxed, as
/// the helpers below return it as their `Err`.
type Failed = Box<CheckResult>;

fn failure(message: impl Into<String>, suggestion: impl Into<String>) -> Failed {
    Box::new(
        CheckResult::error("symlink", t!("doctor.name.broken_symlink"), message)
            .with_suggestion(suggestion),
    )
}

/// Points the env's `python` at its installed interpreter again: find the
/// env's Python version, find that Python via uv, replace the link.
fn relink(output: &crate::output::Output, venv_path: &Path, venv_name: &str) -> Result<(), Failed> {
    if !venv_path.exists() {
        return Err(failure(
            t!("doctor.symlink.env_not_found", name = venv_name),
            format!("scuv create {} <python-version>", venv_name),
        ));
    }

    // Read the venv's Python version (scuv metadata, then pyvenv.cfg fallback).
    let python_version = read_python_version(venv_path).ok_or_else(|| {
        failure(
            t!("doctor.symlink.version_unknown", name = venv_name),
            format!(
                "scuv remove {} && scuv create {} <python-version>",
                venv_name, venv_name
            ),
        )
    })?;
    output.info(&t!(
        "doctor.symlink.found_version",
        version = python_version
    ));

    let python_path = installed_python_path(&python_version)?;
    replace_symlink(
        &python_path,
        &crate::paths::virtualenv_python_exe(venv_path),
    )
}

/// The interpreter uv has installed for `version`.
fn installed_python_path(version: &str) -> Result<PathBuf, Failed> {
    let install_hint = format!("scuv install {}", version);
    let uv = UvClient::new().map_err(|_| {
        failure(
            t!("doctor.symlink.uv_unavailable"),
            t!("doctor.symlink.install_uv"),
        )
    })?;
    match uv.find_python(version) {
        Ok(Some(info)) => info.path.ok_or_else(|| {
            failure(
                t!("doctor.symlink.python_path_missing", version = version),
                install_hint,
            )
        }),
        Ok(None) => Err(failure(
            t!("doctor.symlink.python_missing", version = version),
            install_hint,
        )),
        Err(_) => Err(failure(t!("doctor.symlink.find_failed"), install_hint)),
    }
}

/// Replaces `link` (if anything is there) with a symlink to `target`.
fn replace_symlink(target: &Path, link: &Path) -> Result<(), Failed> {
    if (link.exists() || link.is_symlink())
        && let Err(e) = std::fs::remove_file(link)
    {
        return Err(failure(
            t!("doctor.symlink.remove_failed", error = e.to_string()),
            t!("doctor.check_file_permissions"),
        ));
    }

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(target, link).map_err(|e| {
            failure(
                t!("doctor.symlink.create_failed", error = e.to_string()),
                t!("doctor.check_file_permissions"),
            )
        })
    }

    #[cfg(not(unix))]
    {
        let _ = target;
        Err(Box::new(
            CheckResult::warn(
                "symlink",
                t!("doctor.name.broken_symlink"),
                t!("doctor.symlink.unsupported"),
            )
            .with_suggestion(t!("doctor.symlink.manual")),
        ))
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
    /// test_virtualenv_exists_with_broken_symlink in src/paths/tests.rs).
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
            // `fix` reads the env name from here, not from the message.
            let error = results.iter().find(|r| r.is_error()).unwrap();
            assert_eq!(error.target.as_deref(), Some("brokenlink"));
        });
    }

    /// A symlink under virtualenvs/ is not an env, so a dangling interpreter
    /// inside it is not reported (it is not `scuv list`-ed either). What
    /// fails this: walking with `path.is_dir()`, which follows the link.
    #[cfg(unix)]
    #[test]
    #[serial]
    fn symlink_check_ignores_a_symlinked_entry() {
        with_temp_scoop_home(|temp| {
            use std::os::unix::fs::symlink;

            let elsewhere = tempfile::tempdir().unwrap();
            std::fs::create_dir_all(elsewhere.path().join("bin")).unwrap();
            symlink(
                "/nonexistent/python",
                elsewhere.path().join("bin").join("python"),
            )
            .unwrap();
            let venvs = temp.path().join("virtualenvs");
            std::fs::create_dir_all(&venvs).unwrap();
            symlink(elsewhere.path(), venvs.join("linked")).unwrap();

            assert!(SymlinkCheck.run().is_empty());
        });
    }

    fn quiet() -> crate::output::Output {
        crate::output::Output::new(0, true, crate::output::Colors::NONE, false)
    }

    /// `fix` repairs the env named in `target`, whatever the message says
    /// (it may be translated). The env does not exist here, so the repair
    /// stops at the lookup and reports it. What fails this: `fix` going
    /// back to parsing the message (no quotes, no name) or returning None.
    #[test]
    #[serial]
    fn fix_symlink_repairs_the_env_named_in_target() {
        with_temp_scoop_home(|temp| {
            std::fs::create_dir_all(temp.path().join("virtualenvs")).unwrap();
            let probe = CheckResult::error("symlink", "broken symlink", "lien cassé")
                .with_target("fix-target");

            let r = SymlinkCheck
                .fix(&probe, &quiet())
                .expect("a fix is attempted");
            assert!(r.is_error());
            assert!(
                matches!(&r.status, CheckStatus::Error(msg) if msg.contains("'fix-target' not found")),
                "{r:?}"
            );
            assert!(
                r.suggestion
                    .as_deref()
                    .is_some_and(|s| s.contains("scuv create fix-target"))
            );
        });
    }

    /// Without a target there is nothing to repair, even when the message
    /// still quotes a name.
    #[test]
    fn fix_symlink_needs_a_target() {
        let probe = CheckResult::error(
            "symlink",
            "broken symlink",
            "Python symlink in 'fix-target' is broken",
        );
        assert!(SymlinkCheck.fix(&probe, &quiet()).is_none());
    }

    /// Only an error is repaired. What fails this: dropping the status guard.
    #[test]
    fn fix_symlink_leaves_a_warning_alone() {
        let probe = CheckResult::warn("symlink", "broken symlink", "x").with_target("fix-target");
        assert!(SymlinkCheck.fix(&probe, &quiet()).is_none());
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
