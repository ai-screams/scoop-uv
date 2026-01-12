//! Uninstall command

use rust_i18n::t;

use crate::error::Result;
use crate::output::{Output, UninstallData};
use crate::uv::UvClient;

/// Execute the uninstall command
pub fn execute(output: &Output, version: &str) -> Result<()> {
    let uv = UvClient::new()?;

    output.info(&t!("uninstall.uninstalling", version = version));

    uv.uninstall_python(version)?;

    // JSON output
    if output.is_json() {
        output.json_success(
            "uninstall",
            UninstallData {
                version: version.to_string(),
            },
        );
        return Ok(());
    }

    output.success(&t!("uninstall.success", version = version));

    Ok(())
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
        };

        let json = serde_json::to_string(&data).unwrap();

        // Verify exact JSON structure
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed.is_object());
        assert!(parsed.get("version").is_some());
        assert_eq!(parsed["version"].as_str().unwrap(), "3.12.0");
    }

    /// True roundtrip test: serialize → deserialize → compare
    /// This catches Serialize/Deserialize mismatches (e.g., serde rename typos)
    #[test]
    fn uninstall_data_json_roundtrip() {
        let original = UninstallData {
            version: "3.11.5".to_string(),
        };

        // Serialize to JSON
        let json = serde_json::to_string(&original).unwrap();

        // Deserialize back to struct (this is what makes it a TRUE roundtrip)
        let restored: UninstallData = serde_json::from_str(&json).unwrap();

        // Compare original and restored
        assert_eq!(original, restored);
    }

    // =========================================================================
    // Security & Edge Case Tests
    // =========================================================================

    /// Empty string roundtrip - verifies JSON API handles empty input
    #[test]
    fn uninstall_data_roundtrip_empty_version() {
        let original = UninstallData {
            version: String::new(),
        };

        let json = serde_json::to_string(&original).unwrap();
        let restored: UninstallData = serde_json::from_str(&json).unwrap();

        assert_eq!(original, restored);
        assert!(restored.version.is_empty());
    }

    /// Special characters that require JSON escaping
    /// Verifies no JSON injection is possible
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

    /// Unicode characters - verifies proper UTF-8 handling
    #[test]
    fn uninstall_data_roundtrip_unicode() {
        let test_cases = [
            ("korean", "3.12.0-한글버전"),
            ("emoji", "3.12.0-🐍python"),
            ("chinese", "3.12.0-中文版"),
            ("mixed", "v3.12.0-αβγ-日本語"),
            ("rtl", "3.12.0-עברית"),
        ];

        for (name, version) in test_cases {
            let original = UninstallData {
                version: version.to_string(),
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

    /// Very long string - verifies no panic on large input (DoS resistance)
    #[test]
    fn uninstall_data_roundtrip_long_string() {
        let long_version = "3.12.0-".to_string() + &"x".repeat(10_000);
        let original = UninstallData {
            version: long_version,
        };

        let json = serde_json::to_string(&original).unwrap();
        let restored: UninstallData = serde_json::from_str(&json).unwrap();

        assert_eq!(original, restored);
        assert_eq!(restored.version.len(), 10_007); // "3.12.0-" + 10000 x's
    }

    // =========================================================================
    // execute Error Handling Tests
    // =========================================================================

    /// Test that execute returns UvNotFound when uv is not installed.
    #[test]
    fn execute_returns_uv_not_found_when_uv_missing() {
        // Skip test if uv is installed
        if UvClient::new().is_ok() {
            return; // uv is installed, can't test UvNotFound
        }

        let output = Output::new(0, true, true, false);
        let result = execute(&output, "3.12.0");

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            matches!(err, ScoopError::UvNotFound),
            "Expected UvNotFound, got {:?}",
            err
        );
    }

    // =========================================================================
    // Integration Test with Real uv
    // =========================================================================

    /// Comprehensive test that verifies execute behavior based on uv availability.
    /// Tests both normal mode and JSON mode.
    #[test]
    fn execute_behavior_matches_uv_availability() {
        let uv_available = UvClient::new().is_ok();

        // Test normal mode
        let normal_output = Output::new(0, true, true, false);
        let normal_result = execute(&normal_output, "0.0.1");

        // Test JSON mode
        let json_output = Output::new(0, true, true, true);
        let json_result = execute(&json_output, "0.0.2");

        if uv_available {
            // uv is installed - both modes should succeed
            assert!(
                normal_result.is_ok(),
                "Normal mode with uv should succeed. Got: {:?}",
                normal_result
            );
            assert!(
                json_result.is_ok(),
                "JSON mode with uv should succeed. Got: {:?}",
                json_result
            );
        } else {
            // uv is not installed - both modes should fail with UvNotFound
            assert!(normal_result.is_err(), "Without uv, execute should fail");
            assert!(
                matches!(normal_result.unwrap_err(), ScoopError::UvNotFound),
                "Error should be UvNotFound"
            );

            assert!(json_result.is_err(), "JSON mode without uv should fail");
            assert!(
                matches!(json_result.unwrap_err(), ScoopError::UvNotFound),
                "JSON mode error should be UvNotFound"
            );
        }
    }
}
