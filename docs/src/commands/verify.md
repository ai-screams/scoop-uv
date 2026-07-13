# verify

Verify the health of one or all virtual environments. Where [`doctor`](doctor.md) checks system-wide setup (uv installed, shell wrapper wired up, etc.), `verify` looks inside each env directory and answers: "does Python actually work in here?"

## Usage

```bash
scuv verify                # Check every environment
scuv verify <NAME>         # Check just one environment
scuv verify --json         # Machine-readable output
scuv verify --strict       # Exit 1 if any check has Fail status (default: always 0)
```

## What gets checked

Six checks run per environment, in order:

| Check | Status | What it means |
|-------|--------|---------------|
| `metadata` | Fail | `.scoop-metadata.json` is missing or unreadable |
| `python_binary` | Fail | `bin/python` (`Scripts/python.exe` on Windows) is missing |
| `pyvenv_cfg` | Fail | `pyvenv.cfg` venv marker is missing |
| `activate_script` | Fail | `bin/activate` (`Scripts/Activate.ps1` on Windows) is missing |
| `python_executes` | Fail | `python --version` fails to run (Skip if the binary is already missing) |
| `manifest_match` | Warn | env's Python doesn't match `.scuv.toml` if a manifest exists in the cwd hierarchy (Skip otherwise) |

A check returns one of:

- **Pass** — everything looks right
- **Skip** — irrelevant for this env (e.g. no `.scuv.toml`, or the prerequisite already failed)
- **Warn** — soft issue; env may still work
- **Fail** — hard breakage; env likely unusable

An env is considered **healthy** when every check is Pass or Skip.

## Exit codes

By default, `verify` always exits 0 — even when checks fail. This matches `doctor`'s philosophy: surfacing information shouldn't break CI just because someone wanted to look at the report. Pass `--strict` to opt into `exit 1` when any env has at least one **Fail** check (Warn alone does not trigger the non-zero exit).

## verify vs doctor vs gc

| Command | Scope | Action |
|---------|-------|--------|
| `doctor` | System (uv install, shell wrapper, paths) | Diagnose + optional `--fix` |
| `verify` | Per-env (file presence, exec, manifest) | Diagnose only |
| `gc` | All envs (orphan detection) | Diagnose + optional removal |

`verify` is the "what's wrong with this specific env?" tool. `gc` is the "which envs are so broken they should just be removed?" tool. They share territory but differ in granularity and intent.

## Examples

```bash
# Quick health check on every env
scuv verify

# Specific env, JSON for scripting
scuv verify myproject --json | jq '.data.envs[0].healthy'

# CI gate: fail the build if any env is broken
scuv verify --strict
```

## JSON output

```json
{
  "status": "success",
  "command": "verify",
  "data": {
    "envs": [
      {
        "name": "myenv",
        "healthy": true,
        "python": "3.12.0",
        "checks": [
          { "name": "metadata", "status": "pass" },
          { "name": "python_binary", "status": "pass" },
          { "name": "pyvenv_cfg", "status": "pass" },
          { "name": "activate_script", "status": "pass" },
          { "name": "python_executes", "status": "pass" },
          { "name": "manifest_match", "status": "skip" }
        ]
      }
    ],
    "summary": { "total": 1, "healthy": 1, "issues": 0 }
  }
}
```

## See also

- [`doctor`](doctor.md) — system-level diagnostics
- [`gc`](gc.md) — remove broken envs detected here
- [`info`](info.md) — detailed view of a single env (without health checks)
