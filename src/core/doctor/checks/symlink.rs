//! Check for symbolic link validity.

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
            // Parse: "Python symlink in 'name' is broken"
            msg.split('\'').nth(1).map(|s| s.to_string())
        } else {
            None
        }?;

        output.info(&format!("Attempting to fix symlink for '{}'...", venv_name));

        // Get virtualenv path
        let venvs_dir = paths::virtualenvs_dir().ok()?;
        let venv_path = venvs_dir.join(&venv_name);

        if !venv_path.exists() {
            return Some(
                CheckResult::error(
                    "symlink",
                    "broken symlink",
                    format!("environment '{}' not found", venv_name),
                )
                .with_suggestion(format!("scuv create {} <python-version>", venv_name)),
            );
        }

        // Read metadata to get Python version
        let metadata_path = venv_path.join(Metadata::FILE_NAME);
        let python_version = if metadata_path.exists() {
            match std::fs::read_to_string(&metadata_path) {
                Ok(content) => match serde_json::from_str::<Metadata>(&content) {
                    Ok(meta) => Some(meta.python_version),
                    Err(_) => None,
                },
                Err(_) => None,
            }
        } else {
            // Try to extract from pyvenv.cfg as fallback
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
        };

        let python_version = match python_version {
            Some(v) => v,
            None => {
                return Some(
                    CheckResult::error(
                        "symlink",
                        "broken symlink",
                        format!("could not determine Python version for '{}'", venv_name),
                    )
                    .with_suggestion(format!(
                        "scuv remove {} && scuv create {} <python-version>",
                        venv_name, venv_name
                    )),
                );
            }
        };

        output.info(&format!("Found Python version: {}", python_version));

        // Find Python binary using uv
        let uv = match UvClient::new() {
            Ok(uv) => uv,
            Err(_) => {
                return Some(
                    CheckResult::error("symlink", "broken symlink", "uv not available")
                        .with_suggestion("Install uv first"),
                );
            }
        };

        let python_path = match uv.find_python(&python_version) {
            Ok(Some(info)) => match info.path {
                Some(path) => path,
                None => {
                    return Some(
                        CheckResult::error(
                            "symlink",
                            "broken symlink",
                            format!("Python {} path not found", python_version),
                        )
                        .with_suggestion(format!("scuv install {}", python_version)),
                    );
                }
            },
            Ok(None) => {
                return Some(
                    CheckResult::error(
                        "symlink",
                        "broken symlink",
                        format!("Python {} not installed", python_version),
                    )
                    .with_suggestion(format!("scuv install {}", python_version)),
                );
            }
            Err(_) => {
                return Some(
                    CheckResult::error(
                        "symlink",
                        "broken symlink",
                        "failed to find Python installation",
                    )
                    .with_suggestion(format!("scuv install {}", python_version)),
                );
            }
        };

        // Recreate symlink
        let symlink_path = crate::paths::virtualenv_python_exe(&venv_path);

        // Remove old symlink if exists
        if symlink_path.exists() || symlink_path.is_symlink() {
            if let Err(e) = std::fs::remove_file(&symlink_path) {
                return Some(
                    CheckResult::error(
                        "symlink",
                        "broken symlink",
                        format!("failed to remove old symlink: {}", e),
                    )
                    .with_suggestion("Check file permissions"),
                );
            }
        }

        // Create new symlink
        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            if let Err(e) = symlink(&python_path, &symlink_path) {
                return Some(
                    CheckResult::error(
                        "symlink",
                        "broken symlink",
                        format!("failed to create symlink: {}", e),
                    )
                    .with_suggestion("Check file permissions"),
                );
            }
        }

        #[cfg(not(unix))]
        {
            return Some(
                CheckResult::warn(
                    "symlink",
                    "broken symlink",
                    "symlink fix not supported on this platform",
                )
                .with_suggestion("Manually recreate the symlink"),
            );
        }

        output.success(&format!("Fixed symlink for '{}'", venv_name));

        Some(
            CheckResult::ok("symlink", "broken symlink")
                .with_details(format!("fixed symlink for '{}'", venv_name)),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::test_utils::with_temp_scoop_home;
    use serial_test::serial;

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
            let output = crate::output::Output::new(0, true, true, false);

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
}
