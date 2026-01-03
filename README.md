# scoop

> Scoop up your Python environments — pyenv-style workflow powered by uv

[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)
[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org/)
[![uv](https://img.shields.io/badge/powered%20by-uv-blueviolet.svg)](https://github.com/astral-sh/uv)

---

## What is scoop?

**scoop** is a centralized Python virtual environment manager using [uv](https://github.com/astral-sh/uv) as its backend.

It combines pyenv-virtualenv's workflow with uv's speed.

```
Problem                            scoop Solution
─────────────────────────────────────────────────────
.venv scattered across projects    ~/.scoop/virtualenvs/ centralized
Manual source .venv/bin/activate   scoop activate or auto-activate
pyenv-virtualenv is slow           uv-based, 100x+ faster
```

---

## Quick Start

```bash
# Create a virtual environment
scoop create myproject --python 3.12

# Activate
scoop use myproject        # Set for current directory
cd ~/projects/myproject    # Auto-activates
(myproject) $

# Install packages
pip install requests       # or uv pip install

# Manage
scoop list                 # List all environments
scoop remove myproject     # Delete
```

---

## Installation

```bash
# Coming soon
cargo install scoop
```

### Shell Setup

```bash
# Bash
echo 'eval "$(scoop init bash)"' >> ~/.bashrc

# Zsh
echo 'eval "$(scoop init zsh)"' >> ~/.zshrc
```

---

## Commands

| Command | Description |
|---------|-------------|
| `scoop create <name> --python <version>` | Create virtual environment |
| `scoop use <name>` | Set local environment (.scoop-version + .venv symlink) |
| `scoop use <name> --global` | Set global default environment |
| `scoop list` | List all virtual environments |
| `scoop remove <name>` | Delete virtual environment |
| `scoop install <version>` | Install Python version (via uv) |
| `scoop init <shell>` | Output shell initialization script |

---

## License

Licensed under either of:

- [MIT License](LICENSE-MIT)
- [Apache License, Version 2.0](LICENSE-APACHE)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this work shall be dual licensed as above, without any
additional terms or conditions.

---

## Acknowledgments

- [uv](https://github.com/astral-sh/uv) — Blazing fast Python package manager
- [pyenv](https://github.com/pyenv/pyenv) — Workflow inspiration
