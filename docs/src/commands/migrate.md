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
| **conda** | `$CONDA_PREFIX/envs`, `~/.conda/envs`, `~/anaconda3/envs`, `~/miniconda3/envs`, `~/miniforge3/envs` |

## What moves

scuv reads the source's packages the way `scuv clone` and `scuv export` do:
uv lists them, and the source's own Python reports where each came from
(its [PEP 610](https://packaging.python.org/en/latest/specifications/direct-url/)
`direct_url.json`). The source needs no `pip`. Each package is put back from
there:

| Installed from | Reinstalled as |
| -- | -- |
| the registry (no recorded origin) | `name==version` |
| an editable directory | `--editable <path>` (an old `setup.py develop` install, which records no origin, is read from where uv reports it) |
| a directory | `name @ file://<path>` |
| a wheel or sdist URL | `name @ <url>`, with its strongest recorded hash |
| a git checkout | `name @ git+<url>@<commit>` |

Three kinds of package are treated differently:

- `pip` is not moved: scuv environments carry no installer. It is reported in
  `packages_excluded` and does not count as a failure. `setuptools` and
  `wheel` move like any other package (some packages use `pkg_resources`
  without declaring it).
- A package conda installed (its `INSTALLER` file says `conda`) records the
  directory conda built it in, which does not exist on your machine. scuv
  installs the registry release of the same name and version instead, and
  lists those packages in `packages_from_conda`. One that the registry does
  not have fails like any other package.
- A source whose Python is older than 3.8 cannot report origins and is
  refused (`PACKAGE_ORIGINS_UNREADABLE`) before uv runs. Nothing is
  installed from the registry in place of what could not be read.

Before anything is created or replaced, every package must be one that can
be put back: a local source that exists and is that package's project (a
directory needs a `pyproject.toml` or `setup.py`, and a static
`[project] name` in it, when there is one, must be the package's), none
inside a target `--force` would replace, and a
uv that checks recorded archive hashes (0.11.29 or later). Any of these
failing fails that environment, in a dry run too. So does a source whose
directory and the new environment's overlap (one inside the other, for
example `SCUV_HOME` inside the source).

The packages are then installed in one `uv pip install`. When that fails,
each package is tried on its own (without dependencies) to find the ones
that do not install, and the environment is rebuilt with the rest, so it
holds no leftovers of the failed attempt. Afterwards scuv reads the new
environment back and compares each package's name, version and origin with
the source; one that differs is a failure (`packages_failed`), and a package
the source did not have (a dependency the install added) is listed in
`packages_added`. `--strict` turns any install or verification failure into
a failed migration and removes the new environment.

## End-of-life Python

scuv treats an env as end of life (EOL) when its Python is 2.x or 3.10 and
earlier. 3.9 reached end of life on 2025-10-31 and 3.10 on 2026-10-01, following
CPython's schedule. Only the version number is checked, so a PyPy env gets the
same cutoff. For a source env that is otherwise healthy and has no name
conflict, without `--allow-eol` or `--force`:

- `migrate list` marks the env `⚠` with `(Python 3.10.14 is EOL)`.
- `migrate @env <name>` refuses it and exits `2`.
- `migrate all` skips it and still exits `0` when nothing else failed, so check
  `migrate list` (or the `skipped` array in `--json`) to see what was left behind.

Other states come first. A name conflict is reported as a conflict (and fails
`migrate all` with exit `2`), and a corrupted source is reported as corrupted
(`migrate @env` exits `3`, `migrate all` skips it). Choosing Overwrite at the
interactive conflict prompt only replaces the existing env: an EOL source then
still fails with `MigrationFailed` (exit `2`) before anything is replaced,
unless `--allow-eol` or `--force` was given. Up to scuv 0.20 Overwrite acted
like `--force` and let an EOL env through. `--rename` does not skip the EOL
check of a conflicting env either.
`--dry-run` runs the same checks as a real run (source status, the target
name, an existing target), so a preview that succeeds is one the real run
accepts.

