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
[![MSRV](https://img.shields.io/crates/msrv/scoop-uv?style=flat-square&logo=rust&label=MSRV)](https://www.rust-lang.org/)
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

> 🍨 Think of it like running an ice cream parlor:
> - **The Freezer** (`~/.scoop/`) keeps all your flavors fresh
> - **Flavors** are your virtualenvs — mix once, serve anywhere
> - **One scoop** is all you need to get the right env

| The Old Way (Yuck 🫠)                | The scoop Way (Fresh 🍨)             |
|-------------------------------------|--------------------------------------|
| `.venv` scattered across projects   | `~/.scoop/virtualenvs/` centralized  |
| Manual `source .venv/bin/activate`  | Auto-activate on directory entry     |
| pyenv-virtualenv is slow            | uv-powered, 100x+ faster             |
| Which Python? Which venv? Chaos.    | `scoop doctor` checks everything     |
| Migrating envs? Manual nightmare.   | `scoop migrate --all` does it all    |
| English-only CLI                    | Multi-language support (en, ko, ja, pt-BR) |

---

## The Freezer 🧊

Your ice cream parlor lives here:

```
~/.scoop/                    # 🧊 The Freezer
├── virtualenvs/             # 🍨 All your flavors
│   ├── myproject/           #    → Python 3.12 flavor
│   ├── webapp/              #    → Python 3.11 flavor
│   └── experiment/          #    → Python 3.13 flavor
└── version                  # 🥄 Default scoop preference
```

**Version file priority** (first match wins):
```
SCOOP_VERSION (env)  →  "Override for this shell session" (set by scoop shell)
.scoop-version       →  "I want THIS flavor here" (local + parent walk)
~/.scoop/version     →  "My usual order" (global default)
```

> **Note**: `.python-version` is not currently supported. Use `.scoop-version` for version pinning.

---

## Installation 🍨

### Prerequisites

| Dependency | Install | Why |
|------------|---------|-----|
| **uv** | `curl -LsSf https://astral.sh/uv/install.sh \| sh` | The secret ingredient 🔮 |
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

**Fish**:

```fish
echo 'eval (scoop init fish)' >> ~/.config/fish/config.fish
source ~/.config/fish/config.fish
```

**PowerShell** (Core or Windows PowerShell):

```powershell
Add-Content $PROFILE 'Invoke-Expression (& scoop init powershell)'
. $PROFILE
```

#### Step 2: Verify

```bash
scoop --version
# → scoop 0.7.0 🍨
```

#### What this enables

- ✅ **Auto-activation** — enter a directory with `.scoop-version`, environment activates
- ✅ **Tab completion** — commands, environments, Python versions
- ✅ **Shell wrapper** — `scoop activate/deactivate` works correctly
- ✅ **Migration ready** — import from pyenv, conda, virtualenvwrapper
- ✅ **Multi-language** — English, 한국어, 日本語, Português (BR)

#### Using with pyenv

Add scoop **after** pyenv in your rc file (order matters — scoop gets the last scoop! 🍨):

```bash
# ~/.zshrc
eval "$(pyenv init -)"       # 1. pyenv first
eval "$(scoop init zsh)"     # 2. scoop second
```

#### Options

| Variable | Effect |
|----------|--------|
| `SCOOP_NO_AUTO=1` | Disable auto-activation |
| `SCOOP_HOME=/path` | Custom freezer location (default: `~/.scoop`) |

```bash
# Example: disable auto-activation
echo 'export SCOOP_NO_AUTO=1' >> ~/.zshrc
```

---

## Minimum Supported Rust Version (MSRV) 🦀

scoop follows an **N-1 MSRV policy**:

- **Current MSRV**: 1.85 (required by Rust Edition 2024)
- We support the current stable Rust and one previous version (~6 week lag)
- MSRV updates are considered **non-breaking** for binary users per [Cargo RFC 3537](https://rust-lang.github.io/rfcs/3537-msrv-resolver.html)

### Impact by User Type

| User Type | MSRV Impact | Action |
|-----------|-------------|--------|
| **Binary users** | ✅ None | Download from [releases](https://github.com/ai-screams/scoop-uv/releases) or `cargo install` |
| **Source builders** | ⚠️ Rust >= 1.85 required | Run `rustup update` if needed |
| **Contributors** | 🔧 Test on MSRV before PR | `cargo +1.85 test --all-features` |

### When We Bump MSRV

✅ **We bump when:**
- New Rust features provide significant user benefits
- Critical dependencies require newer versions
- Security fixes only available in newer Rust

❌ **We don't bump for:**
- Time-based schedules without clear benefits
- Minor syntax sugar or aesthetic preferences
- Personal developer preferences

All MSRV changes are documented in [CHANGELOG.md](CHANGELOG.md) with clear rationale.

For more details, see our [MSRV bump guide in CONTRIBUTING.md](CONTRIBUTING.md#bumping-msrv-step-by-step-guide).

---

## Quick Start 🍨

```bash
# Stock up the freezer 🧊
scoop install 3.12

# Mix a new flavor 🍦
scoop create myproject 3.12

# Pick your flavor for this directory (auto-activates!)
scoop use myproject
(myproject) $ pip install requests

# Check what's in the freezer
scoop list                 # List all flavors
scoop list --pythons       # List Python versions
scoop list --json          # For the data nerds 🤓

# Clean up
scoop remove myproject     # Melt it away 💧
```

---

## Commands 🍨

> **Tip:** Most commands support `--json` for machine-readable output.

### Everyday Scooping

| Command                         | Description                            |
|---------------------------------|----------------------------------------|
| `scoop create <name> [version]` | Mix a new flavor (default: latest Python) |
| `scoop use <name>`              | Pick your flavor (auto-activates)      |
| `scoop use <name> --link`       | Also create `.venv` symlink for IDE    |
| `scoop use <name> --global`     | Set as your usual order                |
| `scoop list`                    | What's in the freezer?                 |
| `scoop list --pythons`          | What Python versions do we have?       |
| `scoop list --json`             | Output as JSON                         |
| `scoop info <name>`             | Show detailed info about a flavor      |
| `scoop info <name> --json`      | Output info as JSON                    |
| `scoop remove <name>`           | Melt a flavor away                     |

### Managing the Freezer

| Command                     | Description                              |
|-----------------------------|------------------------------------------|
| `scoop install [version]`   | Stock up on Python (default: latest)     |
| `scoop install --stable`    | Get the oldest supported Python (3.10)   |
| `scoop uninstall <version>` | Remove a Python version                  |

### Health Check 🩺

| Command              | Description                            |
|----------------------|----------------------------------------|
| `scoop doctor`       | Is everything fresh? Check your setup! |
| `scoop doctor --fix` | Auto-fix issues where possible         |
| `scoop doctor --json`| Output diagnostics as JSON             |

### Migration 🚚

| Command                     | Description                              |
|-----------------------------|------------------------------------------|
| `scoop migrate list`        | Show environments to migrate             |
| `scoop migrate @<name>`     | Migrate a single environment             |
| `scoop migrate --all`       | Migrate all environments                 |

> **Supported sources:** pyenv-virtualenv, virtualenvwrapper, conda

### Language 🌏

| Command               | Description                        |
|-----------------------|------------------------------------|
| `scoop lang`          | Show current language              |
| `scoop lang <code>`   | Set language (en, ko, ja, pt-BR)   |
| `scoop lang --list`   | List supported languages           |
| `scoop lang --reset`  | Reset to system default            |

> 🌍 **Want to help translate?** We welcome translations in any language! See [#44](https://github.com/ai-screams/scoop-uv/issues/44) to contribute.

### Shell Integration

| Command                    | Description                        |
|----------------------------|------------------------------------|
| `scoop init <shell>`       | Output shell initialization script |
| `scoop completions <shell>`| Generate completion script         |
| `scoop use system`         | Switch to system Python            |
| `scoop shell <name>`       | Set shell env (eval required)      |
| `scoop shell --unset`      | Clear shell env setting            |

> **Shells supported:** `bash`, `zsh`, `fish`, `powershell`

For complete command reference, see [docs/src/commands/README.md](docs/src/commands/README.md).

---

## Architecture 🏗️

Built with Rust for speed and reliability:

```
src/
├── cli/           # 🎮 Command parsing (clap)
│   └── commands/  # Individual command handlers
├── core/          # 🧠 Domain logic
│   ├── version    # Version file resolution
│   ├── metadata   # Virtualenv metadata (JSON)
│   ├── virtualenv # Virtualenv entity
│   ├── doctor     # Health diagnostics
│   └── migrate/   # Migration (pyenv, conda, venvwrapper)
├── shell/         # 🐚 Shell integration (bash, zsh, fish, powershell)
├── uv/            # ⚡ uv CLI wrapper
├── output/        # 🎨 Terminal UI & JSON output
├── i18n.rs        # 🌏 Internationalization (en, ko, ja, pt-BR)
├── config.rs      # ⚙️ User configuration
└── error, paths, validate  # Utilities
```

**Design principle:** The CLI outputs shell code to stdout, your shell evaluates it. Just like pyenv — battle-tested pattern.

---

## Documentation 🍨

📖 **[Full Documentation](https://ai-screams.github.io/scoop-uv/)**

| Guide | Description |
|-------|-------------|
| [Installation](https://ai-screams.github.io/scoop-uv/installation.html) | Prerequisites and setup |
| [Quick Start](https://ai-screams.github.io/scoop-uv/quick-start.html) | Get started in 5 minutes |
| [Commands](https://ai-screams.github.io/scoop-uv/commands/) | Complete command reference |
| [Shell Integration](https://ai-screams.github.io/scoop-uv/shell-integration.html) | Auto-activation and configuration |
| [Contributing](https://ai-screams.github.io/scoop-uv/development/contributing.html) | Development guide |

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

If you find this project useful, consider buying me a coffee (or an ice cream 🍨)!

<a href="https://buymeacoffee.com/pignuante" target="_blank"><img src="https://cdn.buymeacoffee.com/buttons/v2/default-yellow.png" alt="Buy Me A Coffee" height="50"></a>

---

## Contributors ✨

Thanks goes to these wonderful people ([emoji key](https://allcontributors.org/docs/en/emoji-key)):

<!-- ALL-CONTRIBUTORS-LIST:START - Do not remove or modify this section -->
<!-- prettier-ignore-start -->
<!-- markdownlint-disable -->
<table>
  <tbody>
    <tr>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/azamarassy"><img src="https://avatars.githubusercontent.com/u/143267784?v=4" width="80px;" alt="azamarassy"/><br /><sub><b>azamarassy</b></sub></a><br /><a href="#translation-azamarassy" title="Translation">🌍</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/murillobazz"><img src="https://avatars.githubusercontent.com/u/64990540?v=4" width="80px;" alt="Murillo Bazilio"/><br /><sub><b>Murillo Bazilio</b></sub></a><br /><a href="#translation-murillobazz" title="Translation">🌍</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/Tosinibikunle"><img src="https://avatars.githubusercontent.com/u/87605729?v=4" width="80px;" alt="Tosinibikunle"/><br /><sub><b>Tosinibikunle</b></sub></a><br /><a href="#doc-Tosinibikunle" title="Documentation">📖</a></td>
    </tr>
  </tbody>
</table>

<!-- markdownlint-restore -->
<!-- prettier-ignore-end -->

<!-- ALL-CONTRIBUTORS-LIST:END -->

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
