# prune

Prune the uv cache — deletes unused download archives, wheels, and source artifacts that uv has cached but no longer needs.

## Usage

```bash
scuv prune
```

This is a thin wrapper around [`uv cache prune`](https://docs.astral.sh/uv/reference/cli/#uv-cache-prune). uv decides what's safe to delete; scuv prints `✓ uv cache pruned` and any stdout uv wrote, so you don't have to remember the exact invocation. uv reports its freed-space summary on stderr, which scuv does not forward.

## When to use

- After uninstalling Python versions you no longer need
- When disk space on `~/.cache/uv/` is filling up
- As part of regular cleanup, paired with [`scuv gc`](gc.md) for orphan virtualenvs

## Options

| Option | Description |
|--------|-------------|
| `--json` | Output the result as JSON |

## Examples

```bash
# Standard cleanup
scuv prune

# JSON for a script; data.output holds uv's stdout, which is empty with current uv
scuv prune --json
```

## See also

- [`gc`](gc.md) — garbage-collect orphan virtual environments
- [`doctor`](doctor.md) — health check (does not delete anything)
