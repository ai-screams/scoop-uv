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
| `--python-path <PATH>` | Use a specific Python executable instead of version discovery |

## Examples

```bash
scoop create myproject 3.12      # Create with Python 3.12
scoop create webapp              # Create with latest Python
scoop create myenv 3.11 --force  # Overwrite if exists

# Use a specific Python executable
scoop create myenv --python-path /opt/python-debug/bin/python3
scoop create graal --python-path /opt/graalpy/bin/graalpy
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

## Custom Python Executable

Use `--python-path` to create a virtualenv with a specific Python binary. This is useful for:
- Custom-built Python (debug builds, optimized builds)
- Alternative interpreters (PyPy, GraalPy)
- Python installations in non-standard locations

```bash
# Debug build from source
scoop create debug-env --python-path /opt/python-debug/bin/python3

# PyPy interpreter
scoop create pypy-env --python-path /opt/pypy/bin/pypy3

# GraalPy
scoop create graal-env --python-path /opt/graalpy/bin/graalpy
```

The path must point to a valid, executable Python binary. scoop will:
1. Validate the path (exists, is a file, is executable)
2. Auto-detect the Python version from the binary
3. Store the custom path in the environment's metadata

You can verify the custom path with `scoop info`:

```bash
scoop info debug-env
# Name:         debug-env
# Python:       3.13.0
# Python Path:  /opt/python-debug/bin/python3
# Path:         ~/.scoop/virtualenvs/debug-env
```
