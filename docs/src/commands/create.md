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
