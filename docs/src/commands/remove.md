# remove

Remove a virtual environment.

**Aliases:** `rm`, `delete`

## Usage

```bash
scoop remove <name> [options]
```

## Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `name` | Yes | Name of the virtualenv to remove |

## Options

| Option | Description |
|--------|-------------|
| `--force`, `-f` | Skip confirmation prompt |

## Examples

```bash
scoop remove myproject           # Remove with confirmation
scoop remove myproject --force   # Remove without asking
scoop rm old-env -f              # Using alias
```
