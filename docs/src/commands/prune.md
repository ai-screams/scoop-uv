# prune

Prune the uv cache — deletes unused download archives, wheels, and source artifacts that uv has cached but no longer needs.

## Usage

```bash
scoop prune
```

This is a thin wrapper around [`uv cache prune`](https://docs.astral.sh/uv/reference/cli/#uv-cache-prune). uv decides what's safe to delete; scoop just forwards the result so you don't have to remember the exact invocation.

## When to use

- After uninstalling Python versions you no longer need
- When disk space on `~/.cache/uv/` is filling up
- As part of regular cleanup, paired with [`scoop gc`](gc.md) for orphan virtualenvs

## Options

| Option | Description |
|--------|-------------|
| `--json` | Output the result as JSON |

## Examples

```bash
# Standard cleanup
scoop prune

# Capture freed-bytes for a script
scoop prune --json | jq -r '.data.output'
```

## See also

- [`gc`](gc.md) — garbage-collect orphan virtual environments
- [`doctor`](doctor.md) — health check (does not delete anything)
