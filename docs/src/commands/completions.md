# completions

Generate shell completion script.

## Usage

```bash
scoop completions <shell>
```

## Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `shell` | Yes | Shell type: `bash`, `zsh`, `fish`, `powershell` (alias: `pwsh`) |

## Examples

```bash
scoop completions bash           # Output bash completions
scoop completions zsh            # Output zsh completions
scoop completions fish           # Output fish completions
scoop completions powershell     # Output PowerShell completions
```

```powershell
# PowerShell — add to $PROFILE
scoop completions powershell | Out-String | Invoke-Expression
```

> **Tip:** Usually you don't need this separately - `scoop init` includes completions.
