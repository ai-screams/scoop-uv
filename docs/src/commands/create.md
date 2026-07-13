# create

Create a new virtual environment.

## Usage

```bash
scuv create <name> [python-version]
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
| `--install-python` | Install the requested Python version first if it's not already available (conflicts with `--python-path`) |
| `--json` | Output result as JSON |

## Examples

```bash
scuv create myproject 3.12      # Create with Python 3.12
scuv create webapp              # Create with latest Python
scuv create myenv 3.11 --force  # Overwrite if exists

# Auto-install Python first if the version is missing
scuv create myenv 3.13 --install-python

# Use a specific Python executable
scuv create myenv --python-path /opt/python-debug/bin/python3
scuv create graal --python-path /opt/graalpy/bin/graalpy
```

### Create a Project Environment with Python 3.9.5

```bash
# Install exact Python version (skip if already available)
scuv install 3.9.5

# Create a new project environment using that exact version
scuv create myproject 3.9.5

# Verify the environment uses Python 3.9.5
scuv info myproject
```

If `3.9.5` is not available, install it first with `scuv install 3.9.5`, then check discovery with
`uv python list` and `scuv list --pythons`.

## Python Version Resolution

scuv delegates Python discovery to [uv](https://github.com/astral-sh/uv). The `python-version` argument is passed to `uv venv --python`, which searches for a match in:

1. uv-managed Python installations
2. System Python on `PATH` (Homebrew, apt, pyenv, etc.)
3. Platform-specific locations (Windows only)

```bash
# Uses uv-managed Python 3.12 (if installed via scuv install)
scuv create myenv 3.12

# Also works with system Python — no scuv install needed
# (e.g., if Homebrew has python@3.13)
scuv create myenv 3.13

# Check what Python versions are available
uv python list
scuv list --pythons
```

> **Tip:** If the version isn't found, install it first with `scuv install 3.12`. See [Python Management](../python-management.md) for custom Python paths.

## Custom Python Executable

Use `--python-path` to create a virtualenv with a specific Python binary. This is useful for:
- Custom-built Python (debug builds, optimized builds)
- Alternative interpreters (PyPy, GraalPy)
- Python installations in non-standard locations

```bash
# Debug build from source
scuv create debug-env --python-path /opt/python-debug/bin/python3

# PyPy interpreter
scuv create pypy-env --python-path /opt/pypy/bin/pypy3

# GraalPy
scuv create graal-env --python-path /opt/graalpy/bin/graalpy
```

The path must point to a valid, executable Python binary. scuv will:
1. Validate the path (exists, is a file, is executable)
2. Auto-detect the Python version from the binary
3. Store the custom path in the environment's metadata

You can verify the custom path with `scuv info`:

```bash
scuv info debug-env
# Name:         debug-env
# Python:       3.13.0
# Python Path:  /opt/python-debug/bin/python3
# Path:         ~/.scuv/virtualenvs/debug-env
```
