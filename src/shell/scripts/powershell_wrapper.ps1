# scuv shell integration for PowerShell
# Add to your $PROFILE: Invoke-Expression (& scuv init powershell)

# Get the scuv binary path (avoids conflict with wrapper function)
$script:ScuvBin = (Get-Command scuv -CommandType Application -ErrorAction SilentlyContinue).Source
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
                if ($name) {
                    Invoke-Expression (& $script:ScuvBin activate $name)
                }
            }
        }
        { $_ -in 'activate', 'deactivate', 'shell' } {
            if ($Arguments -match '(-h|--help|-V|--version)') {
                & $script:ScuvBin @Arguments
            } else {
                Invoke-Expression (& $script:ScuvBin @Arguments)
            }
        }
        default {
            & $script:ScuvBin @Arguments
        }
    }
}
