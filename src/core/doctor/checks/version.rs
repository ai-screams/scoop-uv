//! Check for version file validity.

use crate::paths;

use super::super::types::{Check, CheckResult};

/// Classify a non-empty version-file entry into a [`CheckResult`].
///
/// The `system` sentinel (case-insensitive) is the documented value
/// for "use the system Python, no virtualenv active" — it's what
/// `scuv use system` writes — and must NOT be treated as a regular
/// env name to look up. Pre-0.14.1 the doctor's version checks
/// flagged `.scoop-version: system` as a non-existent-env error
/// because the post-self-update doctor pass took it as a regular
/// reference. `use_env::mod.rs:41` uses the same case-insensitive
/// match on the writer side, so we mirror that contract here.
fn classify_version_entry(
    id: &'static str,
    name: &'static str,
    entry: &str,
    venvs_dir: Option<&std::path::Path>,
) -> CheckResult {
    if entry.eq_ignore_ascii_case("system") {
        return CheckResult::ok(id, name).with_details("system Python (no virtualenv)");
    }
    let env_exists = venvs_dir
        .map(|dir| dir.join(entry).exists())
        .unwrap_or(false);
    if env_exists {
        CheckResult::ok(id, name).with_details(format!("set to '{}'", entry))
    } else {
        CheckResult::error(id, name, format!("references non-existent env '{}'", entry))
            .with_suggestion(format!("Run: scuv create {} <python-version>", entry))
    }
}

/// Resolve the local version-file path for `dir` the way `doctor` should see
/// it: the current `.scuv-version` name wins when present, otherwise falls
/// back to the legacy `.scoop-version` name. Mirrors the per-directory
/// precedence in `VersionService::resolve_local_version_file` so `doctor`
/// and `resolve()` never disagree about which file is authoritative during
/// the shim window — but skips that function's `warn_once` side effect,
/// since `doctor` is a read-only diagnostic and the dedicated `legacy` check
/// already surfaces a legacy-only file to the user.
///
/// DEPRECATION(0.16.0): remove the legacy fallback branch.
fn resolve_local_version_file_for_doctor(dir: &std::path::Path) -> std::path::PathBuf {
    let version_file = paths::local_version_file(dir);
    if version_file.exists() {
        version_file
    } else {
        let legacy = dir.join(paths::LEGACY_VERSION_FILE);
        if legacy.exists() {
            legacy
        } else {
            version_file
        }
    }
}

/// Check for version file validity.
pub(super) struct VersionCheck;

