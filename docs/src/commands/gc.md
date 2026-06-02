# gc

Garbage-collect orphan virtual environments — directories under `~/.scoop/virtualenvs/` that no longer look like working environments, plus (optionally) environments that haven't been activated in a while.

## Usage

```bash
scoop gc                          # Preview orphans only (default)
scoop gc --yes                    # Actually remove orphans
scoop gc --aggressive             # Also flag unused Python versions
scoop gc --aggressive --yes       # Remove orphans + unused Pythons
scoop gc --older-than 30d         # Also preview envs idle >30 days
scoop gc --older-than 6w --yes    # Remove orphans + stale envs (≥6 weeks idle)
```

## What counts as an orphan?

An environment directory is considered an orphan if either:

- It has no `.scoop-metadata.json` (it wasn't created by scoop, or the metadata was deleted), or
- Its Python interpreter is missing (`bin/python` on Unix / `Scripts/python.exe` on Windows) — typically because the Python version was uninstalled out from under it

Healthy environments are left untouched.

## `--aggressive`

With `--aggressive`, `gc` also reports uv-managed Python versions that no surviving environment references. Pair with `--yes` to uninstall them via `uv python uninstall`.

Without `--aggressive`, Python versions are never touched — even ones that look unused — because manually installed interpreters might be intentionally kept around for ad-hoc use.

## `--older-than <DURATION>`

Flag environments whose `last_used` timestamp is older than the given duration. Accepts `<n>d` (days), `<n>w` (weeks = 7d), and `<n>y` (years = 365d). Examples: `30d`, `2w`, `1y`.

**Months are deliberately rejected** — `m` is ambiguous between "minute" and "month", and calendar months would require timezone-aware arithmetic for a marginal gain in accuracy on a stale-env heuristic. Use `30d` or `1y` instead.

The maximum allowed value is 200 years (`200y`); larger values are rejected to keep the resulting cutoff inside chrono's representable range.

### Conservative rules

Two cases are **never** flagged as stale, by design:

- **`last_used = None`** — fresh envs that have never been activated since the field landed, *and* envs whose metadata predates the field. Either way we have no positive evidence the env is unused.
- **Corrupt metadata** — if we can't read the metadata, we don't pretend to know its age.

If you want to nuke un-activated envs anyway, run `scoop verify` to find them and remove by name.

### TOCTOU guard

Between the `--older-than` scan and the actual delete, an env may be activated. Each candidate is re-checked just before removal:

- `SkippedRecentlyUsed` — the env was touched after the scan; `last_used` is now at-or-newer than the original cutoff, so it is no longer stale.
- `SkippedNoData` — metadata became unreadable or missing between scan and remove. We refuse to delete envs we can no longer reason about.

Both surface in the JSON envelope as `outcome` values so scripts can distinguish them from `Removed` / `Failed`.

## Options

| Option | Description |
|--------|-------------|
| `-y`, `--yes` | Actually remove the candidates (default: preview only) |
| `--aggressive` | Also remove uv-managed Python versions that no environment uses |
| `--older-than <DURATION>` | Also flag envs idle past the cutoff (`30d` / `2w` / `1y`) |
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
scoop gc --older-than 30d --json
```

```json
{
  "status": "success",
  "command": "gc",
  "data": {
    "dry_run": true,
    "envs": [
      { "name": "broken-env", "path": "/Users/x/.scoop/virtualenvs/broken-env", "reason": "broken_python", "outcome": "pending" },
      { "name": "rogue-dir",  "path": "/Users/x/.scoop/virtualenvs/rogue-dir",  "reason": "missing_metadata", "outcome": "pending" },
      { "name": "old-poc",    "path": "/Users/x/.scoop/virtualenvs/old-poc",    "reason": "stale", "age_days": 62, "outcome": "pending" }
    ],
    "pythons": []
  }
}
```

`reason` stays a flat string for all variants — orphans use the
existing `"missing_metadata"` / `"broken_python"` values; stale
records add `"stale"` plus a sibling `age_days` integer. Old consumers
that match on `reason` keep working unchanged; the only additive
change is the new `outcome` values `skipped_recently_used` and
`skipped_no_data`.

## See also

- [`prune`](prune.md) — clean the uv cache
- [`doctor`](doctor.md) — diagnose without removing
- [`remove`](remove.md) — delete a specific environment by name
