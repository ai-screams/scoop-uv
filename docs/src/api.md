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
    pub python_version: Option<String>,
}
```

**Fields:**
- `name` - Environment name (e.g., `"myproject"`)
- `path` - Absolute path to virtualenv directory
- `python_version` - Python version string if metadata exists (e.g., `Some("3.12.1")`)

**Example:**
```rust
let info = VirtualenvInfo {
    name: "webapp".to_string(),
    path: PathBuf::from("/Users/x/.scoop/virtualenvs/webapp"),
    python_version: Some("3.12.1".to_string()),
};
```

---

#### `VirtualenvService`

Primary service for virtualenv operations.

```rust
pub struct VirtualenvService {
    uv: UvClient,
}

impl VirtualenvService {
    /// Creates a new service with custom uv wrapper
    pub fn new(uv: UvClient) -> Self

    /// Creates a service using system's uv installation
    pub fn auto() -> Result<Self>

    /// Lists all virtualenvs
    pub fn list(&self) -> Result<Vec<VirtualenvInfo>>

    /// Creates a new virtualenv
    pub fn create(&self, name: &str, python_version: &str) -> Result<PathBuf>

    /// Creates a new virtualenv with a specific Python executable
    pub fn create_with_python_path(&self, name: &str, python_version: &str, python_path: &Path) -> Result<PathBuf>

    /// Deletes a virtualenv
    pub fn delete(&self, name: &str) -> Result<()>

    /// Checks if a virtualenv exists
    pub fn exists(&self, name: &str) -> Result<bool>

    /// Gets the path to a virtualenv
    pub fn get_path(&self, name: &str) -> Result<PathBuf>

    /// Reads metadata for a virtualenv (internal use)
    pub fn read_metadata(&self, path: &Path) -> Option<Metadata>
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
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Metadata {
    pub name: String,
    pub python_version: String,
    pub created_at: DateTime<Utc>,   // Timestamp (ISO 8601 when serialized)
    pub created_by: String,          // "scoop X.Y.Z" format
    pub uv_version: Option<String>,  // uv version used
    pub python_path: Option<String>, // Custom Python executable path (if --python-path was used)
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
  "created_by": "scoop 0.6.0",
  "uv_version": "0.1.0"
}
```

> **Note**: `created_at` is `DateTime<Utc>` in Rust but serializes to ISO 8601 string in JSON.

---

### Doctor Module (`core/doctor.rs`)

#### `Check` Trait

Interface for health checks.

```rust
pub trait Check: Send + Sync {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn run(&self) -> Vec<CheckResult>;  // Returns Vec - a check can produce multiple results
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
    Ok,                  // ✅ Check passed
    Warning(String),     // ⚠️  Issue found, but not critical
    Error(String),       // ❌ Critical issue
}
```

---

#### `CheckResult`

Detailed result from a health check.

```rust
pub struct CheckResult {
    pub id: &'static str,
    pub name: &'static str,
    pub status: CheckStatus,
    pub suggestion: Option<String>,
    pub details: Option<String>,
}

impl CheckResult {
    /// Creates an OK result
    pub fn ok(id: &'static str, name: &'static str) -> Self

    /// Creates a warning result
    pub fn warn(id: &'static str, name: &'static str, message: impl Into<String>) -> Self

    /// Creates an error result
    pub fn error(id: &'static str, name: &'static str, message: impl Into<String>) -> Self

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

