# migrate

Migrate virtual environments from other tools (pyenv-virtualenv, virtualenvwrapper, conda).

## Usage

```bash
# List migratable environments
scuv migrate list

# Migrate a single environment
scuv migrate @env <name>

# Migrate all environments
scuv migrate all
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
| `--force` | `@env`, `all` | Overwrite existing scuv env with the same name; bypass EOL Python guard |
| `--yes` | `@env`, `all` | Skip the interactive confirmation prompt |
| `--strict` | `@env`, `all` | Fail on the first package install error inside an env (default: keep going) |
| `--delete-source` | `@env`, `all` | Remove the source env after successful migration |
| `--rename <new-name>` | `@env` | Migrate under a different name |
| `--auto-rename` | `@env` | On name conflict, migrate under `<name>-pyenv` automatically (conflicts with `--force`) |

Global flags (`--quiet`, `--color`, `--no-color`) apply to all subcommands.

## Exit codes

`scuv migrate` follows the [layered exit-code contract](../api.md#process-exit-codes). The mapping differs slightly between `migrate all` (which partitions envs into buckets before iterating) and the single-env paths (`@env`, `list`).

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
| `2` | `MigrationNameConflict` (env exists in scuv home, `--force` not set, and `--yes` or `--json` skips the prompt) **or** `MigrationFailed` (e.g. requested env's Python is EOL and `--force` not set) |
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
| Name conflict with an existing scuv env | The existing scuv env is overwritten in place |
| EOL Python version (e.g. 2.7) | Migrated anyway (the EOL guard is intentionally bypassed) |
| Corrupted source env | **Not bypassed** — still returns `CorruptedEnvironment` (exit 3) |

### `--auto-rename` (name-conflict only)

`--auto-rename` is a convenience for the pure name-conflict case:
when a scuv env with the same name already exists, the migration
proceeds under an auto-generated name. It does **not** override any
other status (EOL / corrupted), and it does **not** delete or overwrite
the existing scuv env.

```bash
$ scuv migrate @env myproject --auto-rename --yes
• Source: myproject (virtualenvwrapper, Python 3.12)
•   Source: ~/.virtualenvs/myproject
• Auto-renaming to 'myproject-pyenv'
• Migrating...
✓ Migrated 'myproject-pyenv'
•   Path: ~/.scuv/virtualenvs/myproject-pyenv
•   Python: 3.12
•   Packages: 1
•
• → Activate: scuv use myproject-pyenv
```

The generated name is `<name>-pyenv` for every source, including
`virtualenvwrapper` and `conda`. When that name is taken as well, scuv
picks a numbered name such as `myproject-2`.

The EOL guard still applies to a renamed env. When the source env both
conflicts by name and runs an end-of-life Python, `--auto-rename` stops
with `Python <version> is end-of-life. Use --force to migrate anyway.`
(exit 2).

## Examples

### List Migratable Environments

```bash
$ scuv migrate list
• Scanning all sources for environments...
✓ Found 2 environment(s):

  [virtualenvwrapper]
    ✓ myproject            Python 3.12           42.3 MB
    ✓ webapp               Python 3.11          118.6 MB

