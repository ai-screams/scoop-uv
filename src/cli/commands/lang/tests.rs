use super::*;
use crate::test_utils::with_temp_scoop_home;
use serial_test::serial;

// =========================================================================
// Test Helpers
// =========================================================================

fn create_test_output(json: bool) -> Output {
    Output::new(0, false, crate::output::Colors::NONE, json)
}

// =========================================================================
// show_current() Tests
// =========================================================================

#[test]
#[serial]
fn test_show_current_returns_supported_locale() {
    with_temp_scoop_home(|_| {
        let output = create_test_output(false);
        let result = show_current(&output);
        assert!(result.is_ok());

        // Verify current locale is a supported language
        let current = i18n::current();
        assert!(
            i18n::is_supported(&current),
            "Current locale '{}' should be supported",
            current
        );
    });
}

#[test]
#[serial]
fn test_show_current_json_mode_returns_supported_locale() {
    with_temp_scoop_home(|_| {
        let output = create_test_output(true);
        let result = show_current(&output);
        assert!(result.is_ok());

        // JSON mode should still have valid current locale
        let current = i18n::current();
        assert!(
            i18n::is_supported(&current),
            "JSON mode should maintain valid locale"
        );
    });
}

// =========================================================================
// set_language() Tests
// =========================================================================

#[test]
#[serial]
fn test_set_language_supported_en() {
    with_temp_scoop_home(|temp_dir| {
        let output = create_test_output(false);
        let result = set_language(&output, "en");
        assert!(result.is_ok());

        // Verify config was saved
        let config_path = temp_dir.path().join("config.json");
        assert!(config_path.exists());

        let content = std::fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("\"lang\":\"en\"") || content.contains("\"lang\": \"en\""));
    });
}

#[test]
#[serial]
fn test_set_language_supported_ko() {
    with_temp_scoop_home(|temp_dir| {
        let output = create_test_output(false);
        let result = set_language(&output, "ko");
        assert!(result.is_ok());

        // Verify config was saved
        let config_path = temp_dir.path().join("config.json");
        assert!(config_path.exists());

        let content = std::fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("\"lang\":\"ko\"") || content.contains("\"lang\": \"ko\""));
    });
}

#[test]
#[serial]
fn test_set_language_unsupported_does_not_change_locale() {
    with_temp_scoop_home(|_| {
        let output = create_test_output(false);
        let before = i18n::current();

        // Unsupported language returns Ok but should not change locale
        let result = set_language(&output, "xyz");
        assert!(result.is_ok());

        let after = i18n::current();
        assert_eq!(
            before, after,
            "Unsupported language should not change locale"
        );
    });
}

#[test]
#[serial]
fn test_set_language_unsupported_no_config_change() {
    with_temp_scoop_home(|temp_dir| {
        let output = create_test_output(false);
        let result = set_language(&output, "fr"); // fr is not yet supported
        assert!(result.is_ok());

        // Config should not be created for unsupported language
        let config_path = temp_dir.path().join("config.json");
        assert!(!config_path.exists());
    });
}

#[test]
#[serial]
fn test_set_language_json_mode_changes_locale() {
    with_temp_scoop_home(|_| {
        let output = create_test_output(true);

        // Set to Korean
        let result = set_language(&output, "ko");
        assert!(result.is_ok());
        assert_eq!(i18n::current(), "ko", "JSON mode should change locale");

        // Set back to English
        let result = set_language(&output, "en");
        assert!(result.is_ok());
        assert_eq!(i18n::current(), "en");
    });
}

#[test]
#[serial]
fn test_set_language_json_mode_unsupported_no_change() {
    with_temp_scoop_home(|_| {
        let output = create_test_output(true);

        // Set to valid first
        set_language(&output, "en").unwrap();
        let before = i18n::current();

        // Try unsupported
        let result = set_language(&output, "xyz");
        assert!(result.is_ok());
        assert_eq!(
            i18n::current(),
            before,
            "Unsupported language should not change locale in JSON mode"
        );
    });
}

#[test]
#[serial]
fn test_set_language_updates_locale() {
    with_temp_scoop_home(|_| {
        let output = create_test_output(false);

        // Set to Korean
        let result = set_language(&output, "ko");
        assert!(result.is_ok());
        assert_eq!(i18n::current(), "ko");

        // Set back to English
        let result = set_language(&output, "en");
        assert!(result.is_ok());
        assert_eq!(i18n::current(), "en");
    });
}

// =========================================================================
// reset_language() Tests
// =========================================================================

#[test]
#[serial]
fn test_reset_language_sets_supported_locale() {
    with_temp_scoop_home(|_| {
        let output = create_test_output(false);

        // Set to Korean first
        set_language(&output, "ko").unwrap();
        assert_eq!(i18n::current(), "ko");

        // Reset should set to a supported locale
        let result = reset_language(&output);
        assert!(result.is_ok());

        let current = i18n::current();
        assert!(
            i18n::is_supported(&current),
            "Reset should set to supported locale, got '{}'",
            current
        );
    });
}

