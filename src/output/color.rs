//! Color decision, made once at startup (#205).
//!
//! Every colored stream follows one decision: scuv's own messages (stderr),
//! `list`'s highlight (stdout), clap's help and errors, the `migrate all`
//! progress bar, log lines and the panic report. The `--color` flag itself
//! belongs to the CLI layer ([`crate::cli::color`]); this module only takes
//! its override as an `Option<bool>`.

use std::ffi::OsStr;

/// Whether each output stream gets color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Colors {
    /// Color on stdout (`list`'s highlight).
    pub stdout: bool,
    /// Color on stderr (messages, progress bar, logs, panic report).
    pub stderr: bool,
}

impl Colors {
    /// No color on either stream.
    pub const NONE: Self = Self {
        stdout: false,
        stderr: false,
    };

    /// Color on both streams.
    pub const ALL: Self = Self {
        stdout: true,
        stderr: true,
    };

    /// Decides for this process: reads `NO_COLOR` and whether stdout and
    /// stderr are terminals. `force` is the explicit override (see
    /// [`resolve`]).
    pub fn detect(force: Option<bool>) -> Self {
        use std::io::IsTerminal;
        let no_color = std::env::var_os("NO_COLOR");
        Self {
            stdout: resolve(force, no_color.as_deref(), std::io::stdout().is_terminal()),
            stderr: resolve(force, no_color.as_deref(), std::io::stderr().is_terminal()),
        }
    }
}

/// Decides color for one stream.
///
/// An explicit override (`Some`) wins over the environment, as no-color.org
/// allows. Without one, color needs a terminal and no non-empty `NO_COLOR`
/// (an empty one is ignored, per no-color.org).
///
/// # Examples
///
/// ```
/// use scoop_uv::output::color::resolve;
/// use std::ffi::OsStr;
///
/// assert!(resolve(None, None, true));
/// assert!(!resolve(None, None, false)); // piped
/// assert!(!resolve(None, Some(OsStr::new("1")), true));
/// assert!(resolve(Some(true), Some(OsStr::new("1")), false));
/// ```
pub fn resolve(force: Option<bool>, no_color: Option<&OsStr>, is_terminal: bool) -> bool {
    force.unwrap_or_else(|| is_terminal && no_color.is_none_or(OsStr::is_empty))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Fails if a branch of `resolve` ignores the override, the terminal
    /// check or the empty-`NO_COLOR` rule.
    #[test]
    fn resolve_covers_every_choice() {
        let set = Some(OsStr::new("1"));
        let empty = Some(OsStr::new(""));
        for tty in [true, false] {
            for env in [None, set, empty] {
                assert!(resolve(Some(true), env, tty), "always {env:?} {tty}");
                assert!(!resolve(Some(false), env, tty), "never {env:?} {tty}");
            }
        }
        assert!(resolve(None, None, true));
        assert!(resolve(None, empty, true), "empty NO_COLOR is ignored");
        assert!(!resolve(None, set, true), "NO_COLOR turns auto off");
        assert!(!resolve(None, None, false), "no terminal, no color");
    }
}
