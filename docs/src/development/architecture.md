# Architecture

scoop is built in Rust using a modular architecture.

## Module Structure

```
src/
├── cli/              # Command-line interface
│   ├── mod.rs        # Cli struct, Commands enum, ShellType
│   └── commands/     # Individual command handlers
├── core/             # Domain logic
│   ├── doctor.rs     # Health diagnostics
│   ├── metadata.rs   # Virtualenv metadata (JSON)
│   ├── version.rs    # Version file resolution
│   ├── virtualenv.rs # Virtualenv entity
│   ├── doctor.rs     # Health check system
│   └── migrate/      # Migration from pyenv/conda/virtualenvwrapper
│       ├── mod.rs
│       ├── discovery.rs  # Source detection
│       ├── migrator.rs   # Migration orchestrator
│       └── ...
├── shell/            # Shell integration
│   ├── mod.rs        # Shell module exports & detection
│   ├── common.rs     # Shared shell utilities & macros
│   ├── bash.rs       # Bash init script
│   ├── zsh.rs        # Zsh init script
│   ├── fish.rs       # Fish init script
│   └── powershell.rs # PowerShell init script
├── output/           # Terminal UI and JSON output
├── uv/               # uv CLI wrapper
├── error.rs          # ScoopError enum
├── paths.rs          # Path utilities
├── validate.rs       # Input validation
├── i18n.rs           # Internationalization
└── config.rs         # Configuration management
```

### Module Dependency Graph

```mermaid
graph TB
    CLI[cli/] --> Core[core/]
    CLI --> Shell[shell/]
    CLI --> Output[output/]
    CLI --> I18N[i18n]
    CLI --> Config[config]

    Core --> UV[uv/]
    Core --> Paths[paths]
    Core --> Error[error]
    Core --> Validate[validate]
    Core --> Config

    Shell --> Paths

    Output --> Error
    Output --> I18N

    UV --> Error

    Config --> Paths
    Config --> Error

    style CLI fill:#e1f5ff
    style Core fill:#fff3e0
    style Shell fill:#f3e5f5
    style Output fill:#e8f5e9
```

## Key Components

### CLI Layer (`cli/`)

