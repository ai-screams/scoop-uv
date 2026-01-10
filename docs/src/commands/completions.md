# completions

Generate shell completion script.

## Usage

```bash
scoop completions <shell>
```

## Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `shell` | Yes | Shell type: `bash`, `zsh`, `fish`, `powershell` |

## Examples

```bash
scoop completions bash           # Output bash completions
scoop completions zsh            # Output zsh completions
scoop completions fish           # Output fish completions
```

> **Tip:** Usually you don't need this separately - `scoop init` includes completions.
