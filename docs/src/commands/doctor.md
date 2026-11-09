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
| `--fix` | Fix issues where possible; [installing a missing uv](#installing-uv) asks first |

## Checks Performed

| Check | What it verifies |
|-------|------------------|
| **uv installation** | uv is installed and meets the minimum version (0.5.19) |
| **SCUV_HOME directory** | `~/.scuv/` exists and is writable |
| **virtual environments** | Every environment has a `bin/python` and a `pyvenv.cfg` (an error when either is missing), and readable scuv metadata (a warning when it is missing or does not parse; `scuv verify <name>` shows the details) |
| **symbolic links** | Python symlinks inside each environment still resolve |
| **shell configuration** | The shell hook is present in your rc file |
| **version files** | `.scuv-version` entries reference environments that exist |
| **project .venv link** | A `.venv` symlink in the current directory still resolves. A dangling link into `~/.scuv/virtualenvs/` (made by `scuv use --link`) is an error that `--fix` removes; a dangling link elsewhere is only a warning |
| **legacy scoop remnants** | Leftover `SCOOP_*` vars, an orphaned `~/.scoop`, or `.scoop-version` / `.scoop.toml` in the current directory — none of them read since v0.16.0, so the check only warns |

## Installing uv

When uv is not on `PATH`, `scuv doctor --fix` offers to install it on macOS and Linux. It
prints the exact commands, then asks. The default answer is no.

The commands download Astral's official installer and run it with
`UV_INSTALL_DIR=~/.local/bin`. The installer puts `uv` and `uvx` there, writes an install
receipt, and may add that directory to your shell profiles. If the directory is not on the
current `PATH`, doctor reports a warning: open a new shell, then run `scuv doctor --fix` again.

doctor does not offer to install when:

- `--json` or `--quiet` is set, or stdin or stderr is not a terminal. Provision uv yourself in
  scripts and CI; there is no flag to skip the question.
- `UV_OFFLINE` is set to anything but empty, `0`, `false`, `no` or `off`.
- `curl` or `sh` is not on `PATH`.
- There is no usable install location: the home directory is not an absolute path, or
  `~/.local/bin` is your `CARGO_HOME`.
- The platform is Windows. Follow the
  [uv installation guide](https://docs.astral.sh/uv/getting-started/installation/) instead.

It stops without installing if `uv` or `uvx` already exists in `~/.local/bin`, and it lists any
`UV_*`, `INSTALLER_*` or `CARGO_DIST_*` variables set in your environment, because the installer
may follow them instead of the commands shown. It never replaces or upgrades a uv that is
already on `PATH`, and `scuv doctor --fix` is the only command that installs uv.

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
- **scuv metadata** — Does `.scoop-metadata.json` exist and parse? Without it the env still works, so doctor only warns; `scuv verify` counts it as a failure. `--fix` does not recreate metadata.

doctor looks at the same directories `scuv list` shows: a symlink placed inside `~/.scuv/virtualenvs/` is not an environment and is not checked. The activate script, running the interpreter and `.scuv.toml` drift are checked by `scuv verify` only.

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
checks that still had errors and no fix of their own run again, and the
report shows the state after the fixes. A check whose fix ran keeps what
the fix said: one `✓ broken symlink` line per relinked env, or the reason
a relink failed (`Python 3.7 not installed` → `scuv install 3.7`).

> **Tip:** Run `scuv doctor` periodically or after uninstalling Python versions to catch broken environments early. See [uninstall command](uninstall.md) for the safe uninstall workflow.
