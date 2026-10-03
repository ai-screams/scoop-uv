//! Fish shell integration
//!
//! Provides Fish shell support for scuv, including:
//! - Wrapper function for `scuv` command
//! - Auto-activate hook via `--on-variable PWD`
//! - Tab completion with option deduplication

use crate::{file_resolution_check, scoop_version_check};

/// Generate fish initialization script.
///
/// Returns a static string containing the Fish shell integration script.
/// This script should be loaded in the user's `config.fish`. Prefer piping to
/// `source` over `eval (...)`: fish's command substitution splits multi-line
/// output into separate arguments, which `eval` then rejoins with spaces —
/// collapsing the newlines this script's function/switch syntax relies on,
/// so the `scuv` function silently fails to get defined. The wrapper and the
/// hook below use the same `| source` idiom for `scuv activate` /
/// `deactivate` / `shell` output, and pass `--shell fish` explicitly: fish
/// does not export `FISH_VERSION`, so the binary cannot detect fish from a
/// child process and would print bash syntax.
///
/// ```fish
/// scuv init fish | source
/// ```
///
/// # Examples
///
/// ```
/// let script = scoop_uv::shell::fish::init_script();
///
/// // Script contains the wrapper function
/// assert!(script.contains("function scuv"));
///
/// // Script contains the auto-activate hook
/// assert!(script.contains("function _scuv_hook --on-variable PWD"));
///
/// // Script contains completion definitions
/// assert!(script.contains("complete -c scuv"));
/// ```
pub fn init_script() -> &'static str {
    concat!(
        // The `scuv` wrapper function (src/shell/scripts/fish_wrapper.fish).
        include_str!("scripts/fish_wrapper.fish"),
        // The auto-activate hook, assembled from the shared checks.
        // The blank line after the wrapper lives here: a script file
        // cannot end in one (end-of-file-fixer trims it).
        r#"
# Auto-activate hook
function _scuv_hook --on-variable PWD
"#,
        scoop_version_check!(fish),
        file_resolution_check!(fish),
        r#"
end

# Set up auto-activate on startup
if not set -q SCUV_NO_AUTO
    _scuv_hook
end

"#,
        // Tab completion (src/shell/scripts/fish_completion.fish).
        include_str!("scripts/fish_completion.fish"),
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
            script.contains("function scuv"),
            "Script missing wrapper function"
        );
        assert!(
            script.contains("function _scuv_hook --on-variable PWD"),
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
            script.contains("complete -c scuv"),
            "Script must register fish completion"
        );
    }

    // =========================================================================
    // Feature Tests: Verify key behaviors
    // =========================================================================

    #[test]
    fn test_init_script_checks_scuv_no_auto() {
        let script = init_script();

        // SCUV_NO_AUTO must be checked to allow disabling auto-activation
        assert!(
            script.contains("SCUV_NO_AUTO"),
            "Script must check SCUV_NO_AUTO environment variable"
        );

        // Must use `set -q` to check if variable is set
        assert!(
            script.contains("not set -q SCUV_NO_AUTO"),
            "Script must use 'set -q' to check SCUV_NO_AUTO"
        );

        // Fails if the scoop-era SCOOP_NO_AUTO read comes back.
        assert!(!script.contains("SCOOP_NO_AUTO"));
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
            script.contains("scuv list --bare"),
            "Script must provide dynamic env name completions"
        );

        // Dynamic completions for Python versions
        assert!(
            script.contains("scuv list --pythons --bare"),
            "Script must provide dynamic Python version completions"
        );
    }

    /// fish splits a command substitution on newlines and `eval` rejoins the
    /// pieces with spaces, so `eval (command scuv activate ...)` collapses the
    /// multi-line `if ... end` script and fails with "Missing end". Every
    /// activation path must pipe the output to `source` instead, and must
    /// pass `--shell fish` explicitly because fish does not export
    /// FISH_VERSION, so `detect_shell` cannot see it from a child process.
    /// Fails if any `eval (` comes back or a call site drops `--shell fish`.
    #[test]
    fn init_script_sources_activation_output_with_explicit_shell() {
        let script = init_script();
        assert!(
            !script.contains("eval ("),
            "fish script must not eval command substitutions"
        );
        assert!(script.contains(r#"command scuv activate --shell fish "$arg" | source"#));
        assert!(script.contains("command scuv $argv[1] --shell fish $argv[2..-1] | source"));
    }

    /// The pass-through arm must not add a second `--shell` when the user
    /// gave one (clap rejects duplicates), must treat only a whole argument
    /// as a help/version flag, and `use system` must deactivate rather than
    /// try to activate the reserved name. Fails if any of the three guards
    /// is dropped.
    #[test]
    fn init_script_guards_explicit_shell_help_flags_and_use_system() {
        let script = init_script();
        assert!(script.contains("else if string match -q -- '--shell*' $argv"));
        assert!(script.contains("string match -qr -- '^(-h|--help|-V|--version)$' $argv"));
        assert!(script.contains("if test \"$arg\" = system\n                            command scuv deactivate --shell fish | source"));
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
        let re = regex::Regex::new(r#"__fish_seen_subcommand_from lang" -a "([^"]+)""#).unwrap();
        let mut found: Vec<&str> = re
            .captures_iter(script)
            .map(|c| c.get(1).unwrap().as_str())
            .collect();
        assert!(!found.is_empty(), "fish script must complete `lang` codes");
        let mut expected: Vec<&str> = crate::i18n::SUPPORTED_LANGS
            .iter()
            .map(|(c, _)| *c)
            .collect();
        expected.sort_unstable();
        found.sort_unstable();
        assert_eq!(found, expected);
    }

    /// The transitional `scoop` forwarder went with 0.16.0; the init script
    /// must not define a `scoop` function again (scoop.sh coexistence).
    /// Fails if a `function scoop` is reintroduced.
    #[test]
    fn init_script_never_defines_scoop() {
        let s = init_script();
        assert!(!s.contains("function scoop"));
        assert!(!s.contains("alias scoop"));
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
