# uvenv

> Swift as dambi, powered by uv — pyenv-style Python environment manager

[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)
[![Rust](https://img.shields.io/badge/rust-1.85+-orange.svg)](https://www.rust-lang.org/)
[![uv](https://img.shields.io/badge/powered%20by-uv-blueviolet.svg)](https://github.com/astral-sh/uv)

---

## What is uvenv?

**uvenv** is a centralized Python virtual environment manager using [uv](https://github.com/astral-sh/uv) as its backend.

It combines pyenv-virtualenv's workflow with uv's speed.

```
Problem                            uvenv Solution
─────────────────────────────────────────────────────
.venv scattered across projects    ~/.uvenv/virtualenvs/ centralized
Manual source .venv/bin/activate   uvenv activate or auto-activate
pyenv-virtualenv is slow           uv-based, 100x+ faster
```

---

## Quick Start

```bash
# Create a virtual environment
uvenv create myproject 3.12

# Activate
uvenv use myproject        # Set for current directory
cd ~/projects/myproject    # Auto-activates
(myproject) $

# Install packages
pip install requests       # or uv pip install

# Manage
uvenv list                 # List all environments
uvenv remove myproject     # Delete
```

---

## Installation

```bash
# Coming soon
cargo install uvenv
```

### Shell Setup

```bash
# Bash
echo 'eval "$(uvenv init bash)"' >> ~/.bashrc

# Zsh
echo 'eval "$(uvenv init zsh)"' >> ~/.zshrc
```

---

## Commands

### Virtual Environment

| Command | Description |
|---------|-------------|
| `uvenv create <name> <version>` | Create virtual environment |
| `uvenv use <name>` | Set local environment (.uvenv-version + .venv symlink) |
| `uvenv use <name> --global` | Set global default environment |
| `uvenv list` | List all virtual environments |
| `uvenv remove <name>` | Delete virtual environment |

### Python Version Management

| Command | Description |
|---------|-------------|
| `uvenv install` | Install latest Python (same as `--latest`) |
| `uvenv install --latest` | Install latest stable Python |
| `uvenv install --stable` | Install oldest fully-supported Python (more stable) |
| `uvenv install 3.12` | Install latest patch of 3.12.x |
| `uvenv install 3.12.3` | Install exact version 3.12.3 |
| `uvenv uninstall <version>` | Remove installed Python version |
| `uvenv list --pythons` | List installed Python versions |

> **Note:** Python versions are managed by [uv](https://github.com/astral-sh/uv) and downloaded automatically when needed.

### Shell Integration

| Command | Description |
|---------|-------------|
| `uvenv init <shell>` | Output shell initialization script |

---

## Development

### Prerequisites

- Rust 1.85+ (Edition 2024)
- [prek](https://github.com/j178/prek) — Pre-commit hooks (Rust-native)

### Setup

```bash
# Clone
git clone https://github.com/ai-screams/uvenv.git
cd uvenv

# Install prek (pre-commit alternative)
uv tool install prek
# or: pip install prek

# Install git hooks
prek install

# Build
cargo build

# Run tests
cargo test
```

### Pre-commit Hooks

Hooks run automatically on `git commit`:

| Hook | Description |
|------|-------------|
| `cargo fmt` | Code formatting |
| `cargo clippy` | Linting |
| `cargo check` | Type checking |
| `trailing-whitespace` | Whitespace fixes |
| `check-toml` | TOML validation |

```bash
# Run all hooks manually
prek run --all-files

# Run specific hook
prek run cargo-clippy
```

### Common Commands

```bash
# Build
cargo build
cargo build --release

# Test
cargo test
cargo test --all-features

# Lint
cargo clippy --all-targets -- -D warnings

# Format
cargo fmt
cargo fmt --check

# Run
cargo run -- --help
cargo run -- list
```

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
