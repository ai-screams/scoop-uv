# use

Set a virtual environment for the current directory and activate it.

## Usage

```bash
scoop use <name> [options]
```

## Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `name` | Yes | Name of the virtualenv to use |

## Options

| Option | Description |
|--------|-------------|
| `--global`, `-g` | Set as global default |
| `--link` | Create `.venv` symlink for IDE compatibility |
| `--no-link` | Do not create `.venv` symlink (default) |

## Behavior

- Creates `.scoop-version` file in current directory
- Immediately activates the environment (if shell hook installed)
- With `--global`: writes to `~/.scoop/version`
- With `--link`: creates `.venv -> ~/.scoop/virtualenvs/<name>`

## Examples

```bash
scoop use myproject              # Use this virtualenv here
scoop use myproject --link       # Also create .venv symlink
scoop use myproject --global     # Set as global default
```
