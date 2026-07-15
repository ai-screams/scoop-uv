//! Doctor diagnostic module for scuv installation health checks.
//!
//! This module provides functionality to diagnose the scuv installation
//! and report any issues with suggested fixes.

mod engine;
mod types;

use std::process::Command;

use crate::core::metadata::Metadata;
use crate::paths;
use crate::uv::UvClient;
use crate::uv::version as uv_version;

pub use engine::Doctor;
pub use types::{Check, CheckResult, CheckStatus};

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
                        .with_details("scuv requires uv to manage Python environments")
                        .with_suggestion(format!("Install uv: {}", Self::install_hint())),
                ]
            }
        }
    }
}

/// Check for SCUV_HOME directory.
struct HomeCheck;

impl Check for HomeCheck {
    fn id(&self) -> &'static str {
        "home"
    }

    fn name(&self) -> &'static str {
        "SCUV_HOME directory"
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
                    .with_suggestion("Set SCUV_HOME environment variable"),
                ]
            }
        }
    }

    fn fix(&self, result: &CheckResult, output: &crate::output::Output) -> Option<CheckResult> {
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
                                CheckResult::ok("home", "SCUV_HOME directory")
                                    .with_details(format!("created {}", home.display())),
                            );
                        }
                        Err(e) => {
                            return Some(
                                CheckResult::error(
                                    "home",
                                    "SCUV_HOME directory",
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
                    "scuv remove {} && scuv create {} <python-version>",
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
                    "scuv remove {} && scuv create {} <python-version>",
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

    fn fix(&self, result: &CheckResult, output: &crate::output::Output) -> Option<CheckResult> {
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
                .with_suggestion(format!("scuv create {} <python-version>", venv_name)),
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
                        "scuv remove {} && scuv create {} <python-version>",
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
                        .with_suggestion(format!("scuv install {}", python_version)),
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
                    .with_suggestion(format!("scuv install {}", python_version)),
                );
            }
            Err(_) => {
                return Some(
                    CheckResult::error(
                        "symlink",
                        "broken symlink",
                        "failed to find Python installation",
                    )
                    .with_suggestion(format!("scuv install {}", python_version)),
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

/// Check for shell configuration (scuv init).
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

        let shell_type = if shell_name == "zsh" { "zsh" } else { "bash" };

        // Check if any config file contains scuv init. A legacy-only
        // `scoop init` line does NOT count as configured: the deprecated
        // `scoop` shell function is defined *inside* `scuv init`'s own
        // output (see shell/bash.rs, shell/zsh.rs), so it only exists once
        // that init line has already run successfully in the session. An rc
        // file that still invokes `eval "$(scoop init ...)"` calls the
        // `scoop` *binary* directly — which no longer ships after upgrade —
        // so the eval fails at shell startup and integration never loads.
        // That must be flagged as a warning, not treated as configured.
        //
        // DEPRECATION(0.16.0): drop the legacy branch once the shim window
        // closes.
        for (_shell_type, config_path) in &config_files {
            if config_path.exists() {
                match std::fs::read_to_string(config_path) {
                    Ok(content) => {
                        if content.contains("scuv init") {
                            return vec![
                                CheckResult::ok(self.id(), self.name())
                                    .with_details(format!("found in {}", config_path.display())),
                            ];
                        }
                        if content.contains("scoop init") {
                            return vec![
                                CheckResult::warn(
                                    self.id(),
                                    self.name(),
                                    "shell config still references the removed `scoop` command (init line fails at startup)",
                                )
                                .with_details(format!("found in {}", config_path.display()))
                                .with_suggestion(format!(
                                    "Replace with: eval \"$(scuv init {})\"",
                                    shell_type
                                )),
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

        // No scuv init (or legacy scoop init) found
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
                "scuv init not found in shell config",
            )
            .with_suggestion(format!(
                "Add to {}: eval \"$(scuv init {})\"",
                config_file, shell_type
            )),
        ]
    }
}

/// Classify a non-empty version-file entry into a [`CheckResult`].
///
/// The `system` sentinel (case-insensitive) is the documented value
/// for "use the system Python, no virtualenv active" — it's what
/// `scuv use system` writes — and must NOT be treated as a regular
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
            .with_suggestion(format!("Run: scuv create {} <python-version>", entry))
    }
}

/// Resolve the local version-file path for `dir` the way `doctor` should see
/// it: the current `.scuv-version` name wins when present, otherwise falls
/// back to the legacy `.scoop-version` name. Mirrors the per-directory
/// precedence in `VersionService::resolve_local_version_file` so `doctor`
/// and `resolve()` never disagree about which file is authoritative during
/// the shim window — but skips that function's `warn_once` side effect,
/// since `doctor` is a read-only diagnostic and the dedicated `legacy` check
/// already surfaces a legacy-only file to the user.
///
/// DEPRECATION(0.16.0): remove the legacy fallback branch.
fn resolve_local_version_file_for_doctor(dir: &std::path::Path) -> std::path::PathBuf {
    let version_file = paths::local_version_file(dir);
    if version_file.exists() {
        version_file
    } else {
        let legacy = dir.join(paths::LEGACY_VERSION_FILE);
        if legacy.exists() {
            legacy
        } else {
            version_file
        }
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
        let local_file = resolve_local_version_file_for_doctor(&current_dir);
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

/// Check for leftover legacy `scoop` state during the `scoop` → `scuv`
/// shim window: env vars, `~/.scoop` without a sibling `~/.scuv`, and
/// legacy-named files in the current directory.
///
/// DEPRECATION(0.16.0): remove this check together with the shims.
fn check_legacy_remnants() -> CheckResult {
    let mut found: Vec<String> = Vec::new();

    for var in [
        paths::LEGACY_HOME_ENV,
        "SCOOP_VERSION",
        "SCOOP_LANG",
        // No runtime warning for this one (it's read per prompt by the shell
        // hook, warning there would spam) — this doctor hint is the only
        // place a user learns to rename it.
        "SCOOP_NO_AUTO",
    ] {
        if std::env::var_os(var).is_some() {
            found.push(format!("${var}"));
        }
    }

    if let Some(home) = dirs::home_dir() {
        if home.join(".scoop").exists() && !home.join(".scuv").exists() {
            found.push("~/.scoop".to_string());
        }
    }

    if let Ok(cwd) = std::env::current_dir() {
        for f in [
            paths::LEGACY_VERSION_FILE,
            crate::core::manifest::LEGACY_MANIFEST_FILE,
        ] {
            if cwd.join(f).exists() {
                found.push(f.to_string());
            }
        }
    }

    if found.is_empty() {
        CheckResult::ok("legacy", "legacy scoop remnants")
    } else {
        CheckResult::warn(
            "legacy",
            "legacy scoop remnants",
            rust_i18n::t!("doctor.legacy_found", items = found.join(", ")),
        )
        .with_suggestion(rust_i18n::t!("doctor.legacy_suggestion"))
    }
}

/// Check for legacy scoop remnants (env vars, dirs, files) left over from
/// the `scoop` → `scuv` rename.
struct LegacyCheck;

impl Check for LegacyCheck {
    fn id(&self) -> &'static str {
        "legacy"
    }

    fn name(&self) -> &'static str {
        "legacy scoop remnants"
    }

    fn run(&self) -> Vec<CheckResult> {
        vec![check_legacy_remnants()]
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
    // The `system` value in .scuv-version (or legacy .scoop-version, or the
    // global version file) is a documented sentinel meaning "use the system
    // Python, no virtualenv active" — written by `scuv use system`.
    // Pre-0.14.1 the post-self-update doctor pass flagged it as a
    // non-existent env, which the user reported as a false-positive after
    // `scoop self update` to 0.14.0.
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
        // round-trip cleanly even though `scuv use system` always writes
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
                .is_some_and(|s| s.contains("scuv create"))
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
            // scuv create). The cargo-mutants -> None replacement
            // would silently drop that guidance.
            let broken = temp.path().join("virtualenvs");
            std::fs::create_dir_all(&broken).unwrap();

            let probe = CheckResult::error(
                "symlink",
                "broken symlink",
                "Python symlink in 'fix-target' is broken".to_string(),
            );
            let output = crate::output::Output::new(0, true, true, false);

            let fixed = SymlinkCheck.fix(&probe, &output);
            assert!(fixed.is_some(), "fix_symlink must return Some");
            let r = fixed.unwrap();
            assert!(r.is_error() || r.is_warning());
            // Suggestion text should point the user at `scuv create`.
            assert!(
                r.suggestion
                    .as_deref()
                    .is_some_and(|s| s.contains("scuv create"))
                    || matches!(&r.status, CheckStatus::Error(msg) if msg.contains("fix-target"))
            );
        });
    }

    // ==========================================================================
    // ShellCheck: scuv init vs. legacy scoop init in rc files
    //
    // A legacy-only `scoop init` line must warn, not pass — the deprecated
    // `scoop` shell function is defined by `scuv init`'s own output (see
    // shell/bash.rs), so it doesn't exist yet when an rc file's
    // `eval "$(scoop init ...)"` line runs at shell startup; that line
    // invokes the (now-removed) `scoop` binary directly and fails.
    // ==========================================================================

    /// Write `content` to both `.bash_profile` and `.bashrc` so the test is
    /// deterministic regardless of which one `ShellCheck` consults on the
    /// host OS (macOS checks `.bash_profile` first, Linux only `.bashrc`).
    fn write_bash_rc(home: &std::path::Path, content: &str) {
        std::fs::write(home.join(".bash_profile"), content).unwrap();
        std::fs::write(home.join(".bashrc"), content).unwrap();
    }

    #[test]
    #[serial]
    fn shell_check_is_ok_when_current_scuv_init_line_present() {
        let home_tmp = tempfile::tempdir().unwrap();
        write_bash_rc(home_tmp.path(), "eval \"$(scuv init bash)\"\n");
        let _g = crate::test_utils::env_guard(&[
            ("SHELL", Some("/bin/bash")),
            ("HOME", Some(home_tmp.path().to_str().unwrap())),
        ]);

        let results = ShellCheck.run();
        assert_eq!(results.len(), 1, "got {results:#?}");
        assert!(results[0].is_ok(), "expected Ok, got {:#?}", results[0]);
    }

    /// Pins the exact branch order in `ShellCheck::run`: a `scuv init` match
    /// must short-circuit before the legacy-only warning branch is even
    /// reached, so an rc file that still has an old, now-inert comment or
    /// leftover `scoop init` reference alongside a working `scuv init` line
    /// is not incorrectly flagged.
    #[test]
    #[serial]
    fn shell_check_ok_when_both_scuv_and_legacy_scoop_init_present() {
        let home_tmp = tempfile::tempdir().unwrap();
        write_bash_rc(
            home_tmp.path(),
            "# old: eval \"$(scoop init bash)\"\neval \"$(scuv init bash)\"\n",
        );
        let _g = crate::test_utils::env_guard(&[
            ("SHELL", Some("/bin/bash")),
            ("HOME", Some(home_tmp.path().to_str().unwrap())),
        ]);

        let results = ShellCheck.run();
        assert_eq!(results.len(), 1, "got {results:#?}");
        assert!(results[0].is_ok(), "expected Ok, got {:#?}", results[0]);
    }

    #[test]
    #[serial]
    fn shell_check_warns_on_legacy_only_scoop_init_line() {
        let home_tmp = tempfile::tempdir().unwrap();
        write_bash_rc(home_tmp.path(), "eval \"$(scoop init bash)\"\n");
        let _g = crate::test_utils::env_guard(&[
            ("SHELL", Some("/bin/bash")),
            ("HOME", Some(home_tmp.path().to_str().unwrap())),
        ]);

        let results = ShellCheck.run();
        assert_eq!(results.len(), 1, "got {results:#?}");
        assert!(
            results[0].is_warning(),
            "expected Warning, got {:#?}",
            results[0]
        );
        let suggestion = results[0].suggestion.as_deref().unwrap_or_default();
        assert!(
            suggestion.contains("scuv init"),
            "suggestion should point at scuv init, got: {suggestion}"
        );
    }

    // ==========================================================================
    // LegacyCheck / check_legacy_remnants: scoop -> scuv shim window
    //
    // These tests override HOME (not just SCUV_HOME) because the legacy
    // check inspects `dirs::home_dir()` directly rather than going through
    // `paths::scoop_home()` — the dev machine's real `~/.scoop` must never
    // leak into the assertions below.
    // ==========================================================================

    #[test]
    #[serial]
    fn legacy_check_is_ok_when_environment_is_clean() {
        let home_tmp = tempfile::tempdir().unwrap();
        let _g = crate::test_utils::env_guard(&[
            (paths::LEGACY_HOME_ENV, None),
            ("SCOOP_VERSION", None),
            ("SCOOP_LANG", None),
            ("SCOOP_NO_AUTO", None),
            (paths::SCUV_HOME_ENV, None),
            ("SCUV_VERSION", None),
            ("SCUV_LANG", None),
            ("HOME", Some(home_tmp.path().to_str().unwrap())),
        ]);
        let _cwd = TempDirCwdGuard::new();

        let result = check_legacy_remnants();
        assert!(
            result.is_ok(),
            "expected Ok on a clean environment, got {result:#?}"
        );
    }

    #[test]
    #[serial]
    fn legacy_check_warns_on_legacy_home_env_var() {
        let home_tmp = tempfile::tempdir().unwrap();
        let _g = crate::test_utils::env_guard(&[
            (
                paths::LEGACY_HOME_ENV,
                Some("/tmp/legacy-home-doesnt-need-to-exist"),
            ),
            ("SCOOP_VERSION", None),
            ("SCOOP_LANG", None),
            ("SCOOP_NO_AUTO", None),
            (paths::SCUV_HOME_ENV, None),
            ("SCUV_VERSION", None),
            ("SCUV_LANG", None),
            ("HOME", Some(home_tmp.path().to_str().unwrap())),
        ]);
        let _cwd = TempDirCwdGuard::new();

        let result = check_legacy_remnants();
        assert!(result.is_warning(), "expected Warning, got {result:#?}");
        let msg = match &result.status {
            CheckStatus::Warning(m) => m.clone(),
            other => panic!("expected Warning, got {other:?}"),
        };
        assert!(
            msg.contains("SCOOP_HOME"),
            "message should name $SCOOP_HOME, got: {msg}"
        );
        assert!(
            result.suggestion.is_some(),
            "a legacy-remnant warning must carry a suggestion"
        );
    }

    #[test]
    #[serial]
    fn legacy_check_warns_on_scoop_version_and_scoop_lang_env_vars() {
        let home_tmp = tempfile::tempdir().unwrap();
        let _g = crate::test_utils::env_guard(&[
            (paths::LEGACY_HOME_ENV, None),
            ("SCOOP_VERSION", Some("myenv")),
            ("SCOOP_LANG", Some("ko")),
            ("SCOOP_NO_AUTO", Some("1")),
            (paths::SCUV_HOME_ENV, None),
            ("SCUV_VERSION", None),
            ("SCUV_LANG", None),
            ("HOME", Some(home_tmp.path().to_str().unwrap())),
        ]);
        let _cwd = TempDirCwdGuard::new();

        let result = check_legacy_remnants();
        assert!(result.is_warning(), "expected Warning, got {result:#?}");
        let msg = match &result.status {
            CheckStatus::Warning(m) => m.clone(),
            other => panic!("expected Warning, got {other:?}"),
        };
        assert!(msg.contains("SCOOP_VERSION"), "got: {msg}");
        assert!(msg.contains("SCOOP_LANG"), "got: {msg}");
        assert!(msg.contains("SCOOP_NO_AUTO"), "got: {msg}");
    }

    #[test]
    #[serial]
    fn legacy_check_warns_on_legacy_home_dir_without_new_dir() {
        let home_tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir(home_tmp.path().join(".scoop")).unwrap();
        let _g = crate::test_utils::env_guard(&[
            (paths::LEGACY_HOME_ENV, None),
            ("SCOOP_VERSION", None),
            ("SCOOP_LANG", None),
            ("SCOOP_NO_AUTO", None),
            (paths::SCUV_HOME_ENV, None),
            ("SCUV_VERSION", None),
            ("SCUV_LANG", None),
            ("HOME", Some(home_tmp.path().to_str().unwrap())),
        ]);
        let _cwd = TempDirCwdGuard::new();

        let result = check_legacy_remnants();
        assert!(result.is_warning(), "expected Warning, got {result:#?}");
        let msg = match &result.status {
            CheckStatus::Warning(m) => m.clone(),
            other => panic!("expected Warning, got {other:?}"),
        };
        assert!(msg.contains("~/.scoop"), "got: {msg}");
    }

    /// `~/.scoop` existing is only a remnant while `~/.scuv` hasn't been
    /// created yet — once the user has migrated, the two can briefly
    /// coexist (e.g. before the user deletes the old one) without that
    /// being something doctor should flag.
    #[test]
    #[serial]
    fn legacy_check_ok_when_both_scoop_and_scuv_home_dirs_exist() {
        let home_tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir(home_tmp.path().join(".scoop")).unwrap();
        std::fs::create_dir(home_tmp.path().join(".scuv")).unwrap();
        let _g = crate::test_utils::env_guard(&[
            (paths::LEGACY_HOME_ENV, None),
            ("SCOOP_VERSION", None),
            ("SCOOP_LANG", None),
            ("SCOOP_NO_AUTO", None),
            (paths::SCUV_HOME_ENV, None),
            ("SCUV_VERSION", None),
            ("SCUV_LANG", None),
            ("HOME", Some(home_tmp.path().to_str().unwrap())),
        ]);
        let _cwd = TempDirCwdGuard::new();

        let result = check_legacy_remnants();
        assert!(result.is_ok(), "expected Ok, got {result:#?}");
    }

    #[test]
    #[serial]
    fn legacy_check_warns_on_legacy_version_and_manifest_files_in_cwd() {
        let home_tmp = tempfile::tempdir().unwrap();
        let _g = crate::test_utils::env_guard(&[
            (paths::LEGACY_HOME_ENV, None),
            ("SCOOP_VERSION", None),
            ("SCOOP_LANG", None),
            ("SCOOP_NO_AUTO", None),
            (paths::SCUV_HOME_ENV, None),
            ("SCUV_VERSION", None),
            ("SCUV_LANG", None),
            ("HOME", Some(home_tmp.path().to_str().unwrap())),
        ]);
        let cwd_guard = TempDirCwdGuard::new();
        std::fs::write(cwd_guard.path().join(".scoop-version"), "myenv").unwrap();
        std::fs::write(cwd_guard.path().join(".scoop.toml"), "").unwrap();

        let result = check_legacy_remnants();
        assert!(result.is_warning(), "expected Warning, got {result:#?}");
        let msg = match &result.status {
            CheckStatus::Warning(m) => m.clone(),
            other => panic!("expected Warning, got {other:?}"),
        };
        assert!(msg.contains(".scoop-version"), "got: {msg}");
        assert!(msg.contains(".scoop.toml"), "got: {msg}");
    }

    #[test]
    #[serial]
    fn legacy_check_suggestion_names_current_env_var_and_removal_version() {
        let _locale = crate::test_utils::LocaleGuard::capture();
        rust_i18n::set_locale("en");

        let home_tmp = tempfile::tempdir().unwrap();
        let _g = crate::test_utils::env_guard(&[
            (
                paths::LEGACY_HOME_ENV,
                Some("/tmp/legacy-home-doesnt-need-to-exist"),
            ),
            ("SCOOP_VERSION", None),
            ("SCOOP_LANG", None),
            ("SCOOP_NO_AUTO", None),
            (paths::SCUV_HOME_ENV, None),
            ("SCUV_VERSION", None),
            ("SCUV_LANG", None),
            ("HOME", Some(home_tmp.path().to_str().unwrap())),
        ]);
        let _cwd = TempDirCwdGuard::new();

        let result = check_legacy_remnants();
        let suggestion = result.suggestion.as_deref().unwrap_or_default();
        assert!(
            suggestion.contains("SCUV_"),
            "suggestion should point at SCUV_*, got: {suggestion}"
        );
        assert!(
            suggestion.contains("v0.16.0"),
            "suggestion should name the removal version, got: {suggestion}"
        );
    }

    #[test]
    fn doctor_registers_legacy_check() {
        let doctor = Doctor::new();
        assert!(
            doctor.checks.iter().any(|c| c.id() == "legacy"),
            "Doctor::new() must register the legacy check"
        );
    }

    // ==========================================================================
    // VersionCheck: dual-read local version file (new `.scuv-version` name
    // wins, a legacy-only `.scoop-version` is still seen) — keeps doctor
    // aligned with VersionService::resolve()'s per-directory precedence
    // during the scoop -> scuv shim window.
    // ==========================================================================

    #[test]
    #[serial]
    fn version_check_sees_legacy_only_local_version_file() {
        with_temp_scoop_home(|temp| {
            let cwd_guard = TempDirCwdGuard::new();
            // Only the legacy filename exists in cwd — no `.scuv-version`.
            std::fs::write(cwd_guard.path().join(".scoop-version"), "ghost-env").unwrap();
            std::fs::create_dir_all(temp.path().join("virtualenvs")).unwrap();

            let results = VersionCheck.run();
            let local = results
                .iter()
                .find(|r| r.id == "version:local")
                .expect("version:local must be reported from a legacy-only .scoop-version file");
            assert!(
                local.is_error(),
                "references a non-existent env, so it should error, got {local:#?}"
            );
            assert!(
                matches!(&local.status, CheckStatus::Error(msg) if msg.contains("ghost-env")),
                "error should name the env read from the legacy file, got {local:#?}"
            );
        });
    }

    #[test]
    #[serial]
    fn version_check_prefers_new_file_over_legacy_when_both_present() {
        with_temp_scoop_home(|temp| {
            let cwd_guard = TempDirCwdGuard::new();
            std::fs::write(cwd_guard.path().join(".scuv-version"), "newenv").unwrap();
            std::fs::write(cwd_guard.path().join(".scoop-version"), "oldenv").unwrap();
            std::fs::create_dir_all(temp.path().join("virtualenvs")).unwrap();

            let results = VersionCheck.run();
            let local = results
                .iter()
                .find(|r| r.id == "version:local")
                .expect("version:local must run");
            assert!(
                matches!(&local.status, CheckStatus::Error(msg) if msg.contains("newenv")),
                "new-named file must win when both are present, got {local:#?}"
            );
        });
    }

    // ==========================================================================
    // Check-trait dispatch coverage. The underlying logic above is tested
    // directly; these pin the thin trait wrappers (id/name/run) so a broken
    // registration or renamed check surfaces in tests, not in `doctor` output.
    // ==========================================================================

    #[test]
    #[serial]
    fn legacy_check_trait_dispatch_reports_identity_and_runs() {
        let home_tmp = tempfile::tempdir().unwrap();
        let _g = crate::test_utils::env_guard(&[
            (paths::LEGACY_HOME_ENV, None),
            ("SCOOP_VERSION", None),
            ("SCOOP_LANG", None),
            ("SCOOP_NO_AUTO", None),
            (paths::SCUV_HOME_ENV, None),
            ("SCUV_VERSION", None),
            ("SCUV_LANG", None),
            ("HOME", Some(home_tmp.path().to_str().unwrap())),
        ]);
        let _cwd = TempDirCwdGuard::new();

        let check: &dyn Check = &LegacyCheck;
        assert_eq!(check.id(), "legacy");
        assert_eq!(check.name(), "legacy scoop remnants");
        let results = check.run();
        assert_eq!(results.len(), 1, "got {results:#?}");
        assert_eq!(results[0].id, "legacy");
        assert!(results[0].is_ok(), "clean env must be Ok, got {results:#?}");
    }

    #[test]
    #[serial]
    fn home_check_trait_dispatch_ok_on_existing_writable_home() {
        with_temp_scoop_home(|temp| {
            let check: &dyn Check = &HomeCheck;
            assert_eq!(check.id(), "home");
            assert_eq!(check.name(), "SCUV_HOME directory");
            let results = check.run();
            assert_eq!(results.len(), 1, "got {results:#?}");
            assert!(results[0].is_ok(), "got {results:#?}");
            let details = results[0].details.as_deref().unwrap_or_default();
            assert!(
                details.contains(temp.path().to_str().unwrap()),
                "details should name the home path, got {details:?}"
            );
        });
    }

    #[test]
    fn uv_check_trait_dispatch_always_reports_under_uv_id() {
        // Environment-tolerant: passes whether or not a usable `uv` is on
        // PATH — what it pins is that run() actually runs and every result
        // carries this check's identity.
        let check: &dyn Check = &UvCheck;
        assert_eq!(check.id(), "uv");
        assert_eq!(check.name(), "uv installation");
        let results = check.run();
        assert!(!results.is_empty(), "run() must produce results");
        assert!(
            results.iter().all(|r| r.id == "uv"),
            "all results must carry the uv id, got {results:#?}"
        );
    }

    // ==========================================================================
    // ShellCheck zsh branch (the `shell_name == "zsh"` boundary): the
    // suggestion must name `scuv init zsh`, not bash, and read .zshrc.
    // ==========================================================================

    #[test]
    #[serial]
    fn shell_check_zsh_warns_on_legacy_only_line_with_zsh_suggestion() {
        let home_tmp = tempfile::tempdir().unwrap();
        std::fs::write(
            home_tmp.path().join(".zshrc"),
            "eval \"$(scoop init zsh)\"\n",
        )
        .unwrap();
        let _g = crate::test_utils::env_guard(&[
            ("SHELL", Some("/bin/zsh")),
            ("HOME", Some(home_tmp.path().to_str().unwrap())),
        ]);

        let results = ShellCheck.run();
        assert_eq!(results.len(), 1, "got {results:#?}");
        assert!(results[0].is_warning(), "got {results:#?}");
        let suggestion = results[0].suggestion.as_deref().unwrap_or_default();
        assert!(
            suggestion.contains("scuv init zsh"),
            "zsh shell must get a zsh suggestion, got {suggestion:?}"
        );
    }

    #[test]
    #[serial]
    fn shell_check_zsh_is_ok_with_current_scuv_init_line() {
        let home_tmp = tempfile::tempdir().unwrap();
        std::fs::write(
            home_tmp.path().join(".zshrc"),
            "eval \"$(scuv init zsh)\"\n",
        )
        .unwrap();
        let _g = crate::test_utils::env_guard(&[
            ("SHELL", Some("/bin/zsh")),
            ("HOME", Some(home_tmp.path().to_str().unwrap())),
        ]);

        let results = ShellCheck.run();
        assert_eq!(results.len(), 1, "got {results:#?}");
        assert!(results[0].is_ok(), "got {results:#?}");
    }

    // ==========================================================================
    // Doctor::fix_home — creates the missing home (+ virtualenvs) only for
    // "not found" errors, and leaves anything else alone.
    // ==========================================================================

    #[test]
    #[serial]
    fn fix_home_creates_missing_home_and_virtualenvs_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let missing_home = tmp.path().join("scuvhome");
        let _g = crate::test_utils::env_guard(&[(
            paths::SCUV_HOME_ENV,
            Some(missing_home.to_str().unwrap()),
        )]);

        let output = crate::output::Output::new(0, true, true, false);
        let broken = CheckResult::error("home", "SCUV_HOME directory", "directory not found");

        let fixed = HomeCheck.fix(&broken, &output);
        let fixed = fixed.expect("a 'not found' home error must be fixable");
        assert!(fixed.is_ok(), "got {fixed:#?}");
        assert!(missing_home.is_dir(), "home dir must be created");
        assert!(
            missing_home.join("virtualenvs").is_dir(),
            "virtualenvs subdir must be created"
        );
    }

    #[test]
    #[serial]
    fn fix_home_ignores_non_not_found_results() {
        let tmp = tempfile::tempdir().unwrap();
        let _g = crate::test_utils::env_guard(&[(
            paths::SCUV_HOME_ENV,
            Some(tmp.path().to_str().unwrap()),
        )]);

        let output = crate::output::Output::new(0, true, true, false);
        let unrelated = CheckResult::error("home", "SCUV_HOME directory", "permission denied");
        assert!(HomeCheck.fix(&unrelated, &output).is_none());

        let warning = CheckResult::warn("home", "SCUV_HOME directory", "directory not found");
        assert!(
            HomeCheck.fix(&warning, &output).is_none(),
            "only Error status is fixable, not Warning"
        );
    }
}
