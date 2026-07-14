//! Bash shell integration

use crate::{file_resolution_check, scoop_version_check};

/// Generate bash initialization script
pub fn init_script() -> &'static str {
    concat!(
        r#"# scuv shell integration for bash

# Wrapper function for scuv
scuv() {
    local command="${1:-}"

    case "$command" in
        use)
            command scuv "$@"
            local ret=$?
            if [[ $ret -eq 0 ]]; then
                shift  # remove 'use'
                local name=""
                for arg in "$@"; do
                    case "$arg" in
                        -*) ;;  # skip options
                        *) name="$arg"; break ;;
                    esac
                done
                if [[ -n "$name" ]]; then
                    # 'use' above already warned about any legacy config; don't warn twice
                    eval "$(SCUV_SUPPRESS_DEPRECATION=1 command scuv activate "$name")"
                fi
            fi
            return $ret
            ;;
        activate|deactivate|shell)
            # Pass through help/version flags without eval
            if [[ "$*" == *--help* ]] || [[ "$*" == *-h* ]] || [[ "$*" == *--version* ]] || [[ "$*" == *-V* ]]; then
                command scuv "$@"
            else
                eval "$(command scuv "$@")"
            fi
            ;;
        *)
            command scuv "$@"
            ;;
    esac
}

# Auto-activate hook
_scuv_hook() {"#,
        scoop_version_check!(bash),
        file_resolution_check!(bash),
        r#"
}

# Set up PROMPT_COMMAND for auto-activate
# DEPRECATION(0.16.0): drop the legacy SCOOP_NO_AUTO fallback check.
if [[ -z "$SCUV_NO_AUTO" && -z "$SCOOP_NO_AUTO" ]]; then
    if [[ -z "$PROMPT_COMMAND" ]]; then
        PROMPT_COMMAND="_scuv_hook"
    else
        PROMPT_COMMAND="_scuv_hook;$PROMPT_COMMAND"
    fi
fi

# Run hook on startup
_scuv_hook

