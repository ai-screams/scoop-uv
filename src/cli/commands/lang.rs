//! Lang command - Set or show language preference

use rust_i18n::t;

use crate::config::Config;
use crate::error::Result;
use crate::i18n::{self, SUPPORTED_LANGS};
use crate::output::Output;

/// Execute the lang command
pub fn execute(output: &Output, lang: Option<&str>, list: bool, reset: bool) -> Result<()> {
    if list {
        return list_languages(output);
    }

    if reset {
        return reset_language(output);
    }

    match lang {
        Some(code) => set_language(output, code),
        None => show_current(output),
    }
}

/// Show current language setting
fn show_current(output: &Output) -> Result<()> {
    let current = i18n::current();
    let name = i18n::language_name(&current).unwrap_or("Unknown");

    if output.is_json() {
        output.json_success(
            "lang",
            serde_json::json!({
                "current": current,
                "name": name,
            }),
        );
    } else {
        output.println(&t!("lang.current", lang = current, name = name));
    }

    Ok(())
}

/// Set language preference
fn set_language(output: &Output, code: &str) -> Result<()> {
    if !i18n::is_supported(code) {
        if output.is_json() {
            output.json_success(
                "lang",
                serde_json::json!({
                    "unsupported": true,
                    "lang": code,
                    "message": t!("lang.unsupported", lang = code).to_string(),
                }),
            );
        } else {
            output.error(&t!("lang.unsupported", lang = code));
            output.info(&t!("lang.hint"));
        }
        return Ok(());
    }

    // Save to config
    let mut config = Config::load()?;
    config.set_lang(Some(code.to_string()));
    config.save()?;

    // Update current locale
    rust_i18n::set_locale(code);

    if output.is_json() {
        let name = i18n::language_name(code).unwrap_or("Unknown");
        output.json_success(
            "lang",
            serde_json::json!({
                "set": code,
                "name": name,
            }),
        );
    } else {
        output.success(&t!("lang.set", lang = code));
    }

    Ok(())
}

/// Reset to system default language
fn reset_language(output: &Output) -> Result<()> {
    // Remove from config
    let mut config = Config::load()?;
    config.set_lang(None);
    config.save()?;

    // Detect system locale
    let detected = i18n::detect_locale();
    rust_i18n::set_locale(&detected);

    if output.is_json() {
        let name = i18n::language_name(&detected).unwrap_or("Unknown");
        output.json_success(
            "lang",
            serde_json::json!({
                "reset": true,
                "detected": detected,
                "name": name,
            }),
        );
    } else {
        output.success(&t!("lang.reset", lang = detected));
    }

    Ok(())
}

/// List all supported languages
fn list_languages(output: &Output) -> Result<()> {
    let current = i18n::current();

    if output.is_json() {
        let languages: Vec<serde_json::Value> = SUPPORTED_LANGS
            .iter()
            .map(|(code, name)| {
                serde_json::json!({
                    "code": code,
                    "name": name,
                    "current": *code == current,
                })
            })
            .collect();

        output.json_success("lang", serde_json::json!({ "languages": languages }));
    } else {
        output.println(&t!("lang.list_header"));
        for (code, name) in SUPPORTED_LANGS {
            let marker = if *code == current { "*" } else { " " };
            output.println(&format!("  {marker} {code}\t{name}"));
        }
    }

    Ok(())
}
