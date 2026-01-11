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
}
