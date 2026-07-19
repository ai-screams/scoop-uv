//! Check for legacy scoop remnants (env vars, dirs, files) left over from
//! the `scoop` → `scuv` rename.

use crate::paths;

use super::super::types::{Check, CheckResult};

/// Check for leftover legacy `scoop` state during the `scoop` → `scuv`
/// shim window: env vars, `~/.scoop` without a sibling `~/.scuv`, and
/// legacy-named files in the current directory.
///
/// DEPRECATION(0.16.0): remove this check together with the shims.
fn check_legacy_remnants() -> CheckResult {
    let mut found: Vec<String> = Vec::new();

    for var in [
        paths::LEGACY_HOME_ENV,
        "SCOOP_VERSION",
        "SCOOP_LANG",
        // No runtime warning for this one (it's read per prompt by the shell
        // hook, warning there would spam) — this doctor hint is the only
        // place a user learns to rename it.
        "SCOOP_NO_AUTO",
    ] {
        if std::env::var_os(var).is_some() {
            found.push(format!("${var}"));
        }
    }

    if let Some(home) = dirs::home_dir() {
        if home.join(".scoop").exists() && !home.join(".scuv").exists() {
            found.push("~/.scoop".to_string());
        }
    }

    if let Ok(cwd) = std::env::current_dir() {
        for f in [
            paths::LEGACY_VERSION_FILE,
            crate::core::manifest::LEGACY_MANIFEST_FILE,
        ] {
            if cwd.join(f).exists() {
                found.push(f.to_string());
            }
        }
    }

    if found.is_empty() {
        CheckResult::ok("legacy", "legacy scoop remnants")
    } else {
        CheckResult::warn(
            "legacy",
            "legacy scoop remnants",
            rust_i18n::t!("doctor.legacy_found", items = found.join(", ")),
        )
        .with_suggestion(rust_i18n::t!("doctor.legacy_suggestion"))
    }
}

/// Check for legacy scoop remnants (env vars, dirs, files) left over from
/// the `scoop` → `scuv` rename.
pub(super) struct LegacyCheck;

