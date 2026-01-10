# Shell Integration

scoop uses a shell wrapper pattern (like pyenv) where the CLI outputs shell code that gets evaluated by the shell.

## How It Works

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│ User runs   │ --> │ CLI outputs │ --> │ Shell evals │
│ scoop use   │     │ export ...  │     │ the output  │
└─────────────┘     └─────────────┘     └─────────────┘
```

The `scoop` shell function wraps the CLI binary:

```bash
scoop() {
    case "$1" in
        use)
            command scoop "$@"
            eval "$(command scoop activate "$name")"
            ;;
        activate|deactivate)
            eval "$(command scoop "$@")"
            ;;
        *)
            command scoop "$@"
            ;;
    esac
}
```

## Setup

### Zsh

```bash
echo 'eval "$(scoop init zsh)"' >> ~/.zshrc
source ~/.zshrc
```

### Bash

```bash
echo 'eval "$(scoop init bash)"' >> ~/.bashrc
source ~/.bashrc
```

### Fish

```fish
echo 'eval (scoop init fish)' >> ~/.config/fish/config.fish
source ~/.config/fish/config.fish
```

## Auto-Activation

When enabled, scoop automatically activates environments based on version files.

**Zsh**: Uses `chpwd` hook (runs on directory change)

```bash
autoload -Uz add-zsh-hook
add-zsh-hook chpwd _scoop_hook
```

**Bash**: Uses `PROMPT_COMMAND`

```bash
PROMPT_COMMAND="_scoop_hook;$PROMPT_COMMAND"
```

**Fish**: Uses `--on-variable PWD` event handler

```fish
function _scoop_hook --on-variable PWD
    # Check for version file and activate/deactivate
end
```

The hook checks for version files and activates/deactivates accordingly.

## Version File Resolution

scoop checks these files in order (first match wins):

| File | Scope |
|------|-------|
| `.scoop-version` | Project-specific |
| `.python-version` | pyenv compatibility |
| `~/.scoop/version` | Global default |

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `SCOOP_HOME` | Base directory | `~/.scoop` |
| `SCOOP_NO_AUTO` | Disable auto-activation | (unset) |
| `SCOOP_ACTIVE` | Currently active environment | (set by scoop) |
| `SCOOP_RESOLVE_MAX_DEPTH` | Limit parent directory traversal | (unlimited) |

### Disable Auto-Activation

```bash
export SCOOP_NO_AUTO=1
```

### Custom Home Directory

```bash
export SCOOP_HOME=/custom/path
```

### Network Filesystem Optimization

For slow network filesystems (NFS, SSHFS), limit directory traversal depth:

```bash
# Only check current directory and up to 3 parents
export SCOOP_RESOLVE_MAX_DEPTH=3

# Only check current directory (fastest)
export SCOOP_RESOLVE_MAX_DEPTH=0
```

## Using with pyenv

Add scoop **after** pyenv in your shell config:

```bash
# ~/.zshrc
eval "$(pyenv init -)"       # 1. pyenv first
eval "$(scoop init zsh)"     # 2. scoop second (takes precedence)
```

## Tab Completion

Shell integration includes completion for:
- Commands and subcommands
- Environment names
- Python versions
- Command options

Completion is automatically enabled by `scoop init`.

## Supported Shells

| Shell | Status |
|-------|--------|
| Zsh | Full support (auto-activation, completion) |
| Bash | Full support (auto-activation, completion) |
| Fish | Full support (auto-activation, completion) |
| PowerShell | Planned (P3) |
