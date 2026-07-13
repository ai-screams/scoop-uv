# Frequently Asked Questions

## What's the difference between scuv and pyenv?

While both tools help you manage Python, they focus on different parts of the workflow:

**pyenv** is primarily a *version manager*. It focuses on:
- Installing multiple versions of the Python interpreter (e.g., 3.9.0, 3.12.1)
- Switching between them globally or per folder

**scuv** is an *environment and workflow manager* powered by [uv](https://github.com/astral-sh/uv). It focuses on:
- Creating and managing isolated virtual environments
- Fast project-specific environment workflows

> **Summary:** You might use pyenv to install Python 3.11 on your machine, but you use scuv to actually build and run your application within a lightning-fast virtual environment using that Python version.

## How is scuv different from uv's `centralized-project-envs` preview?

They solve different problems, and scuv is built *on top of* uv — it's a complement, not a fork
or a competitor.

uv 0.11.25 added a preview feature (`centralized-project-envs`) that relocates a **project's**
`.venv` into uv's cache directory. The environment is still bound to that one project: its
identity is a cache key derived from the workspace path and interpreter (e.g.
`my-project-cp3.12.4-0123abcd`), it cannot be shared between projects, there is no activation
workflow, and `uv cache clean` / `uv cache prune` delete it unconditionally — by design it is a
disposable cache entry that gets transparently recreated.

scuv environments are the opposite in every one of those dimensions: **named, durable, and
project-independent**.

| | uv `centralized-project-envs` | scuv |
|---|---|---|
| Environment identity | hash cache key (not user-controlled) | a name you choose (`scuv create ml 3.12`) |
| Shared across projects | no (key includes workspace path) | yes — any project with a `.scuv-version` file |
| Activation workflow | none (`uv run`-centric; `.venv` link for IDEs) | shell auto-activation, `scuv use`, 4 shells |
| Lifecycle | wiped by `uv cache clean`/`prune`, auto-recreated | durable; `gc` (dry-run first), `verify`, metadata (`last_used`) |
| Extras | — | `clone`, `diff`, `export`/`import`, `.scuv.toml` sync, migration from pyenv / conda / virtualenvwrapper |

The uv team has stated there are ["no current plans to support standalone environments not tied
to a specific project"](https://github.com/astral-sh/uv/pull/18214). That standalone, named,
pyenv-virtualenv-style workflow is exactly what scuv provides — with uv doing the fast parts
underneath.

## Is scuv related to Scoop, the Windows package manager?

No. scuv — "a **sc**oop of **uv**" 🍨 — is a centralized Python virtual environment manager and
is unrelated to [Scoop](https://scoop.sh), the Windows package manager. The project was
originally command-named `scoop`; we renamed the command to `scuv` in v0.15.0 precisely so both
tools can coexist cleanly on Windows. Installing scuv does not shadow or conflict with `scoop`
in any shell, including PowerShell. (The repository and crate keep the historical name
`scoop-uv`.)

## How do I set Python 3.11.0 as the global default for all new shells and environments?

Use this workflow:

```bash
# 1) Install Python 3.11.0 (skip if already available on your system)
scuv install 3.11.0

# 2) Create an environment that uses 3.11.0
scuv create py311 3.11.0

# 3) Make that environment the global default
scuv use py311 --global
```

Important details:

- `--global` stores an environment name in `~/.scuv/version`, not a raw version like `3.11.0`.
- This global default is applied in new shells and directories without a local `.scuv-version`.
- Priority is: `SCUV_VERSION` env var > local `.scuv-version` > global `~/.scuv/version`.

To remove the global default later:

```bash
scuv use --unset --global
```

## How do I create a new virtual environment for a project, explicitly specifying Python 3.9.5?

Use this end-to-end workflow:

```bash
# 1) Install Python 3.9.5 (skip if already available on your system)
scuv install 3.9.5

# 2) Create a new project environment with that exact version
scuv create myproject 3.9.5

# 3) Verify which Python the environment uses
scuv info myproject
```

If creation fails because `3.9.5` is not found, run:

```bash
uv python list
scuv list --pythons
```

Then install the exact version and retry:

```bash
scuv install 3.9.5
scuv create myproject 3.9.5
```

## How do I uninstall a specific Python version and all its associated virtual environments managed by scuv?

Use `--cascade` to remove both the Python version and every environment that depends on it:

```bash
# 1) Optional: preview affected environments
scuv list --python-version 3.12

# 2) Remove Python 3.12 and all associated environments
scuv uninstall 3.12 --cascade

# 3) Verify cleanup
scuv list --pythons
scuv doctor
```

Useful variants:

- Non-interactive mode: `scuv uninstall 3.12 --cascade --force`
- JSON output for automation: `scuv uninstall 3.12 --cascade --json`

Important detail:

- Without `--cascade`, environments are not removed and can become broken.

## Given scuv's auto-activation feature, how would a developer temporarily disable or customize its behavior for a specific project or directory without affecting global settings?

Use one of these local or temporary patterns:

```bash
# Option 1) Disable auto-activation only in the current shell session
export SCUV_NO_AUTO=1
# ...work here...
unset SCUV_NO_AUTO

# Option 2) For one project directory, force system Python locally
cd ~/project
scuv use system

# Option 3) For one project directory, pin a specific environment locally
scuv use myproject

# Option 4) Temporary override in this terminal only (no file changes)
scuv shell system
# ...test...
scuv shell --unset
```

Notes:

- These approaches avoid `--global`, so global defaults are unchanged.
- `.scuv-version` changes from `scuv use ...` are local to the project directory (and inherited by subdirectories).
- `scuv shell ...` affects only the current terminal session.

## Once a scuv environment is active, how would you install project dependencies from a `requirements.txt` file into it?

Run pip inside the active environment:

```bash
# Prompt shows active environment, e.g. (myproject)
pip install -r requirements.txt
```

Useful variants:

- Different file location: `pip install -r path/to/requirements.txt`
- Verify installed dependencies: `pip list`

If `requirements.txt` is in the project root, run the command from that directory.

## How can a developer list all Python versions and their associated virtual environments currently managed by scuv?

Use this sequence:

```bash
# 1) Show all managed Python versions
scuv list --pythons

# 2) Show all environments and their Python versions
scuv list

# 3) Show environments for one specific Python version
scuv list --python-version 3.12
```

For automation:

- Use `--json` for machine-readable output.
- Use `--bare` for name-only output in shell scripts.

Example script to iterate each Python version and print associated environments:

```bash
for v in $(scuv list --pythons --bare); do
  echo "== Python $v =="
  scuv list --python-version "$v" --bare
done
```

If no versions or environments exist yet, these commands simply return empty results.

## If a project requires a Python version not directly available through scuv's default sources, how could a developer integrate a custom or pre-existing Python installation into scuv's management system?

Use one of these two approaches:

```bash
# Option 1) Recommended: point directly to a Python executable
scuv create myenv --python-path /opt/python-debug/bin/python3

# Option 2) Add custom Python to PATH, then use normal version selection
export PATH="/opt/python-debug/bin:$PATH"
scuv create myenv 3.13
```

Validation and diagnostics:

```bash
uv python list      # confirm interpreter discovery
scuv info myenv    # confirm selected Python + Python Path
scuv doctor -v     # detect broken links/metadata issues
```

Where scuv stores this integration:

- Environment metadata file: `~/.scuv/virtualenvs/myenv/.scoop-metadata.json`
- Custom interpreter path is recorded in the `python_path` field.

## Can I use scuv with conda environments?

Not directly. They serve different purposes and operate independently:

**conda** is a *package and environment manager*. It handles:
- Its own binaries and non-Python dependencies
- Heavy data science libraries (MKL, CUDA, cuDNN, etc.)

**scuv** is a *lightweight environment manager* powered by [uv](https://github.com/astral-sh/uv). It:
- Leverages your existing Python installations
- Creates fast, portable virtual environments

> **When to use what:** For heavy data science requiring non-Python libraries → conda. For almost everything else → scuv (significantly faster and more portable).

## How do I uninstall scuv completely?

To remove scuv from your system:

### 1. Delete the data folder

    rm -rf ~/.scuv

### 2. Remove the shell hook

Edit your shell config file and remove the scuv init line:

| Shell | Config File | Line to Remove |
|-------|-------------|----------------|
| Bash | `~/.bashrc` | `eval "$(scuv init bash)"` |
| Zsh | `~/.zshrc` | `eval "$(scuv init zsh)"` |
| Fish | `~/.config/fish/config.fish` | `scuv init fish \| source` |
| PowerShell | `$PROFILE` | `Invoke-Expression (& scuv init powershell)` |

### 3. (Optional) Remove config

    rm -f ~/.scuv/config.json

### 4. Restart your terminal

## Does scuv work on Windows?

scuv supports **PowerShell** on Windows (both PowerShell Core 7.x+ and Windows PowerShell 5.1+). Shell integration including auto-activation and tab completion works fully.

```powershell
# Add to $PROFILE
Invoke-Expression (& scuv init powershell)
```

> **Note:** Command Prompt (cmd.exe) is not supported. Use PowerShell for the full scuv experience.

## Can I use a custom or pre-existing Python with scuv?

Yes, in two ways:

### Option 1: Use --python-path (recommended for custom builds)

Point directly to any Python executable:

```bash
# Custom-built Python
scuv create debug-env --python-path /opt/python-debug/bin/python3

# PyPy interpreter
scuv create pypy-env --python-path /opt/pypy/bin/pypy3

# GraalPy
scuv create graal-env --python-path /opt/graalpy/bin/graalpy
```

scuv validates the path, auto-detects the version, and stores it in metadata.

### Option 2: System Python via uv discovery

scuv uses [uv](https://github.com/astral-sh/uv) for Python discovery, which automatically finds Python installations on your system:

```bash
# Check what Python versions uv can discover
uv python list

# Example output:
# cpython-3.13.1    /opt/homebrew/bin/python3.13     (system)
# cpython-3.12.8    ~/.local/share/uv/python/...     (managed)
# cpython-3.11.5    /usr/bin/python3.11               (system)

# Use a system-installed Python directly (no scuv install needed)
scuv create myenv 3.13
```

For a custom Python in a non-standard location, add it to your `PATH`:

```bash
export PATH="/opt/python-debug/bin:$PATH"
scuv create debug-env 3.13
```

> **See also:** [Python Management](python-management.md) for the full guide on Python discovery, system Python, custom interpreters, and environment variables.

## Can I migrate environments from pyenv or conda?

Yes. scuv can discover and migrate existing environments from pyenv-virtualenv, conda, and virtualenvwrapper:

```bash
# See what can be migrated
scuv migrate list
# pyenv-virtualenv:
#   myproject (Python 3.12.0)
# conda:
#   ml-env (Python 3.10.4)

# Migrate a specific environment
scuv migrate @env myproject

# Migrate everything at once
scuv migrate all
```

The original environments are preserved by default. Use `--delete-source` to remove source envs after successful migration. See [migrate command](commands/migrate.md) for details.
