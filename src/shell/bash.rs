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
"#
}
