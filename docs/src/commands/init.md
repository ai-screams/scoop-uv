# init

Output shell initialization script.

## Usage

```bash
scoop init <shell>
```

## Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `shell` | Yes | Shell type: `bash`, `zsh`, `fish`, `powershell` |

## Setup

Add to your shell configuration:

```bash
# Bash (~/.bashrc)
eval "$(scoop init bash)"

# Zsh (~/.zshrc)
eval "$(scoop init zsh)"
```

## Features Enabled

- Auto-activation when entering directories with `.scoop-version`
- Tab completion for commands, environments, and options
- Wrapper function for `activate`/`deactivate`/`use`

## Examples

```bash
scoop init bash                  # Output bash init script
scoop init zsh                   # Output zsh init script
```
