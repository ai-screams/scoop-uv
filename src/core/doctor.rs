//! Doctor diagnostic module for scoop installation health checks.
//!
//! This module provides functionality to diagnose the scoop installation
//! and report any issues with suggested fixes.

use std::process::Command;

use crate::core::metadata::Metadata;
use crate::paths;
use crate::uv::UvClient;
use crate::uv::version as uv_version;

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
                Box::new(ShellCheck),
                Box::new(VersionCheck),
            ],
        }
    }

    /// Runs all checks and returns results.
    pub fn run_all(&self) -> Vec<CheckResult> {
        self.checks.iter().flat_map(|c| c.run()).collect()
    }

    /// Runs all checks and attempts to fix issues where possible.
    ///
    /// Returns the results after attempting fixes.
    pub fn run_and_fix(&self, output: &crate::output::Output) -> Vec<CheckResult> {
        let mut all_results = Vec::new();

        for check in &self.checks {
            let results = check.run();

            for result in results {
                // Attempt auto-fix for specific error types
                if result.is_error() {
                    if let Some(fixed_result) = self.try_fix(&result, output) {
                        output.doctor_check(&fixed_result);
                        all_results.push(fixed_result);
                        continue;
                    }
                }

                output.doctor_check(&result);
                all_results.push(result);
            }
        }

        all_results
    }

    /// Attempts to fix a specific issue.
    ///
    /// Returns Some(new_result) if fix was attempted, None if not fixable.
    fn try_fix(&self, result: &CheckResult, output: &crate::output::Output) -> Option<CheckResult> {
        match result.id {
            "home" => self.fix_home(result, output),
            "symlink" => self.fix_symlink(result, output),
            _ => None,
        }
    }

    /// Fix SCOOP_HOME directory issues.
    fn fix_home(
        &self,
        result: &CheckResult,
        output: &crate::output::Output,
    ) -> Option<CheckResult> {
        // Only fix "directory not found" errors
        if let CheckStatus::Error(msg) = &result.status {
            if msg.contains("not found") {
                // Create the directory
                if let Ok(home) = paths::scoop_home() {
                    output.info(&format!("Creating {}...", home.display()));

                    match std::fs::create_dir_all(&home) {
                        Ok(_) => {
                            // Also create virtualenvs subdirectory
                            let _ = std::fs::create_dir_all(home.join("virtualenvs"));

                            return Some(
                                CheckResult::ok("home", "SCOOP_HOME directory")
                                    .with_details(format!("created {}", home.display())),
                            );
                        }
                        Err(e) => {
                            return Some(
                                CheckResult::error(
                                    "home",
                                    "SCOOP_HOME directory",
                                    format!("failed to create: {}", e),
                                )
                                .with_suggestion("Check permissions"),
                            );
                        }
                    }
                }
            }
        }
        None
    }

    /// Fix broken Python symlink issues.
    fn fix_symlink(
        &self,
        result: &CheckResult,
        output: &crate::output::Output,
    ) -> Option<CheckResult> {
        // Extract environment name from error message: "Python symlink in 'name' is broken"
        let venv_name = if let CheckStatus::Error(msg) = &result.status {
            // Parse: "Python symlink in 'name' is broken"
            msg.split('\'').nth(1).map(|s| s.to_string())
        } else {
            None
        }?;

        output.info(&format!("Attempting to fix symlink for '{}'...", venv_name));

        // Get virtualenv path
        let venvs_dir = paths::virtualenvs_dir().ok()?;
        let venv_path = venvs_dir.join(&venv_name);

        if !venv_path.exists() {
            return Some(
                CheckResult::error(
                    "symlink",
                    "broken symlink",
                    format!("environment '{}' not found", venv_name),
                )
                .with_suggestion(format!("scoop create {} <python-version>", venv_name)),
            );
        }

        // Read metadata to get Python version
        let metadata_path = venv_path.join(Metadata::FILE_NAME);
        let python_version = if metadata_path.exists() {
            match std::fs::read_to_string(&metadata_path) {
                Ok(content) => match serde_json::from_str::<Metadata>(&content) {
                    Ok(meta) => Some(meta.python_version),
                    Err(_) => None,
                },
                Err(_) => None,
            }
        } else {
            // Try to extract from pyvenv.cfg as fallback
            let pyvenv_cfg = venv_path.join("pyvenv.cfg");
            if pyvenv_cfg.exists() {
                std::fs::read_to_string(&pyvenv_cfg)
                    .ok()
                    .and_then(|content| {
                        for line in content.lines() {
                            if line.starts_with("version") {
                                return line.split('=').nth(1).map(|v| v.trim().to_string());
                            }
                        }
                        None
                    })
            } else {
                None
            }
        };

        let python_version = match python_version {
            Some(v) => v,
            None => {
                return Some(
                    CheckResult::error(
                        "symlink",
                        "broken symlink",
                        format!("could not determine Python version for '{}'", venv_name),
                    )
                    .with_suggestion(format!(
                        "scoop remove {} && scoop create {} <python-version>",
                        venv_name, venv_name
                    )),
                );
            }
        };

        output.info(&format!("Found Python version: {}", python_version));

        // Find Python binary using uv
        let uv = match UvClient::new() {
            Ok(uv) => uv,
            Err(_) => {
                return Some(
                    CheckResult::error("symlink", "broken symlink", "uv not available")
                        .with_suggestion("Install uv first"),
                );
            }
        };

        let python_path = match uv.find_python(&python_version) {
            Ok(Some(info)) => match info.path {
                Some(path) => path,
                None => {
                    return Some(
                        CheckResult::error(
                            "symlink",
                            "broken symlink",
                            format!("Python {} path not found", python_version),
                        )
                        .with_suggestion(format!("scoop install {}", python_version)),
                    );
                }
            },
            Ok(None) => {
                return Some(
                    CheckResult::error(
                        "symlink",
                        "broken symlink",
                        format!("Python {} not installed", python_version),
                    )
                    .with_suggestion(format!("scoop install {}", python_version)),
                );
            }
            Err(_) => {
                return Some(
                    CheckResult::error(
                        "symlink",
                        "broken symlink",
                        "failed to find Python installation",
                    )
                    .with_suggestion(format!("scoop install {}", python_version)),
                );
            }
        };

        // Recreate symlink
        let symlink_path = crate::paths::virtualenv_python_exe(&venv_path);

        // Remove old symlink if exists
        if symlink_path.exists() || symlink_path.is_symlink() {
            if let Err(e) = std::fs::remove_file(&symlink_path) {
                return Some(
                    CheckResult::error(
                        "symlink",
                        "broken symlink",
                        format!("failed to remove old symlink: {}", e),
                    )
                    .with_suggestion("Check file permissions"),
                );
            }
        }

        // Create new symlink
        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            if let Err(e) = symlink(&python_path, &symlink_path) {
                return Some(
                    CheckResult::error(
                        "symlink",
                        "broken symlink",
                        format!("failed to create symlink: {}", e),
                    )
                    .with_suggestion("Check file permissions"),
                );
            }
        }

        #[cfg(not(unix))]
        {
            return Some(
                CheckResult::warn(
                    "symlink",
                    "broken symlink",
                    "symlink fix not supported on this platform",
                )
                .with_suggestion("Manually recreate the symlink"),
            );
        }

        output.success(&format!("Fixed symlink for '{}'", venv_name));

        Some(
            CheckResult::ok("symlink", "broken symlink")
                .with_details(format!("fixed symlink for '{}'", venv_name)),
        )
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

