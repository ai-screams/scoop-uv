//! Zsh shell integration

/// Generate zsh initialization script
pub fn init_script() -> &'static str {
    r#"# uvenv shell integration for zsh

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

# Set up chpwd hook for auto-activate
if [[ -z "$UVENV_NO_AUTO" ]]; then
    autoload -Uz add-zsh-hook
    add-zsh-hook chpwd _uvenv_hook
fi

# Run hook on startup
_uvenv_hook
"#
}
