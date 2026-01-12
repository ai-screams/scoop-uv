//! Configuration management
//!
//! Handles persistent user settings stored in `~/.scoop/config.json`.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::error::Result;
use crate::paths;

/// User configuration stored in `~/.scoop/config.json`
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Preferred language code (e.g., "en", "ko")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

impl Config {
    /// Get config file path: `~/.scoop/config.json`
    pub fn path() -> Result<PathBuf> {
        Ok(paths::scoop_home()?.join("config.json"))
    }

    /// Load config from file.
    ///
    /// Returns default config if file doesn't exist.
    pub fn load() -> Result<Self> {
        let path = Self::path()?;

        if !path.exists() {
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(&path)?;
        let config = serde_json::from_str(&content)?;
        Ok(config)
    }

    /// Save config to file.
    ///
    /// Creates parent directory if needed.
    pub fn save(&self) -> Result<()> {
        let path = Self::path()?;

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, content)?;

        Ok(())
    }

    /// Set language preference.
    pub fn set_lang(&mut self, lang: Option<String>) {
        self.lang = lang;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.lang.is_none());
    }

    #[test]
    fn test_serialize_config() {
        let mut config = Config::default();
        config.set_lang(Some("ko".to_string()));

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"lang\":\"ko\""));
    }

    #[test]
    fn test_deserialize_config() {
        let json = r#"{"lang":"ko"}"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(config.lang, Some("ko".to_string()));
    }

    #[test]
    fn test_deserialize_empty_config() {
        let json = r#"{}"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert!(config.lang.is_none());
    }

    #[test]
    fn test_skip_serializing_none() {
        let config = Config::default();
        let json = serde_json::to_string(&config).unwrap();
        assert!(!json.contains("lang"));
    }

    // =========================================================================
    // Schema Compatibility Tests (Forward & Backward)
    // =========================================================================

    /// Forward compatibility: Unknown fields should be ignored (not cause parse failure)
    /// This allows older versions to read config files from newer versions
    #[test]
    fn test_config_ignores_unknown_fields() {
        let json = r#"{"lang":"en","theme":"dark","unknown_future_field":123}"#;
        let config: Config = serde_json::from_str(json).unwrap();

        // Known field is parsed
        assert_eq!(config.lang, Some("en".to_string()));
        // Unknown fields are silently ignored (no panic, no error)
    }

    /// Backward compatibility: Minimal config from old version should work
    #[test]
    fn test_config_backward_compat_minimal() {
        // Simulating config from v0.1.0 (no lang field existed)
        let json = r#"{}"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert!(config.lang.is_none());
    }

    /// Config roundtrip: serialize → deserialize → compare
    #[test]
    fn test_config_roundtrip() {
        let original = Config {
            lang: Some("ko".to_string()),
        };

        let json = serde_json::to_string(&original).unwrap();
        let restored: Config = serde_json::from_str(&json).unwrap();

        assert_eq!(original.lang, restored.lang);
    }

    /// Unicode language codes should work
    #[test]
    fn test_config_unicode_lang() {
        // While unusual, lang could theoretically contain unicode
        let json = r#"{"lang":"한국어"}"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(config.lang, Some("한국어".to_string()));
    }

    /// Null value for lang should be treated as None
    #[test]
    fn test_config_null_lang() {
        let json = r#"{"lang":null}"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert!(config.lang.is_none());
    }

    /// Empty string for lang is a valid value (different from None)
    #[test]
    fn test_config_empty_string_lang() {
        let json = r#"{"lang":""}"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(config.lang, Some(String::new()));
    }

    // =========================================================================
    // Malformed Input Tests
    // =========================================================================

    /// Malformed JSON should return parse error, not panic
    #[test]
    fn test_config_malformed_json() {
        let invalid_jsons = [
            r#"{"lang":"ko""#,    // Missing closing brace
            r#"{"lang":}"#,       // Missing value
            r#"not json at all"#, // Not JSON
            r#"["lang","ko"]"#,   // Array instead of object
        ];

        for json in invalid_jsons {
            let result: std::result::Result<Config, _> = serde_json::from_str(json);
            assert!(result.is_err(), "Should fail to parse: {}", json);
        }
    }

    /// Wrong type for lang should return error
    #[test]
    fn test_config_wrong_type_for_lang() {
        let json = r#"{"lang":123}"#; // Number instead of string
        let result: std::result::Result<Config, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    // =========================================================================
    // set_lang Method Tests
    // =========================================================================

    #[test]
    fn test_set_lang_some_to_none() {
        let mut config = Config {
            lang: Some("en".to_string()),
        };
        config.set_lang(None);
        assert!(config.lang.is_none());
    }

    #[test]
    fn test_set_lang_none_to_some() {
        let mut config = Config::default();
        config.set_lang(Some("ko".to_string()));
        assert_eq!(config.lang, Some("ko".to_string()));
    }

    #[test]
    fn test_set_lang_overwrites() {
        let mut config = Config {
            lang: Some("en".to_string()),
        };
        config.set_lang(Some("ko".to_string()));
        assert_eq!(config.lang, Some("ko".to_string()));
    }
}
