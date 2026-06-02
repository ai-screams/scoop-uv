# export

Write a portable JSON snapshot of an environment so another machine (or
another teammate) can recreate it with [`scoop import`](import.md).

## Usage

```bash
scoop export <name> [-o <PATH>]
```

## Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `name` | Yes | Name of the environment to export |

## Options

| Option | Description |
|--------|-------------|
| `-o`, `--output <PATH>` | Write to this file instead of stdout |

When `-o` is omitted, the JSON document is written to **stdout** and status
messages stay on stderr. That keeps the command pipe-friendly:

```bash
scoop export myenv > myenv.json
scoop export myenv | jq '.packages | length'
```

## Schema

The exported file is versioned (`scoop_export_version`) so a future format
change is detected cleanly rather than silently mis-parsed.

```json
{
  "scoop_export_version": "1",
  "environment": {
    "name": "myproject",
    "python": "3.12.7",
    "created_at": "2026-05-29T12:34:56+00:00"
  },
  "packages": [
    { "name": "pytest", "version": "8.0.0" },
    { "name": "black",  "version": "24.1.0" }
  ]
}
```

Field notes:
- `environment.python` is the resolved version recorded in the env's metadata
  (e.g. `3.12.7`), not the original specifier you typed.
- `environment.created_at` is RFC 3339 and may be absent for hand-authored or
  pre-metadata exports.
- `packages` is what the venv's own pip reports — versions are pinned exactly so
  imports are reproducible.

> **Note (Unreleased / post-0.12.0):** The export schema is still v1
> and intentionally does **not** include the new `last_used`
> timestamp. `last_used` is local usage telemetry — it describes how
> you've been using *this* env, not what an importer needs to recreate
> it elsewhere. Imported envs start fresh with no `last_used` and the
> field populates the first time the new env is activated locally.

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Export written successfully (or printed to stdout) |
| 1 | Env not found, or writing the destination file failed |

## See Also

- [`scoop import`](import.md) — reverse operation
- [`scoop sync`](sync.md) — declarative manifest with looser pinning
