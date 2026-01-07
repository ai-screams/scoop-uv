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
                        # Check if env name already provided (exclude current word being typed)
                        local has_env=false
                        local prev_args=("${words[@]:2:$((CURRENT-3))}")
                        for w in "${prev_args[@]}"; do
                            [[ $w != -* && -n $w ]] && has_env=true && break
                        done
                        if [[ $has_env == false ]]; then
                            local envs=(${(f)"$(command scoop list --bare 2>/dev/null)"})
                            compadd -a envs
                        fi
                    fi
                    ;;
                remove)
                    if [[ $cur == -* ]]; then
                        local opts=('--force:Skip confirmation')
                        _describe 'option' opts
                    else
                        # Check if env name already provided (exclude current word being typed)
                        local has_env=false
                        local prev_args=("${words[@]:2:$((CURRENT-3))}")
                        for w in "${prev_args[@]}"; do
                            [[ $w != -* && -n $w ]] && has_env=true && break
                        done
                        if [[ $has_env == false ]]; then
                            local envs=(${(f)"$(command scoop list --bare 2>/dev/null)"})
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
                        local envs=(${(f)"$(command scoop list --bare 2>/dev/null)"})
                        compadd -a envs
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
                    # Check if version already provided (exclude current word being typed)
                    local has_ver=false
                    local prev_args=("${words[@]:2:$((CURRENT-3))}")
                    for w in "${prev_args[@]}"; do
                        [[ $w != -* && -n $w ]] && has_ver=true && break
                    done
                    if [[ $has_ver == false ]]; then
                        local pythons=(${(uf)"$(command scoop list --pythons --bare 2>/dev/null)"})
                        compadd -a pythons
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
                        # Count positional args before current word
                        local pos_count=0
                        local prev_args=("${words[@]:2:$((CURRENT-3))}")
                        for w in "${prev_args[@]}"; do
                            [[ $w != -* && -n $w ]] && ((pos_count++))
                        done
                        if [[ $pos_count -eq 1 ]]; then
                            # Second positional arg: python version
                            local pythons=(${(uf)"$(command scoop list --pythons --bare 2>/dev/null)"})
                            compadd -a pythons
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

    // =========================================================================
    // Real Test: Syntax Validation with zsh -n
    // =========================================================================

    /// Validates that the generated script has valid zsh syntax.
    /// This is a REAL test - it actually runs zsh to check the script.
    #[test]
    #[cfg(unix)]
    fn test_init_script_has_valid_zsh_syntax() {
        let script = init_script();

        // Use zsh -n for syntax checking (parse only, don't execute)
        let output = std::process::Command::new("zsh")
            .arg("-n") // syntax check only
            .arg("-c")
            .arg(script)
            .output();

        match output {
            Ok(result) => {
                assert!(
                    result.status.success(),
                    "Zsh script has syntax errors:\n{}",
                    String::from_utf8_lossy(&result.stderr)
                );
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                // zsh not available, skip test
                eprintln!("Skipping zsh syntax test: zsh not found");
            }
            Err(e) => panic!("Failed to run zsh: {}", e),
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
        let required_functions = ["scoop()", "_scoop_hook()", "_scoop()"];

        for func in required_functions {
            assert!(
                script.contains(func),
                "Script missing required function: {}",
                func
            );
        }
    }

    #[test]
    fn test_init_script_registers_chpwd_hook() {
        let script = init_script();

        // zsh uses chpwd hook for directory change detection
        assert!(
            script.contains("add-zsh-hook chpwd _scoop_hook"),
            "Script must register chpwd hook for auto-activation"
        );
    }

    #[test]
    fn test_init_script_registers_completion() {
        let script = init_script();

        // Must register completion function with compdef
        assert!(
            script.contains("compdef _scoop scoop"),
            "Script must register zsh completion"
        );
    }

    #[test]
    fn test_init_script_loads_zsh_hooks_module() {
        let script = init_script();

        // Must autoload zsh hooks module
        assert!(
            script.contains("autoload -Uz add-zsh-hook"),
            "Script must load zsh hooks module"
        );
    }

    // =========================================================================
    // Real Test: Best Practices Validation with shellcheck
    // =========================================================================

    /// Validates that the generated script follows shell best practices.
    /// Note: shellcheck uses bash mode for zsh (no native zsh support),
    /// but catches most common issues.
    #[test]
    #[cfg(unix)]
    fn test_init_script_passes_shellcheck() {
        let script = init_script();

        // Write script to temp file (shellcheck requires file input)
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(temp_file.path(), script).unwrap();

        // shellcheck doesn't have native zsh support, but bash mode catches most issues
        // Use --exclude for zsh-specific constructs that bash doesn't understand
        let output = std::process::Command::new("shellcheck")
            .arg("--shell=bash")
            .arg("--severity=warning")
            // Exclude zsh-specific constructs:
            // SC1087: $line[1] array syntax (zsh doesn't require braces)
            // SC2034: Unused variable (zsh uses typeset -A)
            // SC2154: Variable referenced but not assigned (zsh completion vars)
            // SC2168: 'local' outside function (zsh completion context)
            // SC2206: Quote to prevent word splitting (zsh handles this differently)
            // SC2207: COMPREPLY=($(compgen ...)) is standard completion idiom
            // SC2296: ${(f)...} parameter expansion flags (zsh-specific)
            // SC3030: Array syntax (zsh-specific)
            // SC3057: Associative array syntax (zsh-specific)
            .arg("--exclude=SC1087,SC2034,SC2154,SC2168,SC2206,SC2207,SC2296,SC3030,SC3057")
            .arg(temp_file.path())
            .output();

        match output {
            Ok(result) => {
                assert!(
                    result.status.success(),
                    "shellcheck found issues in zsh script:\n{}",
                    String::from_utf8_lossy(&result.stdout)
                );
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                eprintln!(
                    "Skipping shellcheck test: shellcheck not found (install: brew install shellcheck)"
                );
            }
            Err(e) => panic!("Failed to run shellcheck: {}", e),
        }
    }
}
