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
                if [[ -n "$name" ]]; then
                    eval "$(command scuv activate "$name")"
                fi
            fi
            return $ret
            ;;
        activate|deactivate|shell)
            # Pass through help/version flags without eval
            if [[ "$*" == *--help* ]] || [[ "$*" == *-h* ]] || [[ "$*" == *--version* ]] || [[ "$*" == *-V* ]]; then
                command scuv "$@"
            else
                eval "$(command scuv "$@")"
            fi
            ;;
        *)
            command scuv "$@"
            ;;
    esac
}
