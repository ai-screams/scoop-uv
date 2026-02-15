# remove

Remove a virtual environment.

**Aliases:** `rm`, `delete`

## Usage

```bash
scoop remove <name> [options]
```

## Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `name` | Yes | Name of the virtualenv to remove |

## Options

| Option | Description |
|--------|-------------|
| `--force`, `-f` | Skip confirmation prompt |

## Examples

```bash
scoop remove myproject           # Remove with confirmation
scoop remove myproject --force   # Remove without asking
scoop rm old-env -f              # Using alias
```

## Check Before Removing

To see details about an environment before removing it:

```bash
# Show environment details (Python version, path, packages)
scoop info myproject
# Output:
#   Name:    myproject
#   Python:  3.12.1
#   Path:    ~/.scoop/virtualenvs/myproject
```

## Removing All Environments for a Python Version

To remove all environments that use a specific Python version:

```bash
# List environments to identify which use Python 3.12
scoop list
#   myproject      3.12.1
#   webapp         3.12.1
#   ml-env         3.11.8

# Remove each one
scoop remove myproject --force
scoop remove webapp --force

# Then optionally uninstall the Python version itself
scoop uninstall 3.12
```

> **See also:** [uninstall command](uninstall.md) for the complete workflow to uninstall a Python version and clean up associated environments.
