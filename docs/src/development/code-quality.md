# Code Quality

Comprehensive guide for maintaining code quality in scoop.

## Quick Reference

```bash
# Format code
cargo fmt

# Lint check
cargo clippy --all-targets --all-features -- -D warnings

# All checks (pre-commit style)
cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test

# Pre-commit hooks
prek run --all-files
```

## Formatting (rustfmt)

### Configuration

Located in `rustfmt.toml`:

```toml
edition = "2024"
max_width = 100
tab_spaces = 4
use_field_init_shorthand = true
use_try_shorthand = true
```

### Commands

```bash
# Auto-format all files
cargo fmt

# Check formatting (CI mode, no changes)
cargo fmt --check

# Format specific file
rustfmt src/main.rs

# Show diff instead of applying
cargo fmt -- --check --diff
```

### IDE Integration

**VS Code** (rust-analyzer):

```json
{
  "[rust]": {
    "editor.formatOnSave": true
  }
}
```

**JetBrains (RustRover/CLion)**:

- Settings → Languages → Rust → Rustfmt → Run on save

## Linting (Clippy)

### Basic Usage

```bash
# Standard lint check
cargo clippy

# Treat warnings as errors (CI mode)
cargo clippy -- -D warnings

# All targets (including tests, examples)
cargo clippy --all-targets

# All features enabled
cargo clippy --all-features

# Full CI check
cargo clippy --all-targets --all-features -- -D warnings
```

### Lint Categories

```bash
# Enable specific lint category
cargo clippy -- -W clippy::pedantic

# Deny specific lint
cargo clippy -- -D clippy::unwrap_used

# Allow specific lint
cargo clippy -- -A clippy::too_many_arguments
```

### Common Lints

| Lint                   | Severity | Description                   |
|------------------------|----------|-------------------------------|
| `clippy::unwrap_used`  | Warn     | Use `?` or `expect()` instead |
| `clippy::panic`        | Warn     | Avoid panic in library code   |
| `clippy::todo`         | Warn     | Remove before release         |
| `clippy::dbg_macro`    | Warn     | Remove debug macros           |
| `clippy::print_stdout` | Warn     | Use logging instead           |

### Fixing Lints

```bash
# Auto-fix where possible
cargo clippy --fix

# Allow fixes that change behavior
cargo clippy --fix --allow-dirty --allow-staged
```

### Suppressing Lints

```rust
// Single line
#[allow(clippy::too_many_arguments)]
fn complex_function(...) {}

// Entire module
#![allow(clippy::module_inception)]

// With explanation
#[allow(clippy::unwrap_used)] // Safe: validated in parse()
fn get_value() {}
```

## Pre-commit Hooks (prek)

### Setup

```bash
# Install prek
cargo install prek
# or
uv tool install prek

# Install hooks in repository
prek install
```

### Configuration

Located in `.pre-commit-config.yaml`:

```yaml
repos:
  - repo: local
    hooks:
      - id: cargo-fmt
        name: cargo fmt
        entry: cargo fmt --all --
        language: system
        types: [ rust ]
        pass_filenames: false

      - id: cargo-clippy
        name: cargo clippy
        entry: cargo clippy --all-targets --all-features -- -D warnings
        language: system
        types: [ rust ]
        pass_filenames: false

      - id: cargo-check
        name: cargo check
        entry: cargo check --all-targets
        language: system
        types: [ rust ]
        pass_filenames: false
```

### Usage

```bash
# Run all hooks on staged files
prek run

# Run all hooks on all files
prek run --all-files

# Run specific hook
prek run cargo-fmt
prek run cargo-clippy

# Run multiple specific hooks
prek run cargo-fmt cargo-clippy

# Skip hooks (emergency only!)
git commit --no-verify
```

### Available Hooks

| Hook                  | Description            | When       |
|-----------------------|------------------------|------------|
| `cargo-fmt`           | Code formatting        | Pre-commit |
| `cargo-clippy`        | Linting                | Pre-commit |
| `cargo-check`         | Type checking          | Pre-commit |
| `trailing-whitespace` | Remove trailing spaces | Pre-commit |
| `end-of-file-fixer`   | Ensure newline at EOF  | Pre-commit |
| `check-toml`          | Validate TOML files    | Pre-commit |
| `check-yaml`          | Validate YAML files    | Pre-commit |

## CI Pipeline

### GitHub Actions

Located in `.github/workflows/ci.yml`:

