# Zsh completion for scuv
# Options every subcommand takes (`global = true` in the CLI): append the
# ones not yet on the line to the caller's `opts` array (zsh scoping).
_scuv_global_opts() {
    local w has_quiet=false has_color=false has_nocolor=false
    for w in "${words[@]}"; do
        case "$w" in
            -q|--quiet) has_quiet=true ;;
            --color|--color=*) has_color=true ;;
            --no-color) has_nocolor=true ;;
        esac
    done
    [[ $has_quiet == false ]] && opts+=('-q:Suppress all output' '--quiet:Suppress all output')
    [[ $has_color == false ]] && opts+=('--color:When to use color (auto, always, never)')
    [[ $has_nocolor == false ]] && opts+=('--no-color:Disable colored output')
}

_scuv() {
    local curcontext="$curcontext" state line
    typeset -A opt_args
    local cur="${words[$CURRENT]}"

    # `--color` takes a value on every subcommand, as `--color <TAB>` or
    # `--color=<TAB>`
    if [[ "$cur" == --color=* ]]; then
        compadd -P '--color=' auto always never
        return 0
    fi
    if [[ "${words[CURRENT-1]:-}" == "--color" ]]; then
        local color_vals=(
            'auto:Color on a terminal unless NO_COLOR is set'
            'always:Always color'
            'never:Never color'
        )
        _describe 'color' color_vals
        return 0
    fi

    _arguments -C \
        '1: :->command' \
        '*: :->args'

    case $state in
        command)
            local commands=(
                'list:List all virtual environments'
                'use:Set local environment for current directory'
                'create:Create a new virtual environment'
                'remove:Remove a virtual environment'
                'info:Show detailed information about a virtual environment'
                'install:Install a Python version'
                'uninstall:Uninstall a Python version'
                'doctor:Diagnose installation issues'
                'init:Output shell initialization script'
                'completions:Output shell completion script'
                'activate:Activate a virtual environment'
                'deactivate:Deactivate current virtual environment'
                'shell:Set shell-specific environment'
                'migrate:Migrate environments from other tools'
                'lang:Set or show language preference'
            )
            _describe -V 'command' commands
            ;;
        args)
            case $line[1] in
                use)
                    if [[ $cur == -* ]]; then
                        local opts=('--help:Show help')
                        local has_unset=false has_global=false has_link=false
                        for w in "${words[@]}"; do
                            case "$w" in
                                --unset) has_unset=true ;;
                                --global) has_global=true ;;
                                --link|--no-link) has_link=true ;;
                            esac
                        done
                        [[ $has_unset == false ]] && opts+=('--unset:Remove version setting')
                        [[ $has_link == false ]] && opts+=('--link:Create .venv symlink' '--no-link:Do not create .venv symlink')
                        [[ $has_global == false ]] && opts+=('--global:Set as global default')
                        _scuv_global_opts
                        _describe 'option' opts
                    else
                        # Check if env name already provided (exclude current word being typed)
                        local has_env=false
                        local prev_args=("${words[@]:2:$((CURRENT-3))}")
                        for w in "${prev_args[@]}"; do
                            [[ $w != -* && -n $w ]] && has_env=true && break
                        done
                        if [[ $has_env == false ]]; then
                            local envs=(${(f)"$(command scuv list --bare 2>/dev/null)"})
                            compadd -a envs
                        fi
                    fi
                    ;;
                remove)
                    if [[ $cur == -* ]]; then
                        local opts=('--help:Show help')
                        local has_force=false
                        for w in "${words[@]}"; do
                            case "$w" in
                                --force) has_force=true ;;
                            esac
                        done
                        [[ $has_force == false ]] && opts+=('--force:Skip confirmation')
                        _scuv_global_opts
                        _describe 'option' opts
                    else
                        # Check if env name already provided (exclude current word being typed)
                        local has_env=false
                        local prev_args=("${words[@]:2:$((CURRENT-3))}")
                        for w in "${prev_args[@]}"; do
                            [[ $w != -* && -n $w ]] && has_env=true && break
                        done
                        if [[ $has_env == false ]]; then
                            local envs=(${(f)"$(command scuv list --bare 2>/dev/null)"})
                            compadd -a envs
                        fi
                    fi
                    ;;
                info)
                    if [[ $cur == -* ]]; then
                        local opts=('--help:Show help')
                        local has_json=false has_allpackages=false has_nosize=false
                        for w in "${words[@]}"; do
                            case "$w" in
                                --json) has_json=true ;;
                                --all-packages) has_allpackages=true ;;
                                --no-size) has_nosize=true ;;
                            esac
                        done
                        [[ $has_json == false ]] && opts+=('--json:Output as JSON')
                        [[ $has_allpackages == false ]] && opts+=('--all-packages:Show all installed packages')
                        [[ $has_nosize == false ]] && opts+=('--no-size:Skip directory size calculation')
                        _scuv_global_opts
                        _describe 'option' opts
                    else
                        # Check if env name already provided (exclude current word being typed)
                        local has_env=false
                        local prev_args=("${words[@]:2:$((CURRENT-3))}")
                        for w in "${prev_args[@]}"; do
                            [[ $w != -* && -n $w ]] && has_env=true && break
                        done
                        if [[ $has_env == false ]]; then
                            local envs=(${(f)"$(command scuv list --bare 2>/dev/null)"})
                            compadd -a envs
                        fi
                    fi
                    ;;
                activate)
                    # Check if env name already provided (exclude current word being typed)
                    local has_env=false
                    local prev_args=("${words[@]:2:$((CURRENT-3))}")
                    for w in "${prev_args[@]}"; do
                        [[ $w != -* && -n $w ]] && has_env=true && break
                    done
                    if [[ $has_env == false ]]; then
                        local envs=(${(f)"$(command scuv list --bare 2>/dev/null)"})
                        compadd -a envs
                    fi
                    ;;
                install)
                    if [[ $cur == -* ]]; then
                        local opts=('--help:Show help')
                        local has_version_opt=false
                        for w in "${words[@]}"; do
                            case "$w" in
                                --latest|--stable) has_version_opt=true ;;
                            esac
                        done
                        [[ $has_version_opt == false ]] && opts+=('--latest:Install latest stable Python' '--stable:Install oldest fully-supported Python')
                        _scuv_global_opts
                        _describe 'option' opts
                    fi
                    ;;
                uninstall)
                    if [[ $cur == -* ]]; then
                        local opts=('--help:Show help')
                        local
                        for w in "${words[@]}"; do
                            case "$w" in
                            esac
                        done
                        _scuv_global_opts
                        _describe 'option' opts
                    else
                        # Check if version already provided (exclude current word being typed)
                        local has_ver=false
                        local prev_args=("${words[@]:2:$((CURRENT-3))}")
                        for w in "${prev_args[@]}"; do
                            [[ $w != -* && -n $w ]] && has_ver=true && break
                        done
                        if [[ $has_ver == false ]]; then
                            local pythons=(${(uf)"$(command scuv list --pythons --bare 2>/dev/null)"})
                            compadd -a pythons
                        fi
                    fi
                    ;;
                list)
                    # `--sort` value completion: when the previous word is
                    # `--sort`, surface the enum values instead of the
                    # flag list. The `--sort=` form is handled by zsh's
                    # built-in option-with-value matcher when the user
                    # types `=`.
                    local prev_word="${words[CURRENT-1]:-}"
                    if [[ "$prev_word" == "--sort" ]]; then
                        local sort_vals=(
                            'name:Alphabetical (default)'
                            'created:Newest created first'
                            'last-used:Most recently used first'
                        )
                        _describe 'sort mode' sort_vals
                        return 0
                    fi
                    if [[ $cur == -* ]]; then
                        local opts=('--help:Show help')
                        local has_pythons=false has_sort=false has_json=false
                        for w in "${words[@]}"; do
                            case "$w" in
                                --pythons) has_pythons=true ;;
                                --sort|--sort=*) has_sort=true ;;
                                --json) has_json=true ;;
                            esac
                        done
                        [[ $has_pythons == false ]] && opts+=('--pythons:Show installed Python versions')
                        [[ $has_sort == false ]] && opts+=('--sort:Sort order (name|created|last-used)')
                        [[ $has_json == false ]] && opts+=('--json:Output as JSON')
                        _scuv_global_opts
                        _describe 'option' opts
                    fi
                    ;;
                doctor)
                    if [[ $cur == -* ]]; then
                        local opts=('--help:Show help')
                        # Check which options are already used
                        local has_verbose=false has_json=false
                        for w in "${words[@]}"; do
                            case "$w" in
                                -v|--verbose) has_verbose=true ;;
                                --json) has_json=true ;;
                            esac
                        done
                        # Add only unused options
                        [[ $has_verbose == false ]] && opts+=('-v:Increase verbosity' '--verbose:Increase verbosity')
                        [[ $has_json == false ]] && opts+=('--json:Output as JSON')
                        _scuv_global_opts
                        _describe 'option' opts
                    fi
                    ;;
                create)
                    if [[ $cur == -* ]]; then
                        local opts=('--help:Show help')
                        local has_force=false
                        for w in "${words[@]}"; do
                            case "$w" in
                                --force) has_force=true ;;
                            esac
                        done
                        [[ $has_force == false ]] && opts+=('--force:Overwrite existing environment')
                        _scuv_global_opts
                        _describe 'option' opts
                    else
                        # Count positional args before current word
                        local pos_count=0
                        local prev_args=("${words[@]:2:$((CURRENT-3))}")
                        for w in "${prev_args[@]}"; do
                            [[ $w != -* && -n $w ]] && ((pos_count++))
                        done
                        if [[ $pos_count -eq 1 ]]; then
                            # Second positional arg: python version
                            local pythons=(${(uf)"$(command scuv list --pythons --bare 2>/dev/null)"})
                            compadd -a pythons
                        fi
                    fi
                    ;;
                init|completions)
                    local shells=('bash:Bash shell' 'zsh:Zsh shell' 'fish:Fish shell' 'powershell:PowerShell')
                    _describe 'shell' shells
                    ;;
                lang)
                    if [[ $cur == -* ]]; then
                        local opts=('--help:Show help')
                        local has_list=false has_reset=false has_json=false
                        for w in "${words[@]}"; do
                            case "$w" in
                                --list) has_list=true ;;
                                --reset) has_reset=true ;;
                                --json) has_json=true ;;
                            esac
                        done
                        [[ $has_list == false ]] && opts+=('--list:List supported languages')
                        [[ $has_reset == false ]] && opts+=('--reset:Reset to system default')
                        [[ $has_json == false ]] && opts+=('--json:Output as JSON')
                        _scuv_global_opts
                        _describe 'option' opts
                    else
                        local langs=('en:English' 'ko:Korean' 'ja:Japanese' 'pt-BR:Portuguese (Brazilian)' 'es:Spanish')
                        _describe 'language' langs
                    fi
                    ;;
                shell)
                    if [[ $cur == -* ]]; then
                        local opts=('--help:Show help')
                        local has_unset=false
                        for w in "${words[@]}"; do
                            case "$w" in
                                --unset) has_unset=true ;;
                            esac
                        done
                        [[ $has_unset == false ]] && opts+=('--unset:Clear shell-specific environment')
                        _scuv_global_opts
                        _describe 'option' opts
                    else
                        # Check if env name already provided (exclude current word being typed)
                        local has_env=false
                        local prev_args=("${words[@]:2:$((CURRENT-3))}")
                        for w in "${prev_args[@]}"; do
                            [[ $w != -* && -n $w ]] && has_env=true && break
                        done
                        if [[ $has_env == false ]]; then
                            local envs=(${(f)"$(command scuv list --bare 2>/dev/null)"})
                            compadd -a envs
                        fi
                    fi
                    ;;
                migrate)
                    if [[ $cur == -* ]]; then
                        local opts=('--help:Show help')
                        _scuv_global_opts
                        _describe 'option' opts
                    else
                        local subcmds=('list:List environments available for migration' 'all:Migrate all environments' '@env:Migrate a specific environment')
                        _describe 'subcommand' subcmds
                    fi
                    ;;
            esac
            ;;
    esac
}

# Register completion only if compdef is available (requires compinit)
(( $+functions[compdef] )) && compdef _scuv scuv
