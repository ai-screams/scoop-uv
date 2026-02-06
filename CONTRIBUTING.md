# 🍨 Contributing to scoop

> *"Every great flavor started with someone willing to experiment!"*

Thank you for your interest in contributing to scoop! Whether you're fixing a bug, adding a feature, or improving documentation, your help makes scoop better for everyone.

---

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Ways to Contribute](#ways-to-contribute)
- [Development Setup](#development-setup)
- [Coding Style](#coding-style)
- [Commit Convention](#commit-convention)
- [Pull Request Process](#pull-request-process)
- [Testing Guide](#testing-guide)
- [Project Structure](#project-structure)
- [Getting Help](#getting-help)

---

## Code of Conduct

By participating in this project, you agree to maintain a respectful and inclusive environment. Be kind, be constructive, and remember: we're all here to make Python environment management a little sweeter. 🍦

---

## Ways to Contribute

### 🫠 Report Bugs

Found a melted scoop? [Open a bug report](https://github.com/ai-screams/scoop-uv/issues/new?template=bug_report.yml)

Before reporting:
- Search existing issues to avoid duplicates
- Use the latest version of scoop
- Include reproduction steps

### 🍦 Request Features

Got an idea for a new flavor? [Request a feature](https://github.com/ai-screams/scoop-uv/issues/new?template=feature_request.yml)

### 🧑‍🍳 Contribute Code

Ready to cook up something new? Follow the guide below!

### 📖 Improve Documentation

Docs can always be clearer. PRs welcome for:
- Typo fixes
- Clarifications
- New examples

### 🌍 Translate scoop

Help make scoop accessible to developers worldwide!

**Quick start:**
1. Add translations to `locales/app.yml` (106 keys)
2. Register your language in `src/i18n.rs`
3. Submit PR with title: `docs(i18n): add {Language} translation`

**Philosophy:** We trust translators. Casual tone, creative expressions welcome — clarity is the only rule.

📖 **[Full Translation Guide](https://ai-screams.github.io/scoop-uv/development/translation.html)**

---

## Development Setup

### Prerequisites

| Tool | Version | Install |
|------|---------|---------|
| **Rust** | 1.85+ | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| **uv** | latest | `curl -LsSf https://astral.sh/uv/install.sh \| sh` |
| **prek** | 0.2.23+ | `uv tool install prek` or `cargo install prek` |

### Clone & Build

```bash
# Clone the repository
git clone https://github.com/ai-screams/scoop-uv.git
cd scoop-uv

# Install git hooks (required!)
prek install

# Build
cargo build

# Run tests
cargo test

# Run the CLI
cargo run -- --help
```

### Rust Version & MSRV

scoop requires **Rust 1.85 or newer** (our MSRV - Minimum Supported Rust Version). The project uses `rust-toolchain.toml` to automatically select the correct version.

#### First-Time Setup

```bash
# Clone repository
git clone https://github.com/ai-screams/scoop-uv.git
cd scoop-uv

# Rust 1.85 will be automatically selected via rust-toolchain.toml
rustc --version
# Expected: rustc 1.85.0 (a28077b28 2025-02-20)

# If you see a different version:
rustup update
rustup toolchain install 1.85
```

#### Updating Rust

```bash
# Update to latest within the MSRV channel
rustup update 1.85

# Or update to latest stable (for testing)
rustup update stable
```

#### Testing on MSRV

**Before submitting PRs**, verify compatibility with our MSRV:

```bash
# The project automatically uses 1.85 via rust-toolchain.toml
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features --workspace
cargo build --all-features

# To explicitly test on stable:
rustup override set stable
cargo test --all-features
rustup override unset  # Back to MSRV
```

CI automatically tests both MSRV (1.85) and stable Rust.

#### Adding Dependencies

When adding dependencies:

1. **Check their MSRV**:
   ```bash
   cargo tree --duplicates
   # Look for rust-version in dependencies' Cargo.toml
   ```

2. **Ensure compatibility** with our MSRV (1.85)

3. **If dependency requires newer Rust**:
   - Evaluate if benefit justifies MSRV bump
   - Discuss in PR description
   - Tag maintainers for MSRV policy review

4. **Run cargo-msrv verification**:
   ```bash
   cargo install cargo-msrv
   cargo msrv verify
   ```

#### MSRV Policy for Contributors

- **Current MSRV**: 1.85 (Edition 2024 requirement)
- **Policy**: N-1 (support current stable + 1 previous version)
- **Updates**: MSRV bumps only for clear benefits (features, security, dependencies)
- **Communication**: All MSRV changes documented in CHANGELOG with rationale

See [README - MSRV Section](README.md#minimum-supported-rust-version-msrv-) for full policy.

#### MSRV Verification Tools

```bash
# Verify current MSRV
cargo msrv show

# Verify MSRV is accurate
cargo msrv verify

# Find minimum possible MSRV (slow, ~10-20 minutes)
cargo msrv find
```

#### Bumping MSRV: Step-by-Step Guide

When you need to increase the MSRV (e.g., from 1.85 to 1.86):

**Step 1: Evaluate Justification**

Ask yourself:
- ✅ Does a new Rust feature significantly improve user experience?
- ✅ Does a critical dependency require the newer version?
- ✅ Is there a security fix only in newer Rust?
- ❌ Is this just for aesthetic preferences or minor convenience?

If not justified, don't bump.

**Step 2: Update All Three Locations**

```bash
# 1. Update Cargo.toml
# macOS: sed -i '' 's/...' file
# Linux: sed -i 's/...' file
sed -i.bak 's/rust-version = "1.85"/rust-version = "1.86"/' Cargo.toml && rm Cargo.toml.bak

# 2. Update rust-toolchain.toml
sed -i.bak 's/channel = "1.85"/channel = "1.86"/' rust-toolchain.toml && rm rust-toolchain.toml.bak

# 3. Update CI workflow
sed -i.bak 's/@1.85/@1.86/g' .github/workflows/ci.yml && rm .github/workflows/ci.yml.bak

# Or manually edit the three files in your editor (safer)
```

**Step 3: Test Locally**

```bash
# Install new MSRV
rustup install 1.86

# Test compilation
cargo +1.86 test --all-features

# Test clippy
cargo +1.86 clippy --all-targets --all-features -- -D warnings

# Verify MSRV
cargo msrv verify
```

**Step 4: Update CHANGELOG**

```bash
# Add to [Unreleased] section
cat >> CHANGELOG.md << 'EOF'

### Changed

- **MSRV**: Bumped to 1.86 (reason: [your justification])
  - Example: "for improved async trait support in std"
  - Example: "clap 4.6 requires Rust 1.86"
  - Example: "security fix CVE-YYYY-XXXXX in rustc 1.86"
EOF
```

**Step 5: Create PR**

```bash
git add Cargo.toml rust-toolchain.toml .github/workflows/ci.yml CHANGELOG.md
git commit -m "chore: bump MSRV to 1.86 for [reason]"
git push origin feat/msrv-1.86
```

**Step 6: Verify CI Passes**

- ✅ MSRV job (1.86) passes
- ✅ cargo-msrv verify passes
- ✅ Test job (stable) passes

**What If Something Goes Wrong?**

See [Troubleshooting MSRV Issues](#troubleshooting-msrv-issues) below.

### IDE Setup

#### VS Code

Recommended extensions:
- `rust-analyzer` - Rust language support
- `Even Better TOML` - TOML syntax highlighting
- `Error Lens` - Inline error display

```json
// .vscode/settings.json (optional)
{
    "rust-analyzer.check.command": "clippy",
    "editor.formatOnSave": true
}
```

#### RustRover / IntelliJ

- Enable "Run rustfmt on save"
- Set Clippy as external linter

---

## Coding Style

### 🎨 The Recipe (rustfmt)

We use `rustfmt` with these settings (`rustfmt.toml`):

```toml
edition = "2024"
max_width = 100
tab_spaces = 4
use_field_init_shorthand = true
use_try_shorthand = true
```

**Format before committing:**
```bash
cargo fmt
```

### 🔍 Quality Check (Clippy)

Clippy is our sous chef, catching issues before they reach production:

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

All warnings are treated as errors. Fix them before submitting.

### 📝 Naming Conventions

| Item | Convention | Example |
|------|------------|---------|
| Functions | `snake_case` | `create_virtualenv()` |
| Types/Structs | `PascalCase` | `VirtualenvService` |
| Constants | `SCREAMING_SNAKE` | `SCOOP_HOME_ENV` |
| Modules | `snake_case` | `mod shell_integration;` |
| CLI commands | `kebab-case` | `scoop create-env` |
| Environment vars | `SCREAMING_SNAKE` | `SCOOP_NO_AUTO` |

### 📚 Documentation Style

Document all public APIs:

```rust
/// Creates a new virtual environment.
///
/// # Arguments
///
/// * `name` - The environment name (must be valid identifier)
/// * `python_version` - Python version (e.g., "3.12")
///
/// # Returns
///
/// Path to the created virtual environment.
///
/// # Errors
///
/// Returns [`ScoopError::InvalidName`] if name is invalid.
/// Returns [`ScoopError::UvError`] if uv command fails.
///
/// # Examples
///
/// ```no_run
/// let path = create_virtualenv("myenv", "3.12")?;
/// ```
pub fn create_virtualenv(name: &str, python_version: &str) -> Result<PathBuf> {
    // ...
}
```

### 🚫 Don'ts

```rust
// ❌ Don't use unwrap() in library code
let value = some_option.unwrap();

// ✅ Use proper error handling
let value = some_option.ok_or(ScoopError::NotFound)?;

// ❌ Don't ignore errors silently
let _ = fs::remove_file(path);

// ✅ Handle or propagate errors
fs::remove_file(path)?;
```

---

## Commit Convention

We follow [Conventional Commits](https://www.conventionalcommits.org/) for clear, automated changelogs.

### Format

```
<type>(<scope>): <description>

[optional body]

[optional footer(s)]
```

### Types

| Type | Description | Emoji |
|------|-------------|:-----:|
| `feat` | New feature | ✨ |
| `fix` | Bug fix | 🐛 |
| `docs` | Documentation & translations | 📖 |
| `style` | Code style (no logic change) | 🎨 |
| `refactor` | Code refactoring | ♻️ |
| `perf` | Performance improvement | ⚡ |
| `test` | Add/update tests | 🧪 |
| `build` | Build system/dependencies | 📦 |
| `ci` | CI configuration | 🔧 |
| `chore` | Other changes | 🔨 |

### Scopes (optional)

`cli`, `core`, `shell`, `uv`, `output`, `i18n`, `docs`, `deps`

### Examples

```bash
# Feature
feat(cli): add --json flag to list command

# Bug fix
fix(shell): correct PATH handling in zsh hook

# Documentation
docs(readme): update installation instructions

# Breaking change (add ! after type)
feat(cli)!: rename 'local' command to 'use'

BREAKING CHANGE: The 'local' command has been renamed to 'use'.
Users must update their scripts.
```

### Commit Message Tips

- Use imperative mood: "add" not "added" or "adds"
- Keep first line under 72 characters
- Reference issues: "fix(cli): handle empty input (#42)"
- One logical change per commit

---

## Pull Request Process

### 1. Branch Strategy

```bash
# Create feature branch from main
git checkout main
git pull origin main
git checkout -b feat/your-feature-name

# Or for bug fixes
git checkout -b fix/issue-description
```

**Branch naming:**
- `feat/description` - New features
- `fix/description` - Bug fixes
- `docs/description` - Documentation
- `refactor/description` - Refactoring

### 2. Before Submitting

Run the full check suite:

```bash
# Format code
cargo fmt

# Run linter
cargo clippy --all-targets --all-features -- -D warnings

# Run tests
cargo test --all-features

# Or run all pre-commit hooks
prek run --all-files
```

### 3. PR Checklist

Before submitting, ensure:

- [ ] Code compiles without warnings (`cargo check`)
- [ ] All tests pass (`cargo test`)
- [ ] Code is formatted (`cargo fmt --check`)
- [ ] Clippy is happy (`cargo clippy -- -D warnings`)
- [ ] Documentation is updated (if needed)
- [ ] Commit messages follow convention
- [ ] PR description explains the changes

### 4. PR Template

When you open a PR, include:

```markdown
## Summary

Brief description of changes.

## Motivation

Why is this change needed?

## Changes

- Change 1
- Change 2

## Testing

How did you test this?

## Checklist

- [ ] Tests added/updated
- [ ] Documentation updated
- [ ] Breaking changes documented
```

### 5. Review Process

1. **Automated checks** - CI must pass
2. **Code review** - Maintainer reviews code
3. **Address feedback** - Make requested changes
4. **Approval** - Get at least one approval
5. **Merge** - Maintainer merges via squash

---

## Testing Guide

### Test Structure

```
src/
├── module.rs
│   └── #[cfg(test)] mod tests { ... }  # Unit tests
│
tests/
├── integration_test.rs                  # Integration tests
└── common/mod.rs                        # Shared test utilities
```

### Unit Tests

Located within source files:

```rust
// src/validate.rs

pub fn is_valid_name(name: &str) -> bool {
    // implementation
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_name() {
        assert!(is_valid_name("myenv"));
        assert!(is_valid_name("my-env"));
        assert!(is_valid_name("my_env_123"));
    }

    #[test]
    fn test_invalid_name() {
        assert!(!is_valid_name(""));
        assert!(!is_valid_name("123start"));
        assert!(!is_valid_name("has space"));
    }
}
```

### Integration Tests

Located in `tests/` directory:

```rust
// tests/cli_test.rs

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_list_command() {
    Command::cargo_bin("scoop")
        .unwrap()
        .args(["list"])
        .assert()
        .success();
}

#[test]
fn test_invalid_command() {
    Command::cargo_bin("scoop")
        .unwrap()
        .args(["invalid"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("error"));
}
```

### Running Tests

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_valid_name

# Run tests in specific module
cargo test validate::tests

# Run ignored tests
cargo test -- --ignored

# Run tests with all features
cargo test --all-features
```

### Test Environment

Tests that modify `SCOOP_HOME` use isolation:

```rust
use crate::test_utils::with_temp_scoop_home;

#[test]
fn test_with_isolated_home() {
    with_temp_scoop_home(|temp_dir| {
        // temp_dir is isolated SCOOP_HOME
        // Safe to create/delete environments
    });
}
```

---

## Project Structure

```
scoop-uv/
├── src/
│   ├── main.rs              # Entry point
│   ├── lib.rs               # Library root
│   ├── error.rs             # ScoopError enum
│   ├── paths.rs             # Path utilities
│   ├── validate.rs          # Validation logic
│   │
│   ├── cli/                 # CLI layer
│   │   ├── mod.rs           # Cli struct, Commands
│   │   └── commands/        # Subcommand handlers
│   │
│   ├── core/                # Business logic
│   │   ├── virtualenv.rs    # VirtualenvService
│   │   ├── version.rs       # VersionService
│   │   └── metadata.rs      # Metadata structs
│   │
│   ├── shell/               # Shell integration
│   │   ├── bash.rs          # Bash init script
│   │   └── zsh.rs           # Zsh init script
│   │
│   ├── uv/                  # uv wrapper
│   │   └── client.rs        # UvClient
│   │
│   └── output/              # Terminal output
│       └── spinner.rs       # Progress spinner
│
├── tests/                   # Integration tests
├── docs/                    # Public documentation
└── .docs/                   # Internal docs (git-ignored)
```

---

## Troubleshooting MSRV Issues

Common MSRV-related problems and their solutions:

### "CI MSRV job fails but stable passes"

**Symptom**: MSRV job fails with compilation errors, but stable test job passes.

**Cause**: Code uses Rust features newer than MSRV (1.85).

**Solution**:
```bash
# Test locally with MSRV
cargo +1.85 clippy --all-targets --all-features -- -D warnings
cargo +1.85 build --all-features

# Check which feature is problematic
rustc +1.85 --version  # Verify you're on 1.85
cargo +1.85 check 2>&1 | grep "error"

# Options:
# A) Rewrite code to work on 1.85
# B) Bump MSRV if feature is essential (follow bump guide above)
```

---

### "cargo-msrv verify fails"

**Symptom**: `cargo msrv verify` exits with error.

**Cause 1**: Cargo.toml rust-version doesn't match actual minimum version.

**Solution**:
```bash
# Find true minimum
cargo msrv find  # Takes 10-20 minutes

# Update Cargo.toml to match
# Edit: rust-version = "<found-version>"

# Verify
cargo msrv verify
```

**Cause 2**: Dependency MSRV increased beyond declared MSRV.

**Solution**:
```bash
# Check dependency MSRVs
cargo tree --duplicates

# Options:
# A) Bump MSRV to match dependency requirement
# B) Find alternative dependency with lower MSRV
# C) Pin dependency to older version (not recommended)
```

---

### "Local Rust version doesn't match MSRV"

**Symptom**: `rustc --version` shows wrong version in project directory.

**Cause**: rust-toolchain.toml not being read, or rustup override set.

**Solution**:
```bash
# Check what's overriding
rustup show

# Should see:
# active toolchain: 1.85-aarch64-apple-darwin
# active because: overridden by '.../rust-toolchain.toml'

# If you see "rustup override":
rustup override unset  # Clear manual override

# If rust-toolchain.toml not working:
cat rust-toolchain.toml  # Verify it exists and is correct
rustup update 1.85       # Ensure 1.85 is installed
```

---

### "MSRV versions out of sync"

**Symptom**: Cargo.toml, rust-toolchain.toml, and ci.yml have different MSRV values.

**Cause**: Forgot to update all three locations when bumping MSRV.

**Solution**:
```bash
# Quick check for sync
grep -E "rust-version|channel|@1\." Cargo.toml rust-toolchain.toml .github/workflows/ci.yml

# Should all show same version (e.g., 1.85)

# Fix each file:
# - Cargo.toml:           rust-version = "1.86"
# - rust-toolchain.toml:  channel = "1.86"
# - ci.yml:               dtolnay/rust-toolchain@1.86

# Verify sync
cargo msrv verify
```

**Prevention**: Follow the [MSRV bump guide](#bumping-msrv-step-by-step-guide) which updates all three.

---

### "CI takes too long after MSRV changes"

**Symptom**: CI runs for 10+ minutes after adding MSRV testing.

**Cause**: Rust cache not effective, or cargo-msrv installing on every run.

**Solution**:
```bash
# Check if cache is working
# In GitHub Actions logs, look for:
# "Cache restored from key: ..."

# If cache keeps missing:
# 1. Check cache key hasn't changed unintentionally
# 2. Verify Cargo.lock is committed (it should be)
# 3. Check if cache eviction policy is hitting limits

# For cargo-msrv specifically:
# - Should see "Cache restored" for cargo-msrv binary
# - If not, check cache key matches in msrv-check.yml
```

**Expected CI times**:
- MSRV job: ~5-6 minutes (first run), ~2-3 minutes (cached)
- msrv-verify job: ~3-4 minutes (first run), ~30 seconds (cached)

---

### "Can I use features from Rust > 1.85?"

**Answer**: Only if you bump the MSRV.

**Process**:
1. Check feature availability: https://doc.rust-lang.org/stable/releases.html
2. Evaluate if feature justifies MSRV bump
3. Follow [MSRV bump guide](#bumping-msrv-step-by-step-guide)
4. Update all documentation

**Quick Reference**: Since our MSRV is Rust 1.85+ (Edition 2024), you have access to:
- ✅ Async-await (since 1.39)
- ✅ Const generics (since 1.51)
- ✅ Let-else statements (since 1.65)
- ✅ GATs - Generic Associated Types (since 1.67)
- ✅ RPIT in traits - return impl Trait (since 1.75)
- ✅ Edition 2024 syntax: `gen` keyword reservation, unsafe extern blocks

---

## Getting Help

Stuck? Here's how to get unstuck:

- 💬 [GitHub Discussions](https://github.com/ai-screams/scoop-uv/discussions) - Ask questions
- 🐛 [Issues](https://github.com/ai-screams/scoop-uv/issues) - Report bugs
- 📖 [Documentation](https://github.com/ai-screams/scoop-uv#readme) - Read the docs

---

## Thank You! 🙏

Every contribution, no matter how small, makes scoop better. Whether it's a typo fix or a major feature, we appreciate your time and effort.

> *"A scoop shared is a scoop enjoyed twice!"* 🍨

Happy coding!
