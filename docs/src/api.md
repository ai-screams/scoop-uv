# API Reference

This document provides a reference for scoop's public API, primarily intended for:
- **AI/LLM tools** analyzing or modifying the codebase
- **Contributors** extending scoop's functionality
- **Advanced users** integrating scoop into custom tooling

> **Note**: This is an internal API reference. For CLI usage, see [Commands](commands/README.md).

---

## Core Types

### VirtualEnv Module (`core/virtualenv.rs`)

#### `VirtualenvInfo`

Represents basic information about a virtual environment.

```rust
pub struct VirtualenvInfo {
    pub name: String,
    pub path: PathBuf,
    pub python_version: String,
}
```

**Fields:**
- `name` - Environment name (e.g., `"myproject"`)
- `path` - Absolute path to virtualenv directory
- `python_version` - Python version string (e.g., `"3.12.1"`)

**Example:**
```rust
let info = VirtualenvInfo {
    name: "webapp".to_string(),
    path: PathBuf::from("/Users/x/.scoop/virtualenvs/webapp"),
    python_version: "3.12.1".to_string(),
};
```

---

#### `VirtualenvService`

Primary service for virtualenv operations.

```rust
pub struct VirtualenvService {
    uv: UvWrapper,
}

impl VirtualenvService {
    /// Creates a new service with custom uv wrapper
    pub fn new(uv: UvWrapper) -> Self

    /// Creates a service using system's uv installation
    pub fn auto() -> Result<Self>

    /// Lists all virtualenvs
    pub fn list(&self) -> Result<Vec<VirtualenvInfo>>

    /// Creates a new virtualenv
    pub fn create(&self, name: &str, version: &str) -> Result<()>

    /// Deletes a virtualenv
    pub fn delete(&self, name: &str) -> Result<()>

    /// Checks if a virtualenv exists
    pub fn exists(&self, name: &str) -> Result<bool>

    /// Gets the path to a virtualenv
    pub fn get_path(&self, name: &str) -> Result<PathBuf>

    /// Reads metadata for a virtualenv
    pub fn read_metadata(&self, name: &str) -> Result<Metadata>

    /// Writes metadata for a virtualenv
    pub fn write_metadata(&self, name: &str, metadata: &Metadata) -> Result<()>
}
```

**Common Usage Pattern:**
```rust
// Initialize service
let service = VirtualenvService::auto()?;

// Check if environment exists
if !service.exists("myenv")? {
    // Create with Python 3.12
    service.create("myenv", "3.12")?;
}

// Get environment path
let path = service.get_path("myenv")?;
println!("Environment at: {}", path.display());
```

---

### Metadata Module (`core/metadata.rs`)

#### `Metadata`

Stores JSON metadata for each virtualenv.

```rust
#[derive(Serialize, Deserialize)]
pub struct Metadata {
    pub name: String,
    pub python_version: String,
    pub created_at: String,          // ISO 8601 timestamp
    pub created_by: String,          // "scoop-vX.Y.Z"
    pub uv_version: Option<String>,  // uv version used
}

impl Metadata {
    /// Creates new metadata
    pub fn new(name: String, python_version: String, uv_version: Option<String>) -> Self
}
```

**Storage Location:** `~/.scoop/virtualenvs/<name>/.scoop-metadata.json`

**Example JSON:**
```json
{
  "name": "myproject",
  "python_version": "3.12.1",
  "created_at": "2024-01-15T10:30:00Z",
  "created_by": "scoop-v0.5.1",
  "uv_version": "0.1.0"
}
```

---

### Doctor Module (`core/doctor.rs`)

#### `Check` Trait

Interface for health checks.

```rust
pub trait Check {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn run(&self) -> CheckResult;
}
```

**Implementations:**
- `UvCheck` - Verifies uv is installed
- `HomeCheck` - Checks `SCOOP_HOME` directory
- `VirtualenvCheck` - Validates virtualenvs directory
- `SymlinkCheck` - Checks for broken `.venv` symlinks
- `ShellCheck` - Verifies shell integration
- `VersionCheck` - Validates version files

---

#### `CheckStatus`

Result status for health checks.

```rust
pub enum CheckStatus {
    Ok,      // ✅ Check passed
    Warning, // ⚠️  Issue found, but not critical
    Error,   // ❌ Critical issue
}
```

---

#### `CheckResult`

Detailed result from a health check.

```rust
pub struct CheckResult {
    pub id: String,
    pub name: String,
    pub status: CheckStatus,
    pub suggestion: Option<String>,
    pub details: Option<String>,
}

impl CheckResult {
    /// Creates an OK result
    pub fn ok(id: impl Into<String>, name: impl Into<String>) -> Self

    /// Creates a warning result
    pub fn warn(id: impl Into<String>, name: impl Into<String>) -> Self

    /// Creates an error result
    pub fn error(id: impl Into<String>, name: impl Into<String>) -> Self

    /// Adds a suggestion for fixing the issue
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self

    /// Adds detailed information
    pub fn with_details(mut self, details: impl Into<String>) -> Self

    // Status checks
    pub fn is_ok(&self) -> bool
    pub fn is_warning(&self) -> bool
    pub fn is_error(&self) -> bool
}
```

