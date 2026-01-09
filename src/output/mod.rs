//! Output utilities

mod json;
mod spinner;

pub use json::*;
pub use spinner::Spinner;

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

    /// Check if colors should be used
    pub fn use_color(&self) -> bool {
        !self.no_color
    }
}

// ============================================================================
// JSON Output Helpers
// ============================================================================

use crate::error::ScoopError;
use serde::Serialize;

impl Output {
    /// Print a JSON success response to stdout
    pub fn json_success<T: Serialize>(&self, command: &'static str, data: T) {
        if !self.json {
            return;
        }
        let response = JsonResponse::success(command, data);
        println!(
            "{}",
            serde_json::to_string_pretty(&response).unwrap_or_default()
        );
    }

    /// Print a JSON error response to stderr
    pub fn json_error(&self, command: &'static str, error: &ScoopError) {
        if !self.json {
            return;
        }
        let mut response = JsonErrorResponse::error(command, error.code(), error.to_string());
        if let Some(suggestion) = error.suggestion() {
            response = response.with_suggestion(suggestion);
        }
        eprintln!(
            "{}",
            serde_json::to_string_pretty(&response).unwrap_or_default()
        );
    }
}

impl Default for Output {
    fn default() -> Self {
        Self::new(0, false, false, false)
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
        eprintln!("Checking scoop installation...");
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
        if self.no_color {
            eprintln!("{} {}", icon, message);
        } else {
            eprintln!("{} {}", color_fn(icon), message);
        }

        // Print details in verbose mode
        if self.verbose > 0 {
            if let Some(details) = &result.details {
                if self.no_color {
                    eprintln!("  {}", details);
                } else {
                    eprintln!("  {}", details.dimmed());
                }
            }
        }

        // Print suggestion for errors/warnings
        if let Some(suggestion) = &result.suggestion {
            if self.no_color {
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
            if self.no_color {
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
            if self.no_color {
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
}
