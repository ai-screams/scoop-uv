//! Doctor diagnostic module for scoop installation health checks.
//!
//! This module provides functionality to diagnose the scoop installation
//! and report any issues with suggested fixes.

use std::process::Command;

use crate::paths;

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
/// of the scoop installation.
pub trait Check: Send + Sync {
    /// Returns the check identifier.
    fn id(&self) -> &'static str;

    /// Returns the check name for display.
    fn name(&self) -> &'static str;

    /// Runs the check and returns results.
    ///
    /// A single check may return multiple results (e.g., one per virtualenv).
    fn run(&self) -> Vec<CheckResult>;
}

// ============================================================================
// Doctor Engine
// ============================================================================

/// Doctor diagnostic engine.
///
/// Runs all registered checks and collects results.
pub struct Doctor {
    checks: Vec<Box<dyn Check>>,
}

impl Doctor {
    /// Creates a new Doctor with default checks.
    pub fn new() -> Self {
        Self {
            checks: vec![
                Box::new(UvCheck),
                Box::new(HomeCheck),
                Box::new(VirtualenvCheck),
                Box::new(SymlinkCheck),
            ],
        }
    }

    /// Runs all checks and returns results.
    pub fn run_all(&self) -> Vec<CheckResult> {
        self.checks.iter().flat_map(|c| c.run()).collect()
    }
}

impl Default for Doctor {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Individual Checks
// ============================================================================

/// Check for uv installation.
struct UvCheck;

impl Check for UvCheck {
    fn id(&self) -> &'static str {
        "uv"
    }

    fn name(&self) -> &'static str {
        "uv installation"
    }

    fn run(&self) -> Vec<CheckResult> {
        match Command::new("uv").arg("--version").output() {
            Ok(output) if output.status.success() => {
                let version = String::from_utf8_lossy(&output.stdout);
                let version = version.trim();
                vec![CheckResult::ok(self.id(), self.name()).with_details(version)]
            }
            _ => {
                vec![
                    CheckResult::error(self.id(), self.name(), "uv not found").with_suggestion(
                        "Install uv: curl -LsSf https://astral.sh/uv/install.sh | sh",
                    ),
                ]
            }
        }
    }
}

/// Check for SCOOP_HOME directory.
struct HomeCheck;

impl Check for HomeCheck {
    fn id(&self) -> &'static str {
        "home"
    }

    fn name(&self) -> &'static str {
        "SCOOP_HOME directory"
    }

    fn run(&self) -> Vec<CheckResult> {
        match paths::scoop_home() {
            Ok(path) if path.exists() => {
                // Check write permission
                match path.metadata() {
                    Ok(meta) if !meta.permissions().readonly() => {
                        vec![
                            CheckResult::ok(self.id(), self.name())
                                .with_details(format!("{}", path.display())),
                        ]
                    }
                    _ => {
                        vec![
                            CheckResult::error(self.id(), self.name(), "directory not writable")
                                .with_suggestion(format!("chmod 755 {}", path.display())),
                        ]
                    }
                }
            }
            Ok(path) => {
                vec![
                    CheckResult::error(self.id(), self.name(), "directory not found")
                        .with_suggestion(format!("mkdir -p {}", path.display())),
                ]
            }
            Err(_) => {
                vec![
                    CheckResult::error(
                        self.id(),
                        self.name(),
                        "could not determine home directory",
                    )
                    .with_suggestion("Set SCOOP_HOME environment variable"),
                ]
            }
        }
    }
}

/// Check for virtualenv integrity.
struct VirtualenvCheck;

impl Check for VirtualenvCheck {
    fn id(&self) -> &'static str {
        "venv"
    }

    fn name(&self) -> &'static str {
        "virtual environments"
    }

    fn run(&self) -> Vec<CheckResult> {
        let venvs_dir = match paths::virtualenvs_dir() {
            Ok(dir) => dir,
            Err(_) => {
                return vec![CheckResult::error(
                    self.id(),
                    self.name(),
                    "virtualenvs directory not found",
                )];
            }
        };

        if !venvs_dir.exists() {
            return vec![
                CheckResult::ok(self.id(), self.name()).with_details("no environments yet"),
            ];
        }

        let mut results = Vec::new();
        let mut healthy = 0;
        let mut broken_names = Vec::new();

        if let Ok(entries) = std::fs::read_dir(&venvs_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let name = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown")
                        .to_string();

                    let python_path = path.join("bin").join("python");
                    let pyvenv_cfg = path.join("pyvenv.cfg");

                    if python_path.exists() && pyvenv_cfg.exists() {
                        healthy += 1;
                    } else {
                        broken_names.push(name);
                    }
                }
            }
        }

        // Report broken environments
        for name in &broken_names {
            results.push(
                CheckResult::error(
                    "venv",
                    "broken virtualenv",
                    format!("'{}' is corrupted", name),
                )
                .with_suggestion(format!(
                    "scoop remove {} && scoop create {} <python-version>",
                    name, name
                )),
            );
        }

        // Summary
        if broken_names.is_empty() {
            if healthy > 0 {
                results.push(
                    CheckResult::ok(self.id(), self.name())
                        .with_details(format!("{} environments, all healthy", healthy)),
                );
            } else {
                results.push(
                    CheckResult::ok(self.id(), self.name()).with_details("no environments yet"),
                );
            }
        }

        results
    }
}

