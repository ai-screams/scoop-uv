# Python Management

scoop delegates all Python installation and discovery to [uv](https://github.com/astral-sh/uv). This page explains how Python versions are found, installed, and used with scoop.

## How Python Discovery Works

When you run `scoop create myenv 3.12`, scoop asks uv to create a virtual environment with Python 3.12. uv searches for a matching Python in this order:

1. **uv-managed installations** in `~/.local/share/uv/python/` (installed via `scoop install` or `uv python install`)
2. **System Python on PATH** — executables named `python`, `python3`, or `python3.x`
3. **Platform-specific locations** — Windows registry, Microsoft Store (Windows only)

> **Key behavior:** For managed Pythons, uv prefers the **newest** matching version. For system Pythons, uv uses the **first compatible** version found on PATH.

## Installing Python Versions

### Via scoop (recommended)

```bash
# Install latest Python
scoop install

# Install specific minor version (latest patch)
scoop install 3.12

# Install exact version
scoop install 3.12.3

# List installed versions
scoop list --pythons
```

### Behind the scenes

`scoop install 3.12` runs `uv python install 3.12` internally. uv downloads a standalone Python build from the [python-build-standalone](https://github.com/indygreg/python-build-standalone) project and stores it in `~/.local/share/uv/python/`.

## Using System Python

scoop can use Python versions already installed on your system (via Homebrew, apt, the OS, etc.) — no `scoop install` needed.

```bash
# Check what Python versions uv can find on your system
uv python list

# Example output:
# cpython-3.13.1    /opt/homebrew/bin/python3.13     (system)
# cpython-3.12.8    ~/.local/share/uv/python/...     (managed)
# cpython-3.12.0    /usr/bin/python3                  (system)
# cpython-3.11.5    /usr/bin/python3.11               (system)

# Create environment using system Python 3.13
# (uv finds it automatically — no scoop install needed)
scoop create myenv 3.13
```

If the version you request matches a system Python, uv will use it. You only need `scoop install` if the version is not already available on your system.

## Using Custom Python Installations

If you have a custom-built Python or an alternative interpreter (PyPy, GraalPy) in a non-standard location, you can make it available to scoop by ensuring uv can discover it.

### Add custom Python to PATH

```bash
# Example: custom Python built from source in /opt/python-debug/
export PATH="/opt/python-debug/bin:$PATH"

# Verify uv can find it
uv python list | grep python
# cpython-3.13.0    /opt/python-debug/bin/python3.13

# Now scoop can use it
scoop create debug-env 3.13
```

### Use UV_PYTHON_INSTALL_DIR

For managed Python installations in a custom location:

```bash
# Store uv-managed Pythons in a custom directory
export UV_PYTHON_INSTALL_DIR=/opt/shared-pythons

# Install Python to the custom location
scoop install 3.12

# All team members can share the same Python installations
```

### Python preference settings

Control whether uv prefers managed or system Python:

```bash
# Use only uv-managed Python (ignore system Python)
UV_PYTHON_PREFERENCE=only-managed scoop create myenv 3.12

# Use only system Python (ignore uv-managed)
UV_PYTHON_PREFERENCE=only-system scoop create myenv 3.12

# Prefer system Python over managed (default: managed first)
UV_PYTHON_PREFERENCE=system scoop create myenv 3.12
```

## Migrating from Other Tools

If you have existing virtual environments in pyenv, conda, or virtualenvwrapper, scoop can migrate them:

```bash
# See what can be migrated
scoop migrate list

# Example output:
# pyenv-virtualenv:
#   myproject (Python 3.12.0)
#   webapp (Python 3.11.8)
# conda:
#   ml-env (Python 3.10.4)

# Migrate a specific environment
scoop migrate @myproject

# Migrate everything at once
scoop migrate --all
```

The migration process:
1. Discovers environments from pyenv (`~/.pyenv/versions/`), conda (`conda info --envs`), or virtualenvwrapper (`$WORKON_HOME`)
2. Creates a new scoop environment with the same Python version
3. Reinstalls packages using uv for improved performance
4. **Preserves** the original environment (no deletion)

See [migrate command](commands/migrate.md) for details.

## Troubleshooting

### Python version not found

```bash
$ scoop create myenv 3.14
# Error: Python 3.14 not found

# Solution 1: Install it via scoop
scoop install 3.14

# Solution 2: Check what's available
uv python list
scoop list --pythons
```

### Using a different Python than expected

```bash
# Check which Python uv would select for a version
uv python find 3.12
# /opt/homebrew/bin/python3.12

# Check all available 3.12 installations
uv python list | grep 3.12
# cpython-3.12.8    /opt/homebrew/bin/python3.12     (system)
# cpython-3.12.7    ~/.local/share/uv/python/...     (managed)
```

### Verify environment's Python

```bash
# Check what Python an environment uses
scoop info myenv
# Name:    myenv
# Python:  3.12.8
# Path:    ~/.scoop/virtualenvs/myenv
```

## Removing Python Versions

Uninstalling a Python version does **not** automatically remove virtual environments that use it. Those environments will break (their Python symlink becomes invalid). Follow this workflow to clean up safely.

### Step-by-step

```bash
# 1. Identify environments using the target Python version
scoop list
#   myproject      3.12.1  ← uses 3.12
#   webapp         3.12.1  ← uses 3.12
#   ml-env         3.11.8

# 2. Remove or recreate affected environments
scoop remove myproject --force
scoop remove webapp --force
# Or recreate with a different version:
# scoop remove myproject --force && scoop create myproject 3.13

# 3. Uninstall the Python version
scoop uninstall 3.12

# 4. Verify everything is clean
scoop list --pythons          # Confirm Python removed
scoop doctor                  # Check for broken environments
```

### Recovery from accidental uninstall

If you already uninstalled Python without cleaning up environments:

```bash
# Detect broken environments
scoop doctor -v
#   ⚠ Environment 'myproject': Python symlink broken

# Fix by reinstalling the Python version
scoop install 3.12
scoop doctor --fix

# Or remove the broken environments and start fresh
scoop remove myproject --force
```

See [uninstall command](commands/uninstall.md) and [doctor command](commands/doctor.md) for details.

## Summary

| Scenario | What to do |
|----------|------------|
| Standard Python version | `scoop install 3.12` then `scoop create myenv 3.12` |
| System Python (Homebrew, apt) | Just `scoop create myenv 3.12` — uv finds it automatically |
| Custom Python in non-standard path | Add to `PATH`, then `scoop create myenv <version>` |
| PyPy or alternative interpreter | Add to `PATH`, uv discovers it automatically |
| Existing pyenv/conda environments | `scoop migrate --all` |
| Shared Python installations | Set `UV_PYTHON_INSTALL_DIR` |
| Force system-only Python | Set `UV_PYTHON_PREFERENCE=only-system` |
| Uninstall Python + cleanup envs | Identify envs → `scoop remove` → `scoop uninstall` → `scoop doctor` |
| Fix broken environments | `scoop doctor --fix` (after reinstalling the Python version) |
