# scuv shell integration for PowerShell
# Add to your $PROFILE: Invoke-Expression (& scuv init powershell | Out-String)

# Get the scuv binary path (avoids conflict with wrapper function). The
# first match on PATH, as a shell would pick: with several installed,
# Get-Command returns them all and .Source would join their paths.
$script:ScuvBin = (Get-Command scuv -CommandType Application -ErrorAction SilentlyContinue |
    Select-Object -First 1).Source
if (-not $script:ScuvBin) {
    Write-Warning "scuv binary not found in PATH"
    return
}

# Wrapper function for scuv
function scuv {
    param([Parameter(ValueFromRemainingArguments=$true)]$Arguments)

    $command = if ($Arguments.Count -gt 0) { $Arguments[0] } else { '' }

    switch ($command) {
        'use' {
            & $script:ScuvBin @Arguments
            if ($LASTEXITCODE -eq 0) {
                $name = $Arguments | Where-Object { $_ -notmatch '^-' } | Select-Object -Skip 1 -First 1
                if ($name -ceq 'system') {
                    Invoke-Expression (& $script:ScuvBin deactivate --shell powershell | Out-String)
                } elseif ($name) {
                    Invoke-Expression (& $script:ScuvBin activate --shell powershell $name | Out-String)
                }
            }
        }
        { $_ -in 'activate', 'deactivate', 'shell' } {
            # Out-String: the output is several lines, and Invoke-Expression
            # takes one string. Help/version flags pass through unevaluated
            # (whole-argument, case-sensitive match: data-hub is not -h).
            if ($Arguments | Where-Object { $_ -cin '-h', '--help', '-V', '--version' }) {
                & $script:ScuvBin @Arguments
            } elseif ($Arguments | Where-Object { $_ -clike '--shell*' }) {
                Invoke-Expression (& $script:ScuvBin @Arguments | Out-String)
            } else {
                $rest = @($Arguments | Select-Object -Skip 1)
                Invoke-Expression (& $script:ScuvBin $Arguments[0] --shell powershell @rest | Out-String)
            }
        }
        default {
            & $script:ScuvBin @Arguments
        }
    }
}
