# Command Reference 🍨

> *"Every flavor has a recipe. Here's the menu."*

Complete reference for all scoop commands.

---

## Flavor Creation 🍦

### `scoop create`

Mix a new flavor (virtual environment).

```bash
scoop create <name> [python-version]
```

**Arguments:**
| Argument | Required | Default | Description |
|----------|----------|---------|-------------|
| `name` | Yes | - | Name for your new flavor |
| `python-version` | No | `3` (latest) | Python version (e.g., `3.12`, `3.11.8`) |

**Options:**
| Option | Description |
|--------|-------------|
| `--force`, `-f` | Overwrite existing flavor |

**Examples:**

```bash
scoop create myproject 3.12      # Mix with Python 3.12
scoop create webapp              # Mix with latest Python
scoop create myenv 3.11 --force  # Overwrite if exists
```

---

### `scoop use`

Pick a flavor for the current directory (auto-activates! 🎉).

```bash
scoop use <name> [options]
```

**Arguments:**
| Argument | Required | Description |
|----------|----------|-------------|
| `name` | Yes | Name of the flavor to pick |

**Options:**
| Option | Description |
|--------|-------------|
| `--global`, `-g` | Set as your usual order (global default) |
| `--link` | Create `.venv` symlink for IDE compatibility |
| `--no-link` | Do not create `.venv` symlink (default) |

**Behavior:**

- Creates `.scoop-version` file in current directory
- Immediately activates the environment (if shell hook installed)
- With `--global`: writes to `~/.scoop/version` (your usual order)
- With `--link`: creates `.venv -> ~/.scoop/virtualenvs/<name>`

**Examples:**

```bash
scoop use myproject              # Pick this flavor here
scoop use myproject --link       # Also create .venv symlink
scoop use myproject --global     # Set as your usual order
```

---

### `scoop list`

What's in the freezer? 🧊

```bash
scoop list [options]
```

**Options:**
| Option | Description |
|--------|-------------|
| `--pythons` | Show Python versions instead of flavors |
| `--bare` | Output names only (for scripting) |
| `--json` | Output as JSON (for the data nerds 🤓) |

**Examples:**

```bash
scoop list                  # List all flavors
scoop list --pythons        # List Python versions in stock
scoop list --bare           # Names only, one per line
scoop list --json           # JSON output
```

---

### `scoop remove`

Melt a flavor away. 💧

```bash
scoop remove <name> [options]
```

**Aliases:** `rm`, `delete`

**Arguments:**
| Argument | Required | Description |
|----------|----------|-------------|
| `name` | Yes | Name of the flavor to melt |

**Options:**
| Option | Description |
|--------|-------------|
| `--force`, `-f` | Skip confirmation prompt |

**Examples:**

```bash
scoop remove myproject           # Melt with confirmation
scoop remove myproject --force   # Melt without asking
scoop rm old-env -f              # Using alias
```

---

## Stocking the Freezer 🧊

### `scoop install`

Stock up on Python versions.

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
scoop install                    # Stock up latest
scoop install --latest           # Same as above
scoop install --stable           # Get Python 3.10
scoop install 3.12               # Get latest 3.12.x
scoop install 3.12.3             # Get exact 3.12.3
```

> **Note:** Python versions are managed by [uv](https://github.com/astral-sh/uv) — the secret ingredient. 🔮

---

### `scoop uninstall`

Remove a Python version from stock.

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

## Health Check 🩺

### `scoop doctor`

Is everything fresh? Check your scoop setup!

```bash
scoop doctor [options]
```

**Options:**
| Option | Description |
|--------|-------------|
| `-v`, `--verbose` | Show more details (can repeat: `-vv`) |
| `--json` | Output diagnostics as JSON |
| `--fix` | Auto-fix issues where possible |

**Checks performed:**

- uv installation and version
- Shell integration status
- Environment integrity
- Path configuration
- Version file validity

**Examples:**

```bash
scoop doctor                     # Quick health check
scoop doctor -v                  # Verbose diagnostics
scoop doctor --fix               # Fix what can be fixed
scoop doctor --json              # JSON output for scripting
```

---

## Shell Integration 🐚

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

- 🎯 Auto-activation when entering directories with `.scoop-version`
- ⌨️ Tab completion for commands, environments, and options
- 🔄 Wrapper function for `activate`/`deactivate`/`use`

---

### `scoop completions`

Generate shell completion script.

```bash
scoop completions <shell>
```

**Arguments:**
| Argument | Required | Description |
|----------|----------|-------------|
| `shell` | Yes | Shell type: `bash`, `zsh`, `fish`, `powershell` |

**Examples:**

```bash
scoop completions bash           # Output bash completions
scoop completions zsh            # Output zsh completions
```

> **Tip:** Usually you don't need this — `scoop init` includes completions! 🍨

---

### `scoop activate` (internal)

Activate a flavor. Outputs shell script for `eval`.

```bash
eval "$(scoop activate <name>)"
```

> This command is typically called automatically by the shell wrapper.

---

### `scoop deactivate` (internal)

Deactivate the current flavor. Outputs shell script for `eval`.

```bash
eval "$(scoop deactivate)"
```

> This command is typically called automatically by the shell wrapper.

---

## Global Options 🎛️

Available for all commands:

| Option            | Description            |
|-------------------|------------------------|
| `-q`, `--quiet`   | Suppress all output    |
| `--no-color`      | Disable colored output |
| `-h`, `--help`    | Show help message      |
| `-V`, `--version` | Show version           |

---

## Environment Variables 🌍

| Variable        | Description             | Default    |
|-----------------|-------------------------|------------|
| `SCOOP_HOME`    | The Freezer location    | `~/.scoop` |
| `SCOOP_NO_AUTO` | Disable auto-activation | (unset)    |
| `NO_COLOR`      | Disable colored output  | (unset)    |

---

## The Freezer Layout 🧊

| Location                | Purpose                                  |
|-------------------------|------------------------------------------|
| `~/.scoop/virtualenvs/` | All your flavors live here               |
| `~/.scoop/version`      | Your usual order (global default)        |
| `.scoop-version`        | Local flavor preference                  |
| `.python-version`       | pyenv compatibility (fallback)           |
| `.venv`                 | Symlink to active flavor (with `--link`) |

---

## Tab Completion ⌨️

Tab completion is enabled via `scoop init`:

```bash
scoop <TAB>              # List subcommands
scoop use <TAB>          # List available flavors
scoop use --<TAB>        # List options
scoop install --<TAB>    # List --latest, --stable
scoop create name <TAB>  # List Python versions
```

---

> 🍨 *"The right flavor, at the right time, in one scoop."*
