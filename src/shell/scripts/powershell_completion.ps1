# Tab completion
Register-ArgumentCompleter -Native -CommandName scuv -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commands = @('list', 'create', 'use', 'remove', 'info', 'install', 'uninstall',
                  'doctor', 'init', 'completions', 'activate', 'deactivate', 'shell',
                  'migrate', 'lang', 'self', 'status', 'clone', 'export', 'import',
                  'sync', 'run', 'which', 'prune', 'gc', 'man', 'verify', 'diff')

    # The words before the one being completed, as the parser split them
    # (quotes kept together); the word under the cursor ends at the cursor
    # and is left out.
    $words = @($commandAst.CommandElements |
        Where-Object { $_.Extent.EndOffset -lt $cursorPosition } |
        ForEach-Object { $_.Extent.Text })
    $cmd = if ($words.Count -gt 1) { $words[1] } else { '' }

    # First argument: complete subcommands
    if ($words.Count -le 1 -and $wordToComplete -notmatch '^-') {
        $commands | Where-Object { $_ -like "$wordToComplete*" } | ForEach-Object {
            [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
        }
        return
    }

    # The word being completed is an option's value: shells for --shell,
    # PowerShell's own path completion for -o/--output, nothing otherwise
    $prev = $words[-1]
    if ($prev -ceq '--shell') {
        @('bash', 'zsh', 'fish', 'powershell') | Where-Object { $_ -like "$wordToComplete*" } | ForEach-Object {
            [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
        }
        return
    }
    if ($prev -cin '-o', '--output', '--name', '--color') { return }

    # Positional arguments before the word being completed: options are
    # skipped, and so is the value of an option that takes one
    $positionals = 0
    $skip = $false
    foreach ($t in @($words | Select-Object -Skip 2)) {
        if ($skip) { $skip = $false; continue }
        if ($t -cin '--color', '-o', '--output', '--name', '--shell') { $skip = $true }
        elseif ($t -notmatch '^-') { $positionals++ }
    }

    # self has one subcommand; import and man take a path (PowerShell falls
    # back to path completion when nothing is returned)
    if ($cmd -eq 'self') {
        if ($positionals -eq 0) {
            @('update') | Where-Object { $_ -like "$wordToComplete*" } | ForEach-Object {
                [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
            }
        }
        return
    }

    # Environment names where one goes: the first argument, or either of diff's two
    $maxEnvs = if ($cmd -eq 'diff') { 2 } else { 1 }
    if ($cmd -in 'use', 'remove', 'info', 'activate', 'shell', 'clone', 'export', 'run', 'verify', 'diff') {
        if ($positionals -ge $maxEnvs) { return }
        $envs = & $script:ScuvBin list --bare 2>$null
        if ($envs) {
            $envs | Where-Object { $_ -like "$wordToComplete*" } | ForEach-Object {
                [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
            }
        }
        return
    }

    # Python version completion
    if ($cmd -in 'install', 'uninstall', 'create') {
        $versions = & $script:ScuvBin list --pythons --bare 2>$null | Sort-Object -Unique
        if ($versions) {
            $versions | Where-Object { $_ -like "$wordToComplete*" } | ForEach-Object {
                [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
            }
        }
        return
    }

    # Shell completion for init/completions
    if ($cmd -in 'init', 'completions') {
        @('bash', 'zsh', 'fish', 'powershell') | Where-Object { $_ -like "$wordToComplete*" } | ForEach-Object {
            [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
        }
        return
    }

    # Language completion for lang
    if ($cmd -eq 'lang') {
        @('en', 'ko', 'ja', 'pt-BR', 'es') | Where-Object { $_ -like "$wordToComplete*" } | ForEach-Object {
            [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
        }
        return
    }

    # Migrate subcommand completion
    if ($cmd -eq 'migrate') {
        @('list', 'all') | Where-Object { $_ -like "$wordToComplete*" } | ForEach-Object {
            [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
        }
        return
    }
}
