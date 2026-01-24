//! Internationalization support
//!
//! Locale detection priority:
//! 1. `SCOOP_LANG` environment variable (override)
//! 2. `~/.scoop/config.json` (`scoop lang` command)
//! 3. System locale (sys-locale)
//! 4. Fallback: "en"

use crate::config::Config;

/// Supported languages with their display names.
pub const SUPPORTED_LANGS: &[(&str, &str)] = &[
    ("en", "English"),
    ("ko", "한국어"),
    ("pt-BR", "Português (Brasil)"),
    ("ja", "日本語"),
    // Coming Soon: ja (日本語), zh-CN (简体中文), fr (Français), ar (العربية)
];

/// Initialize locale on startup.
///
/// Call this early in main() before any translated output.
pub fn init() {
    let locale = detect_locale();
    rust_i18n::set_locale(&locale);
}

/// Detect locale based on priority:
/// 1. SCOOP_LANG env → 2. config.json → 3. sys-locale → 4. "en"
pub fn detect_locale() -> String {
    // 1. SCOOP_LANG environment variable (override for scripts/CI)
    if let Ok(lang) = std::env::var("SCOOP_LANG") {
        let normalized = normalize(&lang);
        if is_supported(&normalized) {
            return normalized;
        }
        // If SCOOP_LANG is set but unsupported, extract language code
        let lang_code = normalized.split('-').next().unwrap_or("en");
        if is_supported(lang_code) {
            return lang_code.to_string();
        }
    }

    // 2. Config file (scoop lang command)
    if let Ok(config) = Config::load() {
        if let Some(lang) = config.lang {
            if is_supported(&lang) {
                return lang;
            }
        }
    }

    // 3. System locale
    if let Some(locale) = sys_locale::get_locale() {
        let normalized = normalize(&locale);
        // Extract language code (e.g., "ko-kr" -> "ko")
        let lang_code = normalized.split('-').next().unwrap_or("en");
        if is_supported(lang_code) {
            return lang_code.to_string();
        }
    }

    // 4. Fallback
    "en".to_string()
}

/// Get current locale.
pub fn current() -> String {
    rust_i18n::locale().to_string()
}

/// Check if a language code is supported.
pub fn is_supported(lang: &str) -> bool {
    SUPPORTED_LANGS.iter().any(|(code, _)| *code == lang)
}

/// Get language display name for a code.
pub fn language_name(code: &str) -> Option<&'static str> {
    SUPPORTED_LANGS
        .iter()
        .find(|(c, _)| *c == code)
        .map(|(_, name)| *name)
}

/// Normalize locale string.
///
/// Examples:
/// - "ko_KR.UTF-8" → "ko-kr"
/// - "en_US" → "en-us"
/// - "ja" → "ja"
fn normalize(locale: &str) -> String {
    locale
        .split('.')
        .next()
        .unwrap_or("en")
        .replace('_', "-")
        .to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize() {
        assert_eq!(normalize("ko_KR.UTF-8"), "ko-kr");
        assert_eq!(normalize("en_US"), "en-us");
        assert_eq!(normalize("ja"), "ja");
        assert_eq!(normalize("zh_CN.UTF-8"), "zh-cn");
    }

    #[test]
    fn test_is_supported() {
        assert!(is_supported("en"));
        assert!(is_supported("ko"));
        assert!(!is_supported("fr"));
        assert!(!is_supported("ja")); // Not yet supported
    }

    #[test]
    fn test_language_name() {
        assert_eq!(language_name("en"), Some("English"));
        assert_eq!(language_name("ko"), Some("한국어"));
        assert_eq!(language_name("fr"), None);
    }

    #[test]
    fn test_detect_with_env() {
        // Save original
        let original = std::env::var("SCOOP_LANG").ok();

        // SAFETY: Single-threaded test, env var changes are restored after test
        unsafe {
            // Test with SCOOP_LANG=ko
            std::env::set_var("SCOOP_LANG", "ko");
            assert_eq!(detect_locale(), "ko");

            // Test with SCOOP_LANG=en
            std::env::set_var("SCOOP_LANG", "en");
            assert_eq!(detect_locale(), "en");

            // Restore
            match original {
                Some(val) => std::env::set_var("SCOOP_LANG", val),
                None => std::env::remove_var("SCOOP_LANG"),
            }
        }
    }

    #[test]
    fn test_detect_with_unsupported_env() {
        // Save original
        let original = std::env::var("SCOOP_LANG").ok();

        // SAFETY: Single-threaded test, env var changes are restored after test
        unsafe {
            // Test with unsupported language - should fall through
            std::env::set_var("SCOOP_LANG", "fr");
            let locale = detect_locale();
            // Should either be "en" (fallback) or system locale
            assert!(is_supported(&locale) || locale == "en");

            // Restore
            match original {
                Some(val) => std::env::set_var("SCOOP_LANG", val),
                None => std::env::remove_var("SCOOP_LANG"),
            }
        }
    }
}
