# create

Create a new virtual environment.

## Usage

```bash
scoop create <name> [python-version]
```

## Arguments

| Argument | Required | Default | Description |
|----------|----------|---------|-------------|
| `name` | Yes | - | Name for the new virtualenv |
| `python-version` | No | `3` (latest) | Python version (e.g., `3.12`, `3.11.8`) |

## Options

| Option | Description |
|--------|-------------|
| `--force`, `-f` | Overwrite existing virtualenv |

## Examples

```bash
scoop create myproject 3.12      # Create with Python 3.12
scoop create webapp              # Create with latest Python
scoop create myenv 3.11 --force  # Overwrite if exists
```

## Python Version Resolution

scoop delegates Python discovery to [uv](https://github.com/astral-sh/uv). The `python-version` argument is passed to `uv venv --python`, which searches for a match in:

1. uv-managed Python installations
2. System Python on `PATH` (Homebrew, apt, pyenv, etc.)
3. Platform-specific locations (Windows only)

```bash
# Uses uv-managed Python 3.12 (if installed via scoop install)
scoop create myenv 3.12

# Also works with system Python — no scoop install needed
# (e.g., if Homebrew has python@3.13)
scoop create myenv 3.13

# Check what Python versions are available
uv python list
scoop list --pythons
```

> **Tip:** If the version isn't found, install it first with `scoop install 3.12`. See [Python Management](../python-management.md) for custom Python paths.
