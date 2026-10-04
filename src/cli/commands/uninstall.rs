//! Uninstall command

use std::io::IsTerminal;

use dialoguer::Confirm;
use rust_i18n::t;

use crate::core::VirtualenvService;
use crate::error::{Result, ScoopError};
use crate::output::{Output, UninstallData};
use crate::uv::UvClient;
use crate::validate::PythonVersion;

/// Execute the uninstall command
pub fn execute(output: &Output, version: &str, cascade: bool, force: bool) -> Result<()> {
    let uv = UvClient::new()?;

    let mut removed_envs: Option<Vec<String>> = None;

    // Handle cascade: remove environments using this Python version
    if cascade {
        removed_envs = Some(handle_cascade(output, &uv, version, force)?);
    }

    output.info(&t!("uninstall.uninstalling", version = version));

    uv.uninstall_python(version)?;

    // JSON output
    if output.is_json() {
        output.json_success(
            "uninstall",
            UninstallData {
                version: version.to_string(),
                removed_envs,
            },
        );
        return Ok(());
    }

    output.success(&t!("uninstall.success", version = version));

    Ok(())
}

/// Handle cascade removal of environments using the target Python version.
///
/// Returns the list of environment names that were removed.
fn handle_cascade(
    output: &Output,
    uv: &UvClient,
    version: &str,
    force: bool,
) -> Result<Vec<String>> {
    let service = VirtualenvService::auto()?;
    let envs = service.list()?;

    let matching_envs = match PythonVersion::parse(version) {
        Some(filter) => {
            // The uv-managed Pythons this uninstall leaves behind.
            let remaining: Vec<PythonVersion> = uv
                .list_managed_pythons()?
                .iter()
                .filter_map(|p| PythonVersion::parse(&p.version))
                .filter(|p| !filter.matches(p))
                .collect();
            affected_envs(
                envs.into_iter().map(|e| (e.name, e.python_version)),
                &filter,
                is_plain_version(version).then_some(remaining.as_slice()),
            )
        }
        None => Vec::new(),
    };

    // No matching environments
    if matching_envs.is_empty() {
        if !output.is_json() {
            output.info(&t!("uninstall.cascade_none", version = version));
        }
        return Ok(vec![]);
    }

    // Show matching environments and confirm (unless --force or --json)
    if !force && !output.is_json() {
        output.info(&t!(
            "uninstall.cascade_found",
            count = matching_envs.len(),
            version = version
        ));
        for name in &matching_envs {
            output.info(&t!("uninstall.cascade_env", name = name));
        }

        // Check if stdin is a TTY; if not, abort rather than hanging
        if !std::io::stdin().is_terminal() {
            return Err(ScoopError::CascadeAborted);
        }

        let confirmed = Confirm::new()
            .with_prompt(t!("uninstall.cascade_confirm", version = version).to_string())
            .default(false)
            .interact()
            .unwrap_or(false);

        if !confirmed {
            output.info(&t!("uninstall.cascade_cancelled"));
            return Err(ScoopError::CascadeAborted);
        }
    }

    // Remove matching environments
    let mut removed = Vec::new();
    for name in &matching_envs {
        if !output.is_json() {
            output.info(&t!("uninstall.cascade_removing", name = name));
        }
        service.delete(name)?;
        removed.push(name.clone());
    }

    if !output.is_json() && !removed.is_empty() {
        output.info(&t!("uninstall.cascade_removed", count = removed.len()));
    }

    Ok(removed)
}

/// The envs an uninstall of `filter` takes the Python away from.
///
/// An env whose recorded version `filter` matches is affected, as before
/// (`uninstall 3.12` and an env recording `3.12.1`). An env recording less
/// than `filter` names — `3.12` against `uninstall 3.12.14`, what current uv
/// writes for an env linked to the 3.12 minor version — runs on whichever
/// 3.12.x is installed, so it is affected only when no install it accepts
/// is left in `remaining`. That second rule needs `remaining` and is off
/// when it is `None`: for a request other than plain digits (`3.12.0rc1`,
/// `3.13t`) the numbers alone do not say what uv removes, so a remaining
/// install could be miscounted and a working env deleted. Envs with no
/// recorded version are skipped.
fn affected_envs(
    envs: impl IntoIterator<Item = (String, Option<String>)>,
    filter: &PythonVersion,
    remaining: Option<&[PythonVersion]>,
) -> Vec<String> {
    envs.into_iter()
        .filter(|(_, recorded)| {
            let Some(recorded) = recorded.as_deref().and_then(PythonVersion::parse) else {
                return false;
            };
            filter.matches(&recorded)
                || remaining.is_some_and(|remaining| {
                    recorded.matches(filter) && !remaining.iter().any(|p| recorded.matches(p))
                })
        })
        .map(|(name, _)| name)
        .collect()
}

