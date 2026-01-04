<div align="center">

# 🐿️ scoop

**Swift as dambi, powered by uv — pyenv-style Python environment manager**

<!-- Hero Badges -->
[![CI](https://img.shields.io/github/actions/workflow/status/ai-screams/scoop-uv/ci.yml?style=for-the-badge&logo=github&label=CI)](https://github.com/ai-screams/scoop-uv/actions/workflows/ci.yml)
[![Security](https://img.shields.io/github/actions/workflow/status/ai-screams/scoop-uv/security.yml?style=for-the-badge&logo=github&label=Security)](https://github.com/ai-screams/scoop-uv/actions/workflows/security.yml)
[![Crates.io](https://img.shields.io/crates/v/scoop-uv?style=for-the-badge&logo=rust&color=orange)](https://crates.io/crates/scoop-uv)
[![Downloads](https://img.shields.io/crates/d/scoop-uv?style=for-the-badge&logo=rust&color=blue)](https://crates.io/crates/scoop-uv)

<!-- Docs & Quality -->
[![docs.rs](https://img.shields.io/docsrs/scoop-uv?style=flat-square&logo=docs.rs&label=docs.rs)](https://docs.rs/scoop-uv)
[![MSRV](https://img.shields.io/badge/MSRV-1.85-blue?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![License](https://img.shields.io/crates/l/scoop-uv?style=flat-square)](LICENSE-MIT)
[![dependency status](https://deps.rs/repo/github/ai-screams/scoop-uv/status.svg)](https://deps.rs/repo/github/ai-screams/scoop-uv)

<!-- GitHub Stats -->
[![Stars](https://img.shields.io/github/stars/ai-screams/scoop-uv?style=flat-square&logo=github&label=Stars)](https://github.com/ai-screams/scoop-uv/stargazers)
[![Forks](https://img.shields.io/github/forks/ai-screams/scoop-uv?style=flat-square&logo=github&label=Forks)](https://github.com/ai-screams/scoop-uv/network/members)
[![Issues](https://img.shields.io/github/issues/ai-screams/scoop-uv?style=flat-square&logo=github&label=Issues)](https://github.com/ai-screams/scoop-uv/issues)
[![PRs](https://img.shields.io/github/issues-pr/ai-screams/scoop-uv?style=flat-square&logo=github&label=PRs)](https://github.com/ai-screams/scoop-uv/pulls)
[![Contributors](https://img.shields.io/github/contributors/ai-screams/scoop-uv?style=flat-square&logo=github)](https://github.com/ai-screams/scoop-uv/graphs/contributors)

<!-- Activity -->
[![Last Commit](https://img.shields.io/github/last-commit/ai-screams/scoop-uv?style=flat-square&logo=github)](https://github.com/ai-screams/scoop-uv/commits/main)
[![Commit Activity](https://img.shields.io/github/commit-activity/m/ai-screams/scoop-uv?style=flat-square&logo=github)](https://github.com/ai-screams/scoop-uv/pulse)
[![Repo Size](https://img.shields.io/github/repo-size/ai-screams/scoop-uv?style=flat-square&logo=github)](https://github.com/ai-screams/scoop-uv)
[![Top Language](https://img.shields.io/github/languages/top/ai-screams/scoop-uv?style=flat-square&logo=rust&color=orange)](https://github.com/ai-screams/scoop-uv)

<!-- Project Identity -->
[![Rust](https://img.shields.io/badge/rust-1.85+-orange?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![Powered by uv](https://img.shields.io/badge/powered%20by-uv-blueviolet?style=flat-square&logo=python)](https://github.com/astral-sh/uv)
[![Maintained](https://img.shields.io/badge/maintained-yes-green?style=flat-square)](https://github.com/ai-screams/scoop-uv)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen?style=flat-square)](https://github.com/ai-screams/scoop-uv/pulls)

</div>

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
scoop create myproject 3.12

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
cargo install scoop-uv
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

### Virtual Environment

| Command | Description |
|---------|-------------|
| `scoop create <name> <version>` | Create virtual environment |
| `scoop use <name>` | Set local environment (.scoop-version + .venv symlink) |
| `scoop use <name> --global` | Set global default environment |
| `scoop list` | List all virtual environments |
| `scoop remove <name>` | Delete virtual environment |

### Python Version Management

| Command | Description |
|---------|-------------|
| `scoop install` | Install latest Python (same as `--latest`) |
| `scoop install --latest` | Install latest stable Python |
| `scoop install --stable` | Install oldest fully-supported Python (more stable) |
| `scoop install 3.12` | Install latest patch of 3.12.x |
| `scoop install 3.12.3` | Install exact version 3.12.3 |
| `scoop uninstall <version>` | Remove installed Python version |
| `scoop list --pythons` | List installed Python versions |

> **Note:** Python versions are managed by [uv](https://github.com/astral-sh/uv) and downloaded automatically when needed.

### Shell Integration

| Command | Description |
|---------|-------------|
| `scoop init <shell>` | Output shell initialization script |

---

## Development

### Prerequisites

- Rust 1.85+ (Edition 2024)
- [prek](https://github.com/j178/prek) — Pre-commit hooks (Rust-native)

### Setup

```bash
# Clone
git clone https://github.com/ai-screams/scoop-uv.git
cd scoop-uv

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