**Example - Implementing a Custom Check:**
```rust
struct MyCustomCheck;

impl Check for MyCustomCheck {
    fn id(&self) -> &'static str {
        "my_check"
    }

    fn name(&self) -> &'static str {
        "My Custom Check"
    }

    fn run(&self) -> CheckResult {
        if some_condition() {
            CheckResult::ok(self.id(), self.name())
        } else {
            CheckResult::error(self.id(), self.name())
                .with_suggestion("Run: scoop fix-it")
                .with_details("Expected X, found Y")
        }
    }
}
```

---

#### `Doctor`

Orchestrates all health checks.

```rust
pub struct Doctor {
    checks: Vec<Box<dyn Check>>,
}

impl Doctor {
    /// Creates a doctor with default checks
    pub fn new() -> Self

    /// Runs all checks without fixing
    pub fn run_all(&self) -> Vec<CheckResult>

    /// Runs checks and attempts to fix issues
    pub fn run_and_fix(&self) -> Vec<CheckResult>

    /// Attempts to fix a specific issue
    pub fn try_fix(&self, check_id: &str) -> Result<()>

    // Fix methods for specific issues
    fn fix_home(&self) -> Result<()>
    fn fix_symlink(&self) -> Result<()>
}

impl Default for Doctor {
    fn default() -> Self {
        Self::new()
    }
}
```

**Usage:**
```rust
let doctor = Doctor::new();

// Run diagnostics
let results = doctor.run_all();
for result in results {
    match result.status {
        CheckStatus::Error => eprintln!("❌ {}: {}", result.name, result.id),
        CheckStatus::Warning => println!("⚠️  {}: {}", result.name, result.id),
        CheckStatus::Ok => println!("✅ {}", result.name),
    }
}

// Auto-fix issues
let fixed_results = doctor.run_and_fix();
```

---

## Error Handling

### `ScoopError` (`error.rs`)

Primary error type for all scoop operations.

```rust
#[derive(Error, Debug)]
pub enum ScoopError {
    // Virtualenv errors
    VirtualenvNotFound { name: String },
    VirtualenvExists { name: String },
    InvalidEnvName { name: String, reason: String },

    // Python errors
    PythonNotInstalled { version: String },
    PythonInstallFailed { version: String, details: String },
    PythonUninstallFailed { version: String, details: String },
    InvalidPythonVersion { version: String },
    NoPythonVersionsMatching { version: String },

    // uv errors
    UvNotFound,
    UvCommandFailed { command: String, stderr: String },

    // Path/IO errors
    PathError { path: PathBuf, source: std::io::Error },
    HomeNotFound,
    IoError { source: std::io::Error },

    // Config errors
    VersionFileNotFound { path: PathBuf },
    UnsupportedShell { shell: String },

    // Internal errors
    JsonError { source: serde_json::Error },

    // CLI errors
    InvalidArgument { arg: String, reason: String },

    // Migration errors
    SourcePyenvNotFound,
    SourcePyenvEnvNotFound { name: String },
    SourceVenvwrapperEnvNotFound { name: String },
    SourceCondaEnvNotFound { name: String },
    MigrateCorrupted { name: String, details: String },
    MigrateExtractionFailed { name: String, details: String },
    MigrateFailed { name: String, details: String },
    MigrateNameConflict { name: String },
}

impl ScoopError {
    /// Returns error code for exit status
    pub fn code(&self) -> i32

    /// Returns user-friendly suggestion (if available)
    pub fn suggestion(&self) -> Option<String>

    /// Returns migration-specific exit code
    pub fn migration_exit_code(&self) -> Option<MigrationExitCode>
}
```

**Error Code Conventions:**
- `1-9` - Environment errors
- `10-19` - Python version errors
- `20-29` - uv errors
- `30-39` - Path/IO errors
- `40-49` - Config errors
- `50-59` - Internal errors
- `60-69` - CLI argument errors
- `70-79` - Migration source errors
- `80-89` - Migration process errors

**Example Error Handling:**
```rust
use crate::error::{ScoopError, Result};

fn my_function(name: &str) -> Result<()> {
    if !validate_name(name) {
        return Err(ScoopError::InvalidEnvName {
            name: name.to_string(),
            reason: "Must start with a letter".to_string(),
        });
    }

    // ... operation

    Ok(())
}

// Usage
match my_function("123invalid") {
    Ok(_) => println!("Success"),
    Err(e) => {
        eprintln!("Error: {}", e);
        if let Some(suggestion) = e.suggestion() {
            eprintln!("Suggestion: {}", suggestion);
        }
        std::process::exit(e.code());
    }
}
```

