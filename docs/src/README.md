# Introduction

**scuv** is a centralized Python virtual environment manager powered by [uv](https://github.com/astral-sh/uv).

> One scoop, endless envs — pyenv-style workflow with uv's blazing speed.

## What is scuv?

Think of it like running an ice cream parlor:

- **The Freezer** (`~/.scuv/`) keeps all your flavors fresh
- **Flavors** are your virtualenvs — mix once, serve anywhere
- **One scoop** is all you need to get the right env

| The Old Way | The scuv Way |
|-------------|---------------|
| `.venv` scattered across projects | `~/.scuv/virtualenvs/` centralized |
| Manual `source .venv/bin/activate` | Auto-activate on directory entry |
| pyenv-virtualenv is slow | uv-powered, 100x+ faster |
| Which Python? Which venv? Chaos. | `scuv doctor` checks everything |

## Quick Example

```bash
# Install Python
scuv install 3.12

# Create a virtualenv
scuv create myproject 3.12

# Use it (auto-activates!)
scuv use myproject
(myproject) $ pip install -r requirements.txt

# Check what's available
scuv list
```

## Features

- **Fast** — Powered by uv, virtualenv creation is nearly instant
- **Centralized** — All environments live in `~/.scuv/virtualenvs/`
- **Auto-activation** — Enter a directory, environment activates automatically
- **Shell integration** — Works with bash, zsh, fish, and PowerShell
- **IDE friendly** — `scuv use --link` creates `.venv` symlink for IDE discovery
- **Health checks** — `scuv doctor` diagnoses your setup

## Getting Started

Ready to scoop? Head to the [Installation](installation.md) guide to get started.

## Links

- [GitHub Repository](https://github.com/ai-screams/scoop-uv)
- [API Reference (docs.rs)](https://docs.rs/scoop-uv)
- [Crates.io](https://crates.io/crates/scoop-uv)
- [llms.txt](https://github.com/ai-screams/scoop-uv/blob/main/llms.txt) — AI/LLM-friendly project reference
- [llms-full.txt](https://github.com/ai-screams/scoop-uv/blob/main/llms-full.txt) — Full API reference for AI tools
