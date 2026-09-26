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

/// Generate SCUV_VERSION priority check script for the auto-activate hook.
///
/// This handles Priority 1 in the resolution order: the SCUV_VERSION
/// environment variable set by the `scuv shell` command.
///
/// # Usage
///
/// ```ignore
/// concat!(
///     "function _scuv_hook() {\n",
///     scoop_version_check!(bash),
///     file_resolution_check!(bash),
///     "}\n"
/// )
/// ```
#[macro_export]
macro_rules! scoop_version_check {
    (bash) => {
        r#"
    # Priority 1: SCUV_VERSION environment variable (scuv shell)
    if [[ -n "$SCUV_VERSION" ]]; then
        if [[ "$SCUV_VERSION" == "system" ]]; then
            if [[ -n "$SCUV_ACTIVE" ]]; then
                eval "$(command scuv deactivate)"
            fi
        elif [[ "$SCUV_VERSION" != "$SCUV_ACTIVE" ]]; then
            eval "$(command scuv activate "$SCUV_VERSION")"
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
    # Priority 1: SCUV_VERSION environment variable (scuv shell)
    if set -q SCUV_VERSION
        if test "$SCUV_VERSION" = "system"
            if set -q SCUV_ACTIVE
                command scuv deactivate --shell fish | source
            end
        else if test "$SCUV_VERSION" != "$SCUV_ACTIVE"
            command scuv activate --shell fish "$SCUV_VERSION" | source
        end
        return
    end
"#
    };
    (powershell) => {
        r#"
    # Priority 1: SCUV_VERSION environment variable (scuv shell)
    if ($env:SCUV_VERSION) {
        if ($env:SCUV_VERSION -eq 'system') {
            if ($env:SCUV_ACTIVE) {
                Invoke-Expression (& $script:ScuvBin deactivate)
            }
        } elseif ($env:SCUV_VERSION -ne $env:SCUV_ACTIVE) {
            Invoke-Expression (& $script:ScuvBin activate $env:SCUV_VERSION)
        }
        return
    }
"#
    };
}

/// Generate file-based resolution script for the auto-activate hook.
///
/// This handles Priority 2-3 in the resolution order:
/// - .scuv-version in current directory
/// - .scuv-version in parent directories
/// - Global ~/.scuv/version
#[macro_export]
macro_rules! file_resolution_check {
    (bash) => {
        r#"
    # Priority 2-3: File-based resolution
    local env_name
    env_name="$(command scuv resolve 2>/dev/null)"

    if [[ "$env_name" == "system" ]]; then
        if [[ -n "$SCUV_ACTIVE" ]]; then
            eval "$(command scuv deactivate)"
        fi
    elif [[ -n "$env_name" && "$env_name" != "$SCUV_ACTIVE" ]]; then
        eval "$(command scuv activate "$env_name")"
    elif [[ -z "$env_name" && -n "$SCUV_ACTIVE" ]]; then
        eval "$(command scuv deactivate)"
    fi"#
    };
    (zsh) => {
        file_resolution_check!(bash)
    };
    (fish) => {
        r#"
    # Priority 2-3: File-based resolution
    set -l env_name (command scuv resolve 2>/dev/null)

    if test "$env_name" = "system"
        if set -q SCUV_ACTIVE
            command scuv deactivate --shell fish | source
        end
    else if test -n "$env_name" -a "$env_name" != "$SCUV_ACTIVE"
        command scuv activate --shell fish "$env_name" | source
    else if test -z "$env_name" -a -n "$SCUV_ACTIVE"
        command scuv deactivate --shell fish | source
    end"#
    };
    (powershell) => {
        r#"
    # Priority 2-3: File-based resolution
    $env_name = & $script:ScuvBin resolve 2>$null

    if ($env_name -eq 'system') {
        if ($env:SCUV_ACTIVE) {
            Invoke-Expression (& $script:ScuvBin deactivate)
        }
    } elseif ($env_name -and ($env_name -ne $env:SCUV_ACTIVE)) {
        Invoke-Expression (& $script:ScuvBin activate $env_name)
    } elseif ((-not $env_name) -and $env:SCUV_ACTIVE) {
        Invoke-Expression (& $script:ScuvBin deactivate)
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
        assert!(script.contains("SCUV_VERSION"));
        // Fails if the scoop-era SCOOP_VERSION fallback read comes back.
        assert!(!script.contains("SCOOP_VERSION"));
    }

    /// Verify fish hook uses fish syntax
    #[test]
    fn test_scoop_version_check_fish_uses_fish_syntax() {
        let script = scoop_version_check!(fish);
        assert!(script.contains("set -q SCUV_VERSION"));
        assert!(!script.contains("SCOOP_VERSION"));
        assert!(script.contains("end"));
    }

    /// See `fish::tests::init_script_sources_activation_output_with_explicit_shell`
    /// for why: `eval (...)` collapses newlines and fish does not export
    /// FISH_VERSION. Fails if a fish hook macro evals or drops `--shell fish`.
    #[test]
    fn fish_hook_macros_source_activation_output_with_explicit_shell() {
        for script in [scoop_version_check!(fish), file_resolution_check!(fish)] {
            assert!(!script.contains("eval ("), "got: {script}");
            assert!(
                script.contains("command scuv deactivate --shell fish | source"),
                "got: {script}"
            );
        }
        assert!(
            scoop_version_check!(fish)
                .contains(r#"command scuv activate --shell fish "$SCUV_VERSION" | source"#)
        );
        assert!(
            file_resolution_check!(fish)
                .contains(r#"command scuv activate --shell fish "$env_name" | source"#)
        );
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
        assert!(script.contains("$env:SCUV_VERSION"));
        assert!(!script.contains("SCOOP_VERSION"));
        assert!(script.contains("Invoke-Expression"));
        assert!(script.contains("$script:ScuvBin"));
    }

    /// Verify PowerShell file resolution uses proper variable
    #[test]
    fn test_file_resolution_powershell_uses_env_name() {
        let script = file_resolution_check!(powershell);
        assert!(script.contains("$env_name"));
        assert!(script.contains("$script:ScuvBin"));
    }
}
