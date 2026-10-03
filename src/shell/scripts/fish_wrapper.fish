# scuv shell integration for fish

# Wrapper function for scuv
function scuv
    set -l cmd $argv[1]

    switch "$cmd"
        case use
            command scuv $argv
            set -l ret $status
            if test $ret -eq 0
                for arg in $argv[2..-1]
                    if not string match -q -- '-*' "$arg"
                        # `use` takes system in any case (SYSTEM, System)
                        if test (string lower -- "$arg") = system
                            command scuv deactivate --shell fish | source
                        else
                            command scuv activate --shell fish "$arg" | source
                        end
                        break
                    end
                end
            end
            return $ret

        case activate deactivate shell
            # Pass through help/version flags without sourcing (whole-argument
            # match: an env name such as data-hub must not count as -h)
            if string match -qr -- '^(-h|--help|-V|--version)$' $argv
                command scuv $argv
            else if string match -q -- '--shell*' $argv
                # The user chose the shell; a second --shell would be rejected
                command scuv $argv | source
                return $pipestatus[1]
            else
                command scuv $argv[1] --shell fish $argv[2..-1] | source
                # source leaves $status alone on empty input, so name
                # scuv's own status instead of whatever ran before
                return $pipestatus[1]
            end

        case '*'
            command scuv $argv
    end
end
