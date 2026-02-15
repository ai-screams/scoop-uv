# Frequently Asked Questions

## What's the difference between scoop and pyenv?

While both tools help you manage Python, they focus on different parts of the workflow:

**pyenv** is primarily a *version manager*. It focuses on:
- Installing multiple versions of the Python interpreter (e.g., 3.9.0, 3.12.1)
- Switching between them globally or per folder

**scoop** is an *environment and workflow manager* powered by [uv](https://github.com/astral-sh/uv). It focuses on:
- Creating and managing isolated virtual environments
- Fast project-specific environment workflows

> **Summary:** You might use pyenv to install Python 3.11 on your machine, but you use scoop to actually build and run your application within a lightning-fast virtual environment using that Python version.

## How do I set Python 3.11.0 as the global default for all new shells and environments?

Use this workflow:

```bash
# 1) Install Python 3.11.0 (skip if already available on your system)
scoop install 3.11.0

# 2) Create an environment that uses 3.11.0
scoop create py311 3.11.0

# 3) Make that environment the global default
scoop use py311 --global
```

Important details:

- `--global` stores an environment name in `~/.scoop/version`, not a raw version like `3.11.0`.
- This global default is applied in new shells and directories without a local `.scoop-version`.
- Priority is: `SCOOP_VERSION` env var > local `.scoop-version` > global `~/.scoop/version`.

To remove the global default later:

```bash
scoop use --unset --global
```

## How do I create a new virtual environment for a project, explicitly specifying Python 3.9.5?

Use this end-to-end workflow:

```bash
# 1) Install Python 3.9.5 (skip if already available on your system)
scoop install 3.9.5

# 2) Create a new project environment with that exact version
scoop create myproject 3.9.5

# 3) Verify which Python the environment uses
scoop info myproject
```

If creation fails because `3.9.5` is not found, run:

```bash
uv python list
scoop list --pythons
```

Then install the exact version and retry:

```bash
scoop install 3.9.5
scoop create myproject 3.9.5
```

## How do I uninstall a specific Python version and all its associated virtual environments managed by scoop?

Use `--cascade` to remove both the Python version and every environment that depends on it:

```bash
# 1) Optional: preview affected environments
scoop list --python-version 3.12

# 2) Remove Python 3.12 and all associated environments
scoop uninstall 3.12 --cascade

# 3) Verify cleanup
scoop list --pythons
scoop doctor
```

Useful variants:

- Non-interactive mode: `scoop uninstall 3.12 --cascade --force`
- JSON output for automation: `scoop uninstall 3.12 --cascade --json`

Important detail:

- Without `--cascade`, environments are not removed and can become broken.

## Given Scoop-uv's auto-activation feature, how would a developer temporarily disable or customize its behavior for a specific project or directory without affecting global settings?

Use one of these local or temporary patterns:

```bash
# Option 1) Disable auto-activation only in the current shell session
export SCOOP_NO_AUTO=1
# ...work here...
unset SCOOP_NO_AUTO

# Option 2) For one project directory, force system Python locally
cd ~/project
scoop use system

# Option 3) For one project directory, pin a specific environment locally
scoop use myproject

# Option 4) Temporary override in this terminal only (no file changes)
scoop shell system
# ...test...
scoop shell --unset
```

Notes:

- These approaches avoid `--global`, so global defaults are unchanged.
- `.scoop-version` changes from `scoop use ...` are local to the project directory (and inherited by subdirectories).
- `scoop shell ...` affects only the current terminal session.

## Can I use scoop with conda environments?

Not directly. They serve different purposes and operate independently:

**conda** is a *package and environment manager*. It handles:
- Its own binaries and non-Python dependencies
- Heavy data science libraries (MKL, CUDA, cuDNN, etc.)

**scoop** is a *lightweight environment manager* powered by [uv](https://github.com/astral-sh/uv). It:
- Leverages your existing Python installations
- Creates fast, portable virtual environments

> **When to use what:** For heavy data science requiring non-Python libraries → conda. For almost everything else → scoop (significantly faster and more portable).

## How do I uninstall scoop completely?

To remove scoop from your system:

### 1. Delete the data folder

    rm -rf ~/.scoop

### 2. Remove the shell hook

Edit your shell config file and remove the scoop init line:

| Shell | Config File | Line to Remove |
|-------|-------------|----------------|
| Bash | `~/.bashrc` | `eval "$(scoop init bash)"` |
| Zsh | `~/.zshrc` | `eval "$(scoop init zsh)"` |
| Fish | `~/.config/fish/config.fish` | `eval (scoop init fish)` |
| PowerShell | `$PROFILE` | `Invoke-Expression (& scoop init powershell)` |

### 3. (Optional) Remove config

    rm -f ~/.scoop/config.json

### 4. Restart your terminal

## Does scoop work on Windows?

scoop supports **PowerShell** on Windows (both PowerShell Core 7.x+ and Windows PowerShell 5.1+). Shell integration including auto-activation and tab completion works fully.

```powershell
# Add to $PROFILE
Invoke-Expression (& scoop init powershell)
```

> **Note:** Command Prompt (cmd.exe) is not supported. Use PowerShell for the full scoop experience.

## Can I use a custom or pre-existing Python with scoop?

Yes, in two ways:

### Option 1: Use --python-path (recommended for custom builds)

Point directly to any Python executable:

```bash
# Custom-built Python
scoop create debug-env --python-path /opt/python-debug/bin/python3

# PyPy interpreter
scoop create pypy-env --python-path /opt/pypy/bin/pypy3

# GraalPy
scoop create graal-env --python-path /opt/graalpy/bin/graalpy
```

scoop validates the path, auto-detects the version, and stores it in metadata.

### Option 2: System Python via uv discovery

scoop uses [uv](https://github.com/astral-sh/uv) for Python discovery, which automatically finds Python installations on your system:

```bash
# Check what Python versions uv can discover
uv python list

# Example output:
# cpython-3.13.1    /opt/homebrew/bin/python3.13     (system)
# cpython-3.12.8    ~/.local/share/uv/python/...     (managed)
# cpython-3.11.5    /usr/bin/python3.11               (system)

# Use a system-installed Python directly (no scoop install needed)
scoop create myenv 3.13
```

For a custom Python in a non-standard location, add it to your `PATH`:

```bash
export PATH="/opt/python-debug/bin:$PATH"
scoop create debug-env 3.13
```

> **See also:** [Python Management](python-management.md) for the full guide on Python discovery, system Python, custom interpreters, and environment variables.

## Can I migrate environments from pyenv or conda?

Yes. scoop can discover and migrate existing environments from pyenv-virtualenv, conda, and virtualenvwrapper:

```bash
# See what can be migrated
scoop migrate list
# pyenv-virtualenv:
#   myproject (Python 3.12.0)
# conda:
#   ml-env (Python 3.10.4)

# Migrate a specific environment
scoop migrate @myproject

# Migrate everything at once
scoop migrate --all
```

The original environments are **preserved** (not deleted). See [migrate command](commands/migrate.md) for details.
