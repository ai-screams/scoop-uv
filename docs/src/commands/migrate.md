# migrate

Migrate virtual environments from other tools (pyenv-virtualenv, virtualenvwrapper, conda).

## Usage

```bash
# List migratable environments
scoop migrate list

# Migrate a single environment
scoop migrate @env <name>

# Migrate all environments
scoop migrate all
```

## Subcommands

| Subcommand | Description |
|------------|-------------|
| `list` | List environments available for migration |
| `@env <name>` | Migrate a single environment by name |
| `all` | Migrate all discovered environments |

## Supported Sources

| Source | Detection |
|--------|-----------|
| **pyenv-virtualenv** | `~/.pyenv/versions/` (non-system virtualenvs) |
| **virtualenvwrapper** | `$WORKON_HOME` or `~/.virtualenvs/` |
| **conda** | `conda info --envs` |

## Options

| Option | Subcommand | Description |
|--------|------------|-------------|
| `--source <pyenv\|virtualenvwrapper\|conda>` | all subcommands | Restrict to a single source tool |
| `--json` | all subcommands | Machine-readable output (see [JSON Output](#json-output)) |
| `--dry-run` | `@env`, `all` | Preview without making changes |
| `--force` | `@env`, `all` | Overwrite existing scoop env with the same name; bypass EOL Python guard |
| `--yes` | `@env`, `all` | Skip the interactive confirmation prompt |
| `--strict` | `@env`, `all` | Fail on the first package install error inside an env (default: keep going) |
| `--delete-source` | `@env`, `all` | Remove the source env after successful migration |
| `--rename <new-name>` | `@env` | Migrate under a different name |
| `--auto-rename` | `@env` | On name conflict, append `-<source>` suffix automatically (conflicts with `--force`) |

Global flags (`--quiet`, `--no-color`) apply to all subcommands.

## Exit codes

`scoop migrate` follows the [layered exit-code contract](../api.md#process-exit-codes). The mapping differs slightly between `migrate all` (which partitions envs into buckets before iterating) and the single-env paths (`@env`, `list`).

### `migrate all`

| Code | Returned when |
|------|---------------|
| `0` | (a) all envs migrated, (b) no envs found but source tools are installed, or (c) only non-conflict skips occurred (EOL / corrupted envs in the skipped bucket, no preflight name conflicts, no per-env failures) |
| `2` | At least one per-env failure **or** at least one preflight name conflict without `--force`. Returned via `MigrationBatchFailed` |
| `3` | No source tool (pyenv / virtualenvwrapper / conda) is detected on the system. Returned via `MigrationSourcesNotFound` |

### `migrate @env <name>`

| Code | Returned when |
|------|---------------|
| `0` | The env migrated successfully (or the user chose `Skip` at the interactive conflict prompt) |
| `2` | `MigrationNameConflict` (env exists in scoop home, `--force` not set, non-interactive context) **or** `MigrationFailed` (e.g. requested env's Python is EOL and `--force` not set) |
| `3` | The named source env was not found in the requested source (`PyenvEnvNotFound`, `VenvWrapperEnvNotFound`, `CondaEnvNotFound`) **or** the source env is `CorruptedEnvironment` |

### `migrate list`

Exit `0` is informational. `list` only fails when a discovery I/O error occurs (exit `1`, via the catchall).

Notes:

- Before v0.14, `migrate all` always exited `0` regardless of per-env outcome. CI gates need `0.14+` to distinguish success from a batch where some envs failed.
- When `migrate all` returns `MigrationBatchFailed`, the human summary and JSON envelope are already on stdout/stderr; `main.rs` suppresses the global `error:` prefix to avoid duplicate noise.

## --force vs --auto-rename (single env, `@env`)

The two flags are mutually exclusive (`clap`-enforced via
`conflicts_with = "force"`). For `migrate all` only `--force` is
available; conflicts without `--force` count toward the exit-2
contract.

### `--force` (recommended for guaranteed resolution)

| Status of source env | With `--force` |
|----------------------|----------------|
| Ready | Migrated normally |
| Name conflict with an existing scoop env | The existing scoop env is overwritten in place |
| EOL Python version (e.g. 2.7) | Migrated anyway (the EOL guard is intentionally bypassed) |
| Corrupted source env | **Not bypassed** — still returns `CorruptedEnvironment` (exit 3) |

### `--auto-rename` (name-conflict only)

`--auto-rename` is a convenience for the pure name-conflict case:
when a scoop env with the same name already exists, the migration
proceeds under an auto-generated name. It does **not** override any
other status (EOL / corrupted), and it does **not** delete or overwrite
the existing scoop env.

Known limitations (tracked separately from this docs PR):

- The generated name is currently hard-coded as `<name>-pyenv` even
  for `virtualenvwrapper` / `conda` sources; the doc-comment promise
  of `<name>-<source>` doesn't yet match the code.
- For envs whose status would be both "name conflict" and "EOL",
  conflict detection runs before EOL detection in `determine_status`,
  so the EOL branch is never reached and `--auto-rename` proceeds with
  the EOL Python silently. Prefer `--force` if you need explicit
  control over EOL semantics in mixed-status envs.

For deterministic conflict handling in scripts, prefer `--force` over
`--auto-rename` until these limitations are addressed.

## Examples

### List Migratable Environments

```bash
$ scoop migrate list
📦 Migratable Environments

  pyenv-virtualenv:
    • myproject (Python 3.12.0)
    • webapp (Python 3.11.8)

  conda:
    • ml-env (Python 3.10.4)
```

### Migrate Single Environment

```bash
$ scoop migrate @env myproject
✓ Migrated 'myproject' from pyenv-virtualenv
  Source: ~/.pyenv/versions/myproject
  Target: ~/.scoop/virtualenvs/myproject
```

### Migrate All

```bash
$ scoop migrate all
✓ Migrated 3 environments
  • myproject (pyenv-virtualenv)
  • webapp (pyenv-virtualenv)
  • ml-env (conda)
```

### CI gate (fail the build on batch failure)

```bash
# Exits 2 if any env failed or a name conflict was skipped.
# Exits 3 if no source tool is installed on the runner.
scoop migrate all --yes
```

## JSON Output

All three subcommands accept `--json`. The envelope follows scoop's standard
shape: `{status, command, data}` on success; `{status: "error", command,
error: { code, message, ... }, data}` on failure paths that already
rendered structured data.

### `migrate list --json`

`data` carries the requested `source` filter string (`"pyenv"`,
`"virtualenvwrapper"`, `"conda"`, or `"all"`), the full `environments`
array, and a `summary` bucketed by status.

```json
{
  "status": "success",
  "command": "migrate list",
  "data": {
    "source": "all",
    "environments": [
      {
        "name": "myproject",
        "python_version": "3.12.0",
        "path": "/home/u/.pyenv/versions/myproject",
        "source_type": "pyenv",
        "size_bytes": 12582912,
        "status": "ready"
      },
      {
        "name": "oldenv",
        "python_version": "2.7.18",
        "path": "/home/u/.pyenv/versions/oldenv",
        "source_type": "pyenv",
        "size_bytes": null,
        "status": "python_eol",
        "version": "2.7.18"
      }
    ],
    "summary": { "total": 2, "ready": 1, "conflict": 0, "eol": 1, "corrupted": 0 }
  }
}
```

The `status` field is a serde-tagged enum (`#[serde(tag = "status",
rename_all = "snake_case")]`) — `"ready"`, `"name_conflict"`
(`existing` payload), `"python_eol"` (`version` payload), or
`"corrupted"` (`reason` payload). `size_bytes` is lazily computed and
may be `null` if not yet requested.

### `migrate all --json` — success path

`MigrateAllData` carries five top-level data keys. `conflicts[]` (new in
0.14) is **additive** — name-conflict envs continue to appear in
`skipped[]` for backward compatibility, and `summary.total ==
migrated.len() + failed.len() + skipped.len()` still holds. `conflicts[]`
is a structured view so consumers can branch on the failure class
without parsing the localized `reason` string in `skipped[]`.

```json
{
  "status": "success",
  "command": "migrate all",
  "data": {
    "migrated": [
      {
        "name": "myproject",
        "python_version": "3.12.0",
        "packages_migrated": 42,
        "packages_failed": [],
        "dry_run": false,
        "path": "/home/u/.scoop/virtualenvs/myproject",
        "source_deleted": false,
        "actual_python_version": "3.12.0"
      }
    ],
    "failed": [],
    "skipped": [],
    "conflicts": [],
    "summary": { "total": 1, "success": 1, "failed": 0, "skipped": 0 }
  }
}
```

`actual_python_version` may differ from `python_version` when uv
selected a compatible interpreter (e.g. requested `3.12`, resolved to
`3.12.4`). `source_deleted` reflects whether `--delete-source` was
honored for this env. `dry_run` mirrors the flag the command was
invoked with.

### `migrate all --json` — failure path (exit 2)

Returned when at least one per-env failure occurred, or at least one
preflight name conflict was detected without `--force`. The envelope
embeds the full data view so consumers don't lose detail on the
failure side either.

```json
{
  "status": "error",
  "command": "migrate all",
  "error": {
    "code": "MIGRATE_BATCH_FAILED",
    "message": "Migration finished with 1 failure(s) and 1 name conflict(s)",
    "failed_count": 1,
    "conflict_count": 1
  },
  "data": {
    "migrated": [],
    "failed": [
      {
        "name": "webapp",
        "source_type": "pyenv",
        "error_code": "MIGRATE_EXTRACTION_FAILED",
        "error": "Couldn't extract packages: pip not found at ..."
      }
    ],
    "skipped": [
      { "name": "myproj", "reason": "name conflict (use --force)" }
    ],
    "conflicts": [
      {
        "name": "myproj",
        "source_type": "pyenv",
        "existing": "/home/u/.scoop/virtualenvs/myproj"
      }
    ],
    "summary": { "total": 2, "success": 0, "failed": 1, "skipped": 1 }
  }
}
```

Per-env failure objects carry two additive fields:

- `source_type` (`"pyenv"`, `"virtualenvwrapper"`, `"conda"`) — origin tool.
- `error_code` — the stable `ScoopError::code()` constant (e.g.
  `"MIGRATE_EXTRACTION_FAILED"`, `"MIGRATE_NAME_CONFLICT"`,
  `"UV_COMMAND_FAILED"`). Scripts branch on this instead of parsing
  `error` (which is localized).

### Exit-3 paths — no JSON envelope on stdout

Two distinct exit-3 cases exist; **neither emits a JSON envelope on
stdout, even under `--json`.** The localized error message and
install/lookup suggestion are written to stderr as plain text; stdout
stays empty. Detect via the exit code.

| Command | Error variant | Trigger |
|---------|---------------|---------|
| `migrate all` | `MigrationSourcesNotFound` | No source tool detected at all (pyenv / virtualenvwrapper / conda) |
| `migrate @env <name>` | `PyenvEnvNotFound` / `VenvWrapperEnvNotFound` / `CondaEnvNotFound` | The named env isn't present in the requested (or any) source |
| `migrate @env <name>` | `CorruptedEnvironment` | The named env exists but its layout is broken (missing python, broken pyvenv.cfg, etc) |

Script template:

```bash
if ! scoop migrate all --json > out.json; then
  case $? in
    2) echo "batch failure — read out.json for detail" ;;
    3) echo "no source tool installed" ;;
  esac
fi
```

The exit-2 path (batch failure) is the only `migrate all` failure path
that emits a structured envelope on stdout. Bridging the exit-3
asymmetry would require `batch.rs` (and `single.rs`) to call
`Output::json_error` before returning Err; tracked for a follow-up.

## Migration Process

1. **Discovery**: Scans configured source paths for virtual environments
2. **Extraction**: Identifies Python version and installed packages
3. **Recreation**: Creates new scoop environment with same Python version
4. **Package Install**: Reinstalls packages using `uv pip install`
5. **Cleanup**: Originals are preserved by default; `--delete-source` removes them after successful migration

## Notes

- Original environments are preserved by default; use `--delete-source` to remove sources after migration
- Package versions are preserved where possible
- Migration creates fresh environments using `uv` for improved performance

## Performance

`scoop migrate all` fans out across all CPU cores via [rayon] when migrating
more than one environment. The dominant cost (uv venv + pip install per env)
is I/O-bound on subprocesses, so wall-clock time scales close to linearly
with core count.

`--dry-run` stays sequential — preview output is more useful when ordered.
Progress lines may interleave when multiple envs finish close together. In
the JSON summary, the `migrated[]` and `failed[]` arrays are sorted
alphabetically by env name (so worker thread scheduling doesn't leak into
the output). `skipped[]` and `conflicts[]` preserve the scan / partition
order — which itself is deterministic (source-type then name; see
`scan_all_environments`).

[rayon]: https://docs.rs/rayon
