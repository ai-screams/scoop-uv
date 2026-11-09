//! Output utilities

pub mod color;
mod json;
mod time;

pub use color::Colors;
pub use json::*;
pub use time::{format_age, format_last_used_value};

use owo_colors::OwoColorize;

use crate::core::doctor::{CheckResult, CheckStatus};

// ============================================================================
// Size Formatting
// ============================================================================

/// Format bytes to human-readable string
///
/// # Examples
///
/// ```
/// use scoop_uv::output::format_size;
/// assert_eq!(format_size(0), "0 B");
/// assert_eq!(format_size(1024), "1 KB");
/// assert_eq!(format_size(1_048_576), "1 MB");
/// assert_eq!(format_size(1_073_741_824), "1.0 GB");
/// ```
pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.0} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.0} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Set once by [`allow_diagnostics`]; read by [`diagnostics_allowed`].
static DIAGNOSTICS: std::sync::OnceLock<bool> = std::sync::OnceLock::new();

/// Records whether layers below the commands (such as uv's own warnings
/// passed on by the uv client) may print diagnostics to stderr.
///
/// The rule is the one [`Output::warn`] follows: nothing under `--quiet` or
/// `--json`. Only the first call counts; the binary makes it once, before
/// any command runs. Without a call nothing is printed, so a library caller
/// gets no stray stderr.
///
/// # Examples
///
/// ```
/// scoop_uv::output::allow_diagnostics(false, false);
/// ```
pub fn allow_diagnostics(quiet: bool, json: bool) {
    let _ = DIAGNOSTICS.set(diagnostics_policy(quiet, json));
}

fn diagnostics_policy(quiet: bool, json: bool) -> bool {
    !quiet && !json
}

/// Returns whether [`allow_diagnostics`] said diagnostics may print.
pub(crate) fn diagnostics_allowed() -> bool {
    DIAGNOSTICS.get().copied().unwrap_or(false)
}

/// Output handler for CLI
pub struct Output {
    /// Verbosity level (0 = normal, 1+ = verbose)
    verbose: u8,
    /// Suppress all output
    quiet: bool,
    /// Which streams get color, decided once at startup (see [`color`])
    colors: Colors,
    /// Output as JSON
    json: bool,
    /// Set once a JSON document has gone to stdout, so a failure afterwards
    /// does not add a second one.
    json_printed: std::sync::atomic::AtomicBool,
}

impl Output {
    /// Create a new output handler.
    ///
    /// `colors` is already decided (the binary calls [`Colors::detect`] once
    /// at startup); `Output` reads no environment itself.
    pub fn new(verbose: u8, quiet: bool, colors: Colors, json: bool) -> Self {
        Self {
            verbose,
            quiet,
            colors,
            json,
            json_printed: std::sync::atomic::AtomicBool::new(false),
        }
    }