/// Check for symbolic link validity.
struct SymlinkCheck;

impl Check for SymlinkCheck {
    fn id(&self) -> &'static str {
        "symlink"
    }

    fn name(&self) -> &'static str {
        "symbolic links"
    }

    fn run(&self) -> Vec<CheckResult> {
        let venvs_dir = match paths::virtualenvs_dir() {
            Ok(dir) if dir.exists() => dir,
            _ => return vec![],
        };

        let mut results = Vec::new();
        let mut valid = 0;
        let mut broken_names = Vec::new();

        if let Ok(entries) = std::fs::read_dir(&venvs_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let python_path = path.join("bin").join("python");

                    if python_path.is_symlink() {
                        match std::fs::read_link(&python_path) {
                            Ok(target) if target.exists() => {
                                valid += 1;
                            }
                            Ok(_) => {
                                let name = path
                                    .file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or("unknown")
                                    .to_string();
                                broken_names.push(name);
                            }
                            Err(_) => {
                                // Not a symlink or error reading
                            }
                        }
                    }
                }
            }
        }

        // Report broken symlinks
        for name in &broken_names {
            results.push(
                CheckResult::error(
                    "symlink",
                    "broken symlink",
                    format!("Python symlink in '{}' is broken", name),
                )
                .with_suggestion(format!(
                    "scoop remove {} && scoop create {} <python-version>",
                    name, name
                )),
            );
        }

        // Summary
        if broken_names.is_empty() && valid > 0 {
            results.push(
                CheckResult::ok(self.id(), self.name())
                    .with_details(format!("{} symlinks valid", valid)),
            );
        }

        results
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_result_ok() {
        let result = CheckResult::ok("test", "Test Check");
        assert_eq!(result.id, "test");
        assert_eq!(result.name, "Test Check");
        assert!(result.is_ok());
        assert!(!result.is_warning());
        assert!(!result.is_error());
    }

    #[test]
    fn test_check_result_warn() {
        let result = CheckResult::warn("test", "Test Check", "warning message");
        assert!(result.is_warning());
        assert!(!result.is_ok());
        assert!(!result.is_error());
    }

    #[test]
    fn test_check_result_error() {
        let result = CheckResult::error("test", "Test Check", "error message");
        assert!(result.is_error());
        assert!(!result.is_ok());
        assert!(!result.is_warning());
    }

    #[test]
    fn test_check_result_with_suggestion() {
        let result =
            CheckResult::error("test", "Test Check", "error").with_suggestion("fix it like this");
        assert!(result.suggestion.is_some());
        assert_eq!(result.suggestion.unwrap(), "fix it like this");
    }

    #[test]
    fn test_check_result_with_details() {
        let result = CheckResult::ok("test", "Test Check").with_details("version 1.0.0");
        assert!(result.details.is_some());
        assert_eq!(result.details.unwrap(), "version 1.0.0");
    }

    #[test]
    fn test_check_result_builder_chain() {
        let result = CheckResult::warn("test", "Test Check", "warning")
            .with_suggestion("do this")
            .with_details("more info");

        assert!(result.is_warning());
        assert!(result.suggestion.is_some());
        assert!(result.details.is_some());
    }

    #[test]
    fn test_doctor_has_default_checks() {
        let doctor = Doctor::new();
        assert!(!doctor.checks.is_empty());
    }

    #[test]
    fn test_check_status_equality() {
        assert_eq!(CheckStatus::Ok, CheckStatus::Ok);
        assert_eq!(
            CheckStatus::Warning("a".to_string()),
            CheckStatus::Warning("a".to_string())
        );
        assert_ne!(
            CheckStatus::Warning("a".to_string()),
            CheckStatus::Warning("b".to_string())
        );
    }
}