`--allow-eol` lets such an env migrate and changes nothing else: a name
conflict still stops it. `--force` allows EOL envs too, but it also
overwrites a scuv env of the same name. Up to scuv 0.19.0 `--force` was the
only way through. Neither helps a Python older than 3.8: its package
origins cannot be read, so it is refused (see [What moves](#what-moves)).

Released versions up to 0.18.0 used a cutoff of 3.8, so 3.9 and 3.10 envs
migrated without `--force`.

## Options

| Option | Subcommand | Description |
|--------|------------|-------------|
| `--source <pyenv\|virtualenvwrapper\|conda>` | all subcommands | Restrict to a single source tool |
| `--json` | all subcommands | Machine-readable output (see [JSON Output](#json-output)) |
| `--dry-run` | `@env`, `all` | Preview without making changes |
| `--force` | `@env`, `all` | Overwrite existing scuv env with the same name; bypass EOL Python guard |
| `--allow-eol` | `@env`, `all` | Migrate envs whose Python is end of life; overwrites nothing |
| `--yes` | `@env`, `all` | Skip the interactive confirmation prompt |
| `--strict` | `@env`, `all` | Fail an env when any of its packages does not install or verify, removing its new env (default: migrate the rest and report them) |
| `--delete-source` | `@env`, `all` | Remove the source env after a migration that moved every package (see [Deleting the source](#deleting-the-source)) |
| `--rename <new-name>` | `@env` | Migrate under a different name |
| `--auto-rename` | `@env` | On name conflict, migrate under `<name>-pyenv` automatically (conflicts with `--force`) |

Global flags (`--quiet`, `--color`, `--no-color`) apply to all subcommands.

## Exit codes

`scuv migrate` follows the [layered exit-code contract](../api.md#process-exit-codes). The mapping differs slightly between `migrate all` (which partitions envs into buckets before iterating) and the single-env paths (`@env`, `list`).

### `migrate all`

| Code | Returned when |
|------|---------------|
| `0` | (a) all envs migrated, (b) no envs found but source tools are installed, or (c) only non-conflict skips occurred (EOL / corrupted envs in the skipped bucket, no preflight name conflicts, no per-env failures) |
| `1` | Confirmation was needed (no `--yes`, `--json` or `--dry-run`) but stderr is not a terminal to ask on: `CONFIRMATION_UNAVAILABLE`, naming `--yes`. Nothing is migrated |
| `2` | At least one per-env failure **or** at least one preflight name conflict without `--force`, **or** two sources offering the same env name in one batch. `--force` does not resolve that case: the first source migrates, and the skipped one needs `scuv migrate @env <name> --source <tool> --rename <other>` (without `--source`, `@env` finds the env that already migrated). Returned via `MigrationBatchFailed` |
| `3` | No source tool (pyenv / virtualenvwrapper / conda) is detected on the system. Returned via `MigrationSourcesNotFound` |

### `migrate @env <name>`

| Code | Returned when |
|------|---------------|
| `0` | The env migrated successfully (or the user chose `Skip` at the interactive conflict prompt) |
| `2` | `MigrationNameConflict` (env exists in scuv home, `--force` not set, and `--yes`, `--json` or a non-terminal stderr skips the prompt) **or** `MigrationFailed` (e.g. requested env's Python is EOL and neither `--allow-eol` nor `--force` is set) |
| `3` | The named source env was not found in the requested source (`PyenvEnvNotFound`, `VenvWrapperEnvNotFound`, `CondaEnvNotFound`) **or** the source env is `CorruptedEnvironment` |

### `migrate list`

Exit `0` is informational. A source whose scan fails is left out of the list instead of failing the command, so exit `0` does not prove that every source was read.

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
| EOL Python version (e.g. 3.9, 3.10) | Migrated anyway (the EOL guard is intentionally bypassed); below 3.8 still refused, since package origins cannot be read there |
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
picks the first free numbered name (`myproject-1`, then `myproject-2`, and so on).

The EOL guard still applies to a renamed env. When the source env both
conflicts by name and runs an end-of-life Python, `--auto-rename` stops
with `Python <version> is end-of-life. Use --allow-eol to migrate anyway.`
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
# EOL envs are skipped without changing the exit code; add --allow-eol to migrate them.
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
        "packages_excluded": [
          { "name": "pip", "version": "24.0", "reason": "installer" }
        ],
        "packages_from_conda": [],
        "packages_added": [],
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

`actual_python_version` is the version the new env's `pyvenv.cfg`
records. Current uv writes the minor (`3.12`) when it links the env to a
minor version and the full version otherwise; a dry run, or a `pyvenv.cfg`
without one, repeats the source's `python_version`. `packages_migrated`
counts packages verified in the new env (for a dry run, packages planned).
`packages_failed` lists packages that did not install or did not verify,
each with what went wrong; it does not change the exit code or the
`success` count. `packages_excluded` (pip), `packages_from_conda` and
`packages_added` are described in [What moves](#what-moves).
`source_deleted` reflects whether `--delete-source` was honored for this
env, and `source_kept_reason` (present only when it was not) says why. `dry_run` mirrors
the flag the command was invoked with.

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
        "error_code": "PACKAGE_ORIGINS_UNREADABLE",
        "error": "Could not read where the packages in /home/u/.virtualenvs/brokenpip came from: reading package origins needs Python 3.8 or later, and the env has Python 3.7.17"
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
  `"PACKAGE_ORIGINS_UNREADABLE"`, `"MIGRATE_NAME_CONFLICT"`,
  `"UV_COMMAND_FAILED"`). Scripts branch on this instead of parsing
  `error` (which is localized).

### Exit-3 paths — the generic error envelope

Two distinct exit-3 cases exist. Under `--json` each prints the generic
error envelope on stdout (`status`, `command`, and `error` with `code`,
the localized `message` and `suggestion`), without the `data` view the
exit-2 envelope carries; without `--json` the message and suggestion go
to stderr as text. The exit code stays `3` either way.

| Command | Error variant | Trigger |
|---------|---------------|---------|
| `migrate all` | `MigrationSourcesNotFound` | No source tool detected at all (pyenv / virtualenvwrapper / conda) |
| `migrate @env <name>` | `PyenvEnvNotFound` / `VenvWrapperEnvNotFound` / `CondaEnvNotFound` | The named env isn't present in the source `--source` asked for |
| `migrate @env <name>` | `SourceEnvNotFound` | Without `--source`, no source has the named env (JSON code `SOURCE_ENV_NOT_FOUND`) |
| `migrate @env <name>` | `CorruptedEnvironment` | The named env exists but its layout is broken (missing python, broken pyvenv.cfg, etc) |

Script template:

```bash
scuv migrate all --json > out.json
case $? in
  0) ;;
  2) echo "batch failure — read out.json for detail" ;;
  3) echo "no source tool or env: $(jq -r .error.code out.json)" ;;
esac
```

Only the exit-2 envelope (batch failure) carries the per-env `data`;
the exit-3 envelope has `error` alone. Up to scuv 0.18.1 the exit-3
paths printed text on stderr even with `--json`, and stdout stayed
empty.

## Migration Process

1. **Discovery**: Scans configured source paths for virtual environments
2. **Reading**: Lists the source's packages and their origins (see [What moves](#what-moves))
3. **Checks**: Plans every package and checks what may be deleted, before anything changes
4. **Recreation**: Creates the new scuv environment with the same Python version
5. **Package Install**: Reinstalls the packages with `uv pip install` and verifies the result
6. **Cleanup**: Originals are preserved by default; `--delete-source` removes them as described below

## Deleting the source

`--delete-source` removes a source only when all of these hold:

- every package migrated and verified (`packages_failed` is empty; a
  deliberately excluded pip does not count);
- the source is not a conda environment: it also holds packages migrate
  does not move (native libraries, conda's own builds), so it is always
  kept;
- nothing that must keep working goes through the source: a local package
  source inside it (an editable project in the source directory, say), the
  Python another environment or the new one runs on (uv may have put a
  managed Python there), the way to the scuv environment directory, or
  another source reached through a link inside it.

In `migrate all`, the environments found but not migrated (skipped,
conflicting, or with a name scuv cannot take) count too: their packages
are read and must survive. If any environment cannot be read, its packages
are unknown, so that batch deletes no source and replaces no target.

When a source is kept, the output says why (`source_kept_reason` in JSON).
The same check guards `--force`: replacing an existing scuv environment
that something listed above goes through fails that environment before
anything is removed.

## Notes

- Original environments are preserved by default; use `--delete-source` to remove sources after migration
- Package versions and origins are preserved; see [What moves](#what-moves)
- Migration creates fresh environments using `uv` for improved performance

## Performance

`scuv migrate all` migrates one environment at a time, in order. Every
environment is read and checked before the first one starts, and source
deletions under `--delete-source` happen after the last one finishes, so
no removal can interleave with another environment's creation (for
example, with uv installing a Python inside a directory that is about to
be replaced). uv still downloads and installs in parallel within each
environment. Up to scuv 0.22 environments migrated in parallel.

In the JSON summary, the `migrated[]` and `failed[]` arrays are sorted
alphabetically by env name. `skipped[]` and `conflicts[]` preserve the scan
/ partition order — which itself is deterministic (source-type then name;
see `scan_all_environments`).