    /// Print a success message
    pub fn success(&self, msg: &str) {
        if self.quiet || self.json {
            return;
        }

        if !self.colors.stderr {
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

        if !self.colors.stderr {
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

        if !self.colors.stderr {
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

        if !self.colors.stderr {
            eprintln!("⚠ {msg}");
        } else {
            eprintln!("{} {msg}", "⚠".yellow());
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

    /// Check if colors should be used on stderr (messages, progress bars)
    pub fn use_color_stderr(&self) -> bool {
        self.colors.stderr
    }

    /// Check if colors should be used on stdout (`list`'s highlight)
    pub fn use_color_stdout(&self) -> bool {
        self.colors.stdout
    }
}

// ============================================================================
// JSON Output Helpers
// ============================================================================

use serde::Serialize;

impl Output {
    /// Print a JSON success response to stdout
    pub fn json_success<T: Serialize>(&self, command: &'static str, data: T) {
        if !self.json {
            return;
        }
        let response = JsonResponse::success(command, data);
        self.print_json(&serde_json::to_string_pretty(&response).unwrap_or_default());
    }

    /// Print a JSON document a command built itself to stdout, and remember
    /// that stdout now holds one.
    pub fn print_json(&self, document: &str) {
        println!("{document}");
        self.json_printed
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }

    /// Whether a JSON document has already gone to stdout this run.
    pub fn json_printed(&self) -> bool {
        self.json_printed.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Print a failed command as a JSON error envelope on stdout (only in
    /// JSON mode): its stable code, the translated message and the
    /// suggestion, if any.
    pub fn json_error(&self, command: &'static str, err: &crate::error::ScoopError) {
        if !self.json {
            return;
        }
        let mut response = JsonErrorResponse::error(command, err.code(), err.to_string());
        if let Some(suggestion) = err.suggestion() {
            response = response.with_suggestion(suggestion);
        }
        self.print_json(&serde_json::to_string_pretty(&response).unwrap_or_default());
    }
}

impl Default for Output {
    /// Plain output: with nothing decided, no color is the safe default.
    fn default() -> Self {
        Self::new(0, false, Colors::NONE, false)
    }
}

// ============================================================================
// Doctor Report Output
// ============================================================================

impl Output {
    /// Print doctor report header.
    pub fn doctor_header(&self) {
        if self.quiet || self.json {
            return;
        }
        eprintln!();
        eprintln!("Checking installation...");
        eprintln!();
    }

    /// Print a single check result.
    pub fn doctor_check(&self, result: &CheckResult) {
        if self.json {
            return;
        }

        // Skip OK results in quiet mode
        if self.quiet && result.is_ok() {
            return;
        }

        let (icon, color_fn): (&str, fn(&str) -> String) = match &result.status {
            CheckStatus::Ok => ("✓", |s| s.green().to_string()),
            CheckStatus::Warning(_) => ("⚠", |s| s.yellow().to_string()),
            CheckStatus::Error(_) => ("✗", |s| s.red().to_string()),
        };

        // Build message
        let message = match &result.status {
            CheckStatus::Ok => result.name.to_string(),
            CheckStatus::Warning(msg) => format!("{}: {}", result.name, msg),
            CheckStatus::Error(msg) => format!("{}: {}", result.name, msg),
        };

        // Print with or without color
        if !self.colors.stderr {
            eprintln!("{} {}", icon, message);
        } else {
            eprintln!("{} {}", color_fn(icon), message);
        }

        // Print details in verbose mode
        if self.verbose > 0
            && let Some(details) = &result.details
        {
            if !self.colors.stderr {
                eprintln!("  {}", details);
            } else {
                eprintln!("  {}", details.dimmed());
            }
        }

        // Print suggestion for errors/warnings
        if let Some(suggestion) = &result.suggestion {
            if !self.colors.stderr {
                eprintln!("  → {}", suggestion);
            } else {
                eprintln!("  {} {}", "→".cyan(), suggestion);
            }
        }
    }

    /// Print doctor report summary.
    pub fn doctor_summary(&self, results: &[CheckResult]) {
        if self.json {
            return;
        }

        let errors = results.iter().filter(|r| r.is_error()).count();
        let warnings = results.iter().filter(|r| r.is_warning()).count();

        eprintln!();
        eprintln!("──────────────────────────────────");

        if errors == 0 && warnings == 0 {
            if !self.colors.stderr {
                eprintln!("All checks passed!");
            } else {
                eprintln!("{}", "All checks passed!".green());
            }
        } else {
            let mut parts = Vec::new();
            if errors > 0 {
                parts.push(format!("{} error(s)", errors));
            }
            if warnings > 0 {
                parts.push(format!("{} warning(s)", warnings));
            }

            let summary = format!("Found {}.", parts.join(" and "));
            if !self.colors.stderr {
                eprintln!("{}", summary);
            } else {
                eprintln!("{}", summary.yellow());
            }
        }
    }

    /// Print doctor report as JSON.
    pub fn doctor_json(&self, results: &[CheckResult]) {
        if !self.json {
            return;
        }

        let json_results: Vec<serde_json::Value> = results
            .iter()
            .map(|r| {
                let status = match &r.status {
                    CheckStatus::Ok => "ok",
                    CheckStatus::Warning(_) => "warning",
                    CheckStatus::Error(_) => "error",
                };

                let message = match &r.status {
                    CheckStatus::Ok => None,
                    CheckStatus::Warning(msg) => Some(msg.clone()),
                    CheckStatus::Error(msg) => Some(msg.clone()),
                };

                serde_json::json!({
                    "id": r.id,
                    "name": r.name,
                    "status": status,
                    "message": message,
                    "suggestion": r.suggestion,
                    "details": r.details,
                })
            })
            .collect();

        let errors = results.iter().filter(|r| r.is_error()).count();
        let warnings = results.iter().filter(|r| r.is_warning()).count();
        let ok = results.iter().filter(|r| r.is_ok()).count();

        let output = serde_json::json!({
            "version": env!("CARGO_PKG_VERSION"),
            "summary": {
                "total": results.len(),
                "ok": ok,
                "warnings": warnings,
                "errors": errors,
            },
            "checks": json_results,
        });

        println!(
            "{}",
            serde_json::to_string_pretty(&output).unwrap_or_default()
        );
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Lib tests never call allow_diagnostics, so this is the unset default.
    #[test]
    fn diagnostics_stay_off_until_allowed() {
        assert!(!diagnostics_allowed());
    }

    // Same rule as Output::warn: no diagnostics under --quiet or --json.
    #[rstest::rstest]
    #[case::normal(false, false, true)]
    #[case::quiet(true, false, false)]
    #[case::json(false, true, false)]
    #[case::both(true, true, false)]
    fn diagnostics_follow_the_warn_rule(
        #[case] quiet: bool,
        #[case] json: bool,
        #[case] allowed: bool,
    ) {
        assert_eq!(diagnostics_policy(quiet, json), allowed);
    }

    mod format_size_tests {
        use super::*;

        #[test]
        fn test_bytes() {
            assert_eq!(format_size(0), "0 B");
            assert_eq!(format_size(512), "512 B");
            assert_eq!(format_size(1023), "1023 B");
        }

        #[test]
        fn test_kilobytes() {
            assert_eq!(format_size(1024), "1 KB");
            assert_eq!(format_size(2048), "2 KB");
            assert_eq!(format_size(10240), "10 KB");
        }

        #[test]
        fn test_megabytes() {
            assert_eq!(format_size(1_048_576), "1 MB");
            assert_eq!(format_size(10_485_760), "10 MB");
        }

        #[test]
        fn test_gigabytes() {
            assert_eq!(format_size(1_073_741_824), "1.0 GB");
            assert_eq!(format_size(2_147_483_648), "2.0 GB");
        }

        #[test]
        fn test_boundary_values() {
            // KB boundary
            assert_eq!(format_size(1023), "1023 B");
            assert_eq!(format_size(1024), "1 KB");
            // MB boundary
            assert_eq!(format_size(1_048_575), "1024 KB");
            assert_eq!(format_size(1_048_576), "1 MB");
            // GB boundary
            assert_eq!(format_size(1_073_741_823), "1024 MB");
            assert_eq!(format_size(1_073_741_824), "1.0 GB");
        }
    }

    mod output_flag_tests {
        use super::*;

        #[test]
        fn is_json_returns_correct_value() {
            let json_output = Output::new(0, false, Colors::ALL, true);
            let normal_output = Output::new(0, false, Colors::ALL, false);

            assert!(json_output.is_json());
            assert!(!normal_output.is_json());
        }

        #[test]
        fn is_quiet_returns_correct_value() {
            let quiet_output = Output::new(0, true, Colors::ALL, false);
            let normal_output = Output::new(0, false, Colors::ALL, false);

            assert!(quiet_output.is_quiet());
            assert!(!normal_output.is_quiet());
        }

        /// stdout and stderr are decided separately (`scuv list > file` on a
        /// terminal colors stderr only). Fails if either getter reads the
        /// other stream's flag.
        #[test]
        fn colors_keep_streams_apart() {
            let only_stderr = Colors {
                stdout: false,
                stderr: true,
            };
            let output = Output::new(0, false, only_stderr, false);
            assert!(output.use_color_stderr());
            assert!(!output.use_color_stdout());

            let only_stdout = Colors {
                stdout: true,
                stderr: false,
            };
            let output = Output::new(0, false, only_stdout, false);
            assert!(!output.use_color_stderr());
            assert!(output.use_color_stdout());
        }

        #[test]
        fn default_output_has_expected_flags() {
            let output = Output::default();

            assert!(!output.is_json());
            assert!(!output.is_quiet());
            // Nothing decided means no color. Fails if Default goes back to
            // coloring (it once colored even through a pipe).
            assert!(!output.use_color_stderr());
            assert!(!output.use_color_stdout());
        }

        /// Boundary value: maximum verbosity level
        #[test]
        fn output_handles_max_verbosity() {
            let output = Output::new(u8::MAX, false, Colors::ALL, false);

            // Should not panic, and verbosity should be preserved
            assert_eq!(output.verbosity(), u8::MAX);
        }

        /// Boundary value: all flags enabled simultaneously
        #[test]
        fn output_handles_all_flags_enabled() {
            // quiet, no color and json all at once - potentially conflicting
            let output = Output::new(0, true, Colors::NONE, true);

            // All flags should be set as specified
            assert!(output.is_quiet());
            assert!(!output.use_color_stderr());
            assert!(output.is_json());
        }

        /// Verbosity levels affect behavior correctly
        #[test]
        fn output_verbosity_levels() {
            let v0 = Output::new(0, false, Colors::ALL, false);
            let v1 = Output::new(1, false, Colors::ALL, false);
            let v2 = Output::new(2, false, Colors::ALL, false);

            assert_eq!(v0.verbosity(), 0);
            assert_eq!(v1.verbosity(), 1);
            assert_eq!(v2.verbosity(), 2);

            // Verify ordering
            assert!(v0.verbosity() < v1.verbosity());
            assert!(v1.verbosity() < v2.verbosity());
        }
    }
}
