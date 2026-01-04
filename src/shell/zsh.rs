//! Zsh shell integration

/// Generate zsh initialization script
pub fn init_script() -> &'static str {
    r#"# scoop shell integration for zsh

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

# Set up chpwd hook for auto-activate
if [[ -z "$SCOOP_NO_AUTO" ]]; then
    autoload -Uz add-zsh-hook
    add-zsh-hook chpwd _scoop_hook
fi

# Run hook on startup
_scoop_hook

# Zsh completion for scoop
_scoop() {
    local curcontext="$curcontext" state line
    typeset -A opt_args
    local cur="${words[$CURRENT]}"

    _arguments -C \
        '1: :->command' \
        '*: :->args'

    case $state in
        command)
            local commands=(
                'list:List all virtual environments'
                'create:Create a new virtual environment'
                'use:Set local environment for current directory'
                'remove:Remove a virtual environment'
                'install:Install a Python version'
                'uninstall:Uninstall a Python version'
                'init:Output shell initialization script'
                'completions:Output shell completion script'
                'activate:Activate a virtual environment'
                'deactivate:Deactivate current virtual environment'
            )
            _describe 'command' commands
            ;;
        args)
            case $line[1] in
                use)
                    if [[ $cur == -* ]]; then
                        local opts=(
                            '--global:Set as global default'
                            '--link:Create .venv symlink'
                            '--no-link:Do not create .venv symlink'
                        )
                        _describe 'option' opts
                    else
                        local has_env=false
                        for w in "${words[@]:2}"; do
                            [[ $w != -* && -n $w ]] && has_env=true && break
                        done
                        if [[ $has_env == false ]]; then
                            local envs=(${(f)"$(command scoop list --bare 2>/dev/null)"})
                            _describe 'environment' envs
                        fi
                    fi
                    ;;
                remove)
                    if [[ $cur == -* ]]; then
                        local opts=('--force:Skip confirmation')
                        _describe 'option' opts
                    else
                        local has_env=false
                        for w in "${words[@]:2}"; do
                            [[ $w != -* && -n $w ]] && has_env=true && break
                        done
                        if [[ $has_env == false ]]; then
                            local envs=(${(f)"$(command scoop list --bare 2>/dev/null)"})
                            _describe 'environment' envs
                        fi
                    fi
                    ;;
                activate)
                    local has_env=false
                    for w in "${words[@]:2}"; do
                        [[ $w != -* && -n $w ]] && has_env=true && break
                    done
                    if [[ $has_env == false ]]; then
                        local envs=(${(f)"$(command scoop list --bare 2>/dev/null)"})
                        _describe 'environment' envs
                    fi
                    ;;
                install)
                    if [[ $cur == -* ]]; then
                        local opts=(
                            '--latest:Install latest stable Python'
                            '--stable:Install oldest fully-supported Python'
                        )
                        _describe 'option' opts
                    fi
                    ;;
                uninstall)
                    local has_ver=false
                    for w in "${words[@]:2}"; do
                        [[ $w != -* && -n $w ]] && has_ver=true && break
                    done
                    if [[ $has_ver == false ]]; then
                        local pythons=(${(uf)"$(command scoop list --pythons --bare 2>/dev/null)"})
                        _describe 'python version' pythons
                    fi
                    ;;
                list)
                    if [[ $cur == -* ]]; then
                        local opts=('--pythons:Show installed Python versions')
                        _describe 'option' opts
                    fi
                    ;;
                create)
                    if [[ $cur == -* ]]; then
                        local opts=('--force:Overwrite existing environment')
                        _describe 'option' opts
                    else
                        local pos_count=0
                        for w in "${words[@]:2}"; do
                            [[ $w != -* && -n $w ]] && ((pos_count++))
                        done
                        if [[ $pos_count -eq 1 ]]; then
                            local pythons=(${(uf)"$(command scoop list --pythons --bare 2>/dev/null)"})
                            _describe 'python version' pythons
                        fi
                    fi
                    ;;
                init|completions)
                    local shells=('bash:Bash shell' 'zsh:Zsh shell' 'fish:Fish shell' 'powershell:PowerShell')
                    _describe 'shell' shells
                    ;;
            esac
            ;;
    esac
}
compdef _scoop scoop
"#
}
