//! One-shot deprecation warnings (stderr only; stdout is eval'd by shells).

use std::collections::HashSet;
use std::sync::Mutex;

use once_cell::sync::Lazy;

static SEEN: Lazy<Mutex<HashSet<String>>> = Lazy::new(|| Mutex::new(HashSet::new()));

/// Environment variable that silences deprecation warnings for the process.
///
/// Set by the shell wrapper's chained `use` → `activate` call so a single
/// user action doesn't print the same warning twice (each chained call is a
/// fresh process, so the in-process dedup can't help there). An empty value
/// counts as unset. Scripts may also set it to quiet migration noise.
pub const SUPPRESS_ENV: &str = "SCUV_SUPPRESS_DEPRECATION";

/// Prints `warning: <message>` to stderr once per unique message per process.
///
/// Returns `true` if the warning was emitted, `false` if it was suppressed
/// as a duplicate or via [`SUPPRESS_ENV`]. Suppressed messages are not
/// recorded as seen. Never touches stdout.
///
/// # Examples
///
/// ```
/// use scoop_uv::output::deprecation::warn_once;
/// assert!(warn_once("doctest-unique-message"));
/// assert!(!warn_once("doctest-unique-message"));
/// ```
///
/// # Panics
///
/// Panics if the internal seen-message set's mutex is poisoned (i.e. a
/// previous holder of the lock panicked while holding it).
pub fn warn_once(message: &str) -> bool {
    if std::env::var_os(SUPPRESS_ENV).is_some_and(|v| !v.is_empty()) {
        return false;
    }
    let mut seen = SEEN.lock().expect("deprecation set poisoned");
    if !seen.insert(message.to_string()) {
        return false;
    }
    eprintln!("warning: {message}");
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn warn_once_emits_then_suppresses() {
        // Guard against an inherited SCUV_SUPPRESS_DEPRECATION (would make
        // every warn_once return false) and serialize against the other
        // env-mutating tests here.
        let _g = crate::test_utils::env_guard(&[(SUPPRESS_ENV, None)]);
        assert!(warn_once("task1-test-msg-a"));
        assert!(!warn_once("task1-test-msg-a"));
        assert!(warn_once("task1-test-msg-b"));
    }

    #[test]
    #[serial]
    fn warn_once_respects_suppress_env() {
        {
            let _g = crate::test_utils::env_guard(&[(SUPPRESS_ENV, Some("1"))]);
            assert!(!warn_once("suppress-env-msg"));
        }
        // Suppressed calls are not recorded: without the env var the same
        // message still warns.
        let _g = crate::test_utils::env_guard(&[(SUPPRESS_ENV, None)]);
        assert!(warn_once("suppress-env-msg"));
    }

    #[test]
    #[serial]
    fn warn_once_treats_empty_suppress_env_as_unset() {
        let _g = crate::test_utils::env_guard(&[(SUPPRESS_ENV, Some(""))]);
        assert!(warn_once("suppress-env-empty-msg"));
    }
}