impl Check for LegacyCheck {
    fn id(&self) -> &'static str {
        "legacy"
    }

    fn name(&self) -> &'static str {
        "legacy scoop remnants"
    }

    fn run(&self) -> Vec<CheckResult> {
        vec![check_legacy_remnants()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::core::doctor::CheckStatus;
    use crate::core::doctor::checks::test_support::TempDirCwdGuard;
    use serial_test::serial;

    // ==========================================================================
    // LegacyCheck / check_legacy_remnants: scoop -> scuv shim window
    //
    // These tests override HOME (not just SCUV_HOME) because the legacy
    // check inspects `dirs::home_dir()` directly rather than going through
    // `paths::scoop_home()` — the dev machine's real `~/.scoop` must never
    // leak into the assertions below.
    // ==========================================================================

    #[test]
    #[serial]
    fn legacy_check_is_ok_when_environment_is_clean() {
        let home_tmp = tempfile::tempdir().unwrap();
        let _g = crate::test_utils::env_guard(&[
            (paths::LEGACY_HOME_ENV, None),
            ("SCOOP_VERSION", None),
            ("SCOOP_LANG", None),
            ("SCOOP_NO_AUTO", None),
            (paths::SCUV_HOME_ENV, None),
            ("SCUV_VERSION", None),
            ("SCUV_LANG", None),
            ("HOME", Some(home_tmp.path().to_str().unwrap())),
        ]);
        let _cwd = TempDirCwdGuard::new();

        let result = check_legacy_remnants();
        assert!(
            result.is_ok(),
            "expected Ok on a clean environment, got {result:#?}"
        );
    }

    #[test]
    #[serial]
    fn legacy_check_warns_on_legacy_home_env_var() {
        let home_tmp = tempfile::tempdir().unwrap();
        let _g = crate::test_utils::env_guard(&[
            (
                paths::LEGACY_HOME_ENV,
                Some("/tmp/legacy-home-doesnt-need-to-exist"),
            ),
            ("SCOOP_VERSION", None),
            ("SCOOP_LANG", None),
            ("SCOOP_NO_AUTO", None),
            (paths::SCUV_HOME_ENV, None),
            ("SCUV_VERSION", None),
            ("SCUV_LANG", None),
            ("HOME", Some(home_tmp.path().to_str().unwrap())),
        ]);
        let _cwd = TempDirCwdGuard::new();

        let result = check_legacy_remnants();
        assert!(result.is_warning(), "expected Warning, got {result:#?}");
        let msg = match &result.status {
            CheckStatus::Warning(m) => m.clone(),
            other => panic!("expected Warning, got {other:?}"),
        };
        assert!(
            msg.contains("SCOOP_HOME"),
            "message should name $SCOOP_HOME, got: {msg}"
        );
        assert!(
            result.suggestion.is_some(),
            "a legacy-remnant warning must carry a suggestion"
        );
    }

    #[test]
    #[serial]
    fn legacy_check_warns_on_scoop_version_and_scoop_lang_env_vars() {
        let home_tmp = tempfile::tempdir().unwrap();
        let _g = crate::test_utils::env_guard(&[
            (paths::LEGACY_HOME_ENV, None),
            ("SCOOP_VERSION", Some("myenv")),
            ("SCOOP_LANG", Some("ko")),
            ("SCOOP_NO_AUTO", Some("1")),
            (paths::SCUV_HOME_ENV, None),
            ("SCUV_VERSION", None),
            ("SCUV_LANG", None),
            ("HOME", Some(home_tmp.path().to_str().unwrap())),
        ]);
        let _cwd = TempDirCwdGuard::new();

        let result = check_legacy_remnants();
        assert!(result.is_warning(), "expected Warning, got {result:#?}");
        let msg = match &result.status {
            CheckStatus::Warning(m) => m.clone(),
            other => panic!("expected Warning, got {other:?}"),
        };
        assert!(msg.contains("SCOOP_VERSION"), "got: {msg}");
        assert!(msg.contains("SCOOP_LANG"), "got: {msg}");
        assert!(msg.contains("SCOOP_NO_AUTO"), "got: {msg}");
    }

    #[test]
    #[serial]
    fn legacy_check_warns_on_legacy_home_dir_without_new_dir() {
        let home_tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir(home_tmp.path().join(".scoop")).unwrap();
        let _g = crate::test_utils::env_guard(&[
            (paths::LEGACY_HOME_ENV, None),
            ("SCOOP_VERSION", None),
            ("SCOOP_LANG", None),
            ("SCOOP_NO_AUTO", None),
            (paths::SCUV_HOME_ENV, None),
            ("SCUV_VERSION", None),
            ("SCUV_LANG", None),
            ("HOME", Some(home_tmp.path().to_str().unwrap())),
        ]);
        let _cwd = TempDirCwdGuard::new();

        let result = check_legacy_remnants();
        assert!(result.is_warning(), "expected Warning, got {result:#?}");
        let msg = match &result.status {
            CheckStatus::Warning(m) => m.clone(),
            other => panic!("expected Warning, got {other:?}"),
        };
        assert!(msg.contains("~/.scoop"), "got: {msg}");
    }

    /// `~/.scoop` existing is only a remnant while `~/.scuv` hasn't been
    /// created yet — once the user has migrated, the two can briefly
    /// coexist (e.g. before the user deletes the old one) without that
    /// being something doctor should flag.
    #[test]
    #[serial]
    fn legacy_check_ok_when_both_scoop_and_scuv_home_dirs_exist() {
        let home_tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir(home_tmp.path().join(".scoop")).unwrap();
        std::fs::create_dir(home_tmp.path().join(".scuv")).unwrap();
        let _g = crate::test_utils::env_guard(&[
            (paths::LEGACY_HOME_ENV, None),
            ("SCOOP_VERSION", None),
            ("SCOOP_LANG", None),
            ("SCOOP_NO_AUTO", None),
            (paths::SCUV_HOME_ENV, None),
            ("SCUV_VERSION", None),
            ("SCUV_LANG", None),
            ("HOME", Some(home_tmp.path().to_str().unwrap())),
        ]);
        let _cwd = TempDirCwdGuard::new();

        let result = check_legacy_remnants();
        assert!(result.is_ok(), "expected Ok, got {result:#?}");
    }

    #[test]
    #[serial]
    fn legacy_check_warns_on_legacy_version_and_manifest_files_in_cwd() {
        let home_tmp = tempfile::tempdir().unwrap();
        let _g = crate::test_utils::env_guard(&[
            (paths::LEGACY_HOME_ENV, None),
            ("SCOOP_VERSION", None),
            ("SCOOP_LANG", None),
            ("SCOOP_NO_AUTO", None),
            (paths::SCUV_HOME_ENV, None),
            ("SCUV_VERSION", None),
            ("SCUV_LANG", None),
            ("HOME", Some(home_tmp.path().to_str().unwrap())),
        ]);
        let cwd_guard = TempDirCwdGuard::new();
        std::fs::write(cwd_guard.path().join(".scoop-version"), "myenv").unwrap();
        std::fs::write(cwd_guard.path().join(".scoop.toml"), "").unwrap();

        let result = check_legacy_remnants();
        assert!(result.is_warning(), "expected Warning, got {result:#?}");
        let msg = match &result.status {
            CheckStatus::Warning(m) => m.clone(),
            other => panic!("expected Warning, got {other:?}"),
        };
        assert!(msg.contains(".scoop-version"), "got: {msg}");
        assert!(msg.contains(".scoop.toml"), "got: {msg}");
    }

    #[test]
    #[serial]
    fn legacy_check_suggestion_names_current_env_var_and_removal_version() {
        let _locale = crate::test_utils::LocaleGuard::capture();
        rust_i18n::set_locale("en");

        let home_tmp = tempfile::tempdir().unwrap();
        let _g = crate::test_utils::env_guard(&[
            (
                paths::LEGACY_HOME_ENV,
                Some("/tmp/legacy-home-doesnt-need-to-exist"),
            ),
            ("SCOOP_VERSION", None),
            ("SCOOP_LANG", None),
            ("SCOOP_NO_AUTO", None),
            (paths::SCUV_HOME_ENV, None),
            ("SCUV_VERSION", None),
            ("SCUV_LANG", None),
            ("HOME", Some(home_tmp.path().to_str().unwrap())),
        ]);
        let _cwd = TempDirCwdGuard::new();

        let result = check_legacy_remnants();
        let suggestion = result.suggestion.as_deref().unwrap_or_default();
        assert!(
            suggestion.contains("SCUV_"),
            "suggestion should point at SCUV_*, got: {suggestion}"
        );
        assert!(
            suggestion.contains("v0.16.0"),
            "suggestion should name the removal version, got: {suggestion}"
        );
    }

    // ==========================================================================
    // Check-trait dispatch coverage. The underlying logic above is tested
    // directly; these pin the thin trait wrappers (id/name/run) so a broken
    // registration or renamed check surfaces in tests, not in `doctor` output.
    // ==========================================================================

    #[test]
    #[serial]
    fn legacy_check_trait_dispatch_reports_identity_and_runs() {
        let home_tmp = tempfile::tempdir().unwrap();
        let _g = crate::test_utils::env_guard(&[
            (paths::LEGACY_HOME_ENV, None),
            ("SCOOP_VERSION", None),
            ("SCOOP_LANG", None),
            ("SCOOP_NO_AUTO", None),
            (paths::SCUV_HOME_ENV, None),
            ("SCUV_VERSION", None),
            ("SCUV_LANG", None),
            ("HOME", Some(home_tmp.path().to_str().unwrap())),
        ]);
        let _cwd = TempDirCwdGuard::new();

        let check: &dyn Check = &LegacyCheck;
        assert_eq!(check.id(), "legacy");
        assert_eq!(check.name(), "legacy scoop remnants");
        let results = check.run();
        assert_eq!(results.len(), 1, "got {results:#?}");
        assert_eq!(results[0].id, "legacy");
        assert!(results[0].is_ok(), "clean env must be Ok, got {results:#?}");
    }
}
