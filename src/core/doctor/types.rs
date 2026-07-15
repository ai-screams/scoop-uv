//! Shared types for the doctor module: check status, check results, and the
//! `Check` trait that individual checks implement.

// ============================================================================
// Types
// ============================================================================

/// Check result status.
#[derive(Debug, Clone, PartialEq)]
pub enum CheckStatus {
    /// Check passed successfully.
    Ok,
    /// Check passed with a warning.
    Warning(String),
    /// Check failed with an error.
    Error(String),
}

/// Result of a single check.
#[derive(Debug)]
pub struct CheckResult {
    /// Check identifier (e.g., "uv", "home", "venv:myenv").
    pub id: &'static str,
    /// Check name for display.
    pub name: &'static str,
    /// Check status.
    pub status: CheckStatus,
    /// Suggested fix (when check fails).
    pub suggestion: Option<String>,
    /// Additional details (shown with --verbose).
    pub details: Option<String>,
}

impl CheckResult {
    /// Creates a successful check result.
    pub fn ok(id: &'static str, name: &'static str) -> Self {
        Self {
            id,
            name,
            status: CheckStatus::Ok,
            suggestion: None,
            details: None,
        }
    }

    /// Creates a warning check result.
    pub fn warn(id: &'static str, name: &'static str, message: impl Into<String>) -> Self {
        Self {
            id,
            name,
            status: CheckStatus::Warning(message.into()),
            suggestion: None,
            details: None,
        }
    }

    /// Creates an error check result.
    pub fn error(id: &'static str, name: &'static str, message: impl Into<String>) -> Self {
        Self {
            id,
            name,
            status: CheckStatus::Error(message.into()),
            suggestion: None,
            details: None,
        }
    }

    /// Adds a suggested fix to the result.
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }

    /// Adds details to the result.
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    /// Returns true if the check passed.
    pub fn is_ok(&self) -> bool {
        matches!(self.status, CheckStatus::Ok)
    }

    /// Returns true if the check has a warning.
    pub fn is_warning(&self) -> bool {
        matches!(self.status, CheckStatus::Warning(_))
    }

    /// Returns true if the check failed.
    pub fn is_error(&self) -> bool {
        matches!(self.status, CheckStatus::Error(_))
    }
}

// ============================================================================
// Check Trait
// ============================================================================

/// Trait for implementing health checks.
///
/// Each check should be independent and focused on a single aspect
/// of the scuv installation.
pub trait Check: Send + Sync {
    /// Returns the check identifier.
    fn id(&self) -> &'static str;

    /// Returns the check name for display.
    fn name(&self) -> &'static str;

    /// Runs the check and returns results.
    ///
    /// A single check may return multiple results (e.g., one per virtualenv).
    fn run(&self) -> Vec<CheckResult>;

    /// Attempts to fix a failing check result.
    ///
    /// Returns `Some(new_result)` if a fix was attempted, `None` if this
    /// check has no automatic fix to offer for the given result.
    fn fix(&self, _result: &CheckResult, _output: &crate::output::Output) -> Option<CheckResult> {
        None
    }
}