# Bash completion for scuv
_scuv_complete() {
    local cur cmd i
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"

    # Get subcommand (COMP_WORDS[1])
    cmd=""
    if [[ ${COMP_CWORD} -ge 1 ]]; then
        cmd="${COMP_WORDS[1]}"
    fi

    # First argument: complete subcommands
    if [[ ${COMP_CWORD} -eq 1 ]]; then
        COMPREPLY=($(compgen -W "list use create remove info install uninstall doctor init completions activate deactivate shell migrate lang" -- "$cur"))
        return
    fi

    # Option completion (starts with -)
    if [[ "$cur" == -* ]]; then
        case "$cmd" in
            list)
                # Special-case `--sort` value completion: when the user
                # has typed `--sort=` or `--sort <TAB>`, offer the
                # enum values directly instead of the option list.
                local prev_word="${COMP_WORDS[COMP_CWORD-1]:-}"
                if [[ "$cur" == --sort=* ]]; then
                    local val="${cur#--sort=}"
                    COMPREPLY=($(compgen -W "name created last-used" -P "--sort=" -- "$val"))
                    return 0
                fi
                if [[ "$prev_word" == "--sort" ]]; then
                    COMPREPLY=($(compgen -W "name created last-used" -- "$cur"))
                    return 0
                fi
                local opts="--pythons --sort --json -q --quiet --no-color --help"
                for word in "${COMP_WORDS[@]}"; do
                    case "$word" in
                        --pythons) opts="${opts//--pythons }" ;;
                        --sort|--sort=*) opts="${opts//--sort }" ;;
                        --json) opts="${opts//--json }" ;;
                        -q|--quiet) opts="${opts//-q }"; opts="${opts//--quiet }" ;;
                        --no-color) opts="${opts//--no-color }" ;;
                    esac
                done
                COMPREPLY=($(compgen -W "$opts" -- "$cur"))
                ;;
            doctor)
                local opts="-v --verbose -q --quiet --json --no-color --help"
                # Filter out already used options
                for word in "${COMP_WORDS[@]}"; do
                    case "$word" in
                        -v|--verbose) opts="${opts//-v }"; opts="${opts//--verbose }" ;;
                        -q|--quiet) opts="${opts//-q }"; opts="${opts//--quiet }" ;;
                        --json) opts="${opts//--json }" ;;
                        --no-color) opts="${opts//--no-color }" ;;
                    esac
                done
                COMPREPLY=($(compgen -W "$opts" -- "$cur"))
                ;;
            create)
                local opts="--force -q --quiet --no-color --help"
                for word in "${COMP_WORDS[@]}"; do
                    case "$word" in
                        --force) opts="${opts//--force }" ;;
                        -q|--quiet) opts="${opts//-q }"; opts="${opts//--quiet }" ;;
                        --no-color) opts="${opts//--no-color }" ;;
                    esac
                done
                COMPREPLY=($(compgen -W "$opts" -- "$cur"))
                ;;
            use)
                local opts="--unset --link --global --no-link -q --quiet --no-color --help"
                for word in "${COMP_WORDS[@]}"; do
                    case "$word" in
                        --unset) opts="${opts//--unset }" ;;
                        --global) opts="${opts//--global }" ;;
                        --link|--no-link) opts="${opts//--link }"; opts="${opts//--no-link }" ;;
                        -q|--quiet) opts="${opts//-q }"; opts="${opts//--quiet }" ;;
                        --no-color) opts="${opts//--no-color }" ;;
                    esac
                done
                COMPREPLY=($(compgen -W "$opts" -- "$cur"))
                ;;
            remove)
                local opts="--force -q --quiet --no-color --help"
                for word in "${COMP_WORDS[@]}"; do
                    case "$word" in
                        --force) opts="${opts//--force }" ;;
                        -q|--quiet) opts="${opts//-q }"; opts="${opts//--quiet }" ;;
                        --no-color) opts="${opts//--no-color }" ;;
                    esac
                done
                COMPREPLY=($(compgen -W "$opts" -- "$cur"))
                ;;
            install)
                local opts="--latest --stable -q --quiet --no-color --help"
                for word in "${COMP_WORDS[@]}"; do
                    case "$word" in
                        --latest|--stable) opts="${opts//--latest }"; opts="${opts//--stable }" ;;
                        -q|--quiet) opts="${opts//-q }"; opts="${opts//--quiet }" ;;
                        --no-color) opts="${opts//--no-color }" ;;
                    esac
                done
                COMPREPLY=($(compgen -W "$opts" -- "$cur"))
                ;;
            uninstall)
                local opts="-q --quiet --no-color --help"
                for word in "${COMP_WORDS[@]}"; do
                    case "$word" in
                        -q|--quiet) opts="${opts//-q }"; opts="${opts//--quiet }" ;;
                        --no-color) opts="${opts//--no-color }" ;;
                    esac
                done
                COMPREPLY=($(compgen -W "$opts" -- "$cur"))
                ;;
            info)
                local opts="--json --all-packages --no-size -q --quiet --no-color --help"
                for word in "${COMP_WORDS[@]}"; do
                    case "$word" in
                        --json) opts="${opts//--json }" ;;
                        --all-packages) opts="${opts//--all-packages }" ;;
                        --no-size) opts="${opts//--no-size }" ;;
                        -q|--quiet) opts="${opts//-q }"; opts="${opts//--quiet }" ;;
                        --no-color) opts="${opts//--no-color }" ;;
                    esac
                done
                COMPREPLY=($(compgen -W "$opts" -- "$cur"))
                ;;
            init|completions)
                COMPREPLY=($(compgen -W "--help" -- "$cur"))
                ;;
            lang)
                local opts="--list --reset --json --help"
                for word in "${COMP_WORDS[@]}"; do
                    case "$word" in
                        --list) opts="${opts//--list }" ;;
                        --reset) opts="${opts//--reset }" ;;
                        --json) opts="${opts//--json }" ;;
                    esac
                done
                COMPREPLY=($(compgen -W "$opts" -- "$cur"))
                ;;
            shell)
                local opts="--unset -q --quiet --no-color --help"
                for word in "${COMP_WORDS[@]}"; do
                    case "$word" in
                        --unset) opts="${opts//--unset }" ;;
                        -q|--quiet) opts="${opts//-q }"; opts="${opts//--quiet }" ;;
                        --no-color) opts="${opts//--no-color }" ;;
                    esac
                done
                COMPREPLY=($(compgen -W "$opts" -- "$cur"))
                ;;
            migrate)
                local opts="--help"
                COMPREPLY=($(compgen -W "$opts" -- "$cur"))
                ;;
        esac
        return
    fi

    # Argument completion (by subcommand)
    case "$cmd" in
        use|remove|info|activate|shell)
            # Check if env name already provided
            local has_arg=false
            for ((i=2; i<COMP_CWORD; i++)); do
                if [[ "${COMP_WORDS[i]}" != -* ]]; then
                    has_arg=true
                    break
                fi
            done
            if [[ "$has_arg" == false ]]; then
                COMPREPLY=($(compgen -W "$(command scuv list --bare 2>/dev/null)" -- "$cur"))
            fi
            ;;
        uninstall)
            COMPREPLY=($(compgen -W "$(command scuv list --pythons --bare 2>/dev/null | sort -u)" -- "$cur"))
            ;;
        init|completions)
            COMPREPLY=($(compgen -W "bash zsh fish powershell" -- "$cur"))
            ;;
        create)
            # First arg: name, second arg: python version
            local arg_count=0
            for ((i=2; i<COMP_CWORD; i++)); do
                if [[ "${COMP_WORDS[i]}" != -* ]]; then
                    ((arg_count++))
                fi
            done
            if [[ $arg_count -eq 1 ]]; then
                # Second positional arg: python version
                COMPREPLY=($(compgen -W "$(command scuv list --pythons --bare 2>/dev/null | sort -u)" -- "$cur"))
            fi
            ;;
        lang)
            # Complete language codes
            COMPREPLY=($(compgen -W "en ko ja pt-BR" -- "$cur"))
            ;;
        migrate)
            # Complete migrate subcommands
            COMPREPLY=($(compgen -W "list all @env" -- "$cur"))
            ;;
    esac
}
complete -o nosort -F _scuv_complete scuv

