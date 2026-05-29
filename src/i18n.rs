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
        if let Some(code) = resolve_supported(&lang) {
            return code.to_string();
        }
    }

    // 2. Config file (scoop lang command)
    if let Ok(config) = Config::load() {
        if let Some(lang) = config.lang {
            if let Some(code) = resolve_supported(&lang) {
                return code.to_string();
            }
        }
    }

    // 3. System locale
    if let Some(locale) = sys_locale::get_locale() {
        if let Some(code) = resolve_supported(&locale) {
            return code.to_string();
        }
    }

    // 4. Fallback
    "en".to_string()
}

/// Resolve any locale string to a supported canonical code.
///
/// Tries a full case-insensitive match first — so `pt-BR`, `pt-br`,
/// `pt_BR`, and `pt-BR.UTF-8` all map to the canonical `"pt-BR"` — then
/// falls back to the language-only prefix (e.g. `ko-KR` → `ko`). Returns
/// `None` for unsupported locales.
fn resolve_supported(raw: &str) -> Option<&'static str> {
    let normalized = normalize(raw);
    canonical_supported(&normalized)
        .or_else(|| canonical_supported(normalized.split('-').next().unwrap_or("")))
}

/// Find the canonical supported code matching `candidate` case-insensitively.
///
/// Returns the code exactly as declared in [`SUPPORTED_LANGS`] (e.g. `"pt-BR"`),
/// which is the form `rust_i18n::set_locale` and `locales/app.yml` expect.
fn canonical_supported(candidate: &str) -> Option<&'static str> {
    SUPPORTED_LANGS
        .iter()
        .map(|(code, _)| *code)
        .find(|code| code.eq_ignore_ascii_case(candidate))
}

/// Get current locale.
pub fn current() -> String {
    rust_i18n::locale().to_string()
}

/// Check if a language code is supported (case-insensitive).
pub fn is_supported(lang: &str) -> bool {
    canonical_supported(lang).is_some()
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
    use serial_test::serial;

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
        assert!(is_supported("ja"));
        assert!(is_supported("pt-BR"));
        assert!(!is_supported("fr"));
        assert!(!is_supported("zh-CN")); // Not yet supported
    }

    #[test]
    fn test_language_name() {
        assert_eq!(language_name("en"), Some("English"));
        assert_eq!(language_name("ko"), Some("한국어"));
        assert_eq!(language_name("ja"), Some("日本語"));
        assert_eq!(language_name("pt-BR"), Some("Português (Brasil)"));
        assert_eq!(language_name("fr"), None);
    }

    #[test]
    #[serial]
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
    #[serial]
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

    #[test]
    #[serial]
    fn test_detect_pt_br_from_env_variants() {
        let original = std::env::var("SCOOP_LANG").ok();

        // SAFETY: serialized via #[serial]; env restored after test.
        unsafe {
            // All of these spellings must resolve to the canonical "pt-BR".
            for input in ["pt-BR", "pt-br", "PT-BR", "pt_BR", "pt-BR.UTF-8"] {
                std::env::set_var("SCOOP_LANG", input);
                assert_eq!(
                    detect_locale(),
                    "pt-BR",
                    "SCOOP_LANG={input} should resolve to canonical pt-BR"
                );
            }

            match original {
                Some(val) => std::env::set_var("SCOOP_LANG", val),
                None => std::env::remove_var("SCOOP_LANG"),
            }
        }
    }

    #[test]
    fn test_is_supported_is_case_insensitive() {
        assert!(is_supported("pt-BR"));
        assert!(is_supported("pt-br"));
        assert!(is_supported("PT-BR"));
        assert!(is_supported("EN"));
        assert!(is_supported("Ja"));
        assert!(!is_supported("fr"));
        assert!(!is_supported("zh-CN")); // Not yet supported
    }

    #[test]
    fn test_resolve_supported_full_then_prefix() {
        // Full case-insensitive match (region preserved as canonical).
        assert_eq!(resolve_supported("pt-BR.UTF-8"), Some("pt-BR"));
        assert_eq!(resolve_supported("PT_br"), Some("pt-BR"));
        // Language-only prefix fallback.
        assert_eq!(resolve_supported("ko_KR"), Some("ko"));
        assert_eq!(resolve_supported("en_US.UTF-8"), Some("en"));
        assert_eq!(resolve_supported("ja"), Some("ja"));
        // Unsupported.
        assert_eq!(resolve_supported("fr"), None);
        assert_eq!(resolve_supported("zh-CN"), None);
    }
}
