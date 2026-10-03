# Tab completion
Register-ArgumentCompleter -Native -CommandName scuv -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commands = @('list', 'create', 'use', 'remove', 'info', 'install', 'uninstall',
                  'doctor', 'init', 'completions', 'activate', 'deactivate', 'shell',
                  'migrate', 'lang', 'self', 'status', 'clone', 'export', 'import',
                  'sync', 'run', 'which', 'prune', 'gc', 'man', 'verify', 'diff')

    $tokens = $commandAst.ToString() -split '\s+'
    $cmd = if ($tokens.Count -gt 1) { $tokens[1] } else { '' }

    # First argument: complete subcommands. The AST text drops the trailing
    # space, so `scuv use <TAB>` also has two tokens; only an empty line
    # after `scuv`, or a partly typed second word, is the subcommand slot.
    $atSubcommand = $tokens.Count -eq 1 -or ($tokens.Count -eq 2 -and $wordToComplete)
    if ($atSubcommand -and $wordToComplete -notmatch '^-') {
        $commands | Where-Object { $_ -like "$wordToComplete*" } | ForEach-Object {
            [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
        }
        return
    }

    # Positional arguments before the word being completed: options are
    # skipped, and so is the value of an option that takes one
    $before = @($tokens | Select-Object -Skip 2)
    if ($wordToComplete -and $before.Count -gt 0) { $before = @($before | Select-Object -SkipLast 1) }
    $positionals = 0
    $skip = $false
    foreach ($t in $before) {
        if ($skip) { $skip = $false; continue }
        if ($t -cin '--color', '-o', '--output', '--name', '--shell') { $skip = $true }
        elseif ($t -and $t -notmatch '^-') { $positionals++ }
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
