//! Zsh shell integration

use crate::{file_resolution_check, scoop_version_check};

/// Generate zsh initialization script
pub fn init_script() -> &'static str {
    concat!(
        // The `scuv` wrapper function (src/shell/scripts/zsh_wrapper.zsh).
        include_str!("scripts/zsh_wrapper.zsh"),
        // The auto-activate hook, assembled from the shared checks.
        // The blank line after the wrapper lives here: a script file
        // cannot end in one (end-of-file-fixer trims it).
        r#"
# Auto-activate hook
_scuv_hook() {
"#,
        scoop_version_check!(zsh),
        file_resolution_check!(zsh),
        r#"
}

# Set up chpwd hook for auto-activate
if [[ -z "$SCUV_NO_AUTO" ]]; then
    autoload -Uz add-zsh-hook
    add-zsh-hook chpwd _scuv_hook
fi

# Run hook on startup
_scuv_hook

"#,
        // Tab completion (src/shell/scripts/zsh_completion.zsh).
        include_str!("scripts/zsh_completion.zsh"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // Real Test: Syntax Validation with zsh -n
    // =========================================================================

    /// Validates that the generated script has valid zsh syntax.
    /// This is a REAL test - it actually runs zsh to check the script.
    #[test]
    #[cfg(unix)]
    fn test_init_script_has_valid_zsh_syntax() {
        let script = init_script();

        // Use zsh -n for syntax checking (parse only, don't execute)
        let output = std::process::Command::new("zsh")
            .arg("-n") // syntax check only
            .arg("-c")
            .arg(script)
            .output();

        match output {
            Ok(result) => {
                assert!(
                    result.status.success(),
                    "Zsh script has syntax errors:\n{}",
                    String::from_utf8_lossy(&result.stderr)
                );
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                // zsh not available, skip test
                eprintln!("Skipping zsh syntax test: zsh not found");
            }
            Err(e) => panic!("Failed to run zsh: {}", e),
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
        let required_functions = ["scuv()", "_scuv_hook()", "_scuv()"];

        for func in required_functions {
            assert!(
                script.contains(func),
                "Script missing required function: {}",
                func
            );
        }
    }

    #[test]
    fn test_init_script_registers_chpwd_hook() {
        let script = init_script();

        // zsh uses chpwd hook for directory change detection
        assert!(
            script.contains("add-zsh-hook chpwd _scuv_hook"),
            "Script must register chpwd hook for auto-activation"
        );
    }

    #[test]
    fn test_init_script_registers_completion() {
        let script = init_script();

        // Must register completion function with compdef
        assert!(
            script.contains("compdef _scuv scuv"),
            "Script must register zsh completion"
        );
    }

    /// The auto-activate gate reads `SCUV_NO_AUTO` only — fish/powershell
    /// have the same test; this pins bash/zsh symmetrically.
    /// Fails if the scoop-era `SCOOP_NO_AUTO` read comes back.
    #[test]
    fn init_script_checks_only_scuv_no_auto() {
        let script = init_script();
        assert!(script.contains(r#"[[ -z "$SCUV_NO_AUTO" ]]"#));
        assert!(!script.contains("SCOOP_NO_AUTO"));
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
        let line = script
            .lines()
            .find(|l| l.contains("langs=("))
            .expect("zsh script must define a langs=(...) array");
        // Entries are 'code:Description'; descriptions may contain spaces and parens.
        let re = regex::Regex::new(r"'([^':]+):[^']*'").unwrap();
        let mut found: Vec<&str> = re
            .captures_iter(line)
            .map(|c| c.get(1).unwrap().as_str())
            .collect();
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
    /// Fails if a `scoop()` function is reintroduced.
    #[test]
    fn init_script_never_defines_scoop() {
        let s = init_script();
        assert!(!s.contains("scoop()"));
        assert!(!s.contains("function scoop"));
        assert!(!s.contains("alias scoop"));
    }

    #[test]
    fn test_init_script_loads_zsh_hooks_module() {
        let script = init_script();

        // Must autoload zsh hooks module
        assert!(
            script.contains("autoload -Uz add-zsh-hook"),
            "Script must load zsh hooks module"
        );
    }

    // =========================================================================
    // Real Test: Best Practices Validation with shellcheck
    // =========================================================================

    /// Validates that the generated script follows shell best practices.
    /// Note: shellcheck uses bash mode for zsh (no native zsh support),
    /// but catches most common issues.
    #[test]
    #[cfg(unix)]
    fn test_init_script_passes_shellcheck() {
        let script = init_script();

        // Write script to temp file (shellcheck requires file input)
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(temp_file.path(), script).unwrap();

        // shellcheck doesn't have native zsh support, but bash mode catches most issues
        // Use --exclude for zsh-specific constructs that bash doesn't understand
        let output = std::process::Command::new("shellcheck")
            .arg("--shell=bash")
            .arg("--severity=warning")
            // Exclude zsh-specific constructs:
            // SC1087: $line[1] array syntax (zsh doesn't require braces)
            // SC2034: Unused variable (zsh uses typeset -A)
            // SC2154: Variable referenced but not assigned (zsh completion vars)
            // SC2168: 'local' outside function (zsh completion context)
            // SC2206: Quote to prevent word splitting (zsh handles this differently)
            // SC2207: COMPREPLY=($(compgen ...)) is standard completion idiom
            // SC2296: ${(f)...} parameter expansion flags (zsh-specific)
            // SC3030: Array syntax (zsh-specific)
            // SC3057: Associative array syntax (zsh-specific)
            .arg("--exclude=SC1087,SC2034,SC2154,SC2168,SC2206,SC2207,SC2296,SC3030,SC3057")
            .arg(temp_file.path())
            .output();

        match output {
            Ok(result) => {
                assert!(
                    result.status.success(),
                    "shellcheck found issues in zsh script:\n{}",
                    String::from_utf8_lossy(&result.stdout)
                );
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                eprintln!(
                    "Skipping shellcheck test: shellcheck not found (install: brew install shellcheck)"
                );
            }
            Err(e) => panic!("Failed to run shellcheck: {}", e),
        }
    }
}