impl UvCheck {
    /// Platform-appropriate install or upgrade command for uv.
    fn install_hint() -> &'static str {
        if cfg!(target_os = "macos") {
            "brew install uv  OR  curl -LsSf https://astral.sh/uv/install.sh | sh"
        } else if cfg!(target_os = "windows") {
            "powershell -ExecutionPolicy ByPass -c \"irm https://astral.sh/uv/install.ps1 | iex\""
        } else {
            "curl -LsSf https://astral.sh/uv/install.sh | sh"
        }
    }
}

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
                let raw = String::from_utf8_lossy(&output.stdout);
                let raw = raw.trim();

                // Enforce the minimum supported uv version when parseable.
                // Unparseable output (custom build, unknown format) is treated
                // as a soft pass so we don't block users on a banner change.
                if let Some(version) = uv_version::parse(raw) {
                    if !uv_version::meets_minimum(version) {
                        return vec![
                            CheckResult::error(
                                self.id(),
                                self.name(),
                                format!(
                                    "uv {} is older than the supported minimum ({})",
                                    uv_version::format_version(version),
                                    uv_version::format_version(uv_version::MIN_VERSION),
                                ),
                            )
                            .with_details(raw.to_string())
                            .with_suggestion(format!("Upgrade uv: {}", Self::install_hint())),
                        ];
                    }
                }

                vec![CheckResult::ok(self.id(), self.name()).with_details(raw.to_string())]
            }
            _ => {
                vec![
                    CheckResult::error(self.id(), self.name(), "uv not found in PATH")
                        .with_details("scoop requires uv to manage Python environments")
                        .with_suggestion(format!("Install uv: {}", Self::install_hint())),
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

                    let python_path = crate::paths::virtualenv_python_exe(&path);
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
                    let python_path = crate::paths::virtualenv_python_exe(&path);

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

/// Check for shell configuration (scoop init).
struct ShellCheck;

impl Check for ShellCheck {
    fn id(&self) -> &'static str {
        "shell"
    }

    fn name(&self) -> &'static str {
        "shell configuration"
    }

    fn run(&self) -> Vec<CheckResult> {
        let home = match dirs::home_dir() {
            Some(h) => h,
            None => {
                return vec![CheckResult::error(
                    self.id(),
                    self.name(),
                    "could not determine home directory",
                )];
            }
        };

        // Detect current shell from $SHELL environment variable
        let shell = std::env::var("SHELL").unwrap_or_default();
        let shell_name = std::path::Path::new(&shell)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_lowercase();

        // Determine config files to check based on shell
        let config_files: Vec<(&str, std::path::PathBuf)> = match shell_name.as_str() {
            "zsh" => vec![("zsh", home.join(".zshrc"))],
            "bash" => {
                // macOS uses .bash_profile, Linux uses .bashrc
                if cfg!(target_os = "macos") {
                    vec![
                        ("bash", home.join(".bash_profile")),
                        ("bash", home.join(".bashrc")),
                    ]
                } else {
                    vec![("bash", home.join(".bashrc"))]
                }
            }
            _ => {
                // Unknown shell - check both common configs
                return vec![
                    CheckResult::warn(
                        self.id(),
                        self.name(),
                        format!("unsupported shell: {}", shell_name),
                    )
                    .with_details("Supported shells: bash, zsh")
                    .with_suggestion("Manual setup may be required"),
                ];
            }
        };

        // Check if any config file contains scoop init
        for (_shell_type, config_path) in &config_files {
            if config_path.exists() {
                match std::fs::read_to_string(config_path) {
                    Ok(content) => {
                        // Look for scoop init pattern
                        if content.contains("scoop init") {
                            return vec![
                                CheckResult::ok(self.id(), self.name())
                                    .with_details(format!("found in {}", config_path.display())),
                            ];
                        }
                    }
                    Err(_) => {
                        return vec![CheckResult::warn(
                            self.id(),
                            self.name(),
                            format!("could not read {}", config_path.display()),
                        )];
                    }
                }
            }
        }

        // No scoop init found
        let shell_type = if shell_name == "zsh" { "zsh" } else { "bash" };
        let config_file = if shell_name == "zsh" {
            "~/.zshrc"
        } else if cfg!(target_os = "macos") {
            "~/.bash_profile"
        } else {
            "~/.bashrc"
        };

        vec![
            CheckResult::error(
                self.id(),
                self.name(),
                "scoop init not found in shell config",
            )
            .with_suggestion(format!(
                "Add to {}: eval \"$(scoop init {})\"",
                config_file, shell_type
            )),
        ]
    }
}