---

## Shell Integration

### Shell Types (`cli/mod.rs`)

```rust
pub enum ShellType {
    Bash,
    Zsh,
    Fish,
    Powershell,
}

impl ShellType {
    /// Detects shell from environment
    pub fn detect() -> Result<Self>

    /// Returns init script for this shell
    pub fn init_script(&self) -> String

    /// Returns completion script
    pub fn completion_script(&self) -> String
}
```

**Auto-detection Priority:**
1. `FISH_VERSION` → Fish
2. `PSModulePath` → PowerShell
3. `ZSH_VERSION` → Zsh
4. Default → Bash

---

## Path Utilities (`paths.rs`)

```rust
/// Returns scoop home directory (SCOOP_HOME or ~/.scoop)
pub fn scoop_home() -> Result<PathBuf>

/// Returns virtualenvs directory
pub fn virtualenvs_dir() -> Result<PathBuf>

/// Returns global version file path
pub fn version_file() -> Result<PathBuf>

/// Returns local version file (.scoop-version)
pub fn local_version_file() -> PathBuf

/// Returns pyenv-compatible version file (.python-version)
pub fn python_version_file() -> PathBuf
```

---

## Version Resolution (`core/version.rs`)

**Priority Order:**
1. `SCOOP_VERSION` environment variable
2. `.scoop-version` (current directory)
3. `.python-version` (pyenv compatibility)
4. `~/.scoop/version` (global default)

**Related Functions:**
```rust
pub fn resolve_version() -> Result<String>
pub fn read_version_file(path: &Path) -> Result<String>
pub fn write_version_file(path: &Path, version: &str) -> Result<()>
```

---

## Testing Patterns

### Property-Based Testing

scoop uses `proptest` for property-based testing of critical logic:

```rust
#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn env_name_validation_is_consistent(name in "[a-zA-Z][a-zA-Z0-9_-]*") {
            assert!(validate_name(&name).is_ok());
        }
    }
}
```

### Integration Testing

Test files follow the pattern:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_test_env() -> TempDir {
        // Test setup
    }

    #[test]
    fn test_something() {
        let temp = setup_test_env();
        // Test logic
    }
}
```

---

## Extending scoop

### Adding a New Command

1. **Define command in `cli/mod.rs`:**
```rust
#[derive(Subcommand)]
pub enum Commands {
    // ... existing commands
    MyCommand {
        #[arg(help = "Argument description")]
        arg: String,
    },
}
```

2. **Create handler in `cli/commands/`:**
```rust
// cli/commands/my_command.rs
use crate::error::Result;

pub fn execute(arg: &str) -> Result<()> {
    // Implementation
    Ok(())
}
```

3. **Wire up in `main.rs`:**
```rust
Commands::MyCommand { arg } => {
    commands::my_command::execute(&arg)?
}
```

---

### Adding a New Health Check

```rust
// In core/doctor.rs
struct MyCheck;

impl Check for MyCheck {
    fn id(&self) -> &'static str {
        "my_check"
    }

    fn name(&self) -> &'static str {
        "My Custom Check"
    }

    fn run(&self) -> CheckResult {
        // Check logic
        CheckResult::ok(self.id(), self.name())
    }
}

// Register in Doctor::new()
impl Doctor {
    pub fn new() -> Self {
        Self {
            checks: vec![
                // ... existing checks
                Box::new(MyCheck),
            ],
        }
    }
}
```

---

## Related Documentation

- [Architecture](development/architecture.md) - System design and patterns
- [Commands](commands/README.md) - CLI reference
- [Contributing](development/contributing.md) - Development guide
- [Testing](development/testing.md) - Testing strategies

---

## For AI/LLM Tools

When analyzing or modifying this codebase:

1. **Use symbolic tools** for precise navigation:
   - `find_symbol` to locate specific functions/types
   - `find_referencing_symbols` to understand usage
   - `get_symbols_overview` for module structure

2. **Follow existing patterns**:
   - Error handling: Always return `Result<T>` with `ScoopError`
   - Shell output: CLI outputs shell code, wrapper evals it
   - Testing: Unit tests + integration tests + property tests
   - i18n: Use `t!()` macro for all user-facing strings

3. **Preserve conventions**:
   - Error codes follow the numbering scheme
   - Exit codes: 0 = success, error.code() = failure
   - Path handling via `paths.rs` utilities
   - Shell detection via `ShellType::detect()`

4. **Documentation requirements**:
   - All `pub fn` must have doc comments
   - Examples in doctests where applicable
   - Error conditions documented
   - Shell integration changes require cross-shell testing

---

> **Last Updated:** 2026-02-05
> **scoop Version:** 0.5.1
