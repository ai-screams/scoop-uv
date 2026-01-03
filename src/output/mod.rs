//! Output utilities

mod spinner;

pub use spinner::Spinner;

use owo_colors::OwoColorize;

/// Output handler for CLI
pub struct Output {
    /// Verbosity level (0 = normal, 1+ = verbose)
    verbose: u8,
    /// Suppress all output
    quiet: bool,
    /// Disable colors
    no_color: bool,
    /// Output as JSON
    json: bool,
}

impl Output {
    /// Create a new output handler
    pub fn new(verbose: u8, quiet: bool, no_color: bool, json: bool) -> Self {
        // Also check NO_COLOR environment variable
        let no_color = no_color || std::env::var("NO_COLOR").is_ok();

        Self {
            verbose,
            quiet,
            no_color,
            json,
        }
    }

    /// Print a success message
    pub fn success(&self, msg: &str) {
        if self.quiet || self.json {
            return;
        }

        if self.no_color {
            eprintln!("✓ {msg}");
        } else {
            eprintln!("{} {msg}", "✓".green());
        }
    }

    /// Print an error message
    pub fn error(&self, msg: &str) {
        if self.json {
            return;
        }

        if self.no_color {
            eprintln!("✗ {msg}");
        } else {
            eprintln!("{} {msg}", "✗".red());
        }
    }

    /// Print an info message
    pub fn info(&self, msg: &str) {
        if self.quiet || self.json {
            return;
        }

        if self.no_color {
            eprintln!("• {msg}");
        } else {
            eprintln!("{} {msg}", "•".blue());
        }
    }

    /// Print a warning message
    pub fn warn(&self, msg: &str) {
        if self.quiet || self.json {
            return;
        }

        if self.no_color {
            eprintln!("⚠ {msg}");
        } else {
            eprintln!("{} {msg}", "⚠".yellow());
        }
    }

    /// Print a debug message (only if verbose)
    pub fn debug(&self, msg: &str) {
        if self.quiet || self.json || self.verbose == 0 {
            return;
        }

        if self.no_color {
            eprintln!("  {msg}");
        } else {
            eprintln!("  {}", msg.dimmed());
        }
    }

    /// Print a line to stdout (for list output)
    pub fn println(&self, msg: &str) {
        if self.quiet {
            return;
        }
        println!("{msg}");
    }

    /// Check if JSON output is enabled
    pub fn is_json(&self) -> bool {
        self.json
    }

    /// Check if quiet mode is enabled
    pub fn is_quiet(&self) -> bool {
        self.quiet
    }

    /// Get verbosity level
    pub fn verbosity(&self) -> u8 {
        self.verbose
    }
}

impl Default for Output {
    fn default() -> Self {
        Self::new(0, false, false, false)
    }
}
