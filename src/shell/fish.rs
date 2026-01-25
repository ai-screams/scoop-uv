//! Fish shell integration
//!
//! Provides Fish shell support for scoop, including:
//! - Wrapper function for `scoop` command
//! - Auto-activate hook via `--on-variable PWD`
//! - Tab completion with option deduplication

use crate::{file_resolution_check, scoop_version_check};

/// Generate fish initialization script.
///
/// Returns a static string containing the Fish shell integration script.
/// This script should be evaluated in the user's `config.fish`:
///
/// ```fish
/// eval (scoop init fish)
/// ```
///
/// # Examples
///
/// ```
/// let script = scoop_uv::shell::fish::init_script();
///
/// // Script contains the wrapper function
/// assert!(script.contains("function scoop"));
///
/// // Script contains the auto-activate hook
/// assert!(script.contains("function _scoop_hook --on-variable PWD"));
///
/// // Script contains completion definitions
/// assert!(script.contains("complete -c scoop"));
/// ```
pub fn init_script() -> &'static str {
    concat!(
        r#"# scoop shell integration for fish

# Wrapper function for scoop
function scoop
    set -l cmd $argv[1]

    switch "$cmd"
        case use
            command scoop $argv
            set -l ret $status
            if test $ret -eq 0
                for arg in $argv[2..-1]
                    if not string match -q -- '-*' "$arg"
                        eval (command scoop activate "$arg")
                        break
                    end
                end
            end
            return $ret

        case activate deactivate shell
            # Pass through help/version flags without eval
            if string match -qr -- '(-h|--help|-V|--version)' $argv
                command scoop $argv
            else
                eval (command scoop $argv)
            end

        case '*'
            command scoop $argv
    end
end

# Auto-activate hook
function _scoop_hook --on-variable PWD
"#,
        scoop_version_check!(fish),
        file_resolution_check!(fish),
        r#"
end

# Set up auto-activate on startup
if not set -q SCOOP_NO_AUTO
    _scoop_hook
end

# Fish completion for scoop
complete -c scoop -f

# Subcommands
set -l commands list use create remove info install uninstall doctor init completions activate deactivate migrate lang

complete -c scoop -n "not __fish_seen_subcommand_from $commands" -a "list" -d "List all virtual environments"
complete -c scoop -n "not __fish_seen_subcommand_from $commands" -a "use" -d "Set local environment for current directory"
complete -c scoop -n "not __fish_seen_subcommand_from $commands" -a "create" -d "Create a new virtual environment"
complete -c scoop -n "not __fish_seen_subcommand_from $commands" -a "remove" -d "Remove a virtual environment"
complete -c scoop -n "not __fish_seen_subcommand_from $commands" -a "info" -d "Show detailed information"
complete -c scoop -n "not __fish_seen_subcommand_from $commands" -a "install" -d "Install a Python version"
complete -c scoop -n "not __fish_seen_subcommand_from $commands" -a "uninstall" -d "Uninstall a Python version"
complete -c scoop -n "not __fish_seen_subcommand_from $commands" -a "doctor" -d "Diagnose installation issues"
complete -c scoop -n "not __fish_seen_subcommand_from $commands" -a "init" -d "Output shell initialization script"
complete -c scoop -n "not __fish_seen_subcommand_from $commands" -a "completions" -d "Output shell completion script"
complete -c scoop -n "not __fish_seen_subcommand_from $commands" -a "activate" -d "Activate a virtual environment"
complete -c scoop -n "not __fish_seen_subcommand_from $commands" -a "deactivate" -d "Deactivate current virtual environment"
complete -c scoop -n "not __fish_seen_subcommand_from $commands" -a "migrate" -d "Migrate environments from other tools"
complete -c scoop -n "not __fish_seen_subcommand_from $commands" -a "lang" -d "Set or show language preference"

# Options for 'list' (with duplicate prevention)
complete -c scoop -n "__fish_seen_subcommand_from list; and not __fish_contains_opt pythons" -l pythons -d "Show installed Python versions"
complete -c scoop -n "__fish_seen_subcommand_from list; and not __fish_contains_opt json" -l json -d "Output as JSON"
complete -c scoop -n "__fish_seen_subcommand_from list; and not __fish_contains_opt -s q quiet" -s q -l quiet -d "Suppress output"
complete -c scoop -n "__fish_seen_subcommand_from list; and not __fish_contains_opt no-color" -l no-color -d "Disable colored output"

# Options for 'use' (with duplicate prevention)
complete -c scoop -n "__fish_seen_subcommand_from use; and not __fish_contains_opt unset" -l unset -d "Remove version setting"
complete -c scoop -n "__fish_seen_subcommand_from use; and not __fish_contains_opt global" -l global -d "Set as global default"
complete -c scoop -n "__fish_seen_subcommand_from use; and not __fish_contains_opt link no-link" -l link -d "Create .venv symlink"
complete -c scoop -n "__fish_seen_subcommand_from use; and not __fish_contains_opt link no-link" -l no-link -d "Do not create .venv symlink"
complete -c scoop -n "__fish_seen_subcommand_from use; and not __fish_contains_opt -s q quiet" -s q -l quiet -d "Suppress output"
complete -c scoop -n "__fish_seen_subcommand_from use; and not __fish_contains_opt no-color" -l no-color -d "Disable colored output"

# Options for 'create' (with duplicate prevention)
complete -c scoop -n "__fish_seen_subcommand_from create; and not __fish_contains_opt force" -l force -d "Overwrite existing environment"
complete -c scoop -n "__fish_seen_subcommand_from create; and not __fish_contains_opt -s q quiet" -s q -l quiet -d "Suppress output"
complete -c scoop -n "__fish_seen_subcommand_from create; and not __fish_contains_opt no-color" -l no-color -d "Disable colored output"

# Options for 'remove' (with duplicate prevention)
complete -c scoop -n "__fish_seen_subcommand_from remove; and not __fish_contains_opt force" -l force -d "Skip confirmation"
complete -c scoop -n "__fish_seen_subcommand_from remove; and not __fish_contains_opt -s q quiet" -s q -l quiet -d "Suppress output"
complete -c scoop -n "__fish_seen_subcommand_from remove; and not __fish_contains_opt no-color" -l no-color -d "Disable colored output"

# Options for 'info' (with duplicate prevention)
complete -c scoop -n "__fish_seen_subcommand_from info; and not __fish_contains_opt json" -l json -d "Output as JSON"
complete -c scoop -n "__fish_seen_subcommand_from info; and not __fish_contains_opt all-packages" -l all-packages -d "Show all installed packages"
complete -c scoop -n "__fish_seen_subcommand_from info; and not __fish_contains_opt no-size" -l no-size -d "Skip directory size calculation"
complete -c scoop -n "__fish_seen_subcommand_from info; and not __fish_contains_opt -s q quiet" -s q -l quiet -d "Suppress output"
complete -c scoop -n "__fish_seen_subcommand_from info; and not __fish_contains_opt no-color" -l no-color -d "Disable colored output"

# Options for 'install' (with duplicate prevention, --latest/--stable mutually exclusive)
complete -c scoop -n "__fish_seen_subcommand_from install; and not __fish_contains_opt latest stable" -l latest -d "Install latest stable Python"
complete -c scoop -n "__fish_seen_subcommand_from install; and not __fish_contains_opt latest stable" -l stable -d "Install oldest fully-supported Python"
complete -c scoop -n "__fish_seen_subcommand_from install; and not __fish_contains_opt -s q quiet" -s q -l quiet -d "Suppress output"
complete -c scoop -n "__fish_seen_subcommand_from install; and not __fish_contains_opt no-color" -l no-color -d "Disable colored output"

# Options for 'uninstall' (with duplicate prevention)
complete -c scoop -n "__fish_seen_subcommand_from uninstall; and not __fish_contains_opt -s q quiet" -s q -l quiet -d "Suppress output"
complete -c scoop -n "__fish_seen_subcommand_from uninstall; and not __fish_contains_opt no-color" -l no-color -d "Disable colored output"

# Options for 'doctor' (with duplicate prevention)
complete -c scoop -n "__fish_seen_subcommand_from doctor; and not __fish_contains_opt -s v verbose" -s v -l verbose -d "Increase verbosity"
complete -c scoop -n "__fish_seen_subcommand_from doctor; and not __fish_contains_opt json" -l json -d "Output as JSON"
complete -c scoop -n "__fish_seen_subcommand_from doctor; and not __fish_contains_opt -s q quiet" -s q -l quiet -d "Suppress output"
complete -c scoop -n "__fish_seen_subcommand_from doctor; and not __fish_contains_opt no-color" -l no-color -d "Disable colored output"

# Dynamic completions: virtual environment names
complete -c scoop -n "__fish_seen_subcommand_from use remove info activate" -a "(command scoop list --bare 2>/dev/null)" -d "Virtual environment"

# Dynamic completions: Python versions for uninstall
# Note: scoop list --pythons --bare already returns unique, sorted versions
complete -c scoop -n "__fish_seen_subcommand_from uninstall" -a "(command scoop list --pythons --bare 2>/dev/null)" -d "Python version"

# Dynamic completions: Python versions for create (second positional arg)
complete -c scoop -n "__fish_seen_subcommand_from create; and __fish_is_nth_token 3" -a "(command scoop list --pythons --bare 2>/dev/null)" -d "Python version"

# Shell types for init/completions
complete -c scoop -n "__fish_seen_subcommand_from init completions" -a "bash zsh fish powershell" -d "Shell type"

# Options for 'lang' (with duplicate prevention)
complete -c scoop -n "__fish_seen_subcommand_from lang; and not __fish_contains_opt list" -l list -d "List supported languages"
complete -c scoop -n "__fish_seen_subcommand_from lang; and not __fish_contains_opt reset" -l reset -d "Reset to system default"
complete -c scoop -n "__fish_seen_subcommand_from lang; and not __fish_contains_opt json" -l json -d "Output as JSON"

# Language codes for lang command
complete -c scoop -n "__fish_seen_subcommand_from lang" -a "en" -d "English"
complete -c scoop -n "__fish_seen_subcommand_from lang" -a "ko" -d "Korean"

# Subcommands for 'migrate'
complete -c scoop -n "__fish_seen_subcommand_from migrate; and not __fish_seen_subcommand_from list all @env" -a "list" -d "List environments available for migration"
complete -c scoop -n "__fish_seen_subcommand_from migrate; and not __fish_seen_subcommand_from list all @env" -a "all" -d "Migrate all environments"
complete -c scoop -n "__fish_seen_subcommand_from migrate; and not __fish_seen_subcommand_from list all @env" -a "@env" -d "Migrate a specific environment"
"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // Real Test: Syntax Validation with fish -n
    // =========================================================================

    /// Validates that the generated script has valid fish syntax.
    /// This is a REAL test - it actually runs fish to check the script.
    #[test]
    #[cfg(unix)]
    fn test_init_script_has_valid_fish_syntax() {
        let script = init_script();

        // Use fish -n for syntax checking (parse only, don't execute)
        let output = std::process::Command::new("fish")
            .arg("-n") // syntax check only
            .arg("-c")
            .arg(script)
            .output();

        match output {
            Ok(result) => {
                assert!(
                    result.status.success(),
                    "Fish script has syntax errors:\n{}",
                    String::from_utf8_lossy(&result.stderr)
                );
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                // fish not available, skip test
                eprintln!("Skipping fish syntax test: fish not found");
            }
            Err(e) => panic!("Failed to run fish: {}", e),
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
    fn test_init_script_defines_required_functions() {
        let script = init_script();

        // These functions MUST exist for the shell integration to work
        assert!(
            script.contains("function scoop"),
            "Script missing wrapper function"
        );
        assert!(
            script.contains("function _scoop_hook --on-variable PWD"),
            "Script missing auto-activate hook"
        );
    }

    #[test]
    fn test_init_script_registers_pwd_hook() {
        let script = init_script();

        // PWD hook must be registered for auto-activation
        assert!(
            script.contains("--on-variable PWD"),
            "Script must register PWD hook for auto-activation"
        );
    }

    #[test]
    fn test_init_script_registers_completion() {
        let script = init_script();

        // Must register completion function
        assert!(
            script.contains("complete -c scoop"),
            "Script must register fish completion"
        );
    }

    // =========================================================================
    // Feature Tests: Verify key behaviors
    // =========================================================================

    #[test]
    fn test_init_script_checks_scoop_no_auto() {
        let script = init_script();

        // SCOOP_NO_AUTO must be checked to allow disabling auto-activation
        assert!(
            script.contains("SCOOP_NO_AUTO"),
            "Script must check SCOOP_NO_AUTO environment variable"
        );

        // Must use `set -q` to check if variable is set
        assert!(
            script.contains("not set -q SCOOP_NO_AUTO"),
            "Script must use 'set -q' to check SCOOP_NO_AUTO"
        );
    }

    #[test]
    fn test_init_script_has_option_deduplication() {
        let script = init_script();

        // Option deduplication pattern: `not __fish_contains_opt`
        assert!(
            script.contains("not __fish_contains_opt"),
            "Script must use __fish_contains_opt for option deduplication"
        );

        // Verify common options have deduplication
        assert!(
            script.contains("not __fish_contains_opt json"),
            "Script must prevent duplicate --json option"
        );
        assert!(
            script.contains("not __fish_contains_opt -s q quiet"),
            "Script must prevent duplicate -q/--quiet option"
        );
    }

    #[test]
    fn test_init_script_has_dynamic_completions() {
        let script = init_script();

        // Dynamic completions for environment names
        assert!(
            script.contains("scoop list --bare"),
            "Script must provide dynamic env name completions"
        );

        // Dynamic completions for Python versions
        assert!(
            script.contains("scoop list --pythons --bare"),
            "Script must provide dynamic Python version completions"
        );
    }

    #[test]
    fn test_init_script_has_mutually_exclusive_options() {
        let script = init_script();

        // --link and --no-link are mutually exclusive
        assert!(
            script.contains("not __fish_contains_opt link no-link"),
            "Script must make --link and --no-link mutually exclusive"
        );

        // --latest and --stable are mutually exclusive
        assert!(
            script.contains("not __fish_contains_opt latest stable"),
            "Script must make --latest and --stable mutually exclusive"
        );
    }
}
