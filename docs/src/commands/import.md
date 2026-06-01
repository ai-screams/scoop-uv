# import

Recreate an environment from a [`scoop export`](export.md) JSON file.

## Usage

```bash
scoop import <PATH> [--name <NEW_NAME>] [--force] [--json]
scoop import -    [--name <NEW_NAME>]                # read from stdin
```

## Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `path` | Yes | Path to the export JSON, or `-` to read from stdin |

## Options

| Option | Description |
|--------|-------------|
| `--name <NAME>` | Override the env name from the file (validated like `scoop create`) |
| `-f`, `--force` | Overwrite an existing environment with the same name |
| `--json` | Output as JSON |

## Behaviour

1. Reads and validates the export schema. Mismatched `scoop_export_version`
   produces a clear `EXPORT_UNSUPPORTED_VERSION` error pointing at upgrade
   guidance instead of trying to limp on.
2. Applies `--name` override (if any) and validates the resulting name.
3. If the target env already exists: errors out unless `--force` is set, in
   which case the existing env is removed first.
4. Auto-installs the requested Python if it isn't already available via uv
   (matches the ergonomics of [`scoop sync`](sync.md)).
5. Creates the env, then `uv pip install`s every pinned package
   (`name==version`) in one shot.

## Examples

```bash
# Plain file -> recreates with the schema's original name
scoop import myenv.json

# Pipe from a sibling machine
ssh other 'scoop export myenv' | scoop import -

# Rename on the fly + overwrite if it already exists
scoop import myenv.json --name myenv-2 --force

# Machine-readable summary for CI
scoop import myenv.json --json
```

## JSON Output

```json
{
  "status": "success",
  "command": "import",
  "data": {
    "name": "myenv",
    "python": "3.12.7",
    "packages_installed": 42,
    "source": "/path/to/myenv.json"
  }
}
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Imported successfully |
| 1 | Invalid file, unsupported schema version, invalid name, or env existed without `--force` |

## See Also

- [`scoop export`](export.md) — produce the input file
- [`scoop sync`](sync.md) — for declarative `.scoop.toml`-driven workflows
