# gc

Garbage-collect orphan virtual environments — directories under `~/.scuv/virtualenvs/` that no longer look like working environments, plus (optionally) environments that haven't been activated in a while.

## Usage

```bash
scuv gc                          # Preview orphans only (default)
scuv gc --yes                    # Actually remove orphans
scuv gc --aggressive             # Also flag unused Python versions
scuv gc --aggressive --yes       # Remove orphans + unused Pythons
scuv gc --older-than 30d         # Also preview envs idle >30 days
scuv gc --older-than 6w --yes    # Remove orphans + stale envs (≥6 weeks idle)
```

## What counts as an orphan?

An environment directory is considered an orphan if either:

- It has no `.scoop-metadata.json` (it wasn't created by scuv, or the metadata was deleted), or
- Its Python interpreter is missing (`bin/python` on Unix / `Scripts/python.exe` on Windows) — typically because the Python version was uninstalled out from under it

Healthy environments are left untouched.

## `--aggressive`

With `--aggressive`, `gc` also reports uv-managed Python versions that no surviving environment references. Pair with `--yes` to uninstall them via `uv python uninstall`.

An environment records the version it was created for. uv links environments to a minor version, so one recorded as `3.12` uses whichever 3.12.x is installed, and keeps every installed 3.12.x; one recorded as `3.12.1` keeps only 3.12.1. Pythons that uv does not manage (Homebrew, `/usr/bin/python3`) are never reported.

Without `--aggressive`, Python versions are never touched — even ones that look unused — because manually installed interpreters might be intentionally kept around for ad-hoc use.

### When Pythons are left alone

`gc` reports a Python as unused only when it can tell which Python every remaining environment uses — every environment it lists that is not itself being cleaned up. When it cannot, it skips Python cleanup instead of guessing:

- **The environment directory cannot be read** — if `~/.scuv/virtualenvs` is unreadable from the start, `gc` stops with an error before touching anything. If it becomes unreadable later, during the Python scan, a warning is printed and no Python is reported.
- **A remaining environment has unreadable metadata** — its Python version is unknown, so a warning is printed and no Python is reported. A cleanup candidate with unreadable metadata does not count: it is going away.

Warnings are not printed under `--json` or `--quiet`; the JSON output then has an empty `pythons` array.

With `--yes`, the Python scan runs again right before uninstalling. Every environment it lists at that moment protects the Python it uses, including a candidate that `gc` decided to keep (see [TOCTOU guard](#toctou-guard)) or failed to remove. If this second scan cannot tell which Pythons are in use, nothing is uninstalled, and each Python left alone gets a warning and the JSON outcome `skipped_in_use`. If uv itself can no longer be found at that point, nothing is uninstalled either; those Pythons get the outcome `skipped_no_uv`.

## `--older-than <DURATION>`

Flag environments whose `last_used` timestamp is older than the given duration. Accepts `<n>d` (days), `<n>w` (weeks = 7d), and `<n>y` (years = 365d). Examples: `30d`, `2w`, `1y`.

**Months are deliberately rejected** — `m` is ambiguous between "minute" and "month", and calendar months would require timezone-aware arithmetic for a marginal gain in accuracy on a stale-env heuristic. Use `30d` or `1y` instead.

The maximum allowed value is 200 years (`200y`); larger values are rejected to keep the resulting cutoff inside chrono's representable range.

> **Note on system clock**: the cutoff is `Utc::now() - <duration>`, so the
> threshold moves with the system clock. A host whose clock is wrong (NTP
> compromise, manual `date` set, or hibernated VM that woke up with a stale
> time) can shift which envs `gc --older-than` considers stale. This is a
> best-effort heuristic, not a security boundary — pair it with `--yes` only
> when you trust the clock.

### Conservative rules

Two cases are **never** flagged as stale, by design:

- **`last_used = None`** — fresh envs that have never been activated since the field landed, *and* envs whose metadata predates the field. Either way we have no positive evidence the env is unused.
- **Corrupt metadata** — if we can't read the metadata, we don't pretend to know its age.

If you want to clean up un-activated envs anyway, surface them with
`scuv list --sort last-used` — envs missing `last_used` always sort to
the bottom — and remove individual ones with `scuv remove <name>`.
For scripted enumeration:

```bash
scuv list --json | jq -r '.data.virtualenvs[] | select(.last_used == null) | .name'
```

(Note: `scuv verify` checks per-env health — metadata / interpreter /
manifest drift — and intentionally does NOT flag a healthy env just
because it has never been activated.)

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
scuv gc

# Sample output:
# • Orphan virtualenvs (2):
#   - broken-env (Python interpreter missing)  ~/.scuv/virtualenvs/broken-env
#   - rogue-dir (no .scoop-metadata.json)  ~/.scuv/virtualenvs/rogue-dir
# • (dry run — pass `--yes` to actually remove)

# Stale envs join the same list, in name order
scuv gc --older-than 30d

# Sample output:
# • Orphan virtualenvs (3):
#   - broken-env (Python interpreter missing)  ~/.scuv/virtualenvs/broken-env
#   - old-poc (stale (62 days idle))  ~/.scuv/virtualenvs/old-poc
#   - rogue-dir (no .scoop-metadata.json)  ~/.scuv/virtualenvs/rogue-dir
# • (dry run — pass `--yes` to actually remove)

# Unused uv-managed Pythons get their own list
scuv gc --aggressive

# Sample output:
# • Orphan virtualenvs (2):
#   - broken-env (Python interpreter missing)  ~/.scuv/virtualenvs/broken-env
#   - rogue-dir (no .scoop-metadata.json)  ~/.scuv/virtualenvs/rogue-dir
# • Unused Python versions (2):
#   - Python 3.13.x
#   - Python 3.11.x
# • (dry run — pass `--yes` to actually remove)

# Actually clean up
scuv gc --yes
```

The `- ` item lines go to stdout; the `• ` header and footer lines go to
stderr, so `scuv gc 2>/dev/null` prints only the candidates. With nothing
to remove, `gc` prints `✓ Nothing to clean up — all environments look
healthy` (stderr).

## JSON output

```bash
scuv gc --json
scuv gc --older-than 30d --json
```

```json
{
  "status": "success",
  "command": "gc",
  "data": {
    "dry_run": true,
    "envs": [
      { "name": "broken-env", "path": "/Users/x/.scuv/virtualenvs/broken-env", "reason": "broken_python", "outcome": "pending" },
      { "name": "old-poc",    "path": "/Users/x/.scuv/virtualenvs/old-poc",    "reason": "stale", "age_days": 62, "outcome": "pending" },
      { "name": "rogue-dir",  "path": "/Users/x/.scuv/virtualenvs/rogue-dir",  "reason": "missing_metadata", "outcome": "pending" }
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
