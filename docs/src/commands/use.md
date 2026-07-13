# use

Set a virtual environment for the current directory and activate it.

## Usage

```bash
scuv use <name> [options]
scuv use system [options]
scuv use --unset [options]
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
| `--json` | Output result as JSON |

## Behavior

- Creates `.scuv-version` file in current directory
- Immediately activates the environment (if shell hook installed)
- With `--global`: writes to `~/.scuv/version`
- With `--link`: creates `.venv -> ~/.scuv/virtualenvs/<name>`

### Special Value: `system`

Using `system` as the name tells scuv to use the system Python:

```bash
scuv use system           # Use system Python in this directory
scuv use system --global  # Use system Python as global default
```

This writes the literal string `system` to the version file, which the shell hook interprets as "deactivate any virtual environment."

### The `--unset` Flag

Removes the version file entirely:

```bash
scuv use --unset           # Delete .scuv-version in current directory
scuv use --unset --global  # Delete ~/.scuv/version
```

After unsetting, scuv falls back to the next priority level in version resolution.

## Examples

```bash
# Use a virtual environment in this directory
scuv use myproject

# Also create .venv symlink (for IDE support)
scuv use myproject --link

# Set global default environment
scuv use myproject --global

# Use system Python in this directory
scuv use system

# Use system Python globally
scuv use system --global

# Remove local version setting
scuv use --unset

# Remove global version setting
scuv use --unset --global
```

### Set Python 3.11.0 as Global Default

`--global` stores an environment name, not a raw Python version string.
Create an environment with Python 3.11.0, then set that environment globally:

```bash
scuv install 3.11.0
scuv create py311 3.11.0
scuv use py311 --global
```

This writes `py311` to `~/.scuv/version`, which is used in new shell sessions and
directories that do not have a local `.scuv-version`.

If a local `.scuv-version` file or `SCUV_VERSION` environment variable is present,
it takes precedence over the global setting.

## Version File Format

The `.scuv-version` file contains a single line with either:
- An environment name (e.g., `myproject`)
- The literal string `system`

```bash
$ cat .scuv-version
myproject
```
