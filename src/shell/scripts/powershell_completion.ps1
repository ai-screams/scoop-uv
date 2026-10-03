# Tab completion
Register-ArgumentCompleter -Native -CommandName scuv -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commands = @('list', 'create', 'use', 'remove', 'info', 'install', 'uninstall',
                  'doctor', 'init', 'completions', 'activate', 'deactivate', 'shell',
                  'migrate', 'lang')

    $tokens = $commandAst.ToString() -split '\s+'
    $cmd = if ($tokens.Count -gt 1) { $tokens[1] } else { '' }

    # First argument: complete subcommands
    if ($tokens.Count -le 2 -and $wordToComplete -notmatch '^-') {
        $commands | Where-Object { $_ -like "$wordToComplete*" } | ForEach-Object {
            [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
        }
        return
    }

    # Environment name completion for specific commands
    if ($cmd -in 'use', 'remove', 'info', 'activate', 'shell') {
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