/// Whether `version` is only dot-separated digits (`3.12`, `3.12.14`): a
/// request whose meaning to uv is its numbers.
fn is_plain_version(version: &str) -> bool {
    version
        .split('.')
        .all(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit()))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ScoopError;
    use crate::uv::UvClient;

    // =========================================================================
    // UninstallData JSON Structure Tests
    // =========================================================================

    #[test]
    fn uninstall_data_json_has_correct_field_name() {
        let data = UninstallData {
            version: "3.12.0".to_string(),
            removed_envs: None,
        };

        let json = serde_json::to_string(&data).unwrap();

        // Verify exact JSON structure
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed.is_object());
        assert!(parsed.get("version").is_some());
        assert_eq!(parsed["version"].as_str().unwrap(), "3.12.0");
        // removed_envs should be absent when None (skip_serializing_if)
        assert!(parsed.get("removed_envs").is_none());
    }

    /// True roundtrip test: serialize -> deserialize -> compare
    #[test]
    fn uninstall_data_json_roundtrip() {
        let original = UninstallData {
            version: "3.11.5".to_string(),
            removed_envs: None,
        };

        let json = serde_json::to_string(&original).unwrap();
        let restored: UninstallData = serde_json::from_str(&json).unwrap();

        assert_eq!(original, restored);
    }

    #[test]
    fn uninstall_data_json_with_removed_envs() {
        let data = UninstallData {
            version: "3.12".to_string(),
            removed_envs: Some(vec!["env1".to_string(), "env2".to_string()]),
        };

        let json = serde_json::to_string(&data).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["version"], "3.12");
        let envs = parsed["removed_envs"].as_array().unwrap();
        assert_eq!(envs.len(), 2);
        assert_eq!(envs[0], "env1");
        assert_eq!(envs[1], "env2");
    }

    #[test]
    fn uninstall_data_json_roundtrip_with_removed_envs() {
        let original = UninstallData {
            version: "3.12".to_string(),
            removed_envs: Some(vec!["web".to_string(), "api".to_string()]),
        };

        let json = serde_json::to_string(&original).unwrap();
        let restored: UninstallData = serde_json::from_str(&json).unwrap();

        assert_eq!(original, restored);
    }

    #[test]
    fn uninstall_data_json_empty_removed_envs() {
        let data = UninstallData {
            version: "3.12".to_string(),
            removed_envs: Some(vec![]),
        };

        let json = serde_json::to_string(&data).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        let envs = parsed["removed_envs"].as_array().unwrap();
        assert!(envs.is_empty());
    }

    // =========================================================================
    // Security & Edge Case Tests
    // =========================================================================

    #[test]
    fn uninstall_data_roundtrip_empty_version() {
        let original = UninstallData {
            version: String::new(),
            removed_envs: None,
        };

        let json = serde_json::to_string(&original).unwrap();
        let restored: UninstallData = serde_json::from_str(&json).unwrap();

        assert_eq!(original, restored);
        assert!(restored.version.is_empty());
    }

    #[test]
    fn uninstall_data_roundtrip_json_special_chars() {
        let test_cases = [
            ("quote", r#"3.12.0"injected"#),
            ("backslash", r"3.12.0\path"),
            ("newline", "3.12.0\nmalicious"),
            ("tab", "3.12.0\tvalue"),
            ("null_char", "3.12.0\0null"),
            ("control_chars", "3.12.0\r\n\x08"),
        ];

        for (name, version) in test_cases {
            let original = UninstallData {
                version: version.to_string(),
                removed_envs: None,
            };

            let json = serde_json::to_string(&original).unwrap();
            let restored: UninstallData = serde_json::from_str(&json).unwrap();

            assert_eq!(
                original, restored,
                "Roundtrip failed for '{}': {:?}",
                name, version
            );
        }
    }

    #[test]
    fn uninstall_data_roundtrip_unicode() {
        let test_cases = [
            ("korean", "3.12.0-\u{d55c}\u{ae00}\u{bc84}\u{c804}"),
            ("emoji", "3.12.0-\u{1f40d}python"),
            ("chinese", "3.12.0-\u{4e2d}\u{6587}\u{7248}"),
            (
                "mixed",
                "v3.12.0-\u{03b1}\u{03b2}\u{03b3}-\u{65e5}\u{672c}\u{8a9e}",
            ),
            ("rtl", "3.12.0-\u{05e2}\u{05d1}\u{05e8}\u{05d9}\u{05ea}"),
        ];

        for (name, version) in test_cases {
            let original = UninstallData {
                version: version.to_string(),
                removed_envs: None,
            };

            let json = serde_json::to_string(&original).unwrap();
            let restored: UninstallData = serde_json::from_str(&json).unwrap();

            assert_eq!(
                original, restored,
                "Unicode roundtrip failed for '{}'",
                name
            );
        }
    }

    #[test]
    fn uninstall_data_roundtrip_long_string() {
        let long_version = "3.12.0-".to_string() + &"x".repeat(10_000);
        let original = UninstallData {
            version: long_version,
            removed_envs: None,
        };

        let json = serde_json::to_string(&original).unwrap();
        let restored: UninstallData = serde_json::from_str(&json).unwrap();

        assert_eq!(original, restored);
        assert_eq!(restored.version.len(), 10_007);
    }

    // =========================================================================
    // execute Error Handling Tests
    // =========================================================================

    #[test]
    fn execute_returns_uv_not_found_when_uv_missing() {
        if UvClient::new().is_ok() {
            return;
        }

        let output = Output::new(0, true, crate::output::Colors::NONE, false);
        let result = execute(&output, "3.12.0", false, false);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            matches!(err, ScoopError::UvNotFound),
            "Expected UvNotFound, got {:?}",
            err
        );
    }

    // =========================================================================
    // CascadeAborted Error Tests
    // =========================================================================

    fn v(s: &str) -> PythonVersion {
        PythonVersion::parse(s).unwrap()
    }

    /// Which envs lose their Python. Fails if the exact-match rule changes,
    /// if an env recording less than the uninstalled version (`3.12` vs
    /// `3.12.14`) is skipped when nothing it accepts remains, or is taken
    /// although another install it accepts remains.
    #[rstest::rstest]
    #[case::exact_patch("3.12.14", "3.12.14", &[], true)]
    #[case::minor_filter_covers_patch_env("3.12", "3.12.1", &["3.13.1"], true)]
    #[case::minor_env_nothing_left("3.12.14", "3.12", &["3.13.1"], true)]
    #[case::minor_env_other_patch_left("3.12.14", "3.12", &["3.12.3"], false)]
    #[case::other_patch_env("3.12.14", "3.12.3", &["3.12.3"], false)]
    #[case::other_minor_env("3.12.14", "3.11", &[], false)]
    #[case::major_env_other_minor_left("3.12.14", "3", &["3.11.9"], false)]
    fn affected_envs_cases(
        #[case] uninstall: &str,
        #[case] recorded: &str,
        #[case] remaining: &[&str],
        #[case] affected: bool,
    ) {
        let remaining: Vec<PythonVersion> = remaining.iter().map(|r| v(r)).collect();
        let got = affected_envs(
            [("web".to_string(), Some(recorded.to_string()))],
            &v(uninstall),
            Some(&remaining),
        );
        assert_eq!(got == ["web"], affected, "{got:?}");
    }

    /// An env with no recorded version is never removed. Fails if a missing
    /// version is read as matching everything.
    #[test]
    fn affected_envs_skips_an_env_without_a_version() {
        assert!(affected_envs([("bare".to_string(), None)], &v("3.12"), Some(&[])).is_empty());
    }

    /// Without a trustworthy `remaining` (a non-plain request), only the
    /// exact-match rule applies: a `3.12` env is not taken for 3.12.0rc1.
    /// Fails if the less-specific rule runs without `remaining`.
    #[test]
    fn affected_envs_needs_remaining_for_a_less_specific_record() {
        let env = || [("web".to_string(), Some("3.12".to_string()))];
        assert!(affected_envs(env(), &v("3.12.0"), None).is_empty());
        assert_eq!(affected_envs(env(), &v("3.12.0"), Some(&[])), ["web"]);
    }

    /// Only digits and dots are a plain request. Fails if a suffix
    /// (`rc1`, `t`), an implementation prefix or an empty part passes.
    #[rstest::rstest]
    #[case("3.12.14", true)]
    #[case("3.12", true)]
    #[case("3.12.0rc1", false)]
    #[case("3.13t", false)]
    #[case("cpython@3.12", false)]
    #[case("3..12", false)]
    fn is_plain_version_cases(#[case] version: &str, #[case] plain: bool) {
        assert_eq!(is_plain_version(version), plain);
    }

    /// Runs `uninstall 3.12.14 --cascade --force` against a fake uv that
    /// lists `installed`, with one env `web` recording `3.12`. Returns
    /// whether `web` still exists and what uv was asked to uninstall.
    #[cfg(unix)]
    fn cascade_with_installed(installed: &[&str]) -> (bool, Vec<String>) {
        cascade_uninstalling("3.12.14", installed)
    }

    /// As [`cascade_with_installed`], uninstalling `version`.
    #[cfg(unix)]
    fn cascade_uninstalling(version: &str, installed: &[&str]) -> (bool, Vec<String>) {
        use crate::test_utils::{FakeUv, env_guard};
        let home = tempfile::tempdir().unwrap();
        let uv = FakeUv::new(installed);
        let path = uv.path_var();
        let _env = env_guard(&[
            (
                crate::paths::SCUV_HOME_ENV,
                Some(home.path().to_str().unwrap()),
            ),
            ("PATH", Some(path.as_str())),
        ]);
        let env = home.path().join("virtualenvs").join("web");
        std::fs::create_dir_all(env.join("bin")).unwrap();
        std::fs::write(env.join("pyvenv.cfg"), "version_info = 3.12\n").unwrap();
        std::fs::write(
            env.join(crate::core::Metadata::FILE_NAME),
            r#"{"name":"web","python_version":"3.12","created_at":"2026-01-01T00:00:00Z","created_by":"test","uv_version":null,"python_path":null}"#,
        )
        .unwrap();

        let out = Output::new(0, true, crate::output::Colors::NONE, false);
        execute(&out, version, true, true).unwrap();
        (env.exists(), uv.uninstalled())
    }

    /// The env was created against uv's 3.12 minor-version link and records
    /// `3.12`; 3.12.14 is the only 3.12 install, so uninstalling it breaks
    /// the env and --cascade must remove it. Fails if the cascade filter only
    /// matches envs that record the full version (the bug: the Python went
    /// and the broken env stayed).
    #[cfg(unix)]
    #[test]
    #[serial_test::serial]
    fn cascade_removes_a_minor_version_env_when_no_other_patch_remains() {
        let (env_left, uninstalled) = cascade_with_installed(&["3.12.14"]);
        assert!(!env_left, "the env lost its only Python and must go");
        assert_eq!(uninstalled, ["3.12.14"]);
    }

    /// A pre-release request: its numbers say 3.12.0, but the stable 3.12.0
    /// that stays still serves the env, and the numbers cannot tell the two
    /// apart. Fails if the less-specific rule runs for a non-plain request
    /// (the stable 3.12.0 drops out of what remains and the env is deleted).
    #[cfg(unix)]
    #[test]
    #[serial_test::serial]
    fn cascade_leaves_a_minor_version_env_for_a_pre_release_request() {
        let (env_left, _) = cascade_uninstalling("3.12.0rc1", &["3.12.0"]);
        assert!(env_left, "3.12.0 stays and serves the env");
    }

    /// The same env with 3.12.3 also installed keeps working after 3.12.14
    /// goes, so --cascade must leave it. Fails if a less specific record is
    /// matched without looking at what remains (it would delete a working
    /// env).
    #[cfg(unix)]
    #[test]
    #[serial_test::serial]
    fn cascade_keeps_a_minor_version_env_another_patch_still_serves() {
        let (env_left, uninstalled) = cascade_with_installed(&["3.12.14", "3.12.3"]);
        assert!(env_left, "3.12.3 still serves the env");
        assert_eq!(uninstalled, ["3.12.14"]);
    }

    #[test]
    fn cascade_aborted_error_code() {
        let err = ScoopError::CascadeAborted;
        assert_eq!(err.code(), "UNINSTALL_CASCADE_ABORTED");
    }

    #[test]
    fn cascade_aborted_has_no_suggestion() {
        let err = ScoopError::CascadeAborted;
        assert!(err.suggestion().is_none());
    }
}