/// Classify a non-empty version-file entry into a [`CheckResult`].
///
/// The `system` sentinel (case-insensitive) is the documented value
/// for "use the system Python, no virtualenv active" — it's what
/// `scoop use system` writes — and must NOT be treated as a regular
/// env name to look up. Pre-0.14.1 the doctor's version checks
/// flagged `.scoop-version: system` as a non-existent-env error
/// because the post-self-update doctor pass took it as a regular
/// reference. `use_env::mod.rs:41` uses the same case-insensitive
/// match on the writer side, so we mirror that contract here.
fn classify_version_entry(
    id: &'static str,
    name: &'static str,
    entry: &str,
    venvs_dir: Option<&std::path::Path>,
) -> CheckResult {
    if entry.eq_ignore_ascii_case("system") {
        return CheckResult::ok(id, name).with_details("system Python (no virtualenv)");
    }
    let env_exists = venvs_dir
        .map(|dir| dir.join(entry).exists())
        .unwrap_or(false);
    if env_exists {
        CheckResult::ok(id, name).with_details(format!("set to '{}'", entry))
    } else {
        CheckResult::error(id, name, format!("references non-existent env '{}'", entry))
            .with_suggestion(format!("Run: scoop create {} <python-version>", entry))
    }
}

/// Check for version file validity.
struct VersionCheck;

