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
