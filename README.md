<div align="center">

# scoop

**One scoop, endless envs — pyenv-style Python environment manager powered by uv**

[![CI](https://img.shields.io/github/actions/workflow/status/ai-screams/scoop-uv/ci.yml?style=for-the-badge&logo=github&label=CI)](https://github.com/ai-screams/scoop-uv/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/scoop-uv?style=for-the-badge&logo=rust&color=orange)](https://crates.io/crates/scoop-uv)
[![License](https://img.shields.io/crates/l/scoop-uv?style=for-the-badge)](LICENSE-MIT)

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
