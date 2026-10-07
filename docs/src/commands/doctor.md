# doctor

Check scuv installation health and diagnose issues.

## Usage

```bash
scuv doctor [options]
```

## Options

| Option | Description |
|--------|-------------|
| `-v`, `--verbose` | Show more details (can repeat: `-vv`) |
| `--json` | Output diagnostics as JSON |
| `--fix` | Auto-fix issues where possible |

## Checks Performed

| Check | What it verifies |
|-------|------------------|
| **uv installation** | uv is installed and meets the minimum version (0.5.19) |
| **SCUV_HOME directory** | `~/.scuv/` exists and is writable |
| **virtual environments** | Every environment has a `bin/python` and a `pyvenv.cfg` |
| **symbolic links** | Python symlinks inside each environment still resolve |
| **shell configuration** | The shell hook is present in your rc file |
| **version files** | `.scuv-version` entries reference environments that exist |
| **project .venv link** | A `.venv` symlink in the current directory still resolves. A dangling link into `~/.scuv/virtualenvs/` (made by `scuv use --link`) is an error that `--fix` removes; a dangling link elsewhere is only a warning |
| **legacy scoop remnants** | Leftover `SCOOP_*` vars, an orphaned `~/.scoop`, or `.scoop-version` / `.scoop.toml` in the current directory — none of them read since v0.16.0, so the check only warns |

## Examples

```bash
scuv doctor                     # Quick health check
scuv doctor -v                  # Verbose diagnostics
scuv doctor --fix               # Fix what can be fixed
scuv doctor --json              # JSON output for scripting
```

## Environment Integrity

The doctor checks each virtual environment for:

- **Python symlink** — Does the `python` binary in the environment point to a valid Python installation?
- **pyvenv.cfg** — Does the environment's configuration file exist (and its Python binary)?

Environments can become broken when their underlying Python version is uninstalled. Use `scuv doctor` to detect these issues:

```bash
# After accidentally uninstalling Python 3.12:
scuv doctor -v
# Output:
#
# Checking installation...
#
# ✓ uv installation
#   uv 0.x.y (<commit> <date> <target>)
# ✓ SCUV_HOME directory
#   ~/.scuv
# ✗ broken virtualenv: 'myproject' is corrupted
#   → scuv remove myproject && scuv create myproject <python-version>
# ✗ broken virtualenv: 'webapp' is corrupted
#   → scuv remove webapp && scuv create webapp <python-version>
# ✗ broken symlink: Python symlink in 'myproject' is broken
#   → scuv remove myproject && scuv create myproject <python-version>
# ✗ broken symlink: Python symlink in 'webapp' is broken
#   → scuv remove webapp && scuv create webapp <python-version>
# ✓ shell configuration
#   found in ~/.zshrc
# ✓ version files
#   no version files configured
# ✓ legacy scoop remnants
#
# ──────────────────────────────────
# Found 4 error(s).

# Auto-fix by recreating symlinks (requires Python to be reinstalled)
scuv install 3.12
scuv doctor --fix
# Output:
# Checking installation...
# • Attempting to fix symlink for 'myproject'...
# • Found Python version: 3.12
# ✓ Fixed symlink for 'myproject'
# • Attempting to fix symlink for 'webapp'...
# • Found Python version: 3.12
# ✓ Fixed symlink for 'webapp'
# ✓ uv installation
# ✓ SCUV_HOME directory
# ✓ virtual environments
# ✓ broken symlink
# ✓ shell configuration
# ✓ version files
# ✓ legacy scoop remnants
# ──────────────────────────────────
# All checks passed!
```

`✓` marks a passing check, `⚠` a warning and `✗` an error; `→` lines
suggest a fix. The report goes to stderr. `doctor` exits `2` when any
check errors, `1` when the worst finding is a warning, and `0` when every
check passes (`All checks passed!`).

`--fix` attempts its fixes first and then prints the report. A fix can
clear an error another check found — relinking an env's interpreter also
mends its `broken virtualenv` error — so once anything is fixed, the
checks that still had errors run again, and the report shows the state
after the fixes.

> **Tip:** Run `scuv doctor` periodically or after uninstalling Python versions to catch broken environments early. See [uninstall command](uninstall.md) for the safe uninstall workflow.
