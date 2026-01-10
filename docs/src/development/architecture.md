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
│   └── virtualenv.rs # Virtualenv entity
├── shell/            # Shell integration
│   ├── mod.rs        # Shell module exports
│   ├── bash.rs       # Bash init script
│   └── zsh.rs        # Zsh init script
├── output/           # Terminal UI and JSON output
├── uv/               # uv CLI wrapper
├── error.rs          # ScoopError enum
├── paths.rs          # Path utilities
└── validate.rs       # Input validation
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

```
User Command
    │
    ▼
CLI Parser (clap)
    │
    ▼
Command Handler (cli/commands/)
    │
    ├──► Core Logic (core/)
    │        │
    │        ▼
    │    UV Wrapper (uv/)
    │
    ▼
Output Formatter (output/)
    │
    ▼
stdout/stderr
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| clap | Argument parsing |
| serde | Serialization |
| thiserror | Error types |
| owo-colors | Terminal colors |
| indicatif | Progress bars |
| dirs | Home directory |
| which | Binary lookup |
