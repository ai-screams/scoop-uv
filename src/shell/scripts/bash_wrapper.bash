# scuv shell integration for bash

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
                # `use` takes system in any case (SYSTEM, System)
                case "$name" in
                    [Ss][Yy][Ss][Tt][Ee][Mm]) name=system ;;
                esac
                if [[ "$name" == system ]]; then
                    eval "$(command scuv deactivate --shell bash)"
                elif [[ -n "$name" ]]; then
                    eval "$(command scuv activate --shell bash "$name")"
                fi
            fi
            return $ret
            ;;
        activate|deactivate|shell)
            local arg script
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
                        script="$(command scuv "$@")" || return
                        eval "$script"
                        return
                        ;;
                esac
            done
            # Name the shell: detection reads PSModulePath first, which
            # Windows sets for every process, Git Bash included.
            # Keep scuv's exit status: `eval ""` would turn a failure into 0.
            script="$(command scuv "$1" --shell bash "${@:2}")" || return
            eval "$script"
            ;;
        *)
            command scuv "$@"
            ;;
    esac
}
