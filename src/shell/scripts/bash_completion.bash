# Bash completion for scuv

# Options every subcommand takes (`global = true` in the CLI).
_scuv_global_opts="-q --quiet --color --no-color --help"

# Offer `$1` plus the global options, minus those already on the line.
# Each further argument is a group of options that exclude one another:
# using one drops the whole group (as -q drops --quiet). `--opt=value`
# drops `--opt`.
_scuv_offer() {
    local opts=" $1 $_scuv_global_opts " word group member
    shift
    local groups=("-q --quiet" "$@")
    for word in "${COMP_WORDS[@]:2:COMP_CWORD-2}"; do
        word="${word%%=*}"
        opts="${opts/ $word / }"
        for group in "${groups[@]}"; do
            if [[ " $group " == *" $word "* ]]; then
                for member in $group; do
                    opts="${opts/ $member / }"
                done
            fi
        done
    done
    COMPREPLY=($(compgen -W "$opts" -- "$cur"))
}

_scuv_complete() {
    local cur cmd i
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"

    # Get subcommand (COMP_WORDS[1])
    cmd=""
    if [[ ${COMP_CWORD} -ge 1 ]]; then
        cmd="${COMP_WORDS[1]}"
    fi

    # `--color` takes a value on every subcommand (`--color=<TAB>` or
    # `--color <TAB>`), so offer the choices before the option lists.
    if [[ "$cur" == --color=* ]]; then
        COMPREPLY=($(compgen -W "auto always never" -P "--color=" -- "${cur#--color=}"))
        return
    fi
    if [[ ${COMP_CWORD} -ge 2 && "${COMP_WORDS[COMP_CWORD-1]}" == "--color" ]]; then
        COMPREPLY=($(compgen -W "auto always never" -- "$cur"))
        return
    fi

    # The word being completed is an option's value: shells for --shell,
    # paths for -o/--output, nothing for --name
    case "${COMP_WORDS[COMP_CWORD-1]}" in
        --shell)
            COMPREPLY=($(compgen -W "bash zsh fish powershell" -- "$cur"))
            return
            ;;
        -o|--output)
            local IFS=$'\n'
            COMPREPLY=($(compgen -f -- "$cur"))
            return
            ;;
        --name)
            return
            ;;
    esac

    # First argument: complete subcommands
    if [[ ${COMP_CWORD} -eq 1 ]]; then
        COMPREPLY=($(compgen -W "list use create remove info install uninstall doctor init completions activate deactivate shell migrate lang self status clone export import sync run which prune gc man verify diff" -- "$cur"))
        return
    fi

    # Option completion (starts with -)
    if [[ "$cur" == -* ]]; then
        case "$cmd" in
            list)
                # Special-case `--sort` value completion: when the user
                # has typed `--sort=` or `--sort <TAB>`, offer the
                # enum values directly instead of the option list.
                local prev_word="${COMP_WORDS[COMP_CWORD-1]:-}"
                if [[ "$cur" == --sort=* ]]; then
                    local val="${cur#--sort=}"
                    COMPREPLY=($(compgen -W "name created last-used" -P "--sort=" -- "$val"))
                    return 0
                fi
                if [[ "$prev_word" == "--sort" ]]; then
                    COMPREPLY=($(compgen -W "name created last-used" -- "$cur"))
                    return 0
                fi
                _scuv_offer "--pythons --sort --json"
                ;;
            doctor) _scuv_offer "-v --verbose --json" "-v --verbose" ;;
            create) _scuv_offer "--force" ;;
            use) _scuv_offer "--unset --link --global --no-link" "--link --no-link" ;;
            remove) _scuv_offer "--force" ;;
            install) _scuv_offer "--latest --stable" "--latest --stable" ;;
            info) _scuv_offer "--json --all-packages --no-size" ;;
            lang) _scuv_offer "--list --reset --json" ;;
            shell) _scuv_offer "--unset" ;;
            *) _scuv_offer "" ;;
        esac
        return
    fi

    # Positional arguments already on the line: options are skipped, and so
    # is the value of an option that takes one
    local positionals=0 skip=false
    for ((i=2; i<COMP_CWORD; i++)); do
        if [[ $skip == true ]]; then
            skip=false
            continue
        fi
        case "${COMP_WORDS[i]}" in
            --color|-o|--output|--name|--shell) skip=true ;;
            -*) ;;
            *) ((positionals++)) ;;
        esac
    done

    # Argument completion (by subcommand)
    case "$cmd" in
        use|remove|info|activate|shell|clone|export|run|verify)
            # The environment name comes first
            if [[ $positionals -eq 0 ]]; then
                COMPREPLY=($(compgen -W "$(command scuv list --bare 2>/dev/null)" -- "$cur"))
            fi
            ;;
        diff)
            # Two environment names
            if [[ $positionals -lt 2 ]]; then
                COMPREPLY=($(compgen -W "$(command scuv list --bare 2>/dev/null)" -- "$cur"))
            fi
            ;;
        self)
            if [[ $positionals -eq 0 ]]; then
                COMPREPLY=($(compgen -W "update" -- "$cur"))
            fi
            ;;
        import|man)
            # A file (import) or a directory (man); one name per line, so a
            # path with spaces stays one candidate
            if [[ $positionals -eq 0 ]]; then
                local IFS=$'\n'
                COMPREPLY=($(compgen -f -- "$cur"))
            fi
            ;;
        uninstall)
            COMPREPLY=($(compgen -W "$(command scuv list --pythons --bare 2>/dev/null | sort -u)" -- "$cur"))
            ;;
        init|completions)
            COMPREPLY=($(compgen -W "bash zsh fish powershell" -- "$cur"))
            ;;
        create)
            # First arg: name, second arg: python version
            if [[ $positionals -eq 1 ]]; then
                # Second positional arg: python version
                COMPREPLY=($(compgen -W "$(command scuv list --pythons --bare 2>/dev/null | sort -u)" -- "$cur"))
            fi
            ;;
        lang)
            # Complete language codes
            COMPREPLY=($(compgen -W "en ko ja pt-BR es" -- "$cur"))
            ;;
        migrate)
            # Complete migrate subcommands
            COMPREPLY=($(compgen -W "list all @env" -- "$cur"))
            ;;
    esac
}
# `-o nosort` (keep subcommand order) needs bash 4.4; macOS ships 3.2,
# which rejects the whole line and would leave scuv without completion.
complete -o nosort -F _scuv_complete scuv 2>/dev/null || complete -F _scuv_complete scuv
