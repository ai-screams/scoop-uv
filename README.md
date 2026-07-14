<div align="center">

<img src="assets/logo/logo-with-text.png" width="180" alt="scuv logo">

# 🍨 scuv

> ⚠️ **Work in Progress** — Under active development. API may change.

> 📢 **Renamed in v0.15.0:** the CLI command is now **`scuv`** (formerly `scoop`) — renamed to coexist with [Scoop](https://scoop.sh), the Windows package manager. The crate/repo keep the name `scoop-uv`. Legacy `SCOOP_*`/`.scoop-*` settings are still read (with a deprecation warning) until v0.16.0 — see the [CHANGELOG](CHANGELOG.md) for migration notes.

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

## What is scuv? 🍨

**scuv** scoops up uv's blazing speed — centralizing all your Python virtual environments in one place.

> 🍨 Think of it like running an ice cream parlor:
> - **The Freezer** (`~/.scuv/`) keeps all your flavors fresh
> - **Flavors** are your virtualenvs — mix once, serve anywhere
> - **One scoop** is all you need to get the right env

| The Old Way (Yuck 🫠)                | The scuv Way (Fresh 🍨)             |
|-------------------------------------|--------------------------------------|
| `.venv` scattered across projects   | `~/.scuv/virtualenvs/` centralized  |
| Manual `source .venv/bin/activate`  | Auto-activate on directory entry     |
| pyenv-virtualenv is slow            | uv-powered, 100x+ faster             |
| Which Python? Which venv? Chaos.    | `scuv doctor` checks everything     |
| Migrating envs? Manual nightmare.   | `scuv migrate all` does it all      |
| English-only CLI                    | Multi-language support (en, ko, ja, pt-BR) |

---

## 60-Second Quick Start ⚡

```bash
# 1. Install prerequisites
curl -LsSf https://astral.sh/uv/install.sh | sh  # uv
cargo install scoop-uv                           # installs the scuv command

# 2. Initialize your shell (zsh example)
echo 'eval "$(scuv init zsh)"' >> ~/.zshrc && source ~/.zshrc

# 3. Create your first environment
scuv install 3.12
scuv create myproject 3.12

# 4. Use it (auto-activates when you enter the directory!)
scuv use myproject
(myproject) $ pip install -r requirements.txt
```

**That's it!** 🎉 Your environment is ready. For detailed docs, see **[Full Documentation →](https://ai-screams.github.io/scoop-uv/)**

### Set Python 3.11.0 as Global Default

Use this when you want new shell sessions to default to an environment built on Python 3.11.0:

```bash
scuv install 3.11.0
scuv create py311 3.11.0
scuv use py311 --global
```

This writes `py311` to `~/.scuv/version`.
Priority still applies: `SCUV_VERSION` (shell override) and local `.scuv-version` take precedence.

### Create a Project Env with Python 3.9.5

Use this when you want a new project environment pinned to an exact Python patch version:

```bash
scuv install 3.9.5
scuv create myproject 3.9.5
scuv info myproject
```

If `3.9.5` is missing, check available versions with `uv python list` and
`scuv list --pythons`, then install and retry.

### Uninstall Python + Associated Envs

Use this to remove one Python version and every environment using it:

```bash
# Optional preview
scuv list --python-version 3.12

# Remove Python 3.12 and all dependent environments
scuv uninstall 3.12 --cascade

# Verify cleanup
scuv list --pythons
scuv doctor
```

For CI/scripts, add `--force` to skip confirmation.

### List Python Versions + Associated Envs

Use this to inspect what scuv currently manages:

```bash
# All managed Python versions
scuv list --pythons

# All environments and their Python versions
scuv list

# Environments associated with one Python version
scuv list --python-version 3.12
```

For scripts, use `--json` or `--bare`.

### Integrate Custom or Pre-Existing Python

If the required version is not available from default scuv/uv sources:

```bash
# Recommended: explicit interpreter path
scuv create myenv --python-path /opt/python-debug/bin/python3

# Alternative: PATH-based discovery
export PATH="/opt/python-debug/bin:$PATH"
scuv create myenv 3.13
```

Verify integration with `uv python list`, `scuv info myenv`, and `scuv doctor -v`.

### Project-Scoped Auto-Activation Control

Need temporary or directory-specific behavior without touching global settings?

```bash
# Temporary: current shell only
export SCUV_NO_AUTO=1
unset SCUV_NO_AUTO

# Directory-local behavior (writes .scuv-version in current directory)
scuv use system      # force system Python for this project
scuv use myproject   # pin this project to a specific env

# Terminal-only override (no file changes)
scuv shell system
scuv shell --unset
```

---

## Installation 🍨

### Prerequisites

| Dependency | Install | Why |
|------------|---------|-----|
| **uv** (>= 0.5.14) | `curl -LsSf https://astral.sh/uv/install.sh \| sh` | The secret ingredient 🔮 |
| **Rust** | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` | Build from source |

> Minimum supported `uv` is **0.5.14**. Run `scuv doctor` to verify your installation.

<details>
<summary>❓ <b>FAQ:</b> How is scuv different from uv, and is it related to Scoop (Windows)?</summary>

### How is scuv different from uv's `centralized-project-envs` preview?

They solve different problems, and scuv is built *on top of* uv — it's a complement, not a fork
or a competitor.

uv 0.11.25 added a preview feature (`centralized-project-envs`) that relocates a **project's**
`.venv` into uv's cache directory. The environment is still bound to that one project: its
identity is a cache key derived from the workspace path and interpreter (e.g.
`my-project-cp3.12.4-0123abcd`), it cannot be shared between projects, there is no activation
workflow, and `uv cache clean` / `uv cache prune` delete it unconditionally — by design it is a
disposable cache entry that gets transparently recreated.

scuv environments are the opposite in every one of those dimensions: **named, durable, and
project-independent**.

| | uv `centralized-project-envs` | scuv |
|---|---|---|
| Environment identity | hash cache key (not user-controlled) | a name you choose (`scuv create ml 3.12`) |
| Shared across projects | no (key includes workspace path) | yes — any project with a `.scuv-version` file |
| Activation workflow | none (`uv run`-centric; `.venv` link for IDEs) | shell auto-activation, `scuv use`, 4 shells |
| Lifecycle | wiped by `uv cache clean`/`prune`, auto-recreated | durable; `gc` (dry-run first), `verify`, metadata (`last_used`) |
| Extras | — | `clone`, `diff`, `export`/`import`, `.scuv.toml` sync, migration from pyenv / conda / virtualenvwrapper |

The uv team has stated there are ["no current plans to support standalone environments not tied
to a specific project"](https://github.com/astral-sh/uv/pull/18214). That standalone, named,
pyenv-virtualenv-style workflow is exactly what scuv provides — with uv doing the fast parts
underneath.

### Is scuv related to Scoop, the Windows package manager?

No. scuv — "a **sc**oop of **uv**" 🍨 — is a centralized Python virtual environment manager and
is unrelated to [Scoop](https://scoop.sh), the Windows package manager. The project was
originally command-named `scoop`; we renamed the command to `scuv` in v0.15.0 precisely so both
tools can coexist cleanly on Windows. Installing scuv does not shadow or conflict with `scoop`
in any shell, including PowerShell. (The repository and crate keep the historical name
`scoop-uv`.)

</details>

### Install scuv

```bash
cargo install scoop-uv  # installs the scuv command
```

<details>
<summary>💡 <code>scuv: command not found</code>?</summary>

Cargo installs binaries to `~/.cargo/bin`. Ensure it's in your PATH:

```bash
# Add to ~/.zshrc or ~/.bashrc
export PATH="$HOME/.cargo/bin:$PATH"
```

Or restart your terminal after installing Rust.

</details>

### Upgrading from scoop (≤ 0.14.x) 🔁

The CLI command was renamed in v0.15.0 (`scoop` → `scuv`). One-time migration:

```bash
# 1. Update — installs the new `scuv` binary
scoop self update        # or: cargo install scoop-uv
# ("could not locate the freshly installed `scoop` binary" warning is
#  expected across the rename — the update itself already succeeded)

# 2. Remove the old binary if cargo left one behind
rm -f ~/.cargo/bin/scoop

# 3. Update your shell rc: scoop init → scuv init
#    ~/.zshrc / ~/.bashrc:  eval "$(scuv init zsh)"      # or bash
#    fish:                  scuv init fish | source

# 4. Move your freezer
mv ~/.scoop ~/.scuv

# 5. Restart your shell, then verify
scuv doctor              # flags anything left over
```

Legacy `SCOOP_*` env vars and `.scoop-version` / `.scoop.toml` files keep
working (with a one-shot deprecation warning) until v0.16.0, and typing
`scoop` in bash/zsh/fish still works through a deprecated forwarder that
warns and calls `scuv`. Skipping step 2 is the one dangerous gap: a
leftover old binary keeps running 0.14.x silently, without any warning.

### Shell Setup

#### Step 1: Add to your shell config

**Zsh** (macOS default):

```bash
echo 'eval "$(scuv init zsh)"' >> ~/.zshrc
source ~/.zshrc
```

**Bash**:

```bash
echo 'eval "$(scuv init bash)"' >> ~/.bashrc
source ~/.bashrc
```

**Fish**:

```fish
echo 'scuv init fish | source' >> ~/.config/fish/config.fish
source ~/.config/fish/config.fish
```

**PowerShell** (Core or Windows PowerShell):

```powershell
Add-Content $PROFILE 'Invoke-Expression (& scuv init powershell)'
. $PROFILE
```

#### Step 2: Verify

```bash
scuv --version
# → scuv 0.15.0 🍨
```

#### What this enables

- ✅ **Auto-activation** — enter a directory with `.scuv-version`, environment activates
- ✅ **Tab completion** — commands, environments, Python versions
- ✅ **Shell wrapper** — `scuv activate/deactivate` works correctly
- ✅ **Migration ready** — import from pyenv, conda, virtualenvwrapper
- ✅ **Multi-language** — English, 한국어, 日本語, Português (BR)

#### Using with pyenv

Add scuv **after** pyenv in your rc file (order matters — scuv gets the last scoop! 🍨):

```bash
# ~/.zshrc
eval "$(pyenv init -)"       # 1. pyenv first
eval "$(scuv init zsh)"     # 2. scuv second
```

#### Options

| Variable | Effect |
|----------|--------|
| `SCUV_NO_AUTO=1` | Disable auto-activation |
| `SCUV_HOME=/path` | Custom freezer location (default: `~/.scuv`) |

```bash
# Example: disable auto-activation
echo 'export SCUV_NO_AUTO=1' >> ~/.zshrc
```

---

## The Freezer 🧊

Your ice cream parlor lives here:

```
~/.scuv/                    # 🧊 The Freezer
├── virtualenvs/             # 🍨 All your flavors
│   ├── myproject/           #    → Python 3.12 flavor
│   ├── webapp/              #    → Python 3.11 flavor
│   └── experiment/          #    → Python 3.13 flavor
└── version                  # 🥄 Default scuv preference
```

**Version file priority** (first match wins):
```
SCUV_VERSION (env)  →  "Override for this shell session" (set by scuv shell)
.scuv-version       →  "I want THIS flavor here" (local + parent walk)
~/.scuv/version     →  "My usual order" (global default)
```

> **Note**: `.python-version` is not supported. Use `.scuv-version` for version pinning.

---

## Commands 🍨

> **Tip:** Most commands support `--json` for machine-readable output.

### Essential Commands

| Command | Description |
|---------|-------------|
| `scuv create <name> [version]` | Create a new environment |
| `scuv use <name>` | Activate environment (auto-activates in directory) |
| `scuv list` | List all environments (`--sort name\|created\|last-used`) |
| `scuv status` | Show the currently active environment (includes `Last used:`) |
| `scuv which <exe>` | Resolve an executable inside the active env |
| `scuv run <env> -- <cmd>` | Run a command inside an env without activating |
| `scuv sync` | Apply `.scuv.toml` (create env + install packages) |
| `scuv export <name>` | Snapshot an env as portable JSON |
| `scuv import <file>` | Recreate an env from an export file |
| `scuv clone <src> <dst>` | Duplicate an environment |
| `scuv diff <a> <b>` | Compare two environments (Python, packages, metadata) |
| `scuv remove <name>` | Delete an environment |
| `scuv install [version]` | Install Python version |
| `scuv gc` | Garbage-collect orphan virtualenvs (`--yes` to remove, `--older-than <n>d/w/y` for stale envs) |
| `scuv prune` | Prune the uv cache |
| `scuv verify` | Per-env health check (metadata, python, pyvenv.cfg, ...) |
| `scuv doctor` | Health check your setup |
| `scuv self update` | Update scuv itself to the latest version |

**For the complete command reference**, see **[Commands Documentation →](https://ai-screams.github.io/scoop-uv/commands/)**

<details>
<summary>📖 Full command reference (click to expand)</summary>

### Everyday Scooping

| Command                         | Description                            |
|---------------------------------|----------------------------------------|
| `scuv create <name> [version]` | Mix a new flavor (default: latest Python) |
| `scuv create <name> <ver> --install-python` | Mix a flavor, installing Python first if missing |
| `scuv use <name>`              | Pick your flavor (auto-activates)      |
| `scuv use <name> --link`       | Also create `.venv` symlink for IDE    |
| `scuv use <name> --global`     | Set as your usual order                |
| `scuv list`                    | What's in the freezer?                 |
| `scuv list --pythons`          | What Python versions do we have?       |
| `scuv list --sort last-used`   | Newest activity first (also `name` / `created`) |
| `scuv list --json`             | Output as JSON                         |
| `scuv info <name>`             | Show detailed info (incl. `Last used:`)|
| `scuv info <name> --json`      | Output info as JSON                    |
| `scuv status`                  | Which flavor am I scooping right now? (incl. `Last used:`)|
| `scuv which <exe>`             | Where's that scoop in my freezer?      |
| `scuv run <env> -- <cmd>`      | Scoop on demand — run without unpacking |
| `scuv sync`                    | Read `.scuv.toml` and serve the flavor |
| `scuv sync --with dev --dry-run` | Preview the plan, no scooping yet     |
| `scuv export <name>`           | Bottle a flavor as portable JSON       |
| `scuv import <file>`           | Unbottle it on another machine         |
| `scuv clone <src> <dst>`       | Twin scoop — same flavor, new cup      |
| `scuv diff <a> <b>`            | Spot the difference between two flavors |
| `scuv remove <name>`           | Melt a flavor away                     |

### Managing the Freezer

| Command                     | Description                              |
|-----------------------------|------------------------------------------|
| `scuv install [version]`   | Stock up on Python (default: latest)     |
| `scuv install --stable`    | Get the oldest supported Python (3.10)   |
| `scuv uninstall <version>` | Remove a Python version                  |

### Health Check 🩺

| Command              | Description                            |
|----------------------|----------------------------------------|
| `scuv doctor`       | Is everything fresh? Check your setup! |
| `scuv doctor --fix` | Auto-fix issues where possible         |
| `scuv doctor --json`| Output diagnostics as JSON             |

### Migration 🚚

| Command                     | Description                              |
|-----------------------------|------------------------------------------|
| `scuv migrate list`        | Show environments to migrate             |
| `scuv migrate @env <name>` | Migrate a single environment             |
| `scuv migrate all`         | Migrate all environments (parallel)      |

> **Supported sources:** pyenv-virtualenv, virtualenvwrapper, conda

> Flags: `--source {pyenv|virtualenvwrapper|conda}`, `--dry-run`, `--force`, `--yes`, `--strict`, `--delete-source`, `--json`; `@env` also `--rename`/`--auto-rename`.

### Cleanup 🧹

| Command                  | Description                                                    |
|--------------------------|----------------------------------------------------------------|
| `scuv verify`           | Per-env health diagnosis — 6 checks per env                    |
| `scuv verify --strict`  | Same, but exit 1 on any issue (CI gate)                        |
| `scuv gc`               | Preview orphan virtualenvs (missing metadata or broken Python) |
| `scuv gc --yes`         | Actually remove the orphans                                    |
| `scuv gc --aggressive`  | Also flag unused uv-managed Python versions                    |
| `scuv gc --older-than 30d` | Also flag envs idle past the cutoff (no `last_used` never matches) |
| `scuv prune`            | Prune the uv download/wheel cache (`uv cache prune` wrapper)   |

### Packaging 📦

| Command                | Description                                                         |
|------------------------|---------------------------------------------------------------------|
| `scuv man`            | Print top-level `scuv.1` to stdout (pipe to `man -l -`)            |
| `scuv man <DIR>`      | Write `scuv.1` + one `scuv-<sub>.1` per subcommand into `<DIR>`   |

### Language 🌏

| Command               | Description                        |
|-----------------------|------------------------------------|
| `scuv lang`          | Show current language              |
| `scuv lang <code>`   | Set language (en, ko, ja, pt-BR)   |
| `scuv lang --list`   | List supported languages           |
| `scuv lang --reset`  | Reset to system default            |

> 🌍 **Want to help translate?** We welcome translations in any language! See [#44](https://github.com/ai-screams/scoop-uv/issues/44) to contribute.

### Shell Integration

| Command                    | Description                        |
|----------------------------|------------------------------------|
| `scuv init <shell>`       | Output shell initialization script |
| `scuv completions <shell>`| Generate completion script         |
| `scuv use system`         | Switch to system Python            |
| `scuv shell <name>`       | Set shell env (eval required)      |
| `scuv shell --unset`      | Clear shell env setting            |

> **Shells supported:** `bash`, `zsh`, `fish`, `powershell`

</details>

---

## Documentation 📚

📖 **[Read the Full Documentation →](https://ai-screams.github.io/scoop-uv/)**

| Guide | Description |
|-------|-------------|
| **[Installation Guide](https://ai-screams.github.io/scoop-uv/installation.html)** | Prerequisites, shell setup, and troubleshooting |
| **[Quick Start](https://ai-screams.github.io/scoop-uv/quick-start.html)** | Get productive in 5 minutes |
| **[Command Reference](https://ai-screams.github.io/scoop-uv/commands/)** | Detailed documentation for every command |
| **[Shell Integration](https://ai-screams.github.io/scoop-uv/shell-integration.html)** | Auto-activation, version files, and configuration |
| **[Migration Guide](https://ai-screams.github.io/scoop-uv/migration.html)** | Move from pyenv, conda, or virtualenvwrapper |
| **[Contributing](https://ai-screams.github.io/scoop-uv/development/contributing.html)** | Development setup and contribution guidelines |

---

## Minimum Supported Rust Version (MSRV) 🦀

**Current MSRV:** 1.85 (required by Rust Edition 2024)

scuv follows an **N-1 MSRV policy** — we support the current stable Rust and one previous version (~6 week lag).

| User Type | MSRV Impact | Action |
|-----------|-------------|--------|
| **Binary users** | ✅ None | Download from [releases](https://github.com/ai-screams/scoop-uv/releases) or `cargo install` |
| **Source builders** | ⚠️ Rust >= 1.85 required | Run `rustup update` if needed |
| **Contributors** | 🔧 Test on MSRV before PR | `cargo +1.85 test --all-features` |

<details>
<summary>📋 Full MSRV policy (click to expand)</summary>

### About N-1 Policy

We support the current stable Rust and one previous version (~6 week lag). MSRV updates are considered **non-breaking** for binary users per [Cargo RFC 3537](https://rust-lang.github.io/rfcs/3537-msrv-resolver.html).

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

### Edition 2024 Constraints

scuv uses **Rust Edition 2024**, which requires:
- Minimum Rust 1.85 (hard floor)
- MSRV-aware resolver enabled by default
- Cannot downgrade below 1.85 without changing edition to 2021

### Automation

- **CI**: Tests on both MSRV (1.85) and stable automatically
- **cargo-msrv**: Verifies MSRV on Cargo.toml changes in CI
- **Badge**: README badge auto-updates from Cargo.toml via shields.io
- **Local**: rust-toolchain.toml auto-selects 1.85 in project directory

For more details, see our [MSRV bump guide in CONTRIBUTING.md](CONTRIBUTING.md#bumping-msrv-step-by-step-guide).

</details>

---

<details>
<summary>🏗️ Architecture (for contributors and curious minds)</summary>

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

</details>

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
  that powers scuv's backend. Without uv's incredible speed and reliability, scuv wouldn't exist. Thank you to Charlie
  Marsh and the entire Astral team for revolutionizing Python tooling.

- **[pyenv](https://github.com/pyenv/pyenv)** & **[pyenv-virtualenv](https://github.com/pyenv/pyenv-virtualenv)** —
  The original inspiration for scuv's workflow. pyenv taught us how Python version management should feel,
  and pyenv-virtualenv showed us how to centralize virtual environments elegantly.

- **[virtualenv](https://github.com/pypa/virtualenv)** by [PyPA](https://www.pypa.io/) — The pioneer of Python virtual
  environments. Thank you to Ian Bicking for the original concept that changed how we isolate Python projects.

- **[Python](https://www.python.org/)** — The language that made programming accessible to everyone. scuv exists to
  make Python development even more delightful. Thank you to Guido van Rossum and the Python community.

- **[Rust](https://www.rust-lang.org/)** — The language that makes scuv fast, safe, and reliable. Thank you to the
  Rust team and Ferris 🦀 for proving that systems programming can be both powerful and enjoyable.

---

<div align="center">

<img src="assets/community/ferris/scoop-ferris.png" width="160" alt="scuv ferris">

*I built scuv because I needed it — and now it's yours too.* 🍨

*Grab a scoop, enjoy the flavor, and if you have thoughts to share,*
*the door to the ice cream parlor is always open.*

**[Issues](https://github.com/ai-screams/scoop-uv/issues)** · **[Discussions](https://github.com/ai-screams/scoop-uv/discussions)** · **[PRs Welcome](https://github.com/ai-screams/scoop-uv/pulls)**

</div>
