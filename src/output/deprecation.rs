//! One-shot deprecation warnings (stderr only; stdout is eval'd by shells).

use std::collections::HashSet;
use std::sync::Mutex;

use once_cell::sync::Lazy;

static SEEN: Lazy<Mutex<HashSet<String>>> = Lazy::new(|| Mutex::new(HashSet::new()));

/// Prints `warning: <message>` to stderr once per unique message per process.
///
/// Returns `true` if the warning was emitted, `false` if it was suppressed
/// as a duplicate. Never touches stdout.
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

    #[test]
    fn warn_once_emits_then_suppresses() {
        assert!(warn_once("task1-test-msg-a"));
        assert!(!warn_once("task1-test-msg-a"));
        assert!(warn_once("task1-test-msg-b"));
    }
}