# DEPRECATION(0.16.0): transitional forwarder; never emitted for PowerShell.
if ! command -v scoop >/dev/null 2>&1; then
    scoop() {
        echo "warning: 'scoop' has been renamed to 'scuv'; this alias will be removed in v0.16.0" >&2
        scuv "$@"
    }
fi
"#
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

    /// The auto-activate gate must honor the new variable AND the legacy one
    /// (deprecated read, removed in 0.16.0) — fish/powershell have the same
    /// test; this pins bash/zsh symmetrically.
    #[test]
    fn init_script_checks_both_no_auto_variables() {
        let script = init_script();
        assert!(
            script.contains(r#"[[ -z "$SCUV_NO_AUTO" && -z "$SCOOP_NO_AUTO" ]]"#),
            "auto-activate gate must check SCUV_NO_AUTO with legacy SCOOP_NO_AUTO fallback"
        );
    }

    /// The chained use→activate call must suppress duplicate deprecation
    /// warnings (each chained call is a fresh process).
    #[test]
    fn init_script_suppresses_duplicate_deprecation_in_use_chain() {
        assert!(init_script().contains("SCUV_SUPPRESS_DEPRECATION"));
    }

    #[test]
    fn init_script_defines_deprecated_scoop_forwarder() {
        assert!(init_script().contains("scoop() {"));
        assert!(init_script().contains("renamed to 'scuv'"));
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
