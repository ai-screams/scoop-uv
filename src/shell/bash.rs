//! Bash shell integration

use crate::{file_resolution_check, scoop_version_check};

/// Generate bash initialization script
pub fn init_script() -> &'static str {
    concat!(
        // The `scuv` wrapper function (src/shell/scripts/bash_wrapper.bash).
        include_str!("scripts/bash_wrapper.bash"),
        // The auto-activate hook, assembled from the shared checks.
        // The blank line after the wrapper lives here: a script file
        // cannot end in one (end-of-file-fixer trims it).
        r#"
# Auto-activate hook
_scuv_hook() {"#,
        scoop_version_check!(bash),
        file_resolution_check!(bash),
        r#"
}

# Set up PROMPT_COMMAND for auto-activate
if [[ -z "$SCUV_NO_AUTO" ]]; then
    if [[ -z "$PROMPT_COMMAND" ]]; then
        PROMPT_COMMAND="_scuv_hook"
    else
        PROMPT_COMMAND="_scuv_hook;$PROMPT_COMMAND"
    fi
fi

# Run hook on startup
_scuv_hook

"#,
        // Tab completion (src/shell/scripts/bash_completion.bash).
        include_str!("scripts/bash_completion.bash"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // Real Test: Syntax Validation with bash -n
    // =========================================================================

    /// Validates that the generated script has valid bash syntax.
    /// This is a REAL test - it actually runs bash to check the script.
    #[test]
    #[cfg(unix)]
    fn test_init_script_has_valid_bash_syntax() {
        let script = init_script();

        // Use bash -n for syntax checking (parse only, don't execute)
        let output = std::process::Command::new("bash")
            .arg("-n") // syntax check only
            .arg("-c")
            .arg(script)
            .output();

        match output {
            Ok(result) => {
                assert!(
                    result.status.success(),
                    "Bash script has syntax errors:\n{}",
                    String::from_utf8_lossy(&result.stderr)
                );
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                // bash not available, skip test
                eprintln!("Skipping bash syntax test: bash not found");
            }
            Err(e) => panic!("Failed to run bash: {}", e),
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
        let required_functions = ["scuv()", "_scuv_hook()", "_scuv_complete()"];

        for func in required_functions {
            assert!(
                script.contains(func),
                "Script missing required function: {}",
                func
            );
        }
    }

    #[test]
    fn test_init_script_registers_prompt_hook() {
        let script = init_script();

        // PROMPT_COMMAND must be set for auto-activation
        assert!(
            script.contains("PROMPT_COMMAND"),
            "Script must register PROMPT_COMMAND for auto-activation"
        );
    }

    #[test]
    fn test_init_script_registers_completion() {
        let script = init_script();

        // Must register completion function
        assert!(
            script.contains("complete -o nosort -F _scuv_complete scuv"),
            "Script must register bash completion"
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
        let re = regex::Regex::new(
            r#"# Complete language codes\s*\n\s*COMPREPLY=\(\$\(compgen -W "([^"]+)""#,
        )
        .unwrap();
        let caps = re
            .captures(script)
            .expect("bash script must complete `lang` with compgen -W");
        let mut found: Vec<&str> = caps.get(1).unwrap().as_str().split_whitespace().collect();
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

    // =========================================================================
    // Real Test: Best Practices Validation with shellcheck
    // =========================================================================

    /// Validates that the generated script follows shell best practices.
    /// shellcheck catches common issues like:
    /// - Quoting problems (SC2086)
    /// - Useless use of cat (SC2002)
    /// - Deprecated syntax
    #[test]
    #[cfg(unix)]
    fn test_init_script_passes_shellcheck() {
        let script = init_script();

        // Write script to temp file (shellcheck requires file input)
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(temp_file.path(), script).unwrap();

        let output = std::process::Command::new("shellcheck")
            .arg("--shell=bash")
            .arg("--severity=warning") // Only warnings and above
            // SC2207: COMPREPLY=($(compgen ...)) is standard bash completion idiom
            .arg("--exclude=SC2207")
            .arg(temp_file.path())
            .output();

        match output {
            Ok(result) => {
                assert!(
                    result.status.success(),
                    "shellcheck found issues in bash script:\n{}",
                    String::from_utf8_lossy(&result.stdout)
                );
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                // shellcheck not installed, skip test
                eprintln!(
                    "Skipping shellcheck test: shellcheck not found (install: brew install shellcheck)"
                );
            }
            Err(e) => panic!("Failed to run shellcheck: {}", e),
        }
    }
}
