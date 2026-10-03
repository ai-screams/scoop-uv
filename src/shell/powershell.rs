//! PowerShell shell integration
//!
//! Provides PowerShell support for scuv, including:
//! - Wrapper function for `scuv` command
//! - Auto-activate hook via prompt override
//! - Tab completion with dynamic environment names and Python versions
//!
//! Supports both PowerShell Core (pwsh) and Windows PowerShell 5.1.

use crate::{file_resolution_check, scoop_version_check};

/// Generate PowerShell initialization script.
///
/// Returns a static string containing the PowerShell integration script.
/// This script should be evaluated in the user's `$PROFILE`:
///
/// ```powershell
/// Invoke-Expression (& scuv init powershell)
/// ```
///
/// # Examples
///
/// ```
/// let script = scoop_uv::shell::powershell::init_script();
///
/// // Script contains the wrapper function
/// assert!(script.contains("function scuv"));
///
/// // Script contains the auto-activate hook
/// assert!(script.contains("function _scuv_hook"));
///
/// // Script contains completion definitions
/// assert!(script.contains("Register-ArgumentCompleter"));
/// ```
pub fn init_script() -> &'static str {
    concat!(
        // The `scuv` wrapper function (src/shell/scripts/powershell_wrapper.ps1).
        include_str!("scripts/powershell_wrapper.ps1"),
        // The auto-activate hook, assembled from the shared checks.
        // The blank line after the wrapper lives here: a script file
        // cannot end in one (end-of-file-fixer trims it).
        r#"
# Auto-activate hook
function _scuv_hook {
"#,
        scoop_version_check!(powershell),
        file_resolution_check!(powershell),
        r#"
}

# Override prompt to call hook
if (-not $env:SCUV_NO_AUTO) {
    $global:_scuv_original_prompt = $function:prompt
    function global:prompt {
        _scuv_hook
        & $global:_scuv_original_prompt
    }
}

# Run hook on startup
_scuv_hook

"#,
        // Tab completion (src/shell/scripts/powershell_completion.ps1).
        include_str!("scripts/powershell_completion.ps1"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // Real Test: Syntax Validation with pwsh
    // =========================================================================

    /// Validates that the generated script has valid PowerShell syntax.
    /// This is a REAL test - it actually runs pwsh to check the script.
    #[test]
    fn test_init_script_has_valid_powershell_syntax() {
        let script = init_script();

        // Use pwsh to parse the script (wrap in scriptblock to check syntax)
        // We use [scriptblock]::Create() which parses without executing
        let check_command = format!("$null = [scriptblock]::Create(@'\n{}\n'@)", script);

        let output = std::process::Command::new("pwsh")
            .arg("-NoProfile")
            .arg("-Command")
            .arg(&check_command)
            .output();

        match output {
            Ok(result) => {
                assert!(
                    result.status.success(),
                    "PowerShell script has syntax errors:\n{}",
                    String::from_utf8_lossy(&result.stderr)
                );
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                // pwsh not available, skip test
                eprintln!("Skipping PowerShell syntax test: pwsh not found");
            }
            Err(e) => panic!("Failed to run pwsh: {}", e),
        }
    }

    // =========================================================================
    // Structural Tests: Minimal checks for required components
    // =========================================================================

    #[test]
    fn test_init_script_not_empty() {
        let script = init_script();
        assert!(!script.is_empty(), "Script should not be empty");
    }

    #[test]
    fn test_init_script_defines_wrapper_function() {
        let script = init_script();
        assert!(
            script.contains("function scuv"),
            "Script missing wrapper function"
        );
    }

    #[test]
    fn test_init_script_defines_hook_function() {
        let script = init_script();
        assert!(
            script.contains("function _scuv_hook"),
            "Script missing auto-activate hook"
        );
    }

    #[test]
    fn test_init_script_registers_prompt_hook() {
        let script = init_script();
        assert!(
            script.contains("function global:prompt"),
            "Script must override prompt for auto-activation"
        );
    }

    #[test]
    fn test_init_script_registers_completion() {
        let script = init_script();
        assert!(
            script.contains("Register-ArgumentCompleter"),
            "Script must register PowerShell completion"
        );
    }

    // =========================================================================
    // Feature Tests: Verify key behaviors
    // =========================================================================

    #[test]
    fn test_init_script_checks_scuv_no_auto() {
        let script = init_script();
        assert!(
            script.contains("SCUV_NO_AUTO"),
            "Script must check SCUV_NO_AUTO environment variable"
        );
        // Fails if the scoop-era SCOOP_NO_AUTO read comes back.
        assert!(!script.contains("SCOOP_NO_AUTO"));
    }

    #[test]
    fn test_init_script_uses_invoke_expression() {
        let script = init_script();
        assert!(
            script.contains("Invoke-Expression"),
            "Script must use Invoke-Expression for eval"
        );
    }

    #[test]
    fn test_init_script_has_dynamic_completions() {
        let script = init_script();
        assert!(
            script.contains("list --bare"),
            "Script must provide dynamic env name completions"
        );
        assert!(
            script.contains("list --pythons --bare"),
            "Script must provide dynamic Python version completions"
        );
    }

    #[test]
    fn test_init_script_stores_binary_path() {
        let script = init_script();
        assert!(
            script.contains("$script:ScuvBin"),
            "Script must store binary path to avoid recursion"
        );
        assert!(
            script.contains("Get-Command scuv -CommandType Application"),
            "Script must use Get-Command to find binary"
        );
    }

    #[test]
    fn test_init_script_handles_use_command() {
        let script = init_script();
        assert!(
            script.contains("'use'"),
            "Script must handle 'use' command specially"
        );
    }

    /// The one-shot deprecation warnings went with 0.16.0, and with them
    /// the suppression variable the chained use→activate call used to set.
    /// Fails if SCUV_SUPPRESS_DEPRECATION plumbing is reintroduced.
    #[test]
    fn init_script_has_no_deprecation_suppression() {
        assert!(!init_script().contains("SCUV_SUPPRESS_DEPRECATION"));
    }

    /// The `scuv lang` completion candidates are hand-written in this script;
    /// this pins them to `SUPPORTED_LANGS` so adding a locale cannot skip a
    /// shell. Fails if a code is missing from (or extra in) the list.
    #[test]
    fn lang_completion_list_matches_supported_langs() {
        let script = init_script();
        let re = regex::Regex::new(r"(?s)\$cmd -eq 'lang'.*?@\(([^)]+)\)").unwrap();
        let caps = re
            .captures(script)
            .expect("PowerShell script must complete `lang` with an @(...) array");
        let mut found: Vec<&str> = caps
            .get(1)
            .unwrap()
            .as_str()
            .split(',')
            .map(|tok| tok.trim().trim_matches('\''))
            .collect();
        let mut expected: Vec<&str> = crate::i18n::SUPPORTED_LANGS
            .iter()
            .map(|(c, _)| *c)
            .collect();
        expected.sort_unstable();
        found.sort_unstable();
        assert_eq!(found, expected);
    }

    /// Safety-critical: scoop.sh (the Windows package manager) coexistence.
    /// PowerShell must NEVER define a `scoop` function or alias, or it would
    /// shadow the real `scoop` command for scoop.sh users.
    #[test]
    fn init_script_never_defines_scoop() {
        let s = init_script();
        assert!(!s.to_lowercase().contains("function scoop"));
        assert!(!s.to_lowercase().contains("set-alias scoop"));
    }
}
