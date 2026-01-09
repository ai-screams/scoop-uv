# Introduction

**scoop** is a centralized Python virtual environment manager powered by [uv](https://github.com/astral-sh/uv).

> One scoop, endless envs — pyenv-style workflow with uv's blazing speed.

## What is scoop?

Think of it like running an ice cream parlor:

- **The Freezer** (`~/.scoop/`) keeps all your flavors fresh
- **Flavors** are your virtualenvs — mix once, serve anywhere
- **One scoop** is all you need to get the right env

| The Old Way | The scoop Way |
|-------------|---------------|
| `.venv` scattered across projects | `~/.scoop/virtualenvs/` centralized |
| Manual `source .venv/bin/activate` | Auto-activate on directory entry |
| pyenv-virtualenv is slow | uv-powered, 100x+ faster |
| Which Python? Which venv? Chaos. | `scoop doctor` checks everything |

## Quick Example

```bash
# Install Python
scoop install 3.12

# Create a virtualenv
scoop create myproject 3.12

# Use it (auto-activates!)
scoop use myproject
(myproject) $ pip install requests

# Check what's available
scoop list
```

## Features

- **Fast** — Powered by uv, virtualenv creation is nearly instant
- **Centralized** — All environments live in `~/.scoop/virtualenvs/`
- **Auto-activation** — Enter a directory, environment activates automatically
- **Shell integration** — Works with bash and zsh
- **pyenv compatible** — Reads `.python-version` files
- **Health checks** — `scoop doctor` diagnoses your setup

## Getting Started

Ready to scoop? Head to the [Installation](installation.md) guide to get started.

## Links

- [GitHub Repository](https://github.com/ai-screams/uvenv)
- [API Reference (docs.rs)](https://docs.rs/scoop-uv)
- [Crates.io](https://crates.io/crates/scoop-uv)
