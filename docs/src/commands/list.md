# list

List all virtual environments or installed Python versions.

**Aliases:** `ls`

## Usage

```bash
scoop list [options]
```

## Options

| Option | Description |
|--------|-------------|
| `--pythons` | Show Python versions instead of virtualenvs |
| `--bare` | Output names only (for scripting) |
| `--json` | Output as JSON |

## Examples

```bash
scoop list                  # List all virtualenvs
scoop list --pythons        # List installed Python versions
scoop list --bare           # Names only, one per line
scoop list --json           # JSON output
```
