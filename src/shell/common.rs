//! Common shell script generation utilities
//!
//! This module provides compile-time macros for generating shell-specific
//! hook scripts, avoiding code duplication across bash, zsh, and fish modules.
//!
//! # Design
//!
//! Uses `macro_rules!` with pattern matching on shell type identifiers.
//! Combined with `concat!()`, this enables compile-time string composition
//! while maintaining DRY principles.

/// Generate SCOOP_VERSION priority check script for the auto-activate hook.
///
/// This handles Priority 1 in the resolution order: the SCOOP_VERSION
/// environment variable set by `scoop shell` command.
///
/// # Usage
///
/// ```ignore
/// concat!(
///     "function _scoop_hook() {\n",
///     scoop_version_check!(bash),
///     file_resolution_check!(bash),
///     "}\n"
/// )
/// ```
#[macro_export]
macro_rules! scoop_version_check {
    (bash) => {
        r#"
    # Priority 1: SCOOP_VERSION environment variable (scoop shell)
    if [[ -n "$SCOOP_VERSION" ]]; then
        if [[ "$SCOOP_VERSION" == "system" ]]; then
            if [[ -n "$SCOOP_ACTIVE" ]]; then
                eval "$(command scoop deactivate)"
            fi
        elif [[ "$SCOOP_VERSION" != "$SCOOP_ACTIVE" ]]; then
            eval "$(command scoop activate "$SCOOP_VERSION")"
        fi
        return
    fi
"#
    };
    (zsh) => {
        scoop_version_check!(bash)
    };
    (fish) => {
        r#"
    # Priority 1: SCOOP_VERSION environment variable (scoop shell)
    if set -q SCOOP_VERSION
        if test "$SCOOP_VERSION" = "system"
            if set -q SCOOP_ACTIVE
                eval (command scoop deactivate)
            end
        else if test "$SCOOP_VERSION" != "$SCOOP_ACTIVE"
            eval (command scoop activate "$SCOOP_VERSION")
        end
        return
    end
"#
    };
    (powershell) => {
        r#"
    # Priority 1: SCOOP_VERSION environment variable (scoop shell)
    if ($env:SCOOP_VERSION) {
        if ($env:SCOOP_VERSION -eq 'system') {
            if ($env:SCOOP_ACTIVE) {
                Invoke-Expression (& $script:ScoopBin deactivate)
            }
        } elseif ($env:SCOOP_VERSION -ne $env:SCOOP_ACTIVE) {
            Invoke-Expression (& $script:ScoopBin activate $env:SCOOP_VERSION)
        }
        return
    }
"#
    };
}

/// Generate file-based resolution script for the auto-activate hook.
///
/// This handles Priority 2-5 in the resolution order:
/// - .scoop-version in current directory
/// - .scoop-version in parent directories
/// - .python-version files
/// - Global ~/.scoop/version
#[macro_export]
macro_rules! file_resolution_check {
    (bash) => {
        r#"
    # Priority 2-5: File-based resolution
    local env_name
    env_name="$(command scoop resolve 2>/dev/null)"

    if [[ "$env_name" == "system" ]]; then
        if [[ -n "$SCOOP_ACTIVE" ]]; then
            eval "$(command scoop deactivate)"
        fi
    elif [[ -n "$env_name" && "$env_name" != "$SCOOP_ACTIVE" ]]; then
        eval "$(command scoop activate "$env_name")"
    elif [[ -z "$env_name" && -n "$SCOOP_ACTIVE" ]]; then
        eval "$(command scoop deactivate)"
    fi"#
    };
    (zsh) => {
        file_resolution_check!(bash)
    };
    (fish) => {
        r#"
    # Priority 2-5: File-based resolution
    set -l env_name (command scoop resolve 2>/dev/null)

    if test "$env_name" = "system"
        if set -q SCOOP_ACTIVE
            eval (command scoop deactivate)
        end
    else if test -n "$env_name" -a "$env_name" != "$SCOOP_ACTIVE"
        eval (command scoop activate "$env_name")
    else if test -z "$env_name" -a -n "$SCOOP_ACTIVE"
        eval (command scoop deactivate)
    end"#
    };
    (powershell) => {
        r#"
    # Priority 2-5: File-based resolution
    $env_name = & $script:ScoopBin resolve 2>$null

    if ($env_name -eq 'system') {
        if ($env:SCOOP_ACTIVE) {
            Invoke-Expression (& $script:ScoopBin deactivate)
        }
    } elseif ($env_name -and ($env_name -ne $env:SCOOP_ACTIVE)) {
        Invoke-Expression (& $script:ScoopBin activate $env_name)
    } elseif ((-not $env_name) -and $env:SCOOP_ACTIVE) {
        Invoke-Expression (& $script:ScoopBin deactivate)
    }"#
    };
}

// Re-export macros for use in sibling modules
pub use file_resolution_check;
pub use scoop_version_check;

#[cfg(test)]
mod tests {
    /// Verify bash hook contains priority comment
    #[test]
    fn test_scoop_version_check_bash_contains_priority_comment() {
        let script = scoop_version_check!(bash);
        assert!(script.contains("Priority 1"));
        assert!(script.contains("SCOOP_VERSION"));
    }

    /// Verify fish hook uses fish syntax
    #[test]
    fn test_scoop_version_check_fish_uses_fish_syntax() {
        let script = scoop_version_check!(fish);
        assert!(script.contains("set -q SCOOP_VERSION"));
        assert!(script.contains("end"));
    }

    /// Verify bash file resolution uses local variable
    #[test]
    fn test_file_resolution_bash_uses_local_variable() {
        let script = file_resolution_check!(bash);
        assert!(script.contains("local env_name"));
    }

    /// Verify fish file resolution uses set -l
    #[test]
    fn test_file_resolution_fish_uses_set_l() {
        let script = file_resolution_check!(fish);
        assert!(script.contains("set -l env_name"));
    }

    /// Verify zsh delegates to bash (same syntax)
    #[test]
    fn test_zsh_delegates_to_bash() {
        let bash_version = scoop_version_check!(bash);
        let zsh_version = scoop_version_check!(zsh);
        assert_eq!(bash_version, zsh_version);
    }

    /// Verify PowerShell hook uses PowerShell syntax
    #[test]
    fn test_scoop_version_check_powershell_uses_powershell_syntax() {
        let script = scoop_version_check!(powershell);
        assert!(script.contains("$env:SCOOP_VERSION"));
        assert!(script.contains("Invoke-Expression"));
        assert!(script.contains("$script:ScoopBin"));
    }

    /// Verify PowerShell file resolution uses proper variable
    #[test]
    fn test_file_resolution_powershell_uses_env_name() {
        let script = file_resolution_check!(powershell);
        assert!(script.contains("$env_name"));
        assert!(script.contains("$script:ScoopBin"));
    }
}
