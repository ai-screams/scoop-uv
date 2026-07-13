# Shell Integration

scuv uses a shell wrapper pattern (like pyenv) where the CLI outputs shell code that gets evaluated by the shell.

## How It Works

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│ User runs   │ --> │ CLI outputs │ --> │ Shell evals │
│ scuv use   │     │ export ...  │     │ the output  │
└─────────────┘     └─────────────┘     └─────────────┘
```

The `scuv` shell function wraps the CLI binary:

```bash
scuv() {
    case "$1" in
        use)
            command scuv "$@"
            local name=""
            shift
            for arg in "$@"; do
                case "$arg" in
                    -*) ;;
                    *) name="$arg"; break ;;
                esac
            done
            if [[ -n "$name" ]]; then
                eval "$(command scuv activate "$name")"
            fi
            ;;
        activate|deactivate|shell)
            eval "$(command scuv "$@")"
            ;;
        *)
            command scuv "$@"
            ;;
    esac
}
```

## Setup

### Zsh

```bash
echo 'eval "$(scuv init zsh)"' >> ~/.zshrc
source ~/.zshrc
```

### Bash

```bash
echo 'eval "$(scuv init bash)"' >> ~/.bashrc
source ~/.bashrc
```

### Fish

```fish
echo 'scuv init fish | source' >> ~/.config/fish/config.fish
source ~/.config/fish/config.fish
```

### PowerShell

```powershell
# Add to $PROFILE
Add-Content $PROFILE 'Invoke-Expression (& scuv init powershell)'
# Restart PowerShell
```

## Auto-Activation

When enabled, scuv automatically activates environments based on version files.

**Zsh**: Uses `chpwd` hook (runs on directory change)

```bash
autoload -Uz add-zsh-hook
add-zsh-hook chpwd _scuv_hook
```

**Bash**: Uses `PROMPT_COMMAND`

```bash
PROMPT_COMMAND="_scuv_hook;$PROMPT_COMMAND"
```

**Fish**: Uses `--on-variable PWD` event handler

```fish
function _scuv_hook --on-variable PWD
    # Check for version file and activate/deactivate
end
```

The hook checks for version files and activates/deactivates accordingly.

## Version Resolution Priority

scuv checks these sources in order (first match wins):

| Priority | Source | Set by |
|----------|--------|--------|
| 1 | `SCUV_VERSION` env var | `scuv shell` |
| 2 | `.scuv-version` file | `scuv use` (walks parent directories) |
| 3 | `~/.scuv/version` file | `scuv use --global` |

### The "system" Value

When any source contains the value `system`, scuv deactivates the current virtual environment and uses the system Python.

```bash
scuv use system          # Write "system" to .scuv-version
scuv shell system        # Set SCUV_VERSION=system (this terminal only)
```

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `SCUV_HOME` | Base directory | `~/.scuv` |
| `SCUV_VERSION` | Override version (highest priority) | (unset) |
| `SCUV_NO_AUTO` | Disable auto-activation | (unset) |
| `SCUV_ACTIVE` | Currently active environment | (set by scuv) |
| `SCUV_RESOLVE_MAX_DEPTH` | Limit parent directory traversal | (unlimited) |

### Disable Auto-Activation

```bash
export SCUV_NO_AUTO=1
```

### Temporary and Project-Scoped Control

Disable only in the current shell session (does not affect global settings):

```bash
export SCUV_NO_AUTO=1
# ...work without auto-activation...
unset SCUV_NO_AUTO
```

For one project directory, use local version files instead of global settings:

```bash
cd ~/project

# Keep auto-activation, but force system Python in this project only
scuv use system

# Or pin a specific environment for this project only
scuv use myproject
```

For temporary per-terminal overrides without changing files:

```bash
scuv shell system    # this terminal only
# ...test...
scuv shell --unset   # return to file-based behavior
```

### Custom Home Directory

```bash
export SCUV_HOME=/custom/path
```

### Network Filesystem Optimization

For slow network filesystems (NFS, SSHFS), limit directory traversal depth:

```bash
# Only check current directory and up to 3 parents
export SCUV_RESOLVE_MAX_DEPTH=3

# Only check current directory (fastest)
export SCUV_RESOLVE_MAX_DEPTH=0
```

## Using with pyenv

Add scuv **after** pyenv in your shell config:

```bash
# ~/.zshrc
eval "$(pyenv init -)"       # 1. pyenv first
eval "$(scuv init zsh)"     # 2. scuv second (takes precedence)
```

## Tab Completion

Shell integration includes completion for:
- Commands and subcommands
- Environment names
- Python versions
- Command options

Completion is automatically enabled by `scuv init`.

## Supported Shells

| Shell | Status |
|-------|--------|
| Zsh | Full support (auto-activation, completion) |
| Bash | Full support (auto-activation, completion) |
| Fish | Full support (auto-activation, completion) |
| PowerShell | Full support (auto-activation, completion) |