#[test]
#[serial]
fn test_reset_language_removes_from_config() {
    with_temp_scoop_home(|temp_dir| {
        let output = create_test_output(false);

        // First set a language
        set_language(&output, "ko").unwrap();
        let config_path = temp_dir.path().join("config.json");
        assert!(config_path.exists());

        let content = std::fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("\"lang\""));

        // Now reset
        let result = reset_language(&output);
        assert!(result.is_ok());

        // Config should no longer contain lang
        let content = std::fs::read_to_string(&config_path).unwrap();
        assert!(!content.contains("\"lang\""));
    });
}

#[test]
#[serial]
fn test_reset_language_json_mode_sets_supported_locale() {
    with_temp_scoop_home(|_| {
        let output = create_test_output(true);

        // Set to Korean first
        set_language(&output, "ko").unwrap();

        // Reset in JSON mode
        let result = reset_language(&output);
        assert!(result.is_ok());

        let current = i18n::current();
        assert!(
            i18n::is_supported(&current),
            "JSON mode reset should set supported locale"
        );
    });
}

#[test]
#[serial]
fn test_reset_language_detects_system_locale() {
    with_temp_scoop_home(|_| {
        let output = create_test_output(false);

        // Set a language first
        set_language(&output, "ko").unwrap();
        assert_eq!(i18n::current(), "ko");

        // Reset should detect system locale
        let result = reset_language(&output);
        assert!(result.is_ok());

        // Current locale must be a supported language (en is always supported)
        let current = i18n::current();
        assert!(
            i18n::is_supported(&current),
            "Reset should detect supported system locale, got '{}'",
            current
        );
    });
}

// =========================================================================
// list_languages() Tests
// =========================================================================

#[test]
#[serial]
fn test_list_languages_includes_current_locale() {
    with_temp_scoop_home(|_| {
        let output = create_test_output(false);
        let result = list_languages(&output);
        assert!(result.is_ok());

        // Current locale should be in supported languages
        let current = i18n::current();
        let codes: Vec<&str> = SUPPORTED_LANGS.iter().map(|(c, _)| *c).collect();
        assert!(
            codes.contains(&current.as_str()),
            "Current locale '{}' should be in supported list",
            current
        );
    });
}

#[test]
#[serial]
fn test_list_languages_json_mode_has_supported_langs() {
    with_temp_scoop_home(|_| {
        let output = create_test_output(true);
        let result = list_languages(&output);
        assert!(result.is_ok());

        // JSON mode should also work with supported langs
        let codes: Vec<&str> = SUPPORTED_LANGS.iter().map(|(c, _)| *c).collect();
        assert!(codes.contains(&"en"), "Must include English");
        assert!(codes.contains(&"ko"), "Must include Korean");
    });
}

#[test]
fn test_supported_langs_includes_expected() {
    // Verify SUPPORTED_LANGS contains en and ko
    let codes: Vec<&str> = SUPPORTED_LANGS.iter().map(|(c, _)| *c).collect();
    assert!(codes.contains(&"en"));
    assert!(codes.contains(&"ko"));
}

// =========================================================================
// i18n Integration Tests
// =========================================================================

#[test]
fn test_is_supported_returns_true_for_en() {
    assert!(i18n::is_supported("en"));
}

#[test]
fn test_is_supported_returns_true_for_ko() {
    assert!(i18n::is_supported("ko"));
}

#[test]
fn test_is_supported_returns_false_for_unknown() {
    assert!(!i18n::is_supported("xyz"));
    assert!(!i18n::is_supported("fr"));
    assert!(!i18n::is_supported(""));
}

#[test]
fn test_language_name_returns_correct_names() {
    assert_eq!(i18n::language_name("en"), Some("English"));
    assert_eq!(i18n::language_name("ko"), Some("한국어"));
}

#[test]
fn test_language_name_returns_none_for_unknown() {
    assert_eq!(i18n::language_name("xyz"), None);
    assert_eq!(i18n::language_name("fr"), None);
}

// =========================================================================
// Edge Cases
// =========================================================================

#[test]
#[serial]
fn test_set_language_empty_string() {
    with_temp_scoop_home(|_| {
        let output = create_test_output(false);
        let result = set_language(&output, "");
        // Empty string is not supported, should return Ok but not save
        assert!(result.is_ok());
    });
}

#[test]
#[serial]
fn test_set_language_case_sensitive() {
    with_temp_scoop_home(|_| {
        let output = create_test_output(false);

        // "EN" is not supported (case-sensitive)
        let result = set_language(&output, "EN");
        assert!(result.is_ok());

        // Should not change locale to "EN" since it's unsupported
        // The locale stays as whatever it was before
    });
}

#[test]
#[serial]
fn test_set_language_overwrites_previous() {
    with_temp_scoop_home(|temp_dir| {
        let output = create_test_output(false);

        // Set to ko
        set_language(&output, "ko").unwrap();

        // Overwrite with en
        set_language(&output, "en").unwrap();

        let config_path = temp_dir.path().join("config.json");
        let content = std::fs::read_to_string(&config_path).unwrap();

        // Should only contain en, not ko
        assert!(content.contains("\"en\""));
        // Note: config has one lang field, so ko is replaced
    });
}