impl Check for VersionCheck {
    fn id(&self) -> &'static str {
        "version"
    }

    fn name(&self) -> &'static str {
        "version files"
    }

    fn run(&self) -> Vec<CheckResult> {
        let mut results = Vec::new();
        let venvs_dir = paths::virtualenvs_dir().ok();

        // Check global version file
        if let Ok(global_file) = paths::global_version_file() {
            if global_file.exists() {
                match std::fs::read_to_string(&global_file) {
                    Ok(content) => {
                        let env_name = content.trim();
                        if !env_name.is_empty() {
                            results.push(classify_version_entry(
                                "version:global",
                                "global version",
                                env_name,
                                venvs_dir.as_deref(),
                            ));
                        }
                    }
                    Err(_) => {
                        results.push(
                            CheckResult::warn(
                                "version:global",
                                "global version",
                                "could not read global version file",
                            )
                            .with_suggestion(format!("Check file: {}", global_file.display())),
                        );
                    }
                }
            }
        }

        // Check local version file in current directory
        let current_dir = match std::env::current_dir() {
            Ok(dir) => dir,
            Err(_) => return results,
        };
        let local_file = resolve_local_version_file_for_doctor(&current_dir);
        if local_file.exists() {
            match std::fs::read_to_string(&local_file) {
                Ok(content) => {
                    let env_name = content.trim();
                    if !env_name.is_empty() {
                        results.push(classify_version_entry(
                            "version:local",
                            "local version",
                            env_name,
                            venvs_dir.as_deref(),
                        ));
                    }
                }
                Err(_) => {
                    results.push(
                        CheckResult::warn(
                            "version:local",
                            "local version",
                            "could not read local version file",
                        )
                        .with_suggestion(format!("Check file: {}", local_file.display())),
                    );
                }
            }
        }

        // If no version files found, that's fine
        if results.is_empty() {
            results.push(
                CheckResult::ok(self.id(), self.name()).with_details("no version files configured"),
            );
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::core::doctor::CheckStatus;
    use crate::core::doctor::checks::test_support::TempDirCwdGuard;
    use crate::test_utils::with_temp_scoop_home;
    use serial_test::serial;

    // ==========================================================================
    // VersionCheck: `system` sentinel handling
    //
    // The `system` value in .scuv-version (or legacy .scoop-version, or the
    // global version file) is a documented sentinel meaning "use the system
    // Python, no virtualenv active" — written by `scuv use system`.
    // Pre-0.14.1 the post-self-update doctor pass flagged it as a
    // non-existent env, which the user reported as a false-positive after
    // `scoop self update` to 0.14.0.
    // ==========================================================================

    #[test]
    fn classify_treats_system_as_valid_sentinel() {
        let result = classify_version_entry("version:test", "test version", "system", None);
        assert!(
            result.is_ok(),
            "system must classify as Ok, got {result:#?}"
        );
        assert!(
            result
                .details
                .as_deref()
                .is_some_and(|d| d.contains("system Python")),
            "details should explain the sentinel, got {:?}",
            result.details
        );
    }

    #[test]
    fn classify_is_case_insensitive_for_system_sentinel() {
        // The writer side (use_env::mod.rs:41) uses eq_ignore_ascii_case;
        // doctor's reader must mirror that contract so `SYSTEM` or `System`
        // round-trip cleanly even though `scuv use system` always writes
        // the lowercase form.
        for variant in ["SYSTEM", "System", "sYsTeM"] {
            let result = classify_version_entry("version:test", "test version", variant, None);
            assert!(
                result.is_ok(),
                "case-variant '{variant}' must classify as Ok"
            );
        }
    }

    #[test]
    fn classify_existing_env_is_ok() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join("myenv")).unwrap();
        let result =
            classify_version_entry("version:test", "test version", "myenv", Some(tmp.path()));
        assert!(result.is_ok());
        assert!(
            result
                .details
                .as_deref()
                .is_some_and(|d| d.contains("myenv"))
        );
    }

    #[test]
    fn classify_missing_env_is_error() {
        let tmp = tempfile::tempdir().unwrap();
        // venvs dir exists but env doesn't — should error with suggestion.
        let result = classify_version_entry(
            "version:test",
            "test version",
            "missingenv",
            Some(tmp.path()),
        );
        assert!(result.is_error());
        assert!(
            result
                .suggestion
                .as_deref()
                .is_some_and(|s| s.contains("scuv create"))
        );
    }

    /// Mirror of the local-version test for the global path. Without
    /// this, deleting the `!` in the `if !env_name.is_empty()` guard
    /// on the global branch silently dropped every version:global
    /// result and went unkilled by mutation testing.
    #[test]
    #[serial]
    fn version_check_treats_global_system_sentinel_as_ok() {
        with_temp_scoop_home(|temp| {
            // Global version file lives at <SCOOP_HOME>/version (see
            // paths::global_version_file). Materialise it with the
            // sentinel value.
            std::fs::write(temp.path().join("version"), "system").unwrap();
            // virtualenvs dir presence shouldn't matter for the
            // sentinel, but create it so other checks behave normally.
            std::fs::create_dir_all(temp.path().join("virtualenvs")).unwrap();

            let results = VersionCheck.run();
            let global = results
                .iter()
                .find(|r| r.id == "version:global")
                .expect("version:global must run when ~/.scoop/version exists");
            assert!(
                global.is_ok(),
                "version:global must be Ok for system sentinel, got {global:#?}"
            );
        });
    }

    #[test]
    #[serial]
    fn version_check_treats_local_system_sentinel_as_ok() {
        with_temp_scoop_home(|temp| {
            // Materialise a `.scuv-version: system` file in the CWD.
            let cwd_guard = TempDirCwdGuard::new();
            std::fs::write(cwd_guard.path().join(".scuv-version"), "system").unwrap();

            // Ensure virtualenvs dir exists so its absence doesn't muddle the
            // assertion (we only want the system-sentinel branch to fire).
            std::fs::create_dir_all(temp.path().join("virtualenvs")).unwrap();

            let results = VersionCheck.run();
            let local = results
                .iter()
                .find(|r| r.id == "version:local")
                .expect("version:local check should run when .scuv-version exists");
            assert!(
                local.is_ok(),
                "version:local must be Ok for system sentinel, got {local:#?}"
            );
        });
    }

    /// Regression: the local-version read-error suggestion must name the
    /// actual resolved version-file path, not a hardcoded literal (which
    /// would silently go stale across a rename). A directory at the
    /// version-file path makes `.exists()` true but `read_to_string` fail,
    /// forcing the error branch without relying on platform-specific
    /// permission bits.
    #[test]
    #[serial]
    fn version_check_local_read_error_suggestion_names_version_file() {
        with_temp_scoop_home(|temp| {
            let cwd_guard = TempDirCwdGuard::new();
            std::fs::create_dir(cwd_guard.path().join(paths::VERSION_FILE)).unwrap();
            std::fs::create_dir_all(temp.path().join("virtualenvs")).unwrap();

            let results = VersionCheck.run();
            let local = results
                .iter()
                .find(|r| r.id == "version:local")
                .expect("version:local check should run when the version-file path exists");
            assert!(
                local.is_warning(),
                "expected a warning for an unreadable version file, got {local:#?}"
            );
            let suggestion = local.suggestion.as_deref().unwrap_or_default();
            assert!(
                suggestion.contains(paths::VERSION_FILE),
                "suggestion should name the actual version-file path, got: {suggestion}"
            );
        });
    }

    // ==========================================================================
    // VersionCheck: dual-read local version file (new `.scuv-version` name
    // wins, a legacy-only `.scoop-version` is still seen) — keeps doctor
    // aligned with VersionService::resolve()'s per-directory precedence
    // during the scoop -> scuv shim window.
    // ==========================================================================

    #[test]
    #[serial]
    fn version_check_sees_legacy_only_local_version_file() {
        with_temp_scoop_home(|temp| {
            let cwd_guard = TempDirCwdGuard::new();
            // Only the legacy filename exists in cwd — no `.scuv-version`.
            std::fs::write(cwd_guard.path().join(".scoop-version"), "ghost-env").unwrap();
            std::fs::create_dir_all(temp.path().join("virtualenvs")).unwrap();

            let results = VersionCheck.run();
            let local = results
                .iter()
                .find(|r| r.id == "version:local")
                .expect("version:local must be reported from a legacy-only .scoop-version file");
            assert!(
                local.is_error(),
                "references a non-existent env, so it should error, got {local:#?}"
            );
            assert!(
                matches!(&local.status, CheckStatus::Error(msg) if msg.contains("ghost-env")),
                "error should name the env read from the legacy file, got {local:#?}"
            );
        });
    }

    #[test]
    #[serial]
    fn version_check_prefers_new_file_over_legacy_when_both_present() {
        with_temp_scoop_home(|temp| {
            let cwd_guard = TempDirCwdGuard::new();
            std::fs::write(cwd_guard.path().join(".scuv-version"), "newenv").unwrap();
            std::fs::write(cwd_guard.path().join(".scoop-version"), "oldenv").unwrap();
            std::fs::create_dir_all(temp.path().join("virtualenvs")).unwrap();

            let results = VersionCheck.run();
            let local = results
                .iter()
                .find(|r| r.id == "version:local")
                .expect("version:local must run");
            assert!(
                matches!(&local.status, CheckStatus::Error(msg) if msg.contains("newenv")),
                "new-named file must win when both are present, got {local:#?}"
            );
        });
    }

    // ==========================================================================
    // VersionCheck::run — the "no version files configured" summary is the
    // only result carrying the check's own id/name ("version"/"version files");
    // global/local results use the "version:*" ids. With no global and no local
    // file, id/name mutants (`-> ""`/`-> "xyzzy"`) would flip the summary.
    // ==========================================================================

    #[test]
    #[serial]
    fn version_run_summary_reports_id_and_name_when_no_files() {
        let tmp = tempfile::tempdir().unwrap();
        let _cwd = TempDirCwdGuard::new(); // empty cwd -> no local version file
        let _g = crate::test_utils::env_guard(&[
            (paths::SCUV_HOME_ENV, Some(tmp.path().to_str().unwrap())),
            ("SCUV_VERSION", None),
            ("SCOOP_VERSION", None),
        ]);
        let results = VersionCheck.run();
        // The "no version files configured" summary is the last result and uses
        // self.id()/self.name(); id/name mutants to ""/"xyzzy" would flip these.
        let summary = results.iter().find(|r| r.id == "version");
        assert!(
            summary.is_some(),
            "expected a 'version' summary result: {results:#?}"
        );
        assert_eq!(summary.unwrap().name, "version files");
    }
}
