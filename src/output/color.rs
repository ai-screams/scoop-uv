//! Color decision, made once at startup (#205).
//!
//! Every colored stream follows one choice: scuv's own messages (stderr),
//! `list`'s highlight (stdout), clap's help and errors, the `migrate all`
//! progress bar and the panic report.

use std::ffi::OsStr;

/// The `--color` choice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, clap::ValueEnum)]
pub enum ColorChoice {
    /// Color when the stream is a terminal and `NO_COLOR` is unset or empty
    #[default]
    Auto,
    /// Always color, even through a pipe or with `NO_COLOR` set
    Always,
    /// Never color
    Never,
}

impl From<ColorChoice> for clap::ColorChoice {
    fn from(choice: ColorChoice) -> Self {
        match choice {
            ColorChoice::Auto => Self::Auto,
            ColorChoice::Always => Self::Always,
            ColorChoice::Never => Self::Never,
        }
    }
}

/// Whether each output stream gets color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Colors {
    /// Color on stdout (`list`'s highlight).
    pub stdout: bool,
    /// Color on stderr (messages, progress bar, panic report).
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

    /// Resolves `choice` for this process: reads `NO_COLOR` and whether
    /// stdout and stderr are terminals.
    pub fn detect(choice: ColorChoice) -> Self {
        use std::io::IsTerminal;
        let no_color = std::env::var_os("NO_COLOR");
        Self {
            stdout: resolve(choice, no_color.as_deref(), std::io::stdout().is_terminal()),
            stderr: resolve(choice, no_color.as_deref(), std::io::stderr().is_terminal()),
        }
    }
}

/// Decides color for one stream.
///
/// An explicit `always`/`never` wins over the environment, as no-color.org
/// allows. `auto` wants a terminal and no non-empty `NO_COLOR` (an empty one
/// is ignored, per no-color.org).
///
/// # Examples
///
/// ```
/// use scoop_uv::output::color::{resolve, ColorChoice};
/// use std::ffi::OsStr;
///
/// assert!(resolve(ColorChoice::Auto, None, true));
/// assert!(!resolve(ColorChoice::Auto, None, false)); // piped
/// assert!(!resolve(ColorChoice::Auto, Some(OsStr::new("1")), true));
/// assert!(resolve(ColorChoice::Always, Some(OsStr::new("1")), false));
/// ```
pub fn resolve(choice: ColorChoice, no_color: Option<&OsStr>, is_terminal: bool) -> bool {
    match choice {
        ColorChoice::Always => true,
        ColorChoice::Never => false,
        ColorChoice::Auto => is_terminal && no_color.is_none_or(OsStr::is_empty),
    }
}

/// Finds the color choice on the command line before clap parses it, so
/// clap's own help and errors can follow it.
///
/// The last of `--color <WHEN>`, `--color=<WHEN>` and `--no-color` wins,
/// matching how the parsed flags override each other. Scanning stops at
/// `--`. An unknown value is skipped here and left for clap to reject. This
/// only steers clap's own output; everything else uses the parsed flags.
///
/// # Examples
///
/// ```
/// use scoop_uv::output::color::{choice_from_args, ColorChoice};
///
/// assert_eq!(choice_from_args(["scuv", "list"]), ColorChoice::Auto);
/// assert_eq!(choice_from_args(["scuv", "--color", "never", "list"]), ColorChoice::Never);
/// assert_eq!(choice_from_args(["scuv", "--no-color", "--color=always"]), ColorChoice::Always);
/// assert_eq!(choice_from_args(["scuv", "run", "e", "--", "ls", "--no-color"]), ColorChoice::Auto);
/// ```
pub fn choice_from_args<I, S>(args: I) -> ColorChoice
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut choice = ColorChoice::Auto;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        let arg = arg.as_ref();
        if arg == "--" {
            break;
        }
        let value = if arg == "--no-color" {
            Some(ColorChoice::Never)
        } else if arg == "--color" {
            args.next().and_then(|v| parse(v.as_ref()))
        } else {
            arg.to_str()
                .and_then(|a| a.strip_prefix("--color="))
                .and_then(|v| parse(OsStr::new(v)))
        };
        if let Some(value) = value {
            choice = value;
        }
    }
    choice
}

fn parse(value: &OsStr) -> Option<ColorChoice> {
    <ColorChoice as clap::ValueEnum>::from_str(value.to_str()?, false).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Fails if a branch of `resolve` ignores the choice, the terminal check
    /// or the empty-`NO_COLOR` rule.
    #[test]
    fn resolve_covers_every_choice() {
        let set = Some(OsStr::new("1"));
        let empty = Some(OsStr::new(""));
        for tty in [true, false] {
            for env in [None, set, empty] {
                assert!(
                    resolve(ColorChoice::Always, env, tty),
                    "always {env:?} {tty}"
                );
                assert!(
                    !resolve(ColorChoice::Never, env, tty),
                    "never {env:?} {tty}"
                );
            }
        }
        assert!(resolve(ColorChoice::Auto, None, true));
        assert!(
            resolve(ColorChoice::Auto, empty, true),
            "empty NO_COLOR is ignored"
        );
        assert!(
            !resolve(ColorChoice::Auto, set, true),
            "NO_COLOR turns auto off"
        );
        assert!(
            !resolve(ColorChoice::Auto, None, false),
            "no terminal, no color"
        );
    }

    /// Fails if the scan misses a form, keeps the first instead of the last,
    /// reads past `--`, or lets a bad value reset the choice.
    #[test]
    fn choice_from_args_reads_every_form() {
        use ColorChoice::*;
        let cases: &[(&[&str], ColorChoice)] = &[
            (&["scuv", "list"], Auto),
            (&["scuv", "--no-color", "list"], Never),
            (&["scuv", "list", "--color", "always"], Always),
            (&["scuv", "--color=never", "list"], Never),
            (&["scuv", "--color", "never", "--color=always"], Always),
            (&["scuv", "--color=always", "--no-color"], Never),
            (&["scuv", "--no-color", "--color", "bogus"], Never),
            (&["scuv", "--color"], Auto),
            (&["scuv", "run", "e", "--", "x", "--color=always"], Auto),
        ];
        for (args, want) in cases {
            assert_eq!(choice_from_args(*args), *want, "{args:?}");
        }
    }

    #[test]
    fn color_choice_maps_to_clap() {
        assert_eq!(
            clap::ColorChoice::from(ColorChoice::Auto),
            clap::ColorChoice::Auto
        );
        assert_eq!(
            clap::ColorChoice::from(ColorChoice::Always),
            clap::ColorChoice::Always
        );
        assert_eq!(
            clap::ColorChoice::from(ColorChoice::Never),
            clap::ColorChoice::Never
        );
    }
}
