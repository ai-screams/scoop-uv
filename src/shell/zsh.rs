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
    fn test_init_script_uses_chpwd_hook() {
        let script = init_script();
        // zsh uses chpwd hook instead of PROMPT_COMMAND
        assert!(script.contains("add-zsh-hook chpwd _scoop_hook"));
    }

    #[test]
    fn test_init_script_respects_no_auto_var() {
        let script = init_script();
        assert!(script.contains("SCOOP_NO_AUTO"));
    }

    #[test]
    fn test_init_script_contains_completion() {
        let script = init_script();
        assert!(script.contains("_scoop()"));
        assert!(script.contains("compdef _scoop scoop"));
    }

    #[test]
    fn test_init_script_uses_zsh_completion_system() {
        let script = init_script();
        assert!(script.contains("_arguments"));
        assert!(script.contains("_describe"));
    }

    #[test]
    fn test_init_script_completes_commands_with_descriptions() {
        let script = init_script();
        assert!(script.contains("'list:List all virtual environments'"));
        assert!(script.contains("'create:Create a new virtual environment'"));
        assert!(script.contains("'use:Set local environment for current directory'"));
    }

    #[test]
    fn test_init_script_is_valid_zsh_comment_header() {
        let script = init_script();
        assert!(script.starts_with("# scoop shell integration for zsh"));
    }

    #[test]
    fn test_init_script_loads_zsh_hooks() {
        let script = init_script();
        assert!(script.contains("autoload -Uz add-zsh-hook"));
    }
}
