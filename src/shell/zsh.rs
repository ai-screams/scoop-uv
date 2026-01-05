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

    // ==========================================================================
    // Shell Script Branch Coverage Tests
    // ==========================================================================

    #[test]
    fn test_init_script_use_command_handles_options() {
        let script = init_script();
        // Verify 'use' command skips options when finding env name
        assert!(script.contains("case \"$arg\" in"));
        assert!(script.contains("-*) ;;"));
    }

    #[test]
    fn test_init_script_use_command_has_return_code() {
        let script = init_script();
        // Verify 'use' command preserves return code
        assert!(script.contains("local ret=$?"));
        assert!(script.contains("return $ret"));
    }

    #[test]
    fn test_init_script_hook_handles_deactivation() {
        let script = init_script();
        // Verify hook deactivates when env_name is empty but SCOOP_ACTIVE is set
        assert!(script.contains("-z \"$env_name\" && -n \"$SCOOP_ACTIVE\""));
        assert!(script.contains("command scoop deactivate"));
    }

    #[test]
    fn test_init_script_hook_compares_active_env() {
        let script = init_script();
        // Verify hook only activates if different from current
        assert!(script.contains("$env_name\" != \"$SCOOP_ACTIVE\""));
    }

    #[test]
    fn test_init_script_default_command_passthrough() {
        let script = init_script();
        // Verify default case passes through to command scoop
        assert!(script.contains("*)\n            command scoop \"$@\""));
    }

    #[test]
    fn test_init_script_runs_hook_on_startup() {
        let script = init_script();
        // Verify hook is called on script load
        assert!(script.contains("\n_scoop_hook\n"));
    }

    #[test]
    fn test_init_script_silences_resolve_errors() {
        let script = init_script();
        // Verify resolve command stderr is silenced
        assert!(script.contains("scoop resolve 2>/dev/null"));
    }

    #[test]
    fn test_init_script_completion_state_machine() {
        let script = init_script();
        // Verify zsh completion uses _arguments state machine
        assert!(script.contains("_arguments -C"));
        assert!(script.contains("'1: :->command'"));
        assert!(script.contains("'*: :->args'"));
    }

    #[test]
    fn test_init_script_completion_use_options() {
        let script = init_script();
        // Verify 'use' subcommand has proper option descriptions
        assert!(script.contains("'--global:Set as global default'"));
        assert!(script.contains("'--link:Create .venv symlink'"));
        assert!(script.contains("'--no-link:Do not create .venv symlink'"));
    }

    #[test]
    fn test_init_script_completion_install_options() {
        let script = init_script();
        // Verify 'install' subcommand has proper option descriptions
        assert!(script.contains("'--latest:Install latest stable Python'"));
        assert!(script.contains("'--stable:Install oldest fully-supported Python'"));
    }

    #[test]
    fn test_init_script_completion_remove_force_option() {
        let script = init_script();
        // Verify 'remove' has --force option
        assert!(script.contains("'--force:Skip confirmation'"));
    }

    #[test]
    fn test_init_script_completion_list_pythons_option() {
        let script = init_script();
        // Verify 'list' has --pythons option
        assert!(script.contains("'--pythons:Show installed Python versions'"));
    }

    #[test]
    fn test_init_script_completion_create_force_option() {
        let script = init_script();
        // Verify 'create' has --force option
        assert!(script.contains("'--force:Overwrite existing environment'"));
    }

    #[test]
    fn test_init_script_completion_prevents_duplicate_env() {
        let script = init_script();
        // Verify completion checks if env already provided
        assert!(script.contains("local has_env=false"));
        assert!(script.contains("[[ $has_env == false ]]"));
    }

    #[test]
    fn test_init_script_completion_uninstall_python_list() {
        let script = init_script();
        // Verify uninstall completes with unique python versions
        assert!(script.contains("local pythons=(${(uf)\"$(command scoop list --pythons --bare"));
    }

    #[test]
    fn test_init_script_completion_create_python_version() {
        let script = init_script();
        // Verify create offers python versions for second arg
        assert!(script.contains("if [[ $pos_count -eq 1 ]]"));
    }

    #[test]
    fn test_init_script_uses_compdef() {
        let script = init_script();
        // Verify completion is registered with compdef
        assert!(script.contains("compdef _scoop scoop"));
    }

    #[test]
    fn test_init_script_adds_chpwd_hook() {
        let script = init_script();
        // Verify chpwd hook is registered
        assert!(script.contains("add-zsh-hook chpwd _scoop_hook"));
    }
}
