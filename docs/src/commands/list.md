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
| `--python-version <VERSION>` | Filter environments by Python version (e.g., `3.12`) |
| `--bare` | Output names only (for scripting) |
| `--json` | Output as JSON |

## Examples

```bash
scoop list                           # List all virtualenvs
scoop list --pythons                 # List installed Python versions
scoop list --bare                    # Names only, one per line
scoop list --json                    # JSON output

# Filter by Python version
scoop list --python-version 3.12     # Show only 3.12.x environments
scoop list --python-version 3        # Show all Python 3.x environments
scoop list --python-version 3.12.1   # Exact version match
```

## Version Filtering

The `--python-version` option uses prefix matching to filter environments:

```bash
scoop list --python-version 3.12
# Output:
#   myproject      3.12.1
#   webapp         3.12.0
# (environments using 3.11 or 3.13 are not shown)
```

This is useful for identifying environments before uninstalling a Python version:

```bash
# See which environments will be affected
scoop list --python-version 3.12

# Then uninstall with cascade
scoop uninstall 3.12 --cascade
```

> **Note:** `--python-version` cannot be combined with `--pythons` (which lists Python installations, not environments).
