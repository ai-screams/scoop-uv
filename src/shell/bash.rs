//! Bash shell integration

/// Generate bash initialization script
pub fn init_script() -> &'static str {
    r#"# scoop shell integration for bash

# Wrapper function for scoop
scoop() {
    local command="${1:-}"

    case "$command" in
        use)
            command scoop "$@"
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
                    eval "$(command scoop activate "$name")"
                fi
            fi
            return $ret
            ;;
        activate|deactivate)
            eval "$(command scoop "$@")"
            ;;
        *)
            command scoop "$@"
            ;;
    esac
}

# Auto-activate hook
_scoop_hook() {
    local env_name
    env_name="$(command scoop resolve 2>/dev/null)"

    if [[ -n "$env_name" && "$env_name" != "$SCOOP_ACTIVE" ]]; then
        eval "$(command scoop activate "$env_name")"
    elif [[ -z "$env_name" && -n "$SCOOP_ACTIVE" ]]; then
        eval "$(command scoop deactivate)"
    fi
}

# Set up PROMPT_COMMAND for auto-activate
if [[ -z "$SCOOP_NO_AUTO" ]]; then
    if [[ -z "$PROMPT_COMMAND" ]]; then
        PROMPT_COMMAND="_scoop_hook"
    else
        PROMPT_COMMAND="_scoop_hook;$PROMPT_COMMAND"
    fi
fi

# Run hook on startup
_scoop_hook

# Bash completion for scoop
_scoop_complete() {
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
        COMPREPLY=($(compgen -W "list create use remove install uninstall init completions activate deactivate" -- "$cur"))
        return
    fi

    # Option completion (starts with -)
    if [[ "$cur" == -* ]]; then
        case "$cmd" in
            list)
                COMPREPLY=($(compgen -W "--pythons --help" -- "$cur"))
                ;;
            create)
                COMPREPLY=($(compgen -W "--force --help" -- "$cur"))
                ;;
            use)
                COMPREPLY=($(compgen -W "--global --link --no-link --help" -- "$cur"))
                ;;
            remove)
                COMPREPLY=($(compgen -W "--force --help" -- "$cur"))
                ;;
            install)
                COMPREPLY=($(compgen -W "--latest --stable --help" -- "$cur"))
                ;;
            uninstall)
                COMPREPLY=($(compgen -W "--help" -- "$cur"))
                ;;
            init|completions)
                COMPREPLY=($(compgen -W "--help" -- "$cur"))
                ;;
        esac
        return
    fi

    # Argument completion (by subcommand)
    case "$cmd" in
        use|remove|activate)
            # Check if env name already provided
            local has_arg=false
            for ((i=2; i<COMP_CWORD; i++)); do
                if [[ "${COMP_WORDS[i]}" != -* ]]; then
                    has_arg=true
                    break
                fi
            done
            if [[ "$has_arg" == false ]]; then
                COMPREPLY=($(compgen -W "$(command scoop list --bare 2>/dev/null)" -- "$cur"))
            fi
            ;;
        uninstall)
            COMPREPLY=($(compgen -W "$(command scoop list --pythons --bare 2>/dev/null | sort -u)" -- "$cur"))
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
                COMPREPLY=($(compgen -W "$(command scoop list --pythons --bare 2>/dev/null | sort -u)" -- "$cur"))
            fi
            ;;
    esac
}
complete -F _scoop_complete scoop
"#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_script_not_empty() {
        let script = init_script();
        assert!(!script.is_empty());
    }

    #[test]
    fn test_init_script_contains_wrapper_function() {
        let script = init_script();
        assert!(script.contains("scoop()"));
        assert!(script.contains("case \"$command\" in"));
    }

    #[test]
    fn test_init_script_contains_hook_function() {
        let script = init_script();
        assert!(script.contains("_scoop_hook()"));
    }

    #[test]
    fn test_init_script_handles_use_command() {
        let script = init_script();
        assert!(script.contains("use)"));
        assert!(script.contains("command scoop activate"));
    }

    #[test]
    fn test_init_script_handles_activate_deactivate() {
        let script = init_script();
        assert!(script.contains("activate|deactivate)"));
        assert!(script.contains("eval"));
    }

    #[test]
    fn test_init_script_sets_prompt_command() {
        let script = init_script();
        assert!(script.contains("PROMPT_COMMAND"));
        assert!(script.contains("_scoop_hook"));
    }

    #[test]
    fn test_init_script_respects_no_auto_var() {
        let script = init_script();
        assert!(script.contains("SCOOP_NO_AUTO"));
    }

    #[test]
    fn test_init_script_contains_completion() {
        let script = init_script();
        assert!(script.contains("_scoop_complete()"));
        assert!(script.contains("complete -F _scoop_complete scoop"));
    }

    #[test]
    fn test_init_script_completes_subcommands() {
        let script = init_script();
        assert!(script.contains("list"));
        assert!(script.contains("create"));
        assert!(script.contains("use"));
        assert!(script.contains("remove"));
        assert!(script.contains("install"));
        assert!(script.contains("uninstall"));
    }

    #[test]
    fn test_init_script_is_valid_bash_comment_header() {
        let script = init_script();
        assert!(script.starts_with("# scoop shell integration for bash"));
    }
}
