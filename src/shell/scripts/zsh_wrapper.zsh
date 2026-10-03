# scuv shell integration for zsh

# Disable completion sorting for scuv (preserves command order)
zstyle ':completion:*:scuv:*' sort false

# Wrapper function for scuv
scuv() {
    local command="${1:-}"

    case "$command" in
        use)
            command scuv "$@"
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
                if [[ "$name" == system ]]; then
                    eval "$(command scuv deactivate --shell zsh)"
                elif [[ -n "$name" ]]; then
                    eval "$(command scuv activate --shell zsh "$name")"
                fi
            fi
            return $ret
            ;;
        activate|deactivate|shell)
            local arg
            for arg in "$@"; do
                case "$arg" in
                    # Pass through help/version flags without eval (whole-
                    # argument match: an env name such as data-hub is not -h)
                    -h|--help|-V|--version)
                        command scuv "$@"
                        return
                        ;;
                    # The user chose the shell; a second --shell is rejected
                    --shell|--shell=*)
                        eval "$(command scuv "$@")"
                        return
                        ;;
                esac
            done
            # Name the shell: detection reads PSModulePath first, which
            # Windows sets for every process, Git Bash included.
            eval "$(command scuv "$1" --shell zsh "${@:2}")"
            ;;
        *)
            command scuv "$@"
            ;;
    esac
}
