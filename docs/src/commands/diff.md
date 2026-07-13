# diff

Compare two virtualenvs across Python version, installed packages, and
metadata. Useful for spotting drift between teammates' envs, between a
dev env and an export, or between a baseline and a working env when
something breaks.

## Usage

```bash
scuv diff <env-a> <env-b>                   # human table, exit 0
scuv diff <env-a> <env-b> --json            # machine-readable
scuv diff <env-a> <env-b> --strict          # exit 1 if any diff
scuv diff <env-a> <env-b> --packages-only   # skip metadata section
scuv diff <env-a> <env-b> --metadata-only   # skip package enumeration
```

## What gets compared

Three independent sections, all enabled by default:

| Section | Fields |
|---------|--------|
| **Python** | `python_version` from each env's metadata |
| **Packages** | `name==version` set via `uv pip list --format=json` on each env |
| **Metadata** | `python_version`, `created_at`, `last_used`, `uv_version` |

Package matching uses [PEP 503](https://peps.python.org/pep-0503/)
canonical names (lowercase, `-`/`_`/`.` collapsed to single `-`), so
`Requests` vs `requests` and `flask_sqlalchemy` vs `Flask-SQLAlchemy`
are treated as the same package.

## Options

| Option | Description |
|--------|-------------|
| `--json` | Output as JSON (see [JSON Output](#json-output)) |
| `--strict` | Exit `1` if any differences are detected (default: always exit 0) |
| `--packages-only` | Skip the metadata section; package enumeration still runs |
| `--metadata-only` | Skip package enumeration (no `uv` subprocess); metadata section only |

`--packages-only` and `--metadata-only` are mutually exclusive
(`clap`-enforced).

Global flags (`--quiet`, `--no-color`) apply.

## Exit codes

`scuv diff` follows the [layered exit-code contract](../api.md#process-exit-codes):

| Code | Returned when |
|------|---------------|
| `0` | Default — diff reported (even if envs differ) |
| `1` | `--strict` was set AND at least one difference was detected (`DiffMismatch`). Same precedent as `verify --strict`. |

Operational failures (env not found, `uv` missing, corrupt metadata)
return the appropriate underlying `ScoopError` variant via the
catchall exit `1` path; `--strict` is not required for those.

## Examples

### Identical envs

```bash
$ scuv diff webapp webapp-mirror
Environments are identical
```

### Mixed differences

```bash
$ scuv diff webapp webapp-mirror

Python
  ~ python           a: 3.12.0                   b: 3.11.9

Packages (3 differences)
  - numpy==1.26.0
  + pandas==2.2.0
  ~ requests: 2.31.0 → 2.32.0

Metadata
    python_version   a: 3.12.0                   b: 3.11.9
  ~ created_at       a: 2025-01-10T00:00:00Z     b: 2025-03-22T00:00:00Z
    last_used        a: -                        b: -
    uv_version       a: 0.5.14                   b: 0.5.14
```

The `~` marker flags changed scalar fields; `-`/`+` flag removed/added
packages; absent metadata values render as `-`.

### CI gate (fail the build on drift)

```bash
scuv diff baseline production --strict
# exits 1 if baseline and production diverge
```

## JSON output

`--json` emits one envelope to stdout. Success and failure envelopes
share the same `data` shape; only the top-level wrapper differs.

### Success envelope (default exit 0)

```json
{
  "status": "success",
  "command": "diff",
  "data": {
    "env_a": "webapp",
    "env_b": "webapp-mirror",
    "identical": false,
    "python": { "a": "3.12.0", "b": "3.11.9", "changed": true },
    "packages": {
      "added":   [{"name": "pandas", "version": "2.2.0", "display_name": "pandas"}],
      "removed": [{"name": "numpy",  "version": "1.26.0", "display_name": "numpy"}],
      "changed": [{"name": "requests", "version_a": "2.31.0", "version_b": "2.32.0"}]
    },
    "metadata": {
      "python_version": { "a": "3.12.0", "b": "3.11.9", "changed": true },
      "created_at":     { "a": "2025-01-10T00:00:00Z", "b": "2025-03-22T00:00:00Z", "changed": true },
      "last_used":      { "a": null, "b": null, "changed": false },
      "uv_version":     { "a": "0.5.14", "b": "0.5.14", "changed": false }
    },
    "summary": {
      "differences": 5,
      "python_changed": true,
      "packages_added": 1,
      "packages_removed": 1,
      "packages_changed": 1,
      "metadata_fields_changed": 2
    }
  }
}
```

### Strict failure envelope (`--strict` with differences, exit 1)

```json
{
  "status": "error",
  "command": "diff",
  "error": {
    "code": "DIFF_MISMATCH",
    "message": "'webapp' and 'webapp-mirror' differ (5 difference(s))",
    "env_a": "webapp",
    "env_b": "webapp-mirror",
    "differences": 5
  },
  "data": { "...": "same shape as success" }
}
```

### Scalar diff fields — 2-state contract

Each metadata field in `data.metadata` is a `ScalarDiff` carrying both
sides as `Option<T>`:

| `a`        | `b`        | `changed` |
|------------|------------|-----------|
| `"x"`      | `"x"`      | `false`   |
| `"x"`      | `"y"`      | `true`    |
| `"x"`      | `null`     | `true`    |
| `null`     | `null`     | `false`   |

The `null` side means **the value is not observable on that side** —
regardless of whether the metadata file was missing, the field was
absent, or the field was present-as-null in the JSON. Diff intentionally
does not surface the cause; if you need to know why, run `scuv info`
on the env directly. (The `data.packages` shape uses non-nullable
inner objects because pip never emits a "package present but no
version" state.)

### Mode-suppressed sections

- `--packages-only` → `data.metadata` is `null`.
- `--metadata-only` → `data.packages` is `null`; no `uv` subprocess is
  spawned.

`data.python` and `data.summary` are always present.

## See also

- [`verify`](verify.md) — per-env health checks (different scope: one
  env at a time, not pairwise)
- [`info`](info.md) — detailed view of a single env
- [`list`](list.md) — enumerate all envs
- [`api.md`](../api.md) — full process exit-code contract
