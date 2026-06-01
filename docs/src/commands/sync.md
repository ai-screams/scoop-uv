# sync

Apply a project's declarative `.scoop.toml` — create the env if needed (auto-
installing Python if missing) and reconcile its packages via uv pip.

## Usage

```bash
scoop sync [--with <GROUP>]... [--dry-run] [--json]
```

## Options

| Option | Description |
|--------|-------------|
| `--with <GROUP>` | Install an extra package group on top of `default` (repeatable) |
| `--dry-run` | Print the resolved plan without creating env or installing packages |
| `--json` | Output as JSON |

## Manifest Resolution

`scoop sync` walks from the current directory up to the filesystem root looking
for `.scoop.toml` — the same model as `.scoop-version`. The first manifest it
finds wins. If none is found, the command fails with `MANIFEST_NOT_FOUND` and
points you at this doc.

## `.scoop.toml` Format

```toml
[environment]
name = "myproject"        # required — must pass `is_valid_env_name`
python = "3.12"           # required — same specifier syntax as `scoop create`

[packages]
default = ["pytest", "black", "mypy"]   # always installed
dev = ["ipython", "debugpy"]            # opt-in via `--with dev`
docs = ["mkdocs"]                       # opt-in via `--with docs`
```

Field rules:
- `[environment].name` follows the same validation as `scoop create <NAME>`
  (letter-leading, `[a-zA-Z][a-zA-Z0-9_-]*`, not a reserved subcommand name).
- `[environment].python` is forwarded to `uv venv --python` as-is, so any
  specifier uv accepts works (`3.12`, `3.12.7`, `cpython@3.12`, `pypy@3.10`).
- `[packages]` is optional; `default = []` is valid.
- Any other key inside `[packages]` becomes a named group selectable with
  `--with <name>`.
- Top-level keys other than `[environment]` and `[packages]` are rejected
  (`deny_unknown_fields`) so typo'd sections fail loudly instead of silently
  doing nothing.

## Behaviour

| State | What `scoop sync` does |
|-------|------------------------|
| Env missing | Auto-installs the requested Python (if uv doesn't have it), creates the env, then installs packages |
| Env exists, Python matches | Just installs packages (idempotent — pip resolves and skips already-satisfied entries) |
| Env exists, Python mismatch | **Warns and proceeds.** Recreating an env on a version change is destructive, so it stays explicit: `scoop remove <name>` then `scoop sync` |
| Unknown `--with <group>` | Fails fast before any env work, lists available groups |

`scoop sync` does *not* uninstall packages that are present in the env but
missing from the manifest. That's a deliberate scoping decision for v1 — full
lockstep reconciliation is left to a future `--prune` flag.

## Examples

```bash
# In a project directory with .scoop.toml
scoop sync                                # default group only
scoop sync --with dev                     # default + dev
scoop sync --with dev --with docs         # multiple groups

scoop sync --dry-run                      # preview, no side effects
scoop sync --with dev --dry-run --json    # machine-readable plan
```

### Sample dry-run output

```
• Dry-run plan:
  manifest:    /path/to/project/.scoop.toml
  environment: myproject (Python 3.12)
  groups:      default, dev
  action:      create env + install packages
  packages:    5 total
    - pytest
    - black
    - mypy
    - ipython
    - debugpy
```

### Sample JSON output

```json
{
  "status": "success",
  "command": "sync",
  "data": {
    "manifest_path": "/path/to/.scoop.toml",
    "environment": "myproject",
    "python": "3.12",
    "groups": ["default", "dev"],
    "packages": ["pytest", "black", "mypy", "ipython", "debugpy"],
    "env_created": true,
    "dry_run": false
  }
}
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Sync succeeded (or dry-run produced a plan) |
| 1 | Manifest not found, unknown group, parse error, or pip install failed |

## Not Yet Supported (v1)

These fields are intentionally not parsed in this version:

- `[hooks]` (`post-create`, `post-activate`, ...) — needs a separate threat
  model before scoop will execute arbitrary shell from a checked-in file.
- `python_path` — per-env custom interpreter overrides (parallel to the
  existing `--python-path` flag on `scoop create`).
- Lock file format.

The parser rejects unknown top-level keys, so a manifest using these will fail
cleanly today and stop working when they ship in a future release without a
silent behaviour change.
