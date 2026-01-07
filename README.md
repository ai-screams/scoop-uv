<div align="center">

<img src="assets/logo/logo-with-text.png" width="180" alt="scoop logo">

# 🍨 scoop

> ⚠️ **Work in Progress** — Under active development. API may change.

**One scoop, endless envs — pyenv-style Python environment manager powered by uv**

[![CI](https://img.shields.io/github/actions/workflow/status/ai-screams/scoop-uv/ci.yml?style=for-the-badge&logo=github&label=CI)](https://github.com/ai-screams/scoop-uv/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/scoop-uv?style=for-the-badge&logo=rust&color=orange)](https://crates.io/crates/scoop-uv)
[![Downloads](https://img.shields.io/crates/d/scoop-uv?style=for-the-badge&logo=rust&color=blue)](https://crates.io/crates/scoop-uv)
[![docs.rs](https://img.shields.io/docsrs/scoop-uv?style=for-the-badge&logo=docs.rs&label=docs)](https://docs.rs/scoop-uv)

<details>
<summary>🍨 More badges</summary>

<!-- Quality & Coverage -->
[![Security](https://img.shields.io/github/actions/workflow/status/ai-screams/scoop-uv/security.yml?style=flat-square&logo=github&label=Security)](https://github.com/ai-screams/scoop-uv/actions/workflows/security.yml)
[![Coverage](https://codecov.io/gh/ai-screams/scoop-uv/graph/badge.svg)](https://codecov.io/gh/ai-screams/scoop-uv)
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

</details>

</div>

---

## What is scoop? 🍨

**scoop** scoops up uv's blazing speed — centralizing all your Python virtual environments in one place.

> 🍨 Like an ice cream parlor — all flavors (envs) in one freezer (`~/.scoop/`),
> served instantly with a single scoop. 🍨 is our signature!

| Problem                            | scoop Solution                      |
|------------------------------------|-------------------------------------|
| `.venv` scattered across projects  | `~/.scoop/virtualenvs/` centralized |
| Manual `source .venv/bin/activate` | Auto-activate on directory entry    |
| pyenv-virtualenv is slow           | uv-powered, 100x+ faster            |

---

## Installation 🍨

### Prerequisites

| Dependency | Install | Why |
|------------|---------|-----|
| **uv** | `curl -LsSf https://astral.sh/uv/install.sh \| sh` | Python installation backend |
| **Rust** | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` | Build from source |

### Install scoop

```bash
cargo install scoop-uv
```

<details>
<summary>💡 <code>scoop: command not found</code>?</summary>

Cargo installs binaries to `~/.cargo/bin`. Ensure it's in your PATH:

```bash
# Add to ~/.zshrc or ~/.bashrc
export PATH="$HOME/.cargo/bin:$PATH"
```

Or restart your terminal after installing Rust.

</details>

### Shell Setup

#### Step 1: Add to your shell config

**Zsh** (macOS default):

```bash
echo 'eval "$(scoop init zsh)"' >> ~/.zshrc
source ~/.zshrc
```

**Bash**:

```bash
echo 'eval "$(scoop init bash)"' >> ~/.bashrc
source ~/.bashrc
```

#### Step 2: Verify

```bash
scoop --version
# → scoop 0.x.x 🍨
```

#### What this enables

- ✅ **Auto-activation** — enter a directory with `.scoop-version`, environment activates
- ✅ **Tab completion** — commands, environments, Python versions
- ✅ **Shell wrapper** — `scoop activate/deactivate` works correctly

#### Using with pyenv

Add scoop **after** pyenv in your rc file:

```bash
# ~/.zshrc (order matters!)
eval "$(pyenv init -)"       # 1. pyenv first
eval "$(scoop init zsh)"     # 2. scoop second
```

#### Options

| Variable | Effect |
|----------|--------|
| `SCOOP_NO_AUTO=1` | Disable auto-activation |
| `SCOOP_HOME=/path` | Custom scoop directory (default: `~/.scoop`) |

```bash
# Example: disable auto-activation
echo 'export SCOOP_NO_AUTO=1' >> ~/.zshrc
```

---

## Quick Start 🍨

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

## Commands 🍨

| Command                         | Description                            |
|---------------------------------|----------------------------------------|
| `scoop create <name> [version]` | Create virtual environment             |
| `scoop use <name>`              | Set local environment (auto-activates) |
| `scoop use <name> --link`       | Also create `.venv` symlink for IDE    |
| `scoop use <name> --global`     | Set global default                     |
| `scoop list`                    | List environments                      |
| `scoop list --pythons`          | List installed Python versions         |
| `scoop remove <name>`           | Delete environment                     |
| `scoop install [version]`       | Install Python (default: latest)       |
| `scoop install --stable`        | Install oldest supported Python        |
| `scoop uninstall <version>`     | Remove Python version                  |

For complete command reference, see [docs/commands.md](docs/commands.md).

---

## Documentation 🍨

- [Command Reference](docs/commands.md) — Complete command documentation
- [Development Guide](docs/DEVELOPMENT.md) — Contributing and development setup

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

## Support 🍨

If you find this project useful, consider buying me a coffee!

<a href="https://buymeacoffee.com/pignuante" target="_blank"><img src="https://cdn.buymeacoffee.com/buttons/v2/default-yellow.png" alt="Buy Me A Coffee" height="50"></a>

---

## Acknowledgments 🍨

This project stands on the shoulders of giants:

- **[uv](https://github.com/astral-sh/uv)** by [Astral](https://astral.sh) — The blazing-fast Python package manager
  that powers scoop's backend. Without uv's incredible speed and reliability, scoop wouldn't exist. Thank you to Charlie
  Marsh and the entire Astral team for revolutionizing Python tooling.

- **[pyenv](https://github.com/pyenv/pyenv)** & **[pyenv-virtualenv](https://github.com/pyenv/pyenv-virtualenv)** —
  The original inspiration for scoop's workflow. pyenv taught us how Python version management should feel,
  and pyenv-virtualenv showed us how to centralize virtual environments elegantly.

- **[virtualenv](https://github.com/pypa/virtualenv)** by [PyPA](https://www.pypa.io/) — The pioneer of Python virtual
  environments. Thank you to Ian Bicking for the original concept that changed how we isolate Python projects.

- **[Python](https://www.python.org/)** — The language that made programming accessible to everyone. scoop exists to
  make Python development even more delightful. Thank you to Guido van Rossum and the Python community.

- **[Rust](https://www.rust-lang.org/)** — The language that makes scoop fast, safe, and reliable. Thank you to the
  Rust team and Ferris 🦀 for proving that systems programming can be both powerful and enjoyable.

---

<div align="center">

<img src="assets/community/ferris/scoop-ferris.png" width="160" alt="scoop ferris">

*I built scoop because I needed it — and now it's yours too.* 🍨

*Grab a scoop, enjoy the flavor, and if you have thoughts to share,*
*the door to the ice cream parlor is always open.*

**[Issues](https://github.com/ai-screams/scoop-uv/issues)** · **[Discussions](https://github.com/ai-screams/scoop-uv/discussions)** · **[PRs Welcome](https://github.com/ai-screams/scoop-uv/pulls)**

</div>
