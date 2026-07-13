# LLM Reference

This page provides a concise reference for AI/LLM tools working with scuv.

> **Tip**: The raw text versions are available at
> [`llms.txt`](https://github.com/ai-screams/scoop-uv/blob/main/llms.txt) (concise) and
> [`llms-full.txt`](https://github.com/ai-screams/scoop-uv/blob/main/llms-full.txt) (full API reference).

---

## Overview

**scuv** is a centralized Python virtual environment manager — pyenv-style workflow powered by uv. Written in Rust.

All virtualenvs are stored in `~/.scuv/virtualenvs/`. Override with `SCUV_HOME` env var.

## Commands

| Command | Description |
|---------|-------------|
| `scuv list` | List virtualenvs (aliases: `ls`) |
| `scuv list --pythons` | List installed Python versions |
| `scuv list --sort <name\|created\|last-used>` | Sort order; envs missing the timestamp sort last with name tie-break (0.13.0) |
| `scuv create <name> [version]` | Create virtualenv (default: latest Python) |
| `scuv create <name> <ver> --install-python` | Create env; install Python on demand if missing (v0.11.0) |
| `scuv use <name>` | Set + activate environment |
| `scuv use <name> --global` | Set as global default |
| `scuv use <name> --link` | Also create `.venv` symlink for IDE |
| `scuv use system` | Deactivate, use system Python |
| `scuv use --unset` | Remove version file |
| `scuv remove <name>` | Delete virtualenv (aliases: `rm`, `delete`) |
| `scuv clone <src> <dst>` | Duplicate an env in-place (`--no-packages` for skeleton) (v0.11.0) |
| `scuv install [version]` | Install Python version |
| `scuv uninstall <version>` | Remove Python version |
| `scuv info <name>` | Show virtualenv details (includes `Last used:` row since 0.13.0) |
| `scuv status` | Summarise current state (Active/Configured/System/None) (v0.11.0); includes `Last used:` since 0.13.0 |
| `scuv which <exe>` | Resolve an executable inside the active env (v0.11.0) |
| `scuv run <env> -- <cmd>` | Run a command inside an env without activating (v0.11.0) |
| `scuv sync` | Reconcile the active env with `.scuv.toml` manifest (v0.11.0) |
| `scuv export <name>` | Snapshot an env as JSON (schema v1) (v0.11.0) |
| `scuv import <file>` | Restore an env from a JSON snapshot (v0.11.0) |
| `scuv doctor` | Health check |
| `scuv doctor --fix` | Auto-fix issues |
| `scuv shell <name>` | Set shell-specific env (temporary) |
| `scuv shell --unset` | Clear shell-specific setting |
| `scuv init <shell>` | Output shell init script |
| `scuv completions <shell>` | Generate completion script |
| `scuv lang [code]` | Get/set language (en, ko, ja, pt-BR) |
| `scuv migrate list` | List migratable envs (pyenv, conda, virtualenvwrapper) |
| `scuv migrate @env <name>` | Migrate single environment |
| `scuv migrate all` | Migrate all environments (parallel via rayon since v0.11.0) |
| `scuv gc` | Garbage-collect orphan virtualenvs (`--yes` to actually remove, `--aggressive` also for unused Pythons, `--older-than <n>d/w/y` flags stale envs by `last_used`; envs with no `last_used` are never matched) |
| `scuv prune` | Prune the uv cache (`uv cache prune` wrapper) |
| `scuv verify [NAME]` | Per-env health diagnosis — 6 checks (metadata, python binary, pyvenv.cfg, activate, exec, manifest drift); `--strict` exits 1 on issues |
| `scuv man [DIR]` | Generate man pages (stdout or one file per subcommand in DIR) |
| `scuv diff <a> <b>` | Compare two environments: Python, packages, metadata |
| `scuv self update` | Update scuv itself from crates.io |

Most commands support `--json` for machine-readable output.

Global options: `--quiet`, `--no-color`

## Key Concepts

### Set a Specific Python Version as Global Default

To set Python `3.11.0` as the global default for new shells:

```bash
scuv install 3.11.0
scuv create py311 3.11.0
scuv use py311 --global
```

Important: `--global` stores an environment name (`py311`) in `~/.scuv/version`,
not the raw Python version string. Local `.scuv-version` and `SCUV_VERSION`
override the global default.

### Create a Project Environment with Python 3.9.5

```bash
scuv install 3.9.5
scuv create myproject 3.9.5
scuv info myproject
```

If `3.9.5` is not found, check discovery with `uv python list` and
`scuv list --pythons`, then install and retry.

### Uninstall Python and Associated Environments

```bash
# Optional preview
scuv list --python-version 3.12

# Remove Python 3.12 and all environments that use it
scuv uninstall 3.12 --cascade

# Verify cleanup
scuv list --pythons
scuv doctor
```

For automation, use `scuv uninstall 3.12 --cascade --force`.
Without `--cascade`, dependent environments are not removed and may become broken.

### Temporarily Disable or Customize Auto-Activation (Project-Scoped)

```bash
# Current shell only (temporary disable)
export SCUV_NO_AUTO=1
unset SCUV_NO_AUTO

# Project-local behavior (writes .scuv-version in current dir)
scuv use system
scuv use myproject

# Terminal-only override (no file changes)
scuv shell system
scuv shell --unset
```

Use these without `--global` to avoid changing global settings.

### Install Dependencies from requirements.txt in Active Environment

```bash
# environment already active (prompt shows: (myproject))
pip install -r requirements.txt
```

Use `pip install -r path/to/requirements.txt` for non-root files.
Verify with `pip list`.

### List Python Versions and Associated Environments

```bash
scuv list --pythons
scuv list
scuv list --python-version 3.12
```

Use `--json` for automation and `--bare` for script-friendly output.
For full mapping in shell scripts, iterate versions from `scuv list --pythons --bare`
and query each with `scuv list --python-version <VERSION> --bare`.

### Integrate Custom or Pre-Existing Python

```bash
# Preferred: explicit interpreter path
scuv create myenv --python-path /opt/python-debug/bin/python3

# Alternative: make interpreter discoverable via PATH
export PATH="/opt/python-debug/bin:$PATH"
scuv create myenv 3.13
```

Verify with `uv python list`, `scuv info myenv`, and `scuv doctor -v`.
Custom interpreter path is stored in `~/.scuv/virtualenvs/<name>/.scoop-metadata.json`
(`python_path` field).

### Version Files

Priority (first match wins):
1. `SCUV_VERSION` env var (shell session override, set by `scuv shell`)
2. `.scuv-version` in current directory (local, walks parent directories)
3. `~/.scuv/version` (global default)

### Shell Integration

scuv outputs shell code to stdout; the shell wrapper `eval`s it (pyenv pattern).
Auto-activation triggers on directory change when `.scuv-version` is present.

Supported shells: bash, zsh, fish, PowerShell (Core 7.x+ and Windows PowerShell 5.1+)

Disable auto-activation: `export SCUV_NO_AUTO=1`

### Environment Name Rules

- Pattern: `^[a-zA-Z][a-zA-Z0-9_-]*$` (max 64 chars)
- Must start with a letter
- Reserved words: activate, base, completions, create, deactivate, default, delete, global, help, init, install, list, local, remove, resolve, root, system, uninstall, use, version, versions

### Migration Sources

Import environments from pyenv-virtualenv, virtualenvwrapper, and conda.

### Internationalization

Supported languages: English (`en`), Korean (`ko`), Japanese (`ja`), Portuguese-BR (`pt-BR`)

Priority: `SCUV_LANG` env > `~/.scuv/config.json` > system locale > `en`

## Configuration

- Config file: `~/.scuv/config.json`
- Home directory: `~/.scuv/` (override: `SCUV_HOME`)
- Metadata: `~/.scuv/virtualenvs/<name>/.scoop-metadata.json`