impl Check for VersionCheck {
    fn id(&self) -> &'static str {
        "version"
    }

    fn name(&self) -> &'static str {
        "version files"
    }

    fn run(&self) -> Vec<CheckResult> {
        let mut results = Vec::new();
        let venvs_dir = paths::virtualenvs_dir().ok();

        // Check global version file
        if let Ok(global_file) = paths::global_version_file() {
            if global_file.exists() {
                match std::fs::read_to_string(&global_file) {
                    Ok(content) => {
                        let env_name = content.trim();
                        if !env_name.is_empty() {
                            results.push(classify_version_entry(
                                "version:global",
                                "global version",
                                env_name,
                                venvs_dir.as_deref(),
                            ));
                        }
                    }
                    Err(_) => {
                        results.push(
                            CheckResult::warn(
                                "version:global",
                                "global version",
                                "could not read global version file",
                            )
                            .with_suggestion(format!("Check file: {}", global_file.display())),
                        );
                    }
                }
            }
        }

        // Check local version file in current directory
        let current_dir = match std::env::current_dir() {
            Ok(dir) => dir,
            Err(_) => return results,
        };
        let local_file = paths::local_version_file(&current_dir);
        if local_file.exists() {
            match std::fs::read_to_string(&local_file) {
                Ok(content) => {
                    let env_name = content.trim();
                    if !env_name.is_empty() {
                        results.push(classify_version_entry(
                            "version:local",
                            "local version",
                            env_name,
                            venvs_dir.as_deref(),
                        ));
                    }
                }
                Err(_) => {
                    results.push(
                        CheckResult::warn(
                            "version:local",
                            "local version",
                            "could not read local version file",
                        )
                        .with_suggestion(format!("Check file: {}", local_file.display())),
                    );
                }
            }
        }

        // If no version files found, that's fine
        if results.is_empty() {
            results.push(
                CheckResult::ok(self.id(), self.name()).with_details("no version files configured"),
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

    // ==========================================================================
    // VirtualenvCheck / SymlinkCheck / fix_symlink coverage
    //
    // These tests pin the run() / fix_symlink() implementations against
    // the cargo-mutants `replace -> vec![]` / `replace -> None`
    // mutations. Without them, deleting the entire function body would
    // still pass the suite, masking real regressions.
    // ==========================================================================

    use crate::test_utils::with_temp_scoop_home;
    use serial_test::serial;

    #[test]
    #[serial]
    fn virtualenv_check_emits_broken_for_corrupted_env() {
        with_temp_scoop_home(|temp| {
            // Build a deliberately broken venv: directory exists but
            // pyvenv.cfg is missing and there's no python binary. The
            // check must surface this as a broken-virtualenv error
            // (not silently return an empty vec or an Ok summary).
            let broken = temp.path().join("virtualenvs").join("brokenenv");
            std::fs::create_dir_all(&broken).unwrap();

            let results = VirtualenvCheck.run();
            assert!(!results.is_empty(), "expected at least one CheckResult");
            assert!(
                results.iter().any(|r| r.is_error()
                    && r.name.contains("broken")
                    && matches!(&r.status, CheckStatus::Error(msg) if msg.contains("brokenenv"))),
                "expected an error CheckResult naming the broken env, got {results:#?}"
            );
        });
    }

    #[test]
    #[serial]
    fn virtualenv_check_returns_ok_for_healthy_env() {
        with_temp_scoop_home(|temp| {
            let env = temp.path().join("virtualenvs").join("healthy");
            std::fs::create_dir_all(env.join("bin")).unwrap();
            std::fs::write(env.join("bin").join("python"), "").unwrap();
            std::fs::write(env.join("pyvenv.cfg"), "").unwrap();

            let results = VirtualenvCheck.run();
            assert!(!results.is_empty(), "expected a non-empty result set");
            assert!(
                results.iter().all(|r| r.is_ok()),
                "all results should be Ok, got {results:#?}"
            );
        });
    }

    /// On Unix we can deterministically create a symlink whose target
    /// doesn't exist, which is exactly the failure SymlinkCheck::run
    /// surfaces. cfg-gated to Unix because the symlink primitive
    /// differs on Windows (and the CI matrix that exercises mutants is
    /// Linux only — same rationale as the existing
    /// test_virtualenv_exists_with_broken_symlink in paths.rs).
    #[cfg(unix)]
    #[test]
    #[serial]
    fn symlink_check_emits_error_for_broken_python_symlink() {
        with_temp_scoop_home(|temp| {
            use std::os::unix::fs::symlink;

            let env = temp.path().join("virtualenvs").join("brokenlink");
            std::fs::create_dir_all(env.join("bin")).unwrap();
            symlink("/nonexistent/python", env.join("bin").join("python")).unwrap();

            let results = SymlinkCheck.run();
            assert!(
                !results.is_empty(),
                "broken symlink env must produce at least one result"
            );
            assert!(
                results.iter().any(|r| r.is_error()),
                "expected at least one error result, got {results:#?}"
            );
        });
    }

    // ==========================================================================
    // VersionCheck: `system` sentinel handling
    //
    // The `system` value in .scoop-version (or the global version file) is
    // a documented sentinel meaning "use the system Python, no virtualenv
    // active" — written by `scoop use system`. Pre-0.14.1 the post-self-update
    // doctor pass flagged it as a non-existent env, which the user
    // reported as a false-positive after `scoop self update` to 0.14.0.
    // ==========================================================================

    #[test]
    fn classify_treats_system_as_valid_sentinel() {
        let result = classify_version_entry("version:test", "test version", "system", None);
        assert!(
            result.is_ok(),
            "system must classify as Ok, got {result:#?}"
        );
        assert!(
            result
                .details
                .as_deref()
                .is_some_and(|d| d.contains("system Python")),
            "details should explain the sentinel, got {:?}",
            result.details
        );
    }

    #[test]
    fn classify_is_case_insensitive_for_system_sentinel() {
        // The writer side (use_env::mod.rs:41) uses eq_ignore_ascii_case;
        // doctor's reader must mirror that contract so `SYSTEM` or `System`
        // round-trip cleanly even though `scoop use system` always writes
        // the lowercase form.
        for variant in ["SYSTEM", "System", "sYsTeM"] {
            let result = classify_version_entry("version:test", "test version", variant, None);
            assert!(
                result.is_ok(),
                "case-variant '{variant}' must classify as Ok"
            );
        }
    }

    #[test]
    fn classify_existing_env_is_ok() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join("myenv")).unwrap();
        let result =
            classify_version_entry("version:test", "test version", "myenv", Some(tmp.path()));
        assert!(result.is_ok());
        assert!(
            result
                .details
                .as_deref()
                .is_some_and(|d| d.contains("myenv"))
        );
    }

    #[test]
    fn classify_missing_env_is_error() {
        let tmp = tempfile::tempdir().unwrap();
        // venvs dir exists but env doesn't — should error with suggestion.
        let result = classify_version_entry(
            "version:test",
            "test version",
            "missingenv",
            Some(tmp.path()),
        );
        assert!(result.is_error());
        assert!(
            result
                .suggestion
                .as_deref()
                .is_some_and(|s| s.contains("scoop create"))
        );
    }

    /// Mirror of the local-version test for the global path. Without
    /// this, deleting the `!` in the `if !env_name.is_empty()` guard
    /// on the global branch (doctor.rs:867) silently dropped every
    /// version:global result and went unkilled by mutation testing.
    #[test]
    #[serial]
    fn version_check_treats_global_system_sentinel_as_ok() {
        with_temp_scoop_home(|temp| {
            // Global version file lives at <SCOOP_HOME>/version (see
            // paths::global_version_file). Materialise it with the
            // sentinel value.
            std::fs::write(temp.path().join("version"), "system").unwrap();
            // virtualenvs dir presence shouldn't matter for the
            // sentinel, but create it so other checks behave normally.
            std::fs::create_dir_all(temp.path().join("virtualenvs")).unwrap();

            let results = VersionCheck.run();
            let global = results
                .iter()
                .find(|r| r.id == "version:global")
                .expect("version:global must run when ~/.scoop/version exists");
            assert!(
                global.is_ok(),
                "version:global must be Ok for system sentinel, got {global:#?}"
            );
        });
    }

    #[test]
    #[serial]
    fn version_check_treats_local_system_sentinel_as_ok() {
        with_temp_scoop_home(|temp| {
            // Materialise a `.scuv-version: system` file in the CWD.
            let cwd_guard = TempDirCwdGuard::new();
            std::fs::write(cwd_guard.path().join(".scuv-version"), "system").unwrap();

            // Ensure virtualenvs dir exists so its absence doesn't muddle the
            // assertion (we only want the system-sentinel branch to fire).
            std::fs::create_dir_all(temp.path().join("virtualenvs")).unwrap();

            let results = VersionCheck.run();
            let local = results
                .iter()
                .find(|r| r.id == "version:local")
                .expect("version:local check should run when .scuv-version exists");
            assert!(
                local.is_ok(),
                "version:local must be Ok for system sentinel, got {local:#?}"
            );
        });
    }

    /// Regression: the local-version read-error suggestion must name the
    /// actual resolved version-file path, not a hardcoded literal (which
    /// would silently go stale across a rename). A directory at the
    /// version-file path makes `.exists()` true but `read_to_string` fail,
    /// forcing the error branch without relying on platform-specific
    /// permission bits.
    #[test]
    #[serial]
    fn version_check_local_read_error_suggestion_names_version_file() {
        with_temp_scoop_home(|temp| {
            let cwd_guard = TempDirCwdGuard::new();
            std::fs::create_dir(cwd_guard.path().join(paths::VERSION_FILE)).unwrap();
            std::fs::create_dir_all(temp.path().join("virtualenvs")).unwrap();

            let results = VersionCheck.run();
            let local = results
                .iter()
                .find(|r| r.id == "version:local")
                .expect("version:local check should run when the version-file path exists");
            assert!(
                local.is_warning(),
                "expected a warning for an unreadable version file, got {local:#?}"
            );
            let suggestion = local.suggestion.as_deref().unwrap_or_default();
            assert!(
                suggestion.contains(paths::VERSION_FILE),
                "suggestion should name the actual version-file path, got: {suggestion}"
            );
        });
    }

    /// RAII guard: chdir into a fresh tempdir for the duration of the
    /// test, then restore the original cwd on drop. The VersionCheck's
    /// local-version branch reads `.scuv-version` from the current
    /// directory, so we have to actually move there — env vars can't
    /// substitute.
    struct TempDirCwdGuard {
        _tmp: tempfile::TempDir,
        new_cwd: std::path::PathBuf,
        original: std::path::PathBuf,
    }

    impl TempDirCwdGuard {
        fn new() -> Self {
            let original = std::env::current_dir().expect("cwd readable");
            let tmp = tempfile::tempdir().unwrap();
            let new_cwd = tmp.path().to_path_buf();
            std::env::set_current_dir(&new_cwd).expect("chdir into tempdir");
            Self {
                _tmp: tmp,
                new_cwd,
                original,
            }
        }
        fn path(&self) -> &std::path::Path {
            &self.new_cwd
        }
    }

    impl Drop for TempDirCwdGuard {
        fn drop(&mut self) {
            let _ = std::env::set_current_dir(&self.original);
        }
    }

    #[test]
    #[serial]
    fn fix_symlink_returns_some_for_parseable_error_name() {
        with_temp_scoop_home(|temp| {
            // fix_symlink parses the env name out of a "Python symlink
            // in 'name' is broken" message. If the parse succeeds and
            // the env doesn't exist, it returns Some(error suggesting
            // scoop create). The cargo-mutants -> None replacement
            // would silently drop that guidance.
            let broken = temp.path().join("virtualenvs");
            std::fs::create_dir_all(&broken).unwrap();

            let probe = CheckResult::error(
                "symlink",
                "broken symlink",
                "Python symlink in 'fix-target' is broken".to_string(),
            );
            let doctor = Doctor::new();
            let output = crate::output::Output::new(0, true, true, false);

            let fixed = doctor.fix_symlink(&probe, &output);
            assert!(fixed.is_some(), "fix_symlink must return Some");
            let r = fixed.unwrap();
            assert!(r.is_error() || r.is_warning());
            // Suggestion text should point the user at `scoop create`.
            assert!(
                r.suggestion
                    .as_deref()
                    .is_some_and(|s| s.contains("scoop create"))
                    || matches!(&r.status, CheckStatus::Error(msg) if msg.contains("fix-target"))
            );
        });
    }
}
