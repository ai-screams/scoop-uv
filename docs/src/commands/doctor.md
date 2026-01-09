# doctor

Check scoop installation health and diagnose issues.

## Usage

```bash
scoop doctor [options]
```

## Options

| Option | Description |
|--------|-------------|
| `-v`, `--verbose` | Show more details (can repeat: `-vv`) |
| `--json` | Output diagnostics as JSON |
| `--fix` | Auto-fix issues where possible |

## Checks Performed

- uv installation and version
- Shell integration status
- Environment integrity
- Path configuration
- Version file validity

## Examples

```bash
scoop doctor                     # Quick health check
scoop doctor -v                  # Verbose diagnostics
scoop doctor --fix               # Fix what can be fixed
scoop doctor --json              # JSON output for scripting
```
