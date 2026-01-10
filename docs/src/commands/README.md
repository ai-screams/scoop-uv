# Command Reference

Complete reference for all scoop commands.

## Commands Overview

| Command | Aliases | Description |
|---------|---------|-------------|
| [`scoop list`](list.md) | `ls` | List virtualenvs or Python versions |
| [`scoop create`](create.md) | - | Create virtualenv |
| [`scoop use`](use.md) | - | Set + activate environment |
| [`scoop remove`](remove.md) | `rm`, `delete` | Remove virtualenv |
| [`scoop install`](install.md) | - | Install Python version |
| [`scoop uninstall`](uninstall.md) | - | Uninstall Python version |
| [`scoop doctor`](doctor.md) | - | Diagnose installation |
| [`scoop info`](info.md) | - | Show virtualenv details |
| [`scoop init`](init.md) | - | Shell init script |
| [`scoop completions`](completions.md) | - | Completion script |

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
| `SCOOP_HOME` | Base directory for scoop | `~/.scoop` |
| `SCOOP_NO_AUTO` | Disable auto-activation | (unset) |
| `NO_COLOR` | Disable colored output | (unset) |

## Directory Layout

| Location | Purpose |
|----------|---------|
| `~/.scoop/virtualenvs/` | Virtual environments storage |
| `~/.scoop/version` | Global default environment |
| `.scoop-version` | Local environment preference |
| `.python-version` | pyenv compatibility (fallback) |
| `.venv` | Symlink to active environment (with `--link`) |
