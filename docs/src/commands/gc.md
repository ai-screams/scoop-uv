# gc

Garbage-collect orphan virtual environments — directories under `~/.scoop/virtualenvs/` that no longer look like working environments.

## Usage

```bash
scoop gc                    # Preview only (default)
scoop gc --yes              # Actually remove orphans
scoop gc --aggressive       # Also flag unused Python versions
scoop gc --aggressive --yes # Remove orphans + unused Pythons
```

## What counts as an orphan?

An environment directory is considered an orphan if either:

- It has no `.scoop-metadata.json` (it wasn't created by scoop, or the metadata was deleted), or
- Its Python interpreter is missing (`bin/python` on Unix / `Scripts/python.exe` on Windows) — typically because the Python version was uninstalled out from under it

Healthy environments are left untouched.

## `--aggressive`

With `--aggressive`, `gc` also reports uv-managed Python versions that no surviving environment references. Pair with `--yes` to uninstall them via `uv python uninstall`.

Without `--aggressive`, Python versions are never touched — even ones that look unused — because manually installed interpreters might be intentionally kept around for ad-hoc use.

## Options

| Option | Description |
|--------|-------------|
| `-y`, `--yes` | Actually remove the orphans (default: preview only) |
| `--aggressive` | Also remove uv-managed Python versions that no environment uses |
| `--json` | Output as JSON |

## Examples

```bash
# See what would be removed
scoop gc

# Sample output:
# Orphan virtualenvs (2):
#   - broken-env (Python interpreter missing)  ~/.scoop/virtualenvs/broken-env
#   - rogue-dir  (no .scoop-metadata.json)     ~/.scoop/virtualenvs/rogue-dir
# (dry run — pass `--yes` to actually remove)

# Actually clean up
scoop gc --yes
```

## JSON output

```bash
scoop gc --json
```

```json
{
  "status": "success",
  "command": "gc",
  "data": {
    "dry_run": true,
    "envs": [
      { "name": "broken-env", "path": "/Users/x/.scoop/virtualenvs/broken-env", "reason": "broken_python" }
    ],
    "pythons": []
  }
}
```

## See also

- [`prune`](prune.md) — clean the uv cache
- [`doctor`](doctor.md) — diagnose without removing
- [`remove`](remove.md) — delete a specific environment by name
