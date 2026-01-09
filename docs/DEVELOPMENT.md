# Development Guide 🍨

> *"Want to add a new flavor? Here's the recipe book."*

Guide for contributing to scoop development.

---

## Prerequisites

- **Rust 1.85+** (Edition 2024)
- **uv** - The secret ingredient ([install](https://github.com/astral-sh/uv))
- **prek** - Pre-commit hooks ([install](https://github.com/j178/prek))

---

## Setup

```bash
# Clone the repository
git clone https://github.com/ai-screams/scoop-uv.git
cd scoop-uv

# Install prek (Rust-native pre-commit alternative)
uv tool install prek
# or: cargo install prek

# Install git hooks
prek install

# Build
cargo build

# Run tests
cargo test
```

---

## The Kitchen Layout 🏗️

```
src/
├── main.rs              # 🚪 Entry point
├── lib.rs               # 📚 Library root
├── error.rs             # ❌ Error types (ScoopError)
├── paths.rs             # 📁 Path utilities
├── validate.rs          # ✅ Name/version validation

├── uv/                  # 🔮 uv client wrapper (the secret ingredient)
│   ├── mod.rs
│   └── client.rs

├── core/                # 🧠 Business logic (the flavor science)
│   ├── mod.rs
│   ├── virtualenv.rs    # VirtualenvService
│   ├── version.rs       # VersionService
│   ├── metadata.rs      # Metadata structs
│   └── doctor.rs        # Health diagnostics 🩺

├── cli/                 # 🎮 CLI layer (the counter)
│   ├── mod.rs           # Cli struct, Commands enum
│   └── commands/        # Command handlers
│       ├── mod.rs
│       ├── list.rs
│       ├── create.rs
│       ├── use_env.rs
│       ├── remove.rs
│       ├── install.rs
│       ├── doctor.rs
│       └── ...

├── shell/               # 🐚 Shell integration
│   ├── mod.rs
│   ├── bash.rs
│   └── zsh.rs

└── output/              # 🎨 Output formatting (presentation!)
    ├── mod.rs
    └── spinner.rs

docs/                    # 📖 Public documentation
.docs/                   # 🔒 Internal technical docs
tests/                   # 🧪 Integration tests
```

---

## Common Commands

### Build & Run

```bash
cargo build              # Debug build
cargo build --release    # Release build (optimized)
cargo run -- --help      # Show help
cargo run -- list        # Run commands
cargo run -- doctor      # Check setup health
```

### Quick Quality Check

```bash
# All checks at once (recommended before commit)
cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test
```

For detailed guides, see:

- **[TESTING.md](TESTING.md)** — Comprehensive testing guide
- **[CODE_QUALITY.md](CODE_QUALITY.md)** — Formatting, linting, pre-commit hooks

---

## Architecture

### Key Services

**VirtualenvService** (`src/core/virtualenv.rs`)

- Manages flavors in `~/.scoop/virtualenvs/` (the freezer)
- Wraps uv commands for venv creation

**VersionService** (`src/core/version.rs`)

- Manages `.scoop-version` files
- Resolves current directory → active flavor

**Doctor** (`src/core/doctor.rs`)

- Health diagnostics for scoop setup
- Checks uv, shell integration, paths, environments

**UvClient** (`src/uv/client.rs`)

- Wrapper for `uv` CLI commands
- Python version management (the secret ingredient)

### Shell Integration

Shell scripts are embedded in Rust code:

- `src/shell/bash.rs` - Bash init script
- `src/shell/zsh.rs` - Zsh init script

Key components:

1. **Wrapper function** - Intercepts `use`/`activate`/`deactivate`
2. **Hook function** - Auto-activation on directory change
3. **Completion function** - Tab completion

---

## Adding a New Command

1. Define command in `src/cli/mod.rs`:

```rust
#[derive(Subcommand)]
pub enum Commands {
    // ...
    MyCommand {
        #[arg(short, long)]
        option: bool,
    },
}
```

2. Create handler in `src/cli/commands/my_command.rs`:

```rust
pub fn execute(output: &Output, option: bool) -> Result<()> {
    // Implementation
    Ok(())
}
```

3. Export in `src/cli/commands/mod.rs`

4. Wire up in `src/main.rs`

5. Add shell completion in `src/shell/{bash,zsh}.rs`

---

## Testing

### Unit Tests

Located within source files:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something() {
        // ...
    }
}
```

### Integration Tests

Located in `tests/`:

```rust
use assert_cmd::Command;

#[test]
fn test_cli_command() {
    Command::cargo_bin("scoop")
        .unwrap()
        .args(["list"])
        .assert()
        .success();
}
```

---

## Release Process

Releases are automated via [release-plz](https://release-plz.dev/):

1. Create PR with changes
2. Merge to main
3. release-plz creates release PR
4. Merge release PR → publishes to crates.io

---

## Internal Documentation

See `.docs/` for internal technical references:

- `TECHNICAL_REFERENCE.md` - Implementation details
- `SHELL_GOTCHAS.md` - Shell integration pitfalls
- `IMPLEMENTATION_PLAN.md` - Development roadmap
- `brand/brand.md` - Brand guidelines 🍨

---

## Code Style

- Follow Rust conventions
- Run `cargo fmt` before committing
- Keep functions small and focused
- Document public APIs with `///` comments
- Use `thiserror` for error types
- Korean error messages with solutions (per CLAUDE.md)

---

> 🍨 *"Good code is like good ice cream — crafted with care, enjoyed by many."*
