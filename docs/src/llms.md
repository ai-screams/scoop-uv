# LLM Reference

This page provides a concise reference for AI/LLM tools working with scoop.

> **Tip**: The raw text versions are available at
> [`llms.txt`](https://github.com/ai-screams/scoop-uv/blob/main/llms.txt) (concise) and
> [`llms-full.txt`](https://github.com/ai-screams/scoop-uv/blob/main/llms-full.txt) (full API reference).

---

## Overview

**scoop** is a centralized Python virtual environment manager — pyenv-style workflow powered by uv. Written in Rust.

All virtualenvs are stored in `~/.scoop/virtualenvs/`. Override with `SCOOP_HOME` env var.

## Commands

| Command | Description |
|---------|-------------|
| `scoop list` | List virtualenvs (aliases: `ls`) |
| `scoop list --pythons` | List installed Python versions |
| `scoop create <name> [version]` | Create virtualenv (default: latest Python) |
| `scoop use <name>` | Set + activate environment |
| `scoop use <name> --global` | Set as global default |
| `scoop use <name> --link` | Also create `.venv` symlink for IDE |
| `scoop use system` | Deactivate, use system Python |
| `scoop use --unset` | Remove version file |
| `scoop remove <name>` | Delete virtualenv (aliases: `rm`, `delete`) |
| `scoop install [version]` | Install Python version |
| `scoop uninstall <version>` | Remove Python version |
| `scoop info <name>` | Show virtualenv details |
| `scoop doctor` | Health check |
| `scoop doctor --fix` | Auto-fix issues |
| `scoop shell <name>` | Set shell-specific env (temporary) |
| `scoop shell --unset` | Clear shell-specific setting |
| `scoop init <shell>` | Output shell init script |
| `scoop completions <shell>` | Generate completion script |
| `scoop lang [code]` | Get/set language (en, ko, ja, pt-BR) |
| `scoop migrate list` | List migratable envs (pyenv, conda, virtualenvwrapper) |
| `scoop migrate @<name>` | Migrate single environment |
| `scoop migrate --all` | Migrate all environments |

Most commands support `--json` for machine-readable output.

Global options: `--quiet`, `--no-color`

## Key Concepts

### Version Files

Priority (first match wins):
1. `SCOOP_VERSION` env var (shell session override, set by `scoop shell`)
2. `.scoop-version` in current directory (local, walks parent directories)
3. `~/.scoop/version` (global default)

### Shell Integration

scoop outputs shell code to stdout; the shell wrapper `eval`s it (pyenv pattern).
Auto-activation triggers on directory change when `.scoop-version` is present.

Supported shells: bash, zsh, fish, PowerShell (Core 7.x+ and Windows PowerShell 5.1+)

Disable auto-activation: `export SCOOP_NO_AUTO=1`

### Environment Name Rules

- Pattern: `^[a-zA-Z][a-zA-Z0-9_-]*$` (max 64 chars)
- Must start with a letter
- Reserved words: activate, base, completions, create, deactivate, default, delete, global, help, init, install, list, local, remove, resolve, root, system, uninstall, use, version, versions

### Migration Sources

Import environments from pyenv-virtualenv, virtualenvwrapper, and conda.

### Internationalization

Supported languages: English (`en`), Korean (`ko`), Japanese (`ja`), Portuguese-BR (`pt-BR`)

Priority: `SCOOP_LANG` env > `~/.scoop/config.json` > system locale > `en`

## Configuration

- Config file: `~/.scoop/config.json`
- Home directory: `~/.scoop/` (override: `SCOOP_HOME`)
- Metadata: `~/.scoop/virtualenvs/<name>/.scoop-metadata.json`
