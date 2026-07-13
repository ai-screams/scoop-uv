# Command Reference

Complete reference for all scuv commands.

## Commands Overview

| Command | Aliases | Description |
|---------|---------|-------------|
| [`scuv list`](list.md) | `ls` | List virtualenvs or Python versions |
| [`scuv create`](create.md) | - | Create virtualenv |
| [`scuv use`](use.md) | - | Set + activate environment |
| [`scuv remove`](remove.md) | `rm`, `delete` | Remove virtualenv |
| [`scuv install`](install.md) | - | Install Python version |
| [`scuv uninstall`](uninstall.md) | - | Uninstall Python version |
| [`scuv doctor`](doctor.md) | - | Diagnose installation |
| [`scuv info`](info.md) | - | Show virtualenv details |
| [`scuv status`](status.md) | - | Summarise the currently active env |
| [`scuv which`](which.md) | - | Resolve an executable inside an env |
| [`scuv run`](run.md) | - | Run a command inside an env without activating |
| [`scuv sync`](sync.md) | - | Apply `.scuv.toml` declaratively |
| [`scuv export`](export.md) | - | Write a portable JSON snapshot of an env |
| [`scuv import`](import.md) | - | Recreate an env from an export file (or stdin) |
| [`scuv clone`](clone.md) | - | Duplicate an env (with or without packages) |
| [`scuv migrate`](migrate.md) | - | Migrate from pyenv/conda/venvwrapper |
| [`scuv gc`](gc.md) | - | Garbage-collect orphan virtualenvs |
| [`scuv prune`](prune.md) | - | Prune the uv cache |
| [`scuv verify`](verify.md) | - | Per-env health diagnosis (6 checks) |
| [`scuv lang`](lang.md) | - | Get/set display language |
| [`scuv shell`](shell.md) | - | Set shell-specific env (temporary) |
| [`scuv init`](init.md) | - | Shell init script |
| [`scuv completions`](completions.md) | - | Completion script |
| [`scuv man`](man.md) | - | Generate man pages (for distro packagers) |
| [`scuv self update`](self.md) | - | Reinstall scuv from crates.io (update or pin version) |

## Global Options

Available for all commands:

| Option | Description |
|--------|-------------|
| `-q`, `--quiet` | Suppress all output |
| `--no-color` | Disable colored output |
| `-h`, `--help` | Show help message |
| `-V`, `--version` | Show version |

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `SCUV_HOME` | Base directory for scuv | `~/.scuv` |
| `SCUV_NO_AUTO` | Disable auto-activation | (unset) |
| `SCUV_LANG` | Display language (en, ko, ja, pt-BR) | System locale |
| `NO_COLOR` | Disable colored output | (unset) |
| `SCUV_VERSION` | Shell-session override; highest-priority version selector (set by `scuv shell`) | (unset) |
| `SCUV_ACTIVE` | Name of the currently active environment (set by the activation script; read by `status`/`which`/`run`) | (unset) |
| `SCUV_RESOLVE_MAX_DEPTH` | Caps the parent-directory walk when resolving `.scuv-version` (0 = current dir only; unset = unlimited) | (unset) |

## Directory Layout

| Location | Purpose |
|----------|---------|
| `~/.scuv/virtualenvs/` | Virtual environments storage |
| `~/.scuv/version` | Global default environment |
| `.scuv-version` | Local environment preference |
| `.venv` | Symlink to active environment (with `--link`) |
