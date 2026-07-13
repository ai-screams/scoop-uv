# init

Output shell initialization script.

## Usage

```bash
scuv init <shell>
```

## Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `shell` | Yes | Shell type: `bash`, `zsh`, `fish`, `powershell` (alias: `pwsh`) |

## Setup

Add to your shell configuration:

```bash
# Bash (~/.bashrc)
eval "$(scuv init bash)"

# Zsh (~/.zshrc)
eval "$(scuv init zsh)"
```

```fish
# Fish (~/.config/fish/config.fish)
scuv init fish | source
```

```powershell
# PowerShell ($PROFILE)
Invoke-Expression (& scuv init powershell)
```

## Features Enabled

- Auto-activation when entering directories with `.scuv-version`
- Tab completion for commands, environments, and options
- Wrapper function for `activate`/`deactivate`/`use`

## Examples

```bash
scuv init bash                  # Output bash init script
scuv init zsh                   # Output zsh init script
scuv init fish                  # Output fish init script
scuv init powershell            # Output PowerShell init script
```
