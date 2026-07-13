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
| **uv installation** | uv is installed and accessible |
| **Shell integration** | Shell hook is properly configured |
| **Environment integrity** | Python symlinks are valid, `pyvenv.cfg` exists |
| **Path configuration** | `~/.scuv/` directory structure is correct |
| **Version file validity** | `.scuv-version` files reference existing environments |

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
- **pyvenv.cfg** — Does the environment's configuration file exist and reference a valid Python?

Environments can become broken when their underlying Python version is uninstalled. Use `scuv doctor` to detect these issues:

```bash
# After accidentally uninstalling Python 3.12:
scuv doctor -v
# Output:
#   ✓ uv: installed (0.5.x)
#   ✓ Shell: zsh integration active
#   ⚠ Environment 'myproject': Python symlink broken
#   ⚠ Environment 'webapp': Python symlink broken

# Auto-fix by recreating symlinks (requires Python to be reinstalled)
scuv install 3.12
scuv doctor --fix
# Output:
#   ✓ Fixed 'myproject': Python symlink restored
#   ✓ Fixed 'webapp': Python symlink restored
```

> **Tip:** Run `scuv doctor` periodically or after uninstalling Python versions to catch broken environments early. See [uninstall command](uninstall.md) for the safe uninstall workflow.
