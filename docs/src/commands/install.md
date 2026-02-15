# install

Install a Python version.

## Usage

```bash
scoop install [version] [options]
```

## Arguments

| Argument | Required | Default | Description |
|----------|----------|---------|-------------|
| `version` | No | latest | Python version (e.g., `3.12`, `3.11.8`) |

## Options

| Option | Description |
|--------|-------------|
| `--latest` | Install latest stable Python (default) |
| `--stable` | Install oldest fully-supported Python (3.10) |

## Version Resolution

- No argument or `--latest`: installs latest Python 3.x
- `--stable`: installs Python 3.10 (oldest with active security support)
- `3.12`: installs latest 3.12.x patch
- `3.12.3`: installs exact version

## Examples

```bash
scoop install                    # Install latest
scoop install --latest           # Same as above
scoop install --stable           # Install Python 3.10
scoop install 3.12               # Install latest 3.12.x
scoop install 3.12.3             # Install exact 3.12.3
```

> **Note:** Python versions are managed by [uv](https://github.com/astral-sh/uv).

## Python Discovery

You don't always need `scoop install`. When you run `scoop create`, uv searches for a matching Python in this order:

1. **uv-managed** — installed via `scoop install` or `uv python install`
2. **System PATH** — Homebrew, apt, pyenv, or any Python on your `PATH`
3. **Platform-specific** — Windows registry, Microsoft Store

```bash
# See all Python versions uv can find
uv python list

# Example output:
# cpython-3.13.1    /opt/homebrew/bin/python3.13     (system)
# cpython-3.12.8    ~/.local/share/uv/python/...     (managed)
# cpython-3.11.5    /usr/bin/python3.11               (system)

# Use system Python directly — no scoop install needed
scoop create myenv 3.13
```

If the requested version isn't found anywhere, `scoop create` will fail with an error. Use `scoop install <version>` to download it first.

> **See also:** [Python Management](../python-management.md) for custom Python paths, environment variables, and migration.
