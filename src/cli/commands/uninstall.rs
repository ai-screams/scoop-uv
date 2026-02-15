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
        removed_envs = Some(handle_cascade(output, version, force)?);
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
fn handle_cascade(output: &Output, version: &str, force: bool) -> Result<Vec<String>> {
    let service = VirtualenvService::auto()?;
    let envs = service.list()?;

    // Parse the target version for prefix matching
    let version_filter = PythonVersion::parse(version);

    // Find environments using this Python version
    let matching_envs: Vec<String> = envs
        .into_iter()
        .filter(|env| {
            match (&version_filter, &env.python_version) {
                (Some(filter), Some(env_ver)) => {
                    PythonVersion::parse(env_ver).is_some_and(|v| filter.matches(&v))
                }
                _ => false, // Skip envs with no metadata
            }
        })
        .map(|env| env.name)
        .collect();

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

        let output = Output::new(0, true, true, false);
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