- Uses [clap](https://docs.rs/clap) for argument parsing
- `Cli` struct defines global options
- `Commands` enum defines subcommands
- Each command has an `execute` function in `commands/`

### Core Layer (`core/`)

| Module | Purpose |
|--------|---------|
| `doctor` | Health check system with `Check` trait |
| `metadata` | JSON metadata for virtualenvs |
| `version` | Version file discovery and parsing |
| `virtualenv` | Virtualenv entity and operations |

### Shell Layer (`shell/`)

Generates shell scripts for integration:

- `init_script()` - Returns shell initialization code
- Wrapper function for `scoop` command
- Auto-activation hooks
- Tab completion definitions

### Output Layer (`output/`)

Handles terminal output formatting:

- Colored output using [owo-colors](https://docs.rs/owo-colors)
- JSON output for scripting
- Progress indicators with [indicatif](https://docs.rs/indicatif)

### UV Layer (`uv/`)

Wraps the uv CLI for Python/virtualenv operations:

- Python installation
- Virtualenv creation
- Version listing

## Design Patterns

### Shell Eval Pattern

The CLI outputs shell code to stdout, which the shell evaluates:

```bash
# User runs
scoop activate myenv

# CLI outputs
export VIRTUAL_ENV="/Users/x/.scoop/virtualenvs/myenv"
export PATH="/Users/x/.scoop/virtualenvs/myenv/bin:$PATH"
export SCOOP_ACTIVE="myenv"

# Shell wrapper evaluates this output
eval "$(command scoop activate myenv)"
```

This pattern is used by pyenv, rbenv, and other version managers.

### Error Handling

Uses [thiserror](https://docs.rs/thiserror) for error types:

```rust
#[derive(Debug, Error)]
pub enum ScoopError {
    #[error("가상환경 '{name}'을(를) 찾을 수 없습니다")]
    VirtualenvNotFound { name: String },
    // ...
}
```

### Path Management

Centralizes path logic in `paths.rs`:

- `scoop_home()` - Returns `SCOOP_HOME` or `~/.scoop`
- `virtualenvs_dir()` - Returns virtualenvs directory
- `version_file()` - Returns global version file path

## Data Flow

### Command Execution Flow

```mermaid
sequenceDiagram
    participant User
    participant CLI as CLI Parser
    participant Cmd as Command Handler
    participant Core as Core Logic
    participant UV as UV Wrapper
    participant Output as Output Formatter

    User->>CLI: scoop create myenv 3.12
    CLI->>Cmd: parse & dispatch
    Cmd->>Core: VirtualenvService::create()
    Core->>UV: uv venv create
    UV-->>Core: success/error
    Core-->>Cmd: Result<()>
    Cmd->>Output: format response
    Output-->>User: stdout/stderr
```

### Shell Integration Flow

```mermaid
sequenceDiagram
    participant User
    participant Shell as Shell Wrapper
    participant CLI as scoop CLI
    participant Core as Core Logic

    User->>Shell: scoop use myenv
    Shell->>CLI: command scoop use myenv
    CLI->>Core: resolve version & path
    Core-->>CLI: env vars to export
    CLI-->>Shell: echo shell script
    Shell->>Shell: eval output
    Shell-->>User: (myenv) $
    Note over User: Environment activated
```

### Version Resolution Flow

```mermaid
graph LR
    Start([User runs command]) --> Env{SCOOP_VERSION<br/>env set?<br/><small>shell hook</small>}
    Env -->|Yes| Use[Use env value]
    Env -->|No| Local{.scoop-version<br/>in current/parent<br/>dirs?}
    Local -->|Yes| Use
    Local -->|No| Global{~/.scoop/version<br/>exists?}
    Global -->|Yes| Use
    Global -->|No| None[No version<br/>system Python]

    style Use fill:#c8e6c9
    style None fill:#fff9c4
```

> **Note**: `.python-version` is not currently supported. Version resolution walks up parent directories to find `.scoop-version`.

### Health Check Flow

```mermaid
flowchart TD
    Start([scoop doctor]) --> Init[Initialize Doctor]
    Init --> Run[Run all checks]

    Run --> UV{UV Check}
    UV -->|Pass| Home{Home Check}
    UV -->|Fail| Fix1[Suggest: install uv]

    Home -->|Pass| Venv{Venv Check}
    Home -->|Fail| Fix2[Auto-fix: mkdir]

    Venv -->|Pass| Link{Symlink Check}
    Venv -->|Warn| Warn1[Warn: corrupted env]

    Link -->|Pass| Shell{Shell Check}
    Link -->|Fail| Fix3[Auto-fix: remove broken links]

    Shell -->|Pass| Ver{Version Check}
    Shell -->|Warn| Warn2[Warn: not initialized]

    Ver -->|Pass| Done([All checks passed])
    Ver -->|Warn| Warn3[Warn: invalid version file]

    Fix1 --> Report[Generate Report]
    Fix2 --> Report
    Fix3 --> Report
    Warn1 --> Report
    Warn2 --> Report
    Warn3 --> Report
    Done --> Report

    style Done fill:#c8e6c9
    style Fix1 fill:#ffcdd2
    style Fix2 fill:#ffcdd2
    style Fix3 fill:#ffcdd2
    style Warn1 fill:#fff9c4
    style Warn2 fill:#fff9c4
    style Warn3 fill:#fff9c4
```

## Migration Architecture

scoop supports migrating environments from pyenv-virtualenv, virtualenvwrapper, and conda.

```mermaid
graph TD
    Start([scoop migrate]) --> Detect[Detect Sources]

    Detect --> Pyenv{pyenv-virtualenv}
    Detect --> Venv{virtualenvwrapper}
    Detect --> Conda{conda}

    Pyenv -->|Found| P1[List ~/.pyenv/versions]
    Venv -->|Found| V1[List $WORKON_HOME]
    Conda -->|Found| C1[conda env list]

    P1 --> Parse[Parse metadata]
    V1 --> Parse
    C1 --> Parse

    Parse --> Create[Create in scoop]
    Create --> Copy[Copy packages]
    Copy --> Meta[Write metadata]
    Meta --> Done([Migration complete])

    Pyenv -->|Not found| Skip1[Skip]
    Venv -->|Not found| Skip2[Skip]
    Conda -->|Not found| Skip3[Skip]

    Skip1 --> Check{Any source found?}
    Skip2 --> Check
    Skip3 --> Check

    Check -->|Yes| Done
    Check -->|No| Error[Error: No sources]

    style Done fill:#c8e6c9
    style Error fill:#ffcdd2
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| clap | Argument parsing & completion |
| clap_complete | Shell completion generation |
| serde | JSON serialization |
| serde_json | Metadata persistence |
| thiserror | Error type definitions |
| owo-colors | Terminal colors |
| indicatif | Progress bars & spinners |
| dialoguer | Interactive prompts |
| dirs | Home directory resolution |
| which | Binary lookup (uv, python) |
| regex | Version parsing & validation |
| walkdir | Directory traversal |
| rust-i18n | Internationalization (en, ko, ja, pt-BR) |
| sys-locale | System locale detection |
| chrono | Timestamp generation |

## Extension Points

### Adding a New Shell

1. Create `shell/myshell.rs`:
```rust
pub fn init_script() -> String {
    // Return shell-specific initialization code
}

pub fn completion_script() -> String {
    // Return completion definitions
}
```

2. Add to `ShellType` enum in `cli/mod.rs`:
```rust
pub enum ShellType {
    // ... existing
    MyShell,
}
```

3. Implement detection in `ShellType::detect()`:
```rust
if env::var("MYSHELL_VERSION").is_ok() {
    return Ok(ShellType::MyShell);
}
```

### Adding a New Migration Source

1. Create `core/migrate/mysource.rs`:
```rust
pub struct MySource;

impl MySource {
    pub fn detect() -> bool {
        // Check if source is available
    }

    pub fn list_envs() -> Result<Vec<MigrationCandidate>> {
        // Return list of environments
    }

    pub fn migrate_env(name: &str) -> Result<()> {
        // Perform migration
    }
}
```

2. Register in `cli/commands/migrate.rs`:
```rust
let sources = vec![
    // ... existing
    Box::new(MySource),
];
```

### Adding a New Doctor Check

See [API Reference - Adding a New Health Check](../api.md#adding-a-new-health-check) for details.

## Performance Characteristics

| Operation | Time Complexity | Notes |
|-----------|----------------|-------|
| List envs | O(n) | n = number of virtualenvs |
| Create env | O(1)* | *Depends on uv performance |
| Delete env | O(1) | Simple directory removal |
| Version resolution | O(1) | File reads, no recursion |
| Doctor checks | O(n) | n = number of checks (fixed) |

## Thread Safety

scoop is a single-threaded CLI application. No concurrent operations are performed.

**File locking:** Not implemented. Assumes single user on single machine. Concurrent operations (e.g., two terminals creating the same env) may result in race conditions.

## Security Considerations

1. **Path Traversal:** All user-provided names are validated via regex before use in filesystem operations.
2. **Command Injection:** uv commands are constructed using typed arguments, not string concatenation.
3. **Symlink Safety:** Doctor checks detect and warn about broken symlinks.
4. **Metadata Integrity:** JSON parsing errors are gracefully handled without panics.

## Related Documentation

- [API Reference](../api.md) - Detailed API documentation
- [Testing](testing.md) - Testing strategies
- [Contributing](contributing.md) - Development guide