• To migrate: scuv migrate @env <name>
• To preview: scuv migrate @env <name> --dry-run
```

Each env line starts with `✓` when it is ready to migrate, `⚠` for a name
conflict or an end-of-life Python (followed by the reason, such as
`(Python 3.7.17 is EOL)` or `(conflicts with <path>)`), and `✗` when the
env is corrupted. The grouped env lines go to stdout; the `•`,
`✓ Found` lines go to stderr.

### Migrate Single Environment

```bash
$ scuv migrate @env myproject --yes
• Source: myproject (virtualenvwrapper, Python 3.12)
•   Source: ~/.virtualenvs/myproject
• Migrating...
✓ Migrated 'myproject'
•   Path: ~/.scuv/virtualenvs/myproject
•   Python: 3.12
•   Packages: 1
•
• → Activate: scuv use myproject
```

### Migrate All

```bash
$ scuv migrate all --yes
• Scanning all sources for environments...
• Found 2 environment(s) to migrate:
•   - myproject (Python 3.12)
•   - webapp (Python 3.11)
•
• Starting batch migration...
•
• ────────────────────────────────────────
✓ Migration complete: 2/2 succeeded
```

When some envs fail or conflict, the summary names them and the command
exits 2:

```bash
$ scuv migrate all --yes
• Scanning all sources for environments...
• Found 1 environment(s) to migrate:
•   - brokenpip (Python 3.12)
⚠ 1 environment(s) will be skipped (see summary)
•
• Starting batch migration...
•
• ────────────────────────────────────────
✓ Migration complete: 0/1 succeeded
⚠ Failed environments: brokenpip
⚠ 1 name conflict(s) skipped (use --force):
⚠   - myproject (virtualenvwrapper) conflicts with /home/u/.scuv/virtualenvs/myproject
• → Pass --force to overwrite existing scuv environments
```

### CI gate (fail the build on batch failure)

```bash
# Exits 2 if any env failed or a name conflict was skipped.
# Exits 3 if no source tool is installed on the runner.
scuv migrate all --yes
```

## JSON Output

All three subcommands accept `--json`. The envelope follows scuv's standard
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
        "python_version": "3.12",
        "path": "/home/u/.virtualenvs/myproject",
        "source_type": "virtualenv_wrapper",
        "size_bytes": 44357632,
        "status": { "status": "ready" }
      },
      {
        "name": "oldenv",
        "python_version": "3.7.17",
        "path": "/home/u/.virtualenvs/oldenv",
        "source_type": "virtualenv_wrapper",
        "size_bytes": 31457280,
        "status": { "status": "python_eol", "version": "3.7.17" }
      }
    ],
    "summary": { "total": 2, "ready": 1, "conflict": 0, "eol": 1, "corrupted": 0 }
  }
}
```

The `status` field is an object whose own `status` key names the state:
`{"status": "ready"}`, `{"status": "name_conflict", "existing": "<path>"}`,
`{"status": "python_eol", "version": "<version>"}`, or
`{"status": "corrupted", "reason": "<reason>"}`. `source_type` is
`"pyenv"`, `"virtualenv_wrapper"` or `"conda"`. `size_bytes` is the sum
of the environment's regular file sizes, measured when `migrate list` runs.

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
        "python_version": "3.12",
        "packages_migrated": 1,
        "packages_failed": [],
        "dry_run": false,
        "path": "/home/u/.scuv/virtualenvs/myproject",
        "source_deleted": false,
        "actual_python_version": "3.12"
      }
    ],
    "failed": [],
    "skipped": [],
    "conflicts": [],
    "summary": { "total": 1, "success": 1, "failed": 0, "skipped": 0 }
  }
}
```

`actual_python_version` currently repeats the source env's
`python_version`. `source_deleted` reflects whether `--delete-source` was
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
        "name": "brokenpip",
        "source_type": "virtualenv_wrapper",
        "error_code": "MIGRATE_EXTRACTION_FAILED",
        "error": "Couldn't extract packages: pip not found at /home/u/.virtualenvs/brokenpip/bin/pip"
      }
    ],
    "skipped": [
      { "name": "myproject", "reason": "name conflict (use --force)" }
    ],
    "conflicts": [
      {
        "name": "myproject",
        "source_type": "virtualenv_wrapper",
        "existing": "/home/u/.scuv/virtualenvs/myproject"
      }
    ],
    "summary": { "total": 2, "success": 0, "failed": 1, "skipped": 1 }
  }
}
```

Per-env failure objects carry two additive fields:

- `source_type` (`"pyenv"`, `"virtualenv_wrapper"`, `"conda"`) — origin tool.
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
scuv migrate all --json > out.json
case $? in
  0) ;;
  2) echo "batch failure — read out.json for detail" ;;
  3) echo "no source tool installed" ;;
esac
```

The exit-2 path (batch failure) is the only `migrate all` failure path
that emits a structured envelope on stdout. Bridging the exit-3
asymmetry would require `batch/` (and `single.rs`) to emit a JSON error
envelope before returning Err; tracked for a follow-up.

## Migration Process

1. **Discovery**: Scans configured source paths for virtual environments
2. **Extraction**: Identifies Python version and installed packages
3. **Recreation**: Creates new scuv environment with same Python version
4. **Package Install**: Reinstalls packages using `uv pip install`
5. **Cleanup**: Originals are preserved by default; `--delete-source` removes them after successful migration

## Notes

- Original environments are preserved by default; use `--delete-source` to remove sources after migration
- Package versions are preserved where possible
- Migration creates fresh environments using `uv` for improved performance

## Performance

`scuv migrate all` fans out across all CPU cores via [rayon] when migrating
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
