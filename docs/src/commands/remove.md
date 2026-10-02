# remove

Remove a virtual environment.

**Aliases:** `rm`, `delete`

## Usage

```bash
scuv remove <name> [options]
```

## Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `name` | Yes | Name of the virtualenv to remove |

## Options

| Option | Description |
|--------|-------------|
| `--force`, `-f` | Skip confirmation prompt |
| `--json` | Output result as JSON |

## Examples

```bash
scuv remove myproject           # Remove with confirmation
scuv remove myproject --force   # Remove without asking
scuv rm old-env -f              # Using alias
```

## Project `.venv` Link

If the current directory has a `.venv` symlink to the environment being
removed (made by `scuv use <name> --link`), `remove` deletes that link too,
so uv and editors do not trip over a dangling `.venv`. A real `.venv`
directory, or a link to anything else, is left alone. Under `--json` the
removed link is reported as `unlinked`.

Only the current directory is checked. If you remove the environment from
somewhere else, `scuv doctor` in the project reports the dangling link and
`scuv doctor --fix` removes it.

## Check Before Removing

To see details about an environment before removing it:

```bash
# Show environment details (Python version, path, packages)
scuv info myproject
# Output:
#   Name:    myproject
#   Python:  3.12.1
#   Path:    ~/.scuv/virtualenvs/myproject
```

## Removing All Environments for a Python Version

To remove all environments that use a specific Python version:

```bash
# List environments to identify which use Python 3.12
scuv list
#   myproject      3.12.1
#   webapp         3.12.1
#   ml-env         3.11.8

# Remove each one
scuv remove myproject --force
scuv remove webapp --force

# Then optionally uninstall the Python version itself
scuv uninstall 3.12
```

> **See also:** [uninstall command](uninstall.md) for the complete workflow to uninstall a Python version and clean up associated environments.
