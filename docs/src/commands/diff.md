# diff

Compare two virtualenvs across Python version, installed packages, and
metadata. Useful for spotting drift between teammates' envs, between a
dev env and an export, or between a baseline and a working env when
something breaks.

## Usage

```bash
scuv diff <env-a> <env-b>                   # human table, exit 0
scuv diff <env-a> <env-b> --json            # machine-readable
scuv diff <env-a> <env-b> --strict          # exit 1 if any diff (created_at counts)
scuv diff <env-a> <env-b> --packages-only   # skip metadata section
scuv diff <env-a> <env-b> --metadata-only   # skip package enumeration
```

## What gets compared

Three independent sections, all enabled by default:

| Section | Fields |
|---------|--------|
| **Python** | `python_version` from each env's metadata |
| **Packages** | name, version and origin of each package: `uv pip list --format=json`, plus the PEP 610 origin each env's Python reports (see [`scuv export`](export.md#schema)) |
| **Metadata** | `python_version`, `created_at`, `last_used`, `uv_version` |

Package matching uses [PEP 503](https://peps.python.org/pep-0503/)
canonical names (lowercase, `-`/`_`/`.` collapsed to single `-`), so
`Requests` vs `requests` and `flask_sqlalchemy` vs `Flask-SQLAlchemy`
are treated as the same package.

A package counts as changed when its version **or** its origin differs:
an editable copy of `idna` 3.4 and PyPI's `idna` 3.4 are a difference,
even though the version is the same. Origins compare by what gets
installed: a git checkout by VCS, URL and commit (not the requested
branch), an archive by URL and hashes (a hash in the URL's `#fragment`,
where uv records it, counts as one), a local directory by its path with
any `subdirectory` joined on. `..` in a path is kept, not folded, since a
symlink can make `a/link/..` lead somewhere else than `a`.

## Options

| Option | Description |
|--------|-------------|
| `--json` | Output as JSON (see [JSON Output](#json-output)) |
| `--strict` | Exit `1` if any differences are detected (default: always exit 0) |
| `--packages-only` | Skip the metadata section; package enumeration still runs |
| `--metadata-only` | Skip package enumeration (no `uv` subprocess); metadata section only |

`--packages-only` and `--metadata-only` are mutually exclusive
(`clap`-enforced).

Global flags (`--quiet`, `--color`, `--no-color`) apply.

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
$ scuv diff webapp webapp-twin --packages-only
• Environments are identical
```

Separately created envs normally differ in `created_at` (each env's
metadata records when it was made), so without `--packages-only` the
metadata section reports at least that one change
(`webapp-twin` here is a `scuv clone` of `webapp`). Comparing an env with
itself prints the same single line.

### Mixed differences

```bash
$ scuv diff webapp webapp-mirror
• webapp vs webapp-mirror
•
• Python
•   ~ python           a: 3.12                      b: 3.11
•
• Packages (4 differences)
•   - six==1.17.0
•   + certifi==2026.7.22
•   ~ idna: 3.6 → 3.7
•   ~ mylib: 0.3.0 (editable /home/me/src/mylib) → 0.3.0 (no recorded origin)
•
• Metadata
•   ~ python_version   a: 3.12                      b: 3.11
•   ~ created_at       a: 2026-01-10T09:12:44.398746+00:00  b: 2026-03-22T14:05:31.266187+00:00
•     last_used        a: -                         b: -
•     uv_version       a: uv 0.x.y (<commit> <date> <target>)  b: uv 0.x.y (<commit> <date> <target>)
```

The `~` marker flags changed scalar fields; `-`/`+` flag removed/added
packages; absent metadata values render as `-`. A package with a recorded
origin shows it in parentheses; a changed package shows both origins when
they differ.

The human report goes to stderr, one `• `-prefixed line each; stdout
stays empty unless `--json` is set.

### CI gate (fail the build on drift)

```bash
scuv diff baseline production --packages-only --strict
# exits 1 if the Python or the packages differ
```

Without `--packages-only`, `--strict` also counts the metadata, and
separately created envs normally differ in `created_at`, so the gate would
fail even when the packages match.

## JSON output

`--json` emits one envelope to stdout. A completed comparison and the
`--strict` failure (`DIFF_MISMATCH`) share the same `data` shape; only
the top-level wrapper differs. An error that stops the comparison (an
env that does not exist, unreadable metadata) prints the generic error
envelope instead: `status`, `command` and `error`, with no `data`.
The success and `--strict` envelopes are printed on a single line
(formatted below for reading); the generic error envelope is indented.

### Success envelope (default exit 0)

```json
{
  "status": "success",
  "command": "diff",
  "data": {
    "env_a": "webapp",
    "env_b": "webapp-mirror",
    "identical": false,
    "python": { "a": "3.12", "b": "3.11", "changed": true },
    "packages": {
      "added":   [{"name": "certifi", "version": "2026.7.22", "display_name": "certifi", "source": null}],
      "removed": [{"name": "six", "version": "1.17.0", "display_name": "six", "source": null}],
      "changed": [
        {"name": "idna", "version_a": "3.6", "version_b": "3.7", "source_a": null, "source_b": null},
        {"name": "mylib", "version_a": "0.3.0", "version_b": "0.3.0",
         "source_a": {"url": "file:///home/me/src/mylib", "dir_info": {"editable": true}}, "source_b": null}
      ]
    },
    "metadata": {
      "python_version": { "a": "3.12", "b": "3.11", "changed": true },
      "created_at":     { "a": "2026-01-10T09:12:44.398746+00:00", "b": "2026-03-22T14:05:31.266187+00:00", "changed": true },
      "last_used":      { "a": null, "b": null, "changed": false },
      "uv_version":     { "a": "uv 0.x.y (<commit> <date> <target>)", "b": "uv 0.x.y (<commit> <date> <target>)", "changed": false }
    },
    "summary": {
      "differences": 6,
      "python_changed": true,
      "packages_added": 1,
      "packages_removed": 1,
      "packages_changed": 2,
      "metadata_fields_changed": 1
    }
  }
}
```

`metadata_fields_changed` counts `created_at`, `last_used` and
`uv_version` only. `metadata.python_version` repeats `data.python` and
is left out, so a Python mismatch adds 1 to `differences`, not 2.

### Strict failure envelope (`--strict` with differences, exit 1)

```json
{
  "status": "error",
  "command": "diff",
  "error": {
    "code": "DIFF_MISMATCH",
    "message": "'webapp' and 'webapp-mirror' differ (6 difference(s))",
    "env_a": "webapp",
    "env_b": "webapp-mirror",
    "differences": 6
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

The `null` side means **the value is not observable on that side**:
the metadata file was missing, or an optional field (`last_used`,
`uv_version`) was absent or `null`. A metadata file missing a required
field (`name`, `python_version`, `created_at`, `created_by`) is invalid,
and the command fails instead. Diff intentionally
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
