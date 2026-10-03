//! The `--color` flag: its values and the early scan clap needs (#205).
//!
//! The decision itself (terminal, `NO_COLOR`) lives in
//! [`crate::output::color`], which knows nothing about clap.

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

impl ColorChoice {
    /// The explicit override, if any: `Some(true)` for `always`,
    /// `Some(false)` for `never`, `None` for `auto`.
    ///
    /// # Examples
    ///
    /// ```
    /// use scoop_uv::cli::color::ColorChoice;
    ///
    /// assert_eq!(ColorChoice::Auto.force(), None);
    /// assert_eq!(ColorChoice::Never.force(), Some(false));
    /// ```
    pub fn force(self) -> Option<bool> {
        match self {
            Self::Auto => None,
            Self::Always => Some(true),
            Self::Never => Some(false),
        }
    }
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
/// use scoop_uv::cli::color::{choice_from_args, ColorChoice};
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

    /// Fails if a variant maps to the wrong override or clap choice.
    #[test]
    fn color_choice_maps_to_force_and_clap() {
        assert_eq!(ColorChoice::Auto.force(), None);
        assert_eq!(ColorChoice::Always.force(), Some(true));
        assert_eq!(ColorChoice::Never.force(), Some(false));
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
