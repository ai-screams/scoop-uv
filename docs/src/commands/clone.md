# clone

Duplicate an environment — same Python version, same packages by default —
without going through an export/import roundtrip.

## Usage

```bash
scuv clone <SRC> <DST> [--no-packages] [--force] [--json]
```

## Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `src` | Yes | Name of the source environment |
| `dst` | Yes | Name for the new environment |

## Options

| Option | Description |
|--------|-------------|
| `--no-packages` | Skip package copy — create an empty env at the same Python version |
| `-f`, `--force` | Overwrite the destination if it already exists |
| `--json` | Output as JSON |

## Behaviour

1. Validates `<DST>` (rejects reserved names like `list`, `clone`, …).
2. Refuses a self-clone (`src == dst`).
3. Resolves `<SRC>`'s recorded Python version from its metadata; surfaces a
   clear `CorruptedEnvironment` error when metadata is missing so you know
   recreate-from-scratch is the right next step.
4. Creates `<DST>` at the same Python version.
5. Unless `--no-packages`, lists the source's installed packages via the
   venv's own pip and re-installs them pinned (`name==version`) into the
   destination.

## Examples

```bash
# Full copy
scuv clone myenv myenv-experiment

# Just the shell, no packages
scuv clone myenv myenv-clean --no-packages

# Replace an existing clone
scuv clone myenv myenv-experiment --force
```

## JSON Output

```json
{
  "status": "success",
  "command": "clone",
  "data": {
    "src": "myenv",
    "dst": "myenv-experiment",
    "python": "3.12.7",
    "path": "/Users/me/.scuv/virtualenvs/myenv-experiment",
    "packages_copied": 12,
    "packages_skipped": false
  }
}
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Cloned successfully |
| 1 | Invalid dst name, self-clone, src missing, dst exists without `--force`, or src corrupted |

## See Also

- [`scuv export`](export.md) / [`scuv import`](import.md) — portable JSON
  for cross-machine duplication
- [`scuv create --force`](create.md) — recreate from scratch with a specific
  Python version
