# Command Reference

Complete reference for all scoop commands.

---

## Virtual Environment Commands

### `scoop create`

Create a new virtual environment.

```bash
scoop create <name> [python-version]
```

**Arguments:**
| Argument | Required | Default | Description |
|----------|----------|---------|-------------|
| `name` | Yes | - | Name for the virtual environment |
| `python-version` | No | `3` (latest) | Python version (e.g., `3.12`, `3.11.8`) |

**Options:**
| Option | Description |
|--------|-------------|
| `--force`, `-f` | Overwrite existing environment |

**Examples:**
```bash
scoop create myproject 3.12      # Create with Python 3.12
scoop create webapp              # Create with latest Python
scoop create myenv 3.11 --force  # Overwrite if exists
```

---

### `scoop use`

Set the active environment for the current directory.

```bash
scoop use <name> [options]
```

**Arguments:**
| Argument | Required | Description |
|----------|----------|-------------|
| `name` | Yes | Name of the virtual environment |

**Options:**
| Option | Description |
|--------|-------------|
| `--global`, `-g` | Set as global default (all directories) |
| `--link` | Create `.venv` symlink for IDE compatibility |
| `--no-link` | Do not create `.venv` symlink (default) |

**Behavior:**
- Creates `.scoop-version` file in current directory
- Immediately activates the environment (if shell hook installed)
- With `--global`: writes to `~/.scoop/version`
- With `--link`: creates `.venv -> ~/.scoop/virtualenvs/<name>`

**Examples:**
```bash
scoop use myproject              # Set local environment
scoop use myproject --link       # Also create .venv symlink
scoop use myproject --global     # Set as global default
```

---

### `scoop list`

List virtual environments or Python versions.

```bash
scoop list [options]
```

**Options:**
| Option | Description |
|--------|-------------|
| `--pythons` | Show installed Python versions instead |
| `--bare` | Output names only (for scripting) |

**Examples:**
```bash
scoop list                  # List all environments
scoop list --pythons        # List installed Python versions
scoop list --bare           # Names only, one per line
```

---

### `scoop remove`

Delete a virtual environment.

```bash
scoop remove <name> [options]
```

**Aliases:** `rm`, `delete`

**Arguments:**
| Argument | Required | Description |
|----------|----------|-------------|
| `name` | Yes | Name of the environment to delete |

**Options:**
| Option | Description |
|--------|-------------|
| `--force`, `-f` | Skip confirmation prompt |

**Examples:**
```bash
scoop remove myproject           # Delete with confirmation
scoop remove myproject --force   # Delete without confirmation
scoop rm old-env -f              # Using alias
```

---

## Python Version Commands

### `scoop install`

Install a Python version.

```bash
scoop install [version] [options]
```

**Arguments:**
| Argument | Required | Default | Description |
|----------|----------|---------|-------------|
| `version` | No | latest | Python version (e.g., `3.12`, `3.11.8`) |

**Options:**
| Option | Description |
|--------|-------------|
| `--latest` | Install latest stable Python (default) |
| `--stable` | Install oldest fully-supported Python (3.10) |

**Version Resolution:**
- No argument or `--latest`: installs latest Python 3.x
- `--stable`: installs Python 3.10 (oldest with active security support)
- `3.12`: installs latest 3.12.x patch
- `3.12.3`: installs exact version

**Examples:**
```bash
scoop install                    # Install latest Python
scoop install --latest           # Same as above (explicit)
scoop install --stable           # Install Python 3.10
scoop install 3.12               # Install latest 3.12.x
scoop install 3.12.3             # Install exact 3.12.3
```

> **Note:** Python versions are managed by [uv](https://github.com/astral-sh/uv).

---

### `scoop uninstall`

Remove an installed Python version.

```bash
scoop uninstall <version>
```

**Arguments:**
| Argument | Required | Description |
|----------|----------|-------------|
| `version` | Yes | Python version to remove |

**Examples:**
```bash
scoop uninstall 3.12             # Remove Python 3.12
scoop uninstall 3.11.8           # Remove specific version
```

---

## Shell Integration Commands

### `scoop init`

Output shell initialization script.

```bash
scoop init <shell>
```

**Arguments:**
| Argument | Required | Description |
|----------|----------|-------------|
| `shell` | Yes | Shell type: `bash`, `zsh`, `fish`, `powershell` |

**Usage:**
Add to your shell configuration:

```bash
# Bash (~/.bashrc)
eval "$(scoop init bash)"

# Zsh (~/.zshrc)
eval "$(scoop init zsh)"
```

**Features enabled:**
- Auto-activation when entering directories with `.scoop-version`
- Tab completion for commands, environments, and options
- Wrapper function for `activate`/`deactivate`/`use`

---

### `scoop activate` (internal)

Activate a virtual environment. Outputs shell script for `eval`.

```bash
eval "$(scoop activate <name>)"
```

> This command is typically called automatically by the shell wrapper.

---

### `scoop deactivate` (internal)

Deactivate the current environment. Outputs shell script for `eval`.

```bash
eval "$(scoop deactivate)"
```

> This command is typically called automatically by the shell wrapper.

---

## Global Options

Available for all commands:

| Option | Description |
|--------|-------------|
| `-v`, `--verbose` | Increase output verbosity (can repeat: `-vv`) |
| `-q`, `--quiet` | Suppress all output |
| `--no-color` | Disable colored output |
| `--json` | Output as JSON (where supported) |
| `-h`, `--help` | Show help message |
| `-V`, `--version` | Show version |

---

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `SCOOP_HOME` | Base directory for scoop data | `~/.scoop` |
| `SCOOP_NO_AUTO` | Disable auto-activation | (unset) |
| `NO_COLOR` | Disable colored output | (unset) |

---

## File Locations

| File | Purpose |
|------|---------|
| `~/.scoop/virtualenvs/` | Virtual environments storage |
| `~/.scoop/version` | Global default environment |
| `.scoop-version` | Local environment (per-directory) |
| `.venv` | Symlink to active environment (with `--link`) |

---

## Shell Completion

Tab completion is enabled via `scoop init`:

```bash
scoop <TAB>           # List subcommands
scoop use <TAB>       # List environments
scoop use --<TAB>     # List options
scoop install --<TAB> # List --latest, --stable
scoop create name <TAB>  # List Python versions
```
