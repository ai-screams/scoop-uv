# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

> Note: the per-module AGENTS.md files are untracked via `.git/info/exclude` — update them on disk, never try to commit them. (CLAUDE.md itself is tracked since v0.15.0.)

## Project Overview

**scuv** is a centralized Python virtual environment manager using [uv](https://github.com/astral-sh/uv) as backend. It combines pyenv-virtualenv's workflow with uv's speed.

**Minimum uv version**: 0.5.14 (enforced by `scuv doctor`; the single source of truth is `MIN_VERSION` in `src/uv/version.rs`).

- **Language**: Rust (Edition 2024, MSRV 1.88)
- **License**: MIT OR Apache-2.0
- **Version**: 0.15.0 (command renamed `scoop` → `scuv`; crate/repo stay `scoop-uv`)
- **Tests**: 959 passed (888 unit + 44 integration + 2 i18n + 25 doctest), 0 clippy warnings
- **Test tooling**: rstest (table tests), proptest, cargo-mutants (mutation), cargo-fuzz (nightly `fuzz/` workspace); see `.docs/dev/testing-strategy.md`
- **i18n**: English, Korean, Japanese, Portuguese-BR (rust-i18n)
- **Shells**: bash, zsh, fish, PowerShell

## Rename & Legacy Compatibility (since v0.15.0)

- Legacy names (`SCOOP_*` env, `~/.scoop`, `.scoop-version`, `.scoop.toml`) are READ-only fallbacks with one-shot warnings; every removal site carries a `// DEPRECATION(0.16.0)` comment — sweep those when cutting 0.16.0.
- **PowerShell must NEVER define a `scoop` function/alias** (would shadow scoop.sh, the Windows package manager — the reason for the rename). Enforced by `init_script_never_defines_scoop` test.
- fish init/shell idiom is `scuv init fish | source` — `eval (...)` does NOT work in fish (splits multi-line output).
- Deliberately KEPT legacy identifiers (on-disk/serialized format compat): `.scoop-metadata.json`, export-schema field `scoop_export_version`. Do not "fix" these.

## Collaboration Rules (MUST FOLLOW)

- **NEVER create or merge a PR on your own.** Do not run `gh pr create`, `gh pr merge`, or any equivalent unless the user has **explicitly told you, in that specific request, that you may create and/or merge the PR**.
- Committing, pushing a branch, running CI, and reporting status are allowed without asking — but **creating a PR** and **merging a PR** each require an explicit, current go-ahead from the user.
- A general instruction like "fix this", "handle it", "해결해줘", or "가장 타당하게 진행해줘" does **NOT** authorize PR creation or merge. When in doubt, stop and ask.
- Closing/reopening PRs, force-push, and other externally visible or hard-to-reverse git actions also require explicit confirmation.

## Build & Development Commands

```bash
# Build
cargo build
cargo build --release

# Test
cargo test
cargo test <test_name>           # Single test

# Lint & Format
cargo fmt                        # Format code
cargo fmt --check                # Check formatting
cargo clippy --all-targets --all-features -- -D warnings

# Run
cargo run -- --help
cargo run -- list
```

### Pre-commit (prek)

```bash
prek install                     # Install hooks (first time)
prek run --all-files             # Run all checks
prek run cargo-fmt cargo-clippy  # Run specific hooks
```

### Testing Gotchas

- **After editing `locales/app.yml`, run `touch src/lib.rs` before `cargo test`** — rust_i18n's proc-macro isn't cargo-tracked; a yml-only edit reuses the stale binary and reports false-green.
- Env-var tests MUST use `env_guard` (src/test_utils.rs) + `#[serial]`, controlling both `SCUV_*` and legacy `SCOOP_*` (and `HOME` when dirs are inspected) — the dev machine has a real `~/.scoop`.
- PR CI runs `cargo-mutants --in-diff`: new `Check`-trait impls and thin wrappers need direct dispatch tests or the Mutants gate fails.

## MSRV Policy

**Policy**: N-1 (Moderate)
**Current MSRV**: 1.88 (ecosystem adopted `let`-chains; deps like `ignore` 0.4.30 and `serde-saphyr` require it. Edition 2024's own floor is 1.85.)
**Test Matrix**: `[msrv, stable]`

### Guidelines for AI Agents & Contributors

#### Before Bumping MSRV

1. **Verify benefit justifies change**:
   - ✅ New language features that significantly improve user experience
   - ✅ Critical dependency requires newer Rust
   - ✅ Security fix only available in newer version
   - ❌ Time-based updates without clear benefit
   - ❌ Minor syntax sugar or personal preference

2. **Check dependency constraints**:
   ```bash
   cargo tree --duplicates
   cargo msrv verify
   ```

3. **Test locally on new MSRV**:
   ```bash
   rustup install 1.86  # Example: bumping to 1.86
   cargo +1.86 test --all-features
   cargo +1.86 clippy --all-targets -- -D warnings
   ```

4. **Update all references**:
   - [ ] `Cargo.toml`: `rust-version = "1.86"`
   - [ ] `rust-toolchain.toml`: `channel = "1.86"`
   - [ ] `.github/workflows/ci.yml`: MSRV job toolchain version
   - [ ] `CHANGELOG.md`: Add entry with rationale
   - [ ] README.md badge will auto-update (dynamic)

5. **CHANGELOG entry format**:
   ```markdown
   ### Changed
   - **MSRV**: Bumped to 1.86 (reason: async trait improvements in std)
   ```

#### When Writing Code

- Assume Rust 1.88 as baseline
- Use Edition 2024 syntax freely (`gen` keyword reservation, unsafe extern blocks)
- All Rust 1.88+ features available (async-await, const generics, let-else, let-chains, RPIT in traits, etc.)
- Check feature stability: https://doc.rust-lang.org/stable/releases.html
- If unsure about feature MSRV, test with `cargo +1.88 check`

#### Testing Commands

```bash
# Test on MSRV (automatic via rust-toolchain.toml)
cargo test --all-features
cargo clippy --all-targets -- -D warnings

# Verify MSRV accuracy
cargo msrv verify

# Test on stable (override)
rustup override set stable
cargo test --all-features
rustup override unset
```

### Edition 2024 Constraints

scuv uses **Rust Edition 2024**, which requires:
- Minimum Rust 1.85 (hard floor)
- MSRV-aware resolver enabled by default
- Cannot downgrade below 1.85 without changing edition to 2021

### Automation

- **CI**: Tests on both MSRV (1.88) and stable automatically
- **cargo-msrv**: Verifies MSRV on Cargo.toml changes in CI
- **Badge**: README badge auto-updates from Cargo.toml via shields.io
- **Local**: rust-toolchain.toml auto-selects 1.88 in project directory

### References

- [RFC 3537: MSRV-aware Resolver](https://rust-lang.github.io/rfcs/3537-msrv-resolver.html)
- [MSRV Best Practices](https://github.com/rust-lang/api-guidelines/discussions/231)
- [Cargo CI Guide](https://doc.rust-lang.org/cargo/guide/continuous-integration.html)
- [rust-toolchain.toml Spec](https://rust-lang.github.io/rustup/overrides.html#the-toolchain-file)

## Architecture

### Module Structure

```
src/
├── cli/           # CLI parsing (clap)
│   ├── mod.rs     # Cli struct, Commands enum, ShellType
│   └── commands/  # Subcommand handlers (execute functions)
│       ├── use_env/   # Use command modular (normal, system, unset, symlink)
│       └── migrate/   # Migration subcommands
├── core/          # Domain logic
│   ├── version.rs       # Version file resolution (.scuv-version)
│   ├── metadata.rs      # Virtualenv metadata (JSON; last_used since 0.13)
│   ├── manifest.rs      # .scuv.toml manifest parsing (scuv sync)
│   ├── export_schema.rs # Portable env export/import schema (scuv export/import)
│   ├── virtualenv/      # Virtualenv entity (mod.rs + tests.rs)
│   ├── doctor.rs        # Health check (Doctor, Check trait)
│   └── migrate/         # Migration from pyenv/conda/virtualenvwrapper
├── shell/         # Shell integration (bash, zsh, fish, powershell)
│   ├── bash.rs, zsh.rs, fish.rs, powershell.rs  # Shell-specific scripts
│   └── common.rs  # Shared utilities (version check, 4-shell macros)
├── uv/            # uv CLI wrapper (client.rs) + version policy (version.rs)
├── output/        # Terminal UI & JSON output
│   └── time.rs    # English fuzzy-age formatter for last_used display
├── error/         # ScoopError module (code, display, exit codes, migrate, suggestion; i18n Display)
├── paths.rs       # Path utilities (scoop_home, virtualenvs_dir)
├── validate.rs    # Validation logic
├── i18n.rs        # Internationalization (locale detection, t! macro)
└── config.rs      # Config management (~/.scuv/config.json)

locales/
└── app.yml        # Translation strings (en, ko, ja, pt-BR)
```

Per-module deep dives live in untracked `AGENTS.md` files (src/, src/core/, src/shell/, src/uv/, locales/, tests/, ...).

### Key Patterns

**Shell Integration**: CLI outputs shell code to stdout, shell function uses `eval` to execute (pyenv pattern).

```bash
# User runs: scuv activate myenv
# CLI outputs: export VIRTUAL_ENV="..." export PATH="..."
# Shell wrapper: eval "$(command scuv activate myenv)"
```

**Version File Priority**: `SCUV_VERSION` env > `.scuv-version` (local + parent walk) > `~/.scuv/version`

**Shell Type Detection**: Auto-detects via `FISH_VERSION`, `PSModulePath`, `ZSH_VERSION` (in priority order). Override with `--shell` option.

## CLI Commands

> **Tip:** Most commands support `--json` for machine-readable output (list, create, use, remove, install, uninstall, doctor, info, status, which, lang, migrate).

| Command | Aliases | Description |
|---------|---------|-------------|
| `scuv list` | `ls` | List virtualenvs or Python versions (`--sort name|created|last-used`) |
| `scuv create <NAME> [VER]` | - | Create virtualenv (`--install-python` for lazy install) |
| `scuv use <NAME>` | - | Set + activate environment |
| `scuv remove <NAME>` | `rm`, `delete` | Remove virtualenv |
| `scuv clone <SRC> <DST>` | - | Duplicate an environment |
| `scuv diff <A> <B>` | - | Compare two environments (Python, packages, metadata) |
| `scuv export <NAME>` | - | Snapshot an env as portable JSON |
| `scuv import <FILE>` | - | Recreate an env from an export file |
| `scuv sync` | - | Apply `.scuv.toml` (create env + install packages) |
| `scuv install [VER]` | - | Install Python version |
| `scuv uninstall <VER>` | - | Uninstall Python version |
| `scuv doctor` | - | Diagnose installation |
| `scuv info <NAME>` | - | Show virtualenv details |
| `scuv status` | - | Summarise the currently active environment |
| `scuv which <EXE>` | - | Resolve an executable inside the active env |
| `scuv run <ENV> -- <CMD>` | - | Run a command inside an env without activating |
| `scuv init <SHELL>` | - | Shell init script |
| `scuv completions <SHELL>` | - | Completion script |
| `scuv lang [CODE]` | - | Get/set language (en, ko, ja, pt-BR) |
| `scuv migrate list` | - | List migratable environments |
| `scuv migrate @env <NAME>` | - | Migrate single environment |
| `scuv migrate all` | - | Migrate all environments (parallel via rayon) |
| `scuv gc` | - | GC orphan virtualenvs (default dry-run; `--yes` removes, `--aggressive` also for Pythons, `--older-than <n>d/w/y` flags stale envs by `last_used`) |
| `scuv prune` | - | Prune the uv cache (`uv cache prune` wrapper) |
| `scuv verify [NAME]` | - | Per-env health diagnosis (6 checks; `--strict` for CI gates) |
| `scuv man [DIR]` | - | Generate man pages (stdout or `scuv.1` + `scuv-<sub>.1` files in DIR) |
| `scuv self update` | - | Update scuv itself from crates.io |
| `scuv use system` | - | Use system Python (deactivate) |
| `scuv use --unset` | - | Remove local/global version file |
| `scuv shell <NAME>` | - | Set shell-specific env (eval required) |
| `scuv shell --unset` | - | Clear shell-specific setting |

### Global Options

| Option | Description |
|--------|-------------|
| `--quiet` | Minimal output |
| `--no-color` | Disable colors |

### Common Options

| Option | Availability | Description |
|--------|--------------|-------------|
| `--json` | Most commands | JSON output (for scripts) |
| `-h`, `--help` | All commands | Show help |
| `-V`, `--version` | All commands | Show version |

## Naming Conventions

| Item | Rule | Example |
|------|------|---------|
| CLI commands | lowercase | `scuv create` |
| Environment variables | SCREAMING_SNAKE | `SCUV_HOME` |
| Error types | PascalCase | `ScoopError` |
| Version file | dot-prefix | `.scuv-version` |

### Environment Name Rules

- Regex: `^[a-zA-Z][a-zA-Z0-9_-]*$`
- Must start with letter (not number, to distinguish from version strings like "3.12")
- Reserved words: `activate`, `base`, `completions`, `create`, `deactivate`, `default`, `delete`, `global`, `help`, `init`, `install`, `list`, `local`, `remove`, `resolve`, `root`, `system`, `uninstall`, `use`, `version`, `versions`

## Documentation Style

### Doc Comment Rules

- `///` for items (functions, structs), `//!` for modules
- Summary line: verb-first, period-end (e.g., `/// Returns the...`)
- Required sections for `pub` items:
  - `# Examples` - Always include usage examples
  - `# Errors` - When returning `Result`
  - `# Panics` - When panic is possible
  - `# Safety` - For `unsafe fn`

### Doctest

```rust
/// Creates a new environment.
///
/// # Examples
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let env = VirtualEnv::create("myenv", "3.12")?;
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns [`ScoopError::InvalidName`] if the name is invalid.
pub fn create(name: &str, version: &str) -> Result<VirtualEnv, ScoopError>
```

- Use `#` prefix for hidden setup code
- Use `?` operator instead of `unwrap()`
- Attributes: `no_run`, `should_panic`, `compile_fail`

## Design Documents

`.docs/` contains internal design documents (git excluded):

| Category | Path | Description |
|----------|------|-------------|
| ADR | `adr/0001-architecture.md` | Architecture decisions |
| Plan | `plan/mvp.md` | MVP feature scope |
| Spec | `spec/naming.md` | Naming conventions |
| Spec | `spec/cli-options.md` | CLI options spec |
| Dev | `dev/code-quality.md` | Linting/formatting setup |
| Dev | `dev/documentation.md` | Rust documentation best practices |

### Document Naming Rules

- **ADR**: `NNNN-kebab-title.md` (e.g., `0001-architecture.md`)
- **Stable docs**: `kebab-case.md` (e.g., `doctor.md`, `cli-options.md`)
- **WIP docs**: `[type] YYYY-MM-DD-kebab-title.md`
  - Workflow types: `[brainstorm]`, `[research]`, `[impl]`, `[refactor]`
  - Conventional types: `[feat]`, `[fix]`, `[docs]`, `[chore]`, `[perf]`, `[test]`
  - e.g., `[research] 2026-01-10-migrate.md`, `[impl] 2026-01-10-migrate-phase1.md`
- **Forbidden**: spaces, Korean filenames, SCREAMING_SNAKE_CASE

### Frontmatter (Required)

All `.docs/` files must include YAML frontmatter:

```yaml
---
title: filename-without-extension
tags:
  - scuv           # Required for all files
  - [folder-tag]    # adr, design, spec, dev, plan, wip, done
  - [content-tags]  # Relevant keywords
---
```

## Error Handling

Use `thiserror` with manual `Display` impl for i18n support:

```rust
// ScoopError uses thiserror for Error trait, but Display is manual for i18n
#[derive(Error, Debug)]
pub enum ScoopError {
    #[error("")]  // Placeholder - Display impl handles actual message
    VirtualenvNotFound { name: String },
}

impl std::fmt::Display for ScoopError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::VirtualenvNotFound { name } => {
                write!(f, "{}", t!("error.virtualenv_not_found", name = name))
            }
            // ...
        }
    }
}
```

## Internationalization (i18n)

**Locale Priority**:
1. `SCUV_LANG` environment variable
2. `~/.scuv/config.json` setting
3. System locale (sys-locale)
4. Default: `en`

**Usage**:
```rust
use crate::i18n::t;

// In CLI output
println!("{}", t!("create.creating", name = name));

// In error messages (via Display impl)
t!("error.virtualenv_not_found", name = name)
```

**Translation file**: `locales/app.yml`
- 226 keys total (error.* 41, suggestion.* 16); parity across all 4 locales enforced by tests/i18n_completeness.rs
- ko conventions: no semicolons in ko values; "scuv"(스커브) has no batchim — particles are 가/를/는/와/로 (never 이/을/은/과/으로). Hand-edit ko/ja, never blind-sed.
- `docs/po/ko.po`: regenerate via `MDBOOK_OUTPUT='{"xgettext": {}}' mdbook build -d po && msgmerge --update po/ko.po po/messages.pot`; CI (tag push) requires the committed file to round-trip byte-identical.

## Docker Development

```bash
# Interactive shells (auto-builds scuv on entry)
make docker-shell          # bash
make docker-shell-zsh      # zsh
make docker-shell-fish     # fish

# Run tests (integration tests run in Docker)
make test-integration    # Docker integration tests
make test-all            # unit + integration
```

**Features**:
- Auto-build on source change detection (`.rs`, `locales/*.yml`)
- Live reload for shell scripts (no image rebuild needed)
- Workspace scuv takes precedence over image-installed version
