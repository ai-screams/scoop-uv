# Testing

Comprehensive guide for testing scoop.

## Quick Reference

```bash
cargo test                          # Run all tests
cargo test json                     # Run tests containing "json"
cargo test -- --nocapture           # Show println! output
cargo clippy -- -D warnings         # Lint check
```

## Test Structure

```
tests/
└── cli.rs                    # CLI integration tests

src/
├── error.rs                  # Unit tests for error types
├── validate.rs               # Unit tests for validation
├── paths.rs                  # Unit tests for path utilities
├── output/
│   └── json.rs               # Unit tests for JSON output
├── core/
│   ├── virtualenv.rs         # Unit tests for virtualenv service
│   ├── version.rs            # Unit tests for version service
│   ├── metadata.rs           # Unit tests for metadata
│   └── doctor.rs             # Unit tests for doctor
├── shell/
│   ├── bash.rs               # Shell script tests
│   └── zsh.rs                # Shell script tests
└── uv/
    └── client.rs             # Unit tests for uv client
```

## Running Tests

### All Tests

```bash
# Run all tests
cargo test

# Run with all features enabled
cargo test --all-features

# Run in release mode (faster execution)
cargo test --release
```

### Filtered Tests

```bash
# By name pattern
cargo test json                     # Tests containing "json"
cargo test error                    # Tests containing "error"
cargo test virtualenv               # Tests containing "virtualenv"

# By module path
cargo test output::json             # Tests in output/json.rs
cargo test error::tests             # Tests in error.rs
cargo test core::version            # Tests in core/version.rs
cargo test cli::commands            # Tests in cli/commands/

# Single test
cargo test test_json_response_success_creates_correct_status
```

### Test Output

```bash
# Show stdout/stderr (println!, dbg!, etc.)
cargo test -- --nocapture

# Show test names as they run
cargo test -- --nocapture --test-threads=1

# Only show failed tests
cargo test -- --quiet
```

### Debugging

```bash
# Run single-threaded (easier to debug)
cargo test -- --test-threads=1

# Run ignored tests
cargo test -- --ignored

# Run specific test with output
cargo test test_name -- --nocapture --test-threads=1
```

## Test Categories

### Unit Tests (520 tests)

Located within source files using `#[cfg(test)]`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something() {
        assert_eq!(1 + 1, 2);
    }
}
```

Key test modules:

| Module                    | Tests | Coverage                        |
|---------------------------|-------|---------------------------------|
| `error::tests`            | 54    | Error types, codes, suggestions |
| `output::json::tests`     | 35    | JSON serialization, edge cases  |
| `validate::tests`         | 30    | Name/version validation         |
| `core::version::tests`    | 18    | Version file resolution         |
| `core::virtualenv::tests` | 12    | Virtualenv service              |
| `paths::tests`            | 16    | Path utilities                  |
| `shell::*::tests`         | 14    | Shell scripts (shellcheck)      |

### Integration Tests (41 tests)

Located in `tests/cli.rs`:

```bash
# Run only integration tests
cargo test --test cli
```

Categories:

- **Error cases** - Invalid inputs, missing arguments
- **Output format** - Help, version, JSON output
- **Command behavior** - list, create, use, remove

Some tests are marked `#[ignore]` because they require `uv` installed:

```bash
# Run ignored tests (requires uv)
cargo test -- --ignored
```

### Doc Tests (6 tests)

Examples in documentation comments:

```rust
/// Validates environment name.
///
/// # Examples
///
/// ```
/// use scoop_uv::validate::is_valid_env_name;
/// assert!(is_valid_env_name("myenv"));
/// assert!(!is_valid_env_name("123bad"));
/// ```
pub fn is_valid_env_name(name: &str) -> bool { ... }
```

```bash
# Run only doc tests
cargo test --doc
```

### Property Tests

Using `proptest` for randomized testing:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_valid_names_accepted(name in "[a-zA-Z][a-zA-Z0-9_-]{0,49}") {
        assert!(is_valid_env_name(&name));
    }
}
```

Located in `src/validate.rs`.

## Writing Tests

### Unit Test Template

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // ========================================
    // Test Group Name
    // ========================================

    #[test]
    fn test_function_name_expected_behavior() {
        // Arrange
        let input = "test input";

        // Act
        let result = function_under_test(input);

        // Assert
        assert_eq!(result, expected_value);
    }

    #[test]
    fn test_function_name_edge_case() {
        let result = function_under_test("");
        assert!(result.is_err());
    }
}
```

### Integration Test Template

```rust
// tests/cli.rs
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_command_success() {
    Command::cargo_bin("scoop")
        .unwrap()
        .args(["list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("expected output"));
}

#[test]
fn test_command_failure() {
    Command::cargo_bin("scoop")
        .unwrap()
        .args(["use", "nonexistent"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}
```

### JSON Output Testing

```rust
#[test]
fn test_json_serialization() {
    let data = MyData { field: "value".into() };
    let json = serde_json::to_string(&data).unwrap();

    // Check JSON structure
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["field"], "value");
}

#[test]
fn test_optional_field_omitted() {
    let data = MyData { optional: None, .. };
    let json = serde_json::to_string(&data).unwrap();

    // skip_serializing_if = "Option::is_none"
    assert!(!json.contains("optional"));
}
```

## Test Utilities

Located in `src/test_utils.rs`:

```rust
use scoop_uv::test_utils::*;

#[test]
fn test_with_temp_environment() {
    with_temp_scoop_home(|temp_dir| {
        // SCOOP_HOME is set to temp_dir
        // Cleanup happens automatically
    });
}

#[test]
fn test_with_mock_venv() {
    with_temp_scoop_home(|temp_dir| {
        create_mock_venv("myenv", Some("3.12"));
        // Virtual environment created at temp_dir/virtualenvs/myenv
    });
}
```

## Coverage

### Using cargo-tarpaulin

```bash
# Install
cargo install cargo-tarpaulin

# Run with HTML report
cargo tarpaulin --out Html --output-dir coverage

# Run with specific target
cargo tarpaulin --out Html --output-dir coverage --packages scoop-uv

# View report
open coverage/tarpaulin-report.html
```

### Using cargo-llvm-cov

```bash
# Install
cargo install cargo-llvm-cov

# Run with HTML report
cargo llvm-cov --html

# View report
open target/llvm-cov/html/index.html
```

## CI/CD Testing

Tests run automatically on:

- Every push to any branch
- Every pull request

GitHub Actions workflow (`.github/workflows/ci.yml`):

```yaml
- name: Run tests
  run: cargo test --all-features

- name: Run clippy
  run: cargo clippy --all-targets -- -D warnings
```

## Troubleshooting

### Test Hangs

```bash
# Run single-threaded to identify hanging test
cargo test -- --test-threads=1
```

### Flaky Tests

```bash
# Run specific test multiple times
for i in {1..10}; do cargo test test_name || break; done
```

### Environment Issues

```bash
# Clear test artifacts
cargo clean

# Rebuild and test
cargo test
```

### Shell Tests Fail

ShellCheck must be installed for shell script tests:

```bash
# macOS
brew install shellcheck

# Linux
apt install shellcheck
```

## Best Practices

1. **Test naming**: `test_<function>_<scenario>_<expected>`
2. **Arrange-Act-Assert**: Clear test structure
3. **One assertion per test**: When practical
4. **Test edge cases**: Empty, unicode, special chars, boundaries
5. **No test interdependencies**: Each test should be isolated
6. **Fast tests**: Mock external dependencies
