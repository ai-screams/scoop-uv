//! Bash shell integration

/// Generate bash initialization script
pub fn init_script() -> &'static str {
    r#"# uvenv shell integration for bash

# Wrapper function for uvenv
uvenv() {
    local command="${1:-}"

    case "$command" in
        activate|deactivate)
            eval "$(command uvenv "$@")"
            ;;
        *)
            command uvenv "$@"
            ;;
    esac
}

# Auto-activate hook
_uvenv_hook() {
    local env_name
    env_name="$(command uvenv resolve 2>/dev/null)"

    if [[ -n "$env_name" && "$env_name" != "$UVENV_ACTIVE" ]]; then
        eval "$(command uvenv activate "$env_name")"
    elif [[ -z "$env_name" && -n "$UVENV_ACTIVE" ]]; then
        eval "$(command uvenv deactivate)"
    fi
}

# Set up PROMPT_COMMAND for auto-activate
if [[ -z "$UVENV_NO_AUTO" ]]; then
    if [[ -z "$PROMPT_COMMAND" ]]; then
        PROMPT_COMMAND="_uvenv_hook"
    else
        PROMPT_COMMAND="_uvenv_hook;$PROMPT_COMMAND"
    fi
fi

# Run hook on startup
_uvenv_hook
"#
}
