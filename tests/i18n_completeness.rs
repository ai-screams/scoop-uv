//! Locale completeness checks.
//!
//! 1. Every translation key in `locales/app.yml` must exist in every supported
//!    locale (en/ko/ja/pt-BR).
//! 2. Every `t!("...")` key used in `src/` must have a translation.
//!
//! Both are pure file reads — no process-global locale is touched, so they run
//! in parallel and need no `#[serial]`.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

const LOCALES: &[&str] = &["en", "ko", "ja", "pt-BR"];

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Parse `app.yml` into `key -> set of locales that define it`.
fn translation_keys() -> BTreeMap<String, BTreeSet<String>> {
    let path = manifest_dir().join("locales/app.yml");
    let raw = std::fs::read_to_string(&path).expect("read locales/app.yml");
    let doc: BTreeMap<String, serde_yaml::Value> =
        serde_yaml::from_str(&raw).expect("parse locales/app.yml");

    let mut keys = BTreeMap::new();
    for (key, value) in doc {
        if key.starts_with('_') {
            continue; // _version and other metadata
        }
        let Some(mapping) = value.as_mapping() else {
            continue;
        };
        let locales: BTreeSet<String> = mapping
            .keys()
            .filter_map(|k| k.as_str().map(str::to_string))
            .collect();
        keys.insert(key, locales);
    }
    keys
}

#[test]
fn every_key_exists_in_all_locales() {
    let keys = translation_keys();
    assert!(!keys.is_empty(), "no translation keys parsed");

    let mut missing = Vec::new();
    for (key, locales) in &keys {
        for loc in LOCALES {
            if !locales.contains(*loc) {
                missing.push(format!("  {key} -> [{loc}]"));
            }
        }
    }

    assert!(
        missing.is_empty(),
        "Translation keys missing in some locales:\n{}",
        missing.join("\n")
    );
}

#[test]
fn every_t_key_in_code_has_a_translation() {
    let known = translation_keys();
    let mut missing = Vec::new();

    for file in rs_files(&manifest_dir().join("src")) {
        let content = std::fs::read_to_string(&file).expect("read source file");
        let code = strip_line_comments(&content);
        for key in extract_t_keys(&code) {
            if !known.contains_key(&key) {
                missing.push(format!("  {key}  ({})", file.display()));
            }
        }
    }

    assert!(
        missing.is_empty(),
        "`t!` keys used in code but absent from locales/app.yml:\n{}",
        missing.join("\n")
    );
}

/// Drop `//`-prefixed lines so doc/example keys don't count as real usage.
fn strip_line_comments(content: &str) -> String {
    content
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Extract literal keys from `t!("key" ...)`, tolerating whitespace/newlines
/// between `t!(` and the opening quote (multi-line invocations).
///
/// Only string-literal keys are detected — `t!(SOME_CONST)` or concatenated
/// keys are not covered. Every `t!` in this codebase uses a literal key, so
/// this is complete today; revisit if a non-literal key is ever introduced.
fn extract_t_keys(code: &str) -> BTreeSet<String> {
    let bytes = code.as_bytes();
    let needle = b"t!(";
    let mut keys = BTreeSet::new();
    let mut i = 0;

    while let Some(pos) = window_find(&bytes[i..], needle) {
        let idx = i + pos; // absolute index of the `t`

        // Require a standalone `t!` token; otherwise `t!(` also matches the
        // tails of `format!(`, `assert!(`, `print!(`, etc.
        let token_start = idx == 0 || {
            let prev = bytes[idx - 1];
            !(prev.is_ascii_alphanumeric() || prev == b'_')
        };

        if token_start {
            let mut j = idx + needle.len();
            while j < bytes.len() && (bytes[j] as char).is_whitespace() {
                j += 1;
            }
            if j < bytes.len() && bytes[j] == b'"' {
                let start = j + 1;
                if let Some(end) = bytes[start..].iter().position(|&b| b == b'"') {
                    keys.insert(code[start..start + end].to_string());
                    i = start + end + 1;
                    continue;
                }
            }
        }
        i = idx + needle.len();
    }
    keys
}

fn window_find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|w| w == needle)
}

fn rs_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for entry in std::fs::read_dir(&d).expect("read_dir") {
            let path = entry.expect("dir entry").path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "rs") {
                out.push(path);
            }
        }
    }
    out
}
