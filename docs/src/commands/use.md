# use

Set a virtual environment for the current directory and activate it.

## Usage

```bash
scoop use <name> [options]
scoop use system [options]
scoop use --unset [options]
```

## Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `name` | No | Name of the virtualenv, or `system` for system Python |

## Options

| Option | Description |
|--------|-------------|
| `--unset` | Remove version file (local or global) |
| `--global`, `-g` | Set as global default |
| `--link` | Create `.venv` symlink for IDE compatibility |
| `--no-link` | Do not create `.venv` symlink (default) |

## Behavior

- Creates `.scoop-version` file in current directory
- Immediately activates the environment (if shell hook installed)
- With `--global`: writes to `~/.scoop/version`
- With `--link`: creates `.venv -> ~/.scoop/virtualenvs/<name>`

### Special Value: `system`

Using `system` as the name tells scoop to use the system Python:

```bash
scoop use system           # Use system Python in this directory
scoop use system --global  # Use system Python as global default
```

This writes the literal string `system` to the version file, which the shell hook interprets as "deactivate any virtual environment."

### The `--unset` Flag

Removes the version file entirely:

```bash
scoop use --unset           # Delete .scoop-version in current directory
scoop use --unset --global  # Delete ~/.scoop/version
```

After unsetting, scoop falls back to the next priority level in version resolution.

## Examples

```bash
# Use a virtual environment in this directory
scoop use myproject

# Also create .venv symlink (for IDE support)
scoop use myproject --link

# Set global default environment
scoop use myproject --global

# Use system Python in this directory
scoop use system

# Use system Python globally
scoop use system --global

# Remove local version setting
scoop use --unset

# Remove global version setting
scoop use --unset --global
```

## Version File Format

The `.scoop-version` file contains a single line with either:
- An environment name (e.g., `myproject`)
- The literal string `system`

```bash
$ cat .scoop-version
myproject
```
