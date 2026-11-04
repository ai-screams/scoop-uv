# Shell Integration

scuv uses a shell wrapper pattern (like pyenv) where the CLI outputs shell code that gets evaluated by the shell. `scuv use` itself only writes the version file; the wrapper then evaluates `scuv activate` (or `deactivate` for `system`).

## How It Works

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│ User runs    │ --> │ CLI outputs  │ --> │ Shell evals  │
│ scuv activate│     │ export ...   │     │ the output   │
└──────────────┘     └──────────────┘     └──────────────┘
```

The `scuv` shell function wraps the CLI binary (simplified; the real one also
passes help flags through and respects an explicit `--shell`):

```bash
scuv() {
    case "$1" in
        use)
            command scuv "$@" || return
            local name=""
            shift
            for arg in "$@"; do
                case "$arg" in
                    -*) ;;
                    *) name="$arg"; break ;;
                esac
            done
            if [[ "$name" == [Ss][Yy][Ss][Tt][Ee][Mm] ]]; then
                eval "$(command scuv deactivate --shell bash)"
            elif [[ -n "$name" ]]; then
                eval "$(command scuv activate --shell bash "$name")"
            fi
            ;;
        activate|deactivate|shell)
            # Capture first, so a failed call keeps its exit status
            local script
            script="$(command scuv "$1" --shell bash "${@:2}")" || return
            eval "$script"
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
Add-Content $PROFILE 'Invoke-Expression (& scuv init powershell | Out-String)'
# Restart PowerShell
```

Keep the `| Out-String`: `scuv init powershell` prints many lines, and
`Invoke-Expression` takes one string, not the array of lines `&` returns.
A profile line without it never defines the `scuv` function; if yours
lacks it, update the line.

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
function _scuv_hook
    # Check for version file and activate/deactivate
end

function _scuv_on_pwd --on-variable PWD
    _scuv_hook
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
| `SCUV_NO_AUTO` | Read at `scuv init`; see [Disable Auto-Activation](#disable-auto-activation) | (unset) |
| `SCUV_ACTIVE` | Currently active environment | (set by scuv) |
| `SCUV_RESOLVE_MAX_DEPTH` | Limit parent directory traversal | (unlimited) |

### Disable Auto-Activation

`SCUV_NO_AUTO` is read once, when your shell sources `scuv init`, so set it to a non-empty
value before the `scuv init` line in your rc file:

```bash
export SCUV_NO_AUTO=1
eval "$(scuv init bash)"
```

It turns off both automatic activations in every shell: the one when the shell starts and
the one when you change directory (bash and PowerShell check on each prompt, zsh and fish
on each `cd`). An empty value counts as unset. `scuv activate`, `scuv use` and `scuv shell`
still work.

Changing the variable in a running shell takes effect when the init script is sourced again
(`source ~/.zshrc`, for example), which also removes the hook the earlier sourcing added.
Up to scuv 0.18.1 the variable turned off only one of the two, and which one depended on
the shell.

### Temporary and Project-Scoped Control

Stop activation in the current shell session only (does not affect global settings):

```bash
scuv shell system
# ...work without auto-activation...
scuv shell --unset
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
