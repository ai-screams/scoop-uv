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
        r#"# scuv shell integration for PowerShell
# Add to your $PROFILE: Invoke-Expression (& scuv init powershell)

# Get the scuv binary path (avoids conflict with wrapper function)
$script:ScuvBin = (Get-Command scuv -CommandType Application -ErrorAction SilentlyContinue).Source
if (-not $script:ScuvBin) {
    Write-Warning "scuv binary not found in PATH"
    return
}

# Wrapper function for scuv
function scuv {
    param([Parameter(ValueFromRemainingArguments=$true)]$Arguments)

    $command = if ($Arguments.Count -gt 0) { $Arguments[0] } else { '' }

    switch ($command) {
        'use' {
            & $script:ScuvBin @Arguments
            if ($LASTEXITCODE -eq 0) {
                $name = $Arguments | Where-Object { $_ -notmatch '^-' } | Select-Object -Skip 1 -First 1
                if ($name) {
                    # 'use' above already warned about any legacy config; don't warn twice.
                    # Save/restore so a user-set suppression value survives this call.
                    $hadSuppress = Test-Path Env:SCUV_SUPPRESS_DEPRECATION
                    $prevSuppress = if ($hadSuppress) { $env:SCUV_SUPPRESS_DEPRECATION } else { $null }
                    $env:SCUV_SUPPRESS_DEPRECATION = '1'
                    try {
                        Invoke-Expression (& $script:ScuvBin activate $name)
                    } finally {
                        if ($hadSuppress) {
                            $env:SCUV_SUPPRESS_DEPRECATION = $prevSuppress
                        } else {
                            Remove-Item Env:SCUV_SUPPRESS_DEPRECATION -ErrorAction SilentlyContinue
                        }
                    }
                }
            }
        }
        { $_ -in 'activate', 'deactivate', 'shell' } {
            if ($Arguments -match '(-h|--help|-V|--version)') {
                & $script:ScuvBin @Arguments
            } else {
                Invoke-Expression (& $script:ScuvBin @Arguments)
            }
        }
        default {
            & $script:ScuvBin @Arguments
        }
    }
}

# Auto-activate hook
function _scuv_hook {
"#,
        scoop_version_check!(powershell),
        file_resolution_check!(powershell),
        r#"
}

# Override prompt to call hook
# DEPRECATION(0.16.0): drop the legacy SCOOP_NO_AUTO fallback check.
if ((-not $env:SCUV_NO_AUTO) -and (-not $env:SCOOP_NO_AUTO)) {
    $global:_scuv_original_prompt = $function:prompt
    function global:prompt {
        _scuv_hook
        & $global:_scuv_original_prompt
    }
}

# Run hook on startup
_scuv_hook

# Tab completion
Register-ArgumentCompleter -Native -CommandName scuv -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commands = @('list', 'create', 'use', 'remove', 'info', 'install', 'uninstall',
                  'doctor', 'init', 'completions', 'activate', 'deactivate', 'shell',
                  'migrate', 'lang')

    $tokens = $commandAst.ToString() -split '\s+'
    $cmd = if ($tokens.Count -gt 1) { $tokens[1] } else { '' }

    # First argument: complete subcommands
    if ($tokens.Count -le 2 -and $wordToComplete -notmatch '^-') {
        $commands | Where-Object { $_ -like "$wordToComplete*" } | ForEach-Object {
            [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
        }
        return
    }

    # Environment name completion for specific commands
    if ($cmd -in 'use', 'remove', 'info', 'activate', 'shell') {
        $envs = & $script:ScuvBin list --bare 2>$null
        if ($envs) {
            $envs | Where-Object { $_ -like "$wordToComplete*" } | ForEach-Object {
                [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
            }
        }
        return
    }

    # Python version completion
    if ($cmd -in 'install', 'uninstall', 'create') {
        $versions = & $script:ScuvBin list --pythons --bare 2>$null | Sort-Object -Unique
        if ($versions) {
            $versions | Where-Object { $_ -like "$wordToComplete*" } | ForEach-Object {
                [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
            }
        }
        return
    }

    # Shell completion for init/completions
    if ($cmd -in 'init', 'completions') {
        @('bash', 'zsh', 'fish', 'powershell') | Where-Object { $_ -like "$wordToComplete*" } | ForEach-Object {
            [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
        }
        return
    }

    # Language completion for lang
    if ($cmd -eq 'lang') {
        @('en', 'ko', 'ja', 'pt-BR') | Where-Object { $_ -like "$wordToComplete*" } | ForEach-Object {
            [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
        }
        return
    }

    # Migrate subcommand completion
    if ($cmd -eq 'migrate') {
        @('list', 'all') | Where-Object { $_ -like "$wordToComplete*" } | ForEach-Object {
            [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
        }
        return
    }
}
"#
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
        // Legacy SCOOP_NO_AUTO must still gate auto-activation (deprecated fallback).
        assert!(
            script.contains("SCOOP_NO_AUTO"),
            "Script must still honor legacy SCOOP_NO_AUTO"
        );
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

    /// The chained use→activate call must suppress duplicate deprecation
    /// warnings (each chained call is a fresh process), and must restore any
    /// pre-existing user value rather than leak `1` into the session.
    #[test]
    fn init_script_suppresses_duplicate_deprecation_in_use_chain() {
        let s = init_script();
        assert!(s.contains("SCUV_SUPPRESS_DEPRECATION"));
        assert!(
            s.contains("$env:SCUV_SUPPRESS_DEPRECATION = $prevSuppress"),
            "must restore a user-set suppression value"
        );
        assert!(
            s.contains("Remove-Item Env:SCUV_SUPPRESS_DEPRECATION"),
            "must clear the variable when the user had not set it"
        );
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
