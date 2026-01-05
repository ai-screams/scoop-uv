<div align="center">

# 🍨 scoop

> ⚠️ **Work in Progress** — Under active development. API may change.

**One scoop, endless envs — pyenv-style Python environment manager powered by uv**

<!-- Hero Badges -->
[![CI](https://img.shields.io/github/actions/workflow/status/ai-screams/scoop-uv/ci.yml?style=for-the-badge&logo=github&label=CI)](https://github.com/ai-screams/scoop-uv/actions/workflows/ci.yml)
[![Security](https://img.shields.io/github/actions/workflow/status/ai-screams/scoop-uv/security.yml?style=for-the-badge&logo=github&label=Security)](https://github.com/ai-screams/scoop-uv/actions/workflows/security.yml)
[![Crates.io](https://img.shields.io/crates/v/scoop-uv?style=for-the-badge&logo=rust&color=orange)](https://crates.io/crates/scoop-uv)
[![Downloads](https://img.shields.io/crates/d/scoop-uv?style=for-the-badge&logo=rust&color=blue)](https://crates.io/crates/scoop-uv)

<!-- Docs & Quality -->
[![docs.rs](https://img.shields.io/docsrs/scoop-uv?style=flat-square&logo=docs.rs&label=docs.rs)](https://docs.rs/scoop-uv)
[![MSRV](https://img.shields.io/badge/MSRV-1.85-blue?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![License](https://img.shields.io/crates/l/scoop-uv?style=flat-square)](LICENSE-MIT)
[![Dependencies](https://img.shields.io/librariesio/release/cargo/scoop-uv?style=flat-square&label=dependencies)](https://libraries.io/cargo/scoop-uv)

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
[![Platform](https://img.shields.io/badge/platform-linux%20%7C%20macos-blue?style=flat-square)](https://github.com/ai-screams/scoop-uv)
[![Rust](https://img.shields.io/badge/rust-1.85+-orange?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![Powered by uv](https://img.shields.io/badge/powered%20by-uv-blueviolet?style=flat-square&logo=python)](https://github.com/astral-sh/uv)
[![Maintained](https://img.shields.io/badge/maintained-yes-green?style=flat-square)](https://github.com/ai-screams/scoop-uv)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen?style=flat-square)](https://github.com/ai-screams/scoop-uv/pulls)

</div>

---

## What is scoop?

**scoop** is a centralized Python virtual environment manager using [uv](https://github.com/astral-sh/uv) as its backend.

```
Problem                            scoop Solution
─────────────────────────────────────────────────────
.venv scattered across projects    ~/.scoop/virtualenvs/ centralized
Manual source .venv/bin/activate   scoop activate or auto-activate
pyenv-virtualenv is slow           uv-based, 100x+ faster
```

---

## Installation

```bash
cargo install scoop-uv
```

### Shell Setup

Add to your shell configuration:

```bash
# Bash (~/.bashrc)
eval "$(scoop init bash)"

# Zsh (~/.zshrc)
eval "$(scoop init zsh)"
```

This enables:
- Auto-activation when entering directories with `.scoop-version`
- Tab completion for commands, environments, and options

---

## Quick Start

```bash
# Install Python
scoop install 3.12

# Create a virtual environment
scoop create myproject 3.12

# Set for current directory (auto-activates)
scoop use myproject
(myproject) $ pip install requests

# Manage environments
scoop list                 # List all environments
scoop remove myproject     # Delete environment
```

---

## Commands

| Command | Description |
|---------|-------------|
| `scoop create <name> [version]` | Create virtual environment |
| `scoop use <name>` | Set local environment (auto-activates) |
| `scoop use <name> --link` | Also create `.venv` symlink for IDE |
| `scoop use <name> --global` | Set global default |
| `scoop list` | List environments |
| `scoop list --pythons` | List installed Python versions |
| `scoop remove <name>` | Delete environment |
| `scoop install [version]` | Install Python (default: latest) |
| `scoop install --stable` | Install oldest supported Python |
| `scoop uninstall <version>` | Remove Python version |

For complete command reference, see [docs/commands.md](docs/commands.md).

---

## Documentation

- [Command Reference](docs/commands.md) - Complete command documentation
- [Development Guide](docs/DEVELOPMENT.md) - Contributing and development setup

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

## Support

If you find this project useful, consider buying me a coffee!

<a href="https://buymeacoffee.com/pignuante" target="_blank"><img src="https://cdn.buymeacoffee.com/buttons/v2/default-yellow.png" alt="Buy Me A Coffee" height="50"></a>

---

## Acknowledgments

This project stands on the shoulders of giants:

- **[uv](https://github.com/astral-sh/uv)** by [Astral](https://astral.sh) — The blazing-fast Python package manager that powers scoop's backend. Without uv's incredible speed and reliability, scoop wouldn't exist. Thank you to Charlie Marsh and the entire Astral team for revolutionizing Python tooling.

- **[pyenv](https://github.com/pyenv/pyenv)** — The original inspiration for scoop's workflow. pyenv taught us how environment management should feel.
