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
    local cur prev
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"

    case "$prev" in
        use|remove|activate)
            COMPREPLY=($(compgen -W "$(command scoop list --bare 2>/dev/null)" -- "$cur"))
            return
            ;;
        uninstall)
            COMPREPLY=($(compgen -W "$(command scoop list --pythons --bare 2>/dev/null | sort -u)" -- "$cur"))
            return
            ;;
        scoop)
            COMPREPLY=($(compgen -W "list create use remove install uninstall init completions activate deactivate" -- "$cur"))
            return
            ;;
    esac

    # Handle options
    if [[ "$cur" == -* ]]; then
        case "$prev" in
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
        esac
    fi
}
complete -F _scoop_complete scoop
"#
}