```yaml
name: CI

on: [ push, pull_request ]

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-action@stable
        with:
          components: rustfmt, clippy

      - name: Format check
        run: cargo fmt --all -- --check

      - name: Clippy
        run: cargo clippy --all-targets --all-features -- -D warnings

      - name: Test
        run: cargo test --all-features
```

### Local CI Simulation

```bash
# Run exactly what CI runs
cargo fmt --check && \
cargo clippy --all-targets --all-features -- -D warnings && \
cargo test --all-features
```

## Code Style Guidelines

### Naming Conventions

| Item      | Convention        | Example             |
|-----------|-------------------|---------------------|
| Modules   | `snake_case`      | `version_file`      |
| Functions | `snake_case`      | `get_version()`     |
| Types     | `PascalCase`      | `VirtualenvService` |
| Constants | `SCREAMING_SNAKE` | `MAX_NAME_LENGTH`   |
| Lifetimes | short lowercase   | `'a`, `'src`        |

### Documentation

```rust
/// Creates a new virtual environment.
///
/// # Arguments
///
/// * `name` - Environment name (must be valid)
/// * `python` - Python version (e.g., "3.12")
///
/// # Returns
///
/// Path to the created environment.
///
/// # Errors
///
/// Returns [`ScoopError::InvalidEnvName`] if name is invalid.
///
/// # Examples
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let path = create_env("myenv", "3.12")?;
/// # Ok(())
/// # }
/// ```
pub fn create_env(name: &str, python: &str) -> Result<PathBuf> {
    // ...
}
```

### Error Handling

```rust
// Good: Use ? operator
fn process() -> Result<()> {
    let file = File::open(path)?;
    let data = read_data(&file)?;
    Ok(())
}

// Good: Contextual errors
fn process() -> Result<()> {
    let file = File::open(path)
        .map_err(|e| ScoopError::Io(e))?;
    Ok(())
}

// Avoid: unwrap() in library code
fn bad() {
    let file = File::open(path).unwrap(); // Bad
}

// OK: expect() with explanation
fn acceptable() {
    let home = dirs::home_dir()
        .expect("home directory must exist");
}
```

### Import Organization

```rust
// 1. Standard library
use std::collections::HashMap;
use std::path::PathBuf;

// 2. External crates
use clap::Parser;
use serde::Serialize;
use thiserror::Error;

// 3. Local modules
use crate::error::Result;
use crate::paths;
```

## Security Considerations

### Dependency Auditing

```bash
# Install cargo-audit
cargo install cargo-audit

# Run audit
cargo audit

# Fix vulnerabilities
cargo audit fix
```

### MSRV (Minimum Supported Rust Version)

- Current MSRV: **1.85**
- Defined in `Cargo.toml`:
  ```toml
  [package]
  rust-version = "1.85"
  ```

### Unsafe Code

- Avoid `unsafe` unless absolutely necessary
- Document safety invariants
- Use `#![forbid(unsafe_code)]` in library crates

## Performance

### Profiling

```bash
# Build with debug info for release
cargo build --release

# Use flamegraph
cargo install flamegraph
cargo flamegraph --bin scoop -- list
```

### Benchmarks

```bash
# Run benchmarks (if defined)
cargo bench

# Using criterion
cargo bench --bench my_benchmark
```

## Continuous Improvement

### Regular Checks

```bash
# Weekly dependency update check
cargo outdated

# Security audit
cargo audit

# MSRV check
cargo msrv verify
```

### Upgrade Dependencies

```bash
# Update Cargo.lock
cargo update

# Upgrade to latest compatible versions
cargo upgrade  # requires cargo-edit
```

## Troubleshooting

### Clippy False Positives

```rust
// Silence with explanation
#[allow(clippy::needless_return)]
fn explicit_return() -> i32 {
    return 42; // Intentional for readability
}
```

### Format Conflicts

```rust
// Skip formatting for specific block
#[rustfmt::skip]
const MATRIX: [[i32; 3]; 3] = [
    [1, 0, 0],
    [0, 1, 0],
    [0, 0, 1],
];
```

### CI vs Local Differences

```bash
# Ensure same toolchain as CI
rustup update stable
rustup default stable

# Check Rust version
rustc --version
```

## Summary Checklist

Before committing:

- [ ] `cargo fmt` - Code formatted
- [ ] `cargo clippy -- -D warnings` - No lint warnings
- [ ] `cargo test` - All tests pass
- [ ] `cargo doc` - Documentation builds
- [ ] No `todo!()` or `dbg!()` left in code
- [ ] Public APIs documented
- [ ] Error messages are helpful