    fn run(&self) -> Vec<CheckResult> {
        if some_condition() {
            vec![CheckResult::ok(self.id(), self.name())]
        } else {
            vec![CheckResult::error(self.id(), self.name(), "Error message here")
                .with_suggestion("Run: scoop fix-it")
                .with_details("Expected X, found Y")]
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
    pub fn run_and_fix(&self, output: &crate::output::Output) -> Vec<CheckResult>
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
    match &result.status {
        CheckStatus::Error(msg) => eprintln!("❌ {}: {}", result.name, msg),
        CheckStatus::Warning(msg) => println!("⚠️  {}: {}", result.name, msg),
        CheckStatus::Ok => println!("✅ {}", result.name),
    }
}

// Auto-fix issues (requires Output for progress display)
use scoop::output::Output;
let output = Output::new(0, false, false, false);
let fixed_results = doctor.run_and_fix(&output);
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
    PythonInstallFailed { version: String, message: String },
    PythonUninstallFailed { version: String, message: String },
    InvalidPythonVersion { version: String },
    NoPythonVersions { pattern: String },

    // uv errors
    UvNotFound,
    UvCommandFailed { command: String, message: String },

    // Path/IO errors
    PathError(String),                    // Tuple variant
    HomeNotFound,
    Io(#[from] std::io::Error),          // Tuple variant with From
    Json(#[from] serde_json::Error),     // Tuple variant with From

    // Config errors
    VersionFileNotFound { path: PathBuf },
    UnsupportedShell { shell: String },

    // CLI errors
    InvalidArgument { message: String },

    // Migration errors
    PyenvNotFound,
    PyenvEnvNotFound { name: String },
    VenvWrapperEnvNotFound { name: String },
    CondaEnvNotFound { name: String },
    CorruptedEnvironment { name: String, reason: String },
    PackageExtractionFailed { reason: String },
    MigrationFailed { reason: String },
    MigrationNameConflict { name: String, existing: PathBuf },

    // Python path errors
    InvalidPythonPath { path: PathBuf, reason: String },

    // Cascade errors
    CascadeAborted,
}

impl ScoopError {
    /// Returns error code string (e.g., "ENV_NOT_FOUND", "UV_COMMAND_FAILED")
    pub fn code(&self) -> &'static str

    /// Returns user-friendly suggestion (if available)
    pub fn suggestion(&self) -> Option<String>

    /// Returns migration-specific exit code
    pub fn migration_exit_code(&self) -> MigrationExitCode
}
```

**Error Code String Prefixes:**
- `ENV_*` - Environment errors (e.g., `ENV_NOT_FOUND`, `ENV_ALREADY_EXISTS`)
- `PYTHON_*` - Python version errors (e.g., `PYTHON_NOT_INSTALLED`)
- `UV_*` - uv errors (e.g., `UV_NOT_INSTALLED`, `UV_COMMAND_FAILED`)
- `IO_*` - Path/IO errors (e.g., `IO_ERROR`, `PATH_ERROR`)
- `CONFIG_*` - Config errors (e.g., `CONFIG_VERSION_FILE_NOT_FOUND`)
- `SHELL_*` - Shell errors (e.g., `SHELL_NOT_SUPPORTED`)
- `ARG_*` - CLI argument errors (e.g., `ARG_INVALID`)
- `SOURCE_*` - Migration source errors (e.g., `SOURCE_PYENV_NOT_FOUND`)
- `MIGRATE_*` - Migration process errors (e.g., `MIGRATE_FAILED`)
- `UNINSTALL_*` - Uninstall errors (e.g., `UNINSTALL_CASCADE_ABORTED`)

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
        eprintln!("Error code: {}", e.code());
        std::process::exit(1);  // Non-zero exit for error
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

// Note: ShellType is a plain enum without methods
// Shell operations are handled by module-level functions:
// - shell::detect_shell() -> ShellType  (in shell/mod.rs)
// - shell::bash::init_script() -> &'static str
// - shell::zsh::init_script() -> &'static str
// - shell::fish::init_script() -> &'static str
// - shell::powershell::init_script() -> &'static str
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

/// Returns global version file path (~/.scoop/version)
pub fn global_version_file() -> Result<PathBuf>

/// Returns local version file path in the given directory
pub fn local_version_file(dir: &std::path::Path) -> PathBuf
```

---

## Version Resolution (`core/version.rs`)

### `VersionService`

Service for managing version files.

```rust
pub struct VersionService;

impl VersionService {
    /// Set the local version for a directory
    pub fn set_local(dir: &Path, env_name: &str) -> Result<()>

    /// Set the global version
    pub fn set_global(env_name: &str) -> Result<()>

    /// Get the local version for a directory
    pub fn get_local(dir: &Path) -> Option<String>

    /// Get the global version
    pub fn get_global() -> Option<String>

    /// Resolve the version for a directory (local -> parent walk -> global)
    pub fn resolve(dir: &Path) -> Option<String>

    /// Resolve from current directory
    pub fn resolve_current() -> Option<String>

    /// Unset local version (removes .scoop-version)
    pub fn unset_local(dir: &Path) -> Result<()>

    /// Unset global version (removes ~/.scoop/version)
    pub fn unset_global() -> Result<()>
}
```

**Resolution Priority Order:**
1. `SCOOP_VERSION` environment variable (checked at shell hook level, not in VersionService)
2. `.scoop-version` in current directory
3. `.scoop-version` in parent directories (walks up)
4. `~/.scoop/version` (global default)

> **Note**: `.python-version` is not supported.

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

    fn run(&self) -> Vec<CheckResult> {
        // Check logic
        vec![CheckResult::ok(self.id(), self.name())]
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
   - Error codes are string constants (e.g., "ENV_NOT_FOUND", "UV_COMMAND_FAILED")
   - Process exit codes: 0 = success, 1 = failure
   - Path handling via `paths.rs` utilities
   - Shell detection via `shell::detect_shell()` function

4. **Documentation requirements**:
   - All `pub fn` must have doc comments
   - Examples in doctests where applicable
   - Error conditions documented
   - Shell integration changes require cross-shell testing

---

> **Last Updated:** 2026-02-15
> **scoop Version:** 0.7.0
