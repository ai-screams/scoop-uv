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
/// environment variable set by the `scuv shell` command (falling back to the
/// legacy SCOOP_VERSION, deprecated).
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
    # DEPRECATION(0.16.0): drop the legacy SCOOP_VERSION fallback read.
    local _scuv_pin="${SCUV_VERSION:-$SCOOP_VERSION}"
    if [[ -n "$_scuv_pin" ]]; then
        if [[ "$_scuv_pin" == "system" ]]; then
            if [[ -n "$SCUV_ACTIVE" ]]; then
                eval "$(command scuv deactivate)"
            fi
        elif [[ "$_scuv_pin" != "$SCUV_ACTIVE" ]]; then
            eval "$(command scuv activate "$_scuv_pin")"
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
    # DEPRECATION(0.16.0): drop the legacy SCOOP_VERSION fallback read.
    if set -q SCUV_VERSION; or set -q SCOOP_VERSION
        set -l _scuv_pin $SCUV_VERSION
        if not set -q SCUV_VERSION
            set _scuv_pin $SCOOP_VERSION
        end
        if test "$_scuv_pin" = "system"
            if set -q SCUV_ACTIVE
                eval (command scuv deactivate)
            end
        else if test "$_scuv_pin" != "$SCUV_ACTIVE"
            eval (command scuv activate "$_scuv_pin")
        end
        return
    end
"#
    };
    (powershell) => {
        r#"
    # Priority 1: SCUV_VERSION environment variable (scuv shell)
    # DEPRECATION(0.16.0): drop the legacy SCOOP_VERSION fallback read.
    $_scuvPin = if ($env:SCUV_VERSION) { $env:SCUV_VERSION } else { $env:SCOOP_VERSION }
    if ($_scuvPin) {
        if ($_scuvPin -eq 'system') {
            if ($env:SCUV_ACTIVE) {
                Invoke-Expression (& $script:ScuvBin deactivate)
            }
        } elseif ($_scuvPin -ne $env:SCUV_ACTIVE) {
            Invoke-Expression (& $script:ScuvBin activate $_scuvPin)
        }
        return
    }
"#
    };
}

/// Generate file-based resolution script for the auto-activate hook.
///
/// This handles Priority 2-3 in the resolution order:
/// - .scuv-version in current directory (legacy .scoop-version fallback)
/// - .scuv-version in parent directories (legacy .scoop-version fallback)
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
            eval (command scuv deactivate)
        end
    else if test -n "$env_name" -a "$env_name" != "$SCUV_ACTIVE"
        eval (command scuv activate "$env_name")
    else if test -z "$env_name" -a -n "$SCUV_ACTIVE"
        eval (command scuv deactivate)
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
        // SCUV_VERSION is the primary read; legacy SCOOP_VERSION is still
        // honored as a fallback (deprecated).
        assert!(script.contains("SCUV_VERSION"));
        assert!(script.contains("SCOOP_VERSION"));
    }

    /// Verify fish hook uses fish syntax
    #[test]
    fn test_scoop_version_check_fish_uses_fish_syntax() {
        let script = scoop_version_check!(fish);
        assert!(script.contains("set -q SCUV_VERSION"));
        assert!(script.contains("SCOOP_VERSION"));
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
        assert!(script.contains("$env:SCUV_VERSION"));
        assert!(script.contains("$env:SCOOP_VERSION"));
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
