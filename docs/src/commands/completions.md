# completions

Generate shell completion script.

## Usage

```bash
scuv completions <shell>
```

## Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `shell` | Yes | Shell type: `bash`, `zsh`, `fish`, `powershell` (alias: `pwsh`) |

## Examples

```bash
scuv completions bash           # Output bash completions
scuv completions zsh            # Output zsh completions
scuv completions fish           # Output fish completions
scuv completions powershell     # Output PowerShell completions
```

```powershell
# PowerShell — add to $PROFILE
scuv completions powershell | Out-String | Invoke-Expression
```

> **Tip:** Usually you don't need this separately - `scuv init` includes completions.
