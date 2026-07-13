# uninstall

Remove an installed Python version.

## Usage

```bash
scuv uninstall <version>
```

## Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `version` | Yes | Python version to remove |

## Options

| Option | Description |
|--------|-------------|
| `--cascade` | Also remove all virtual environments using this Python version |
| `--force`, `-f` | Skip confirmation for cascade removal (requires `--cascade`) |
| `--json` | Output result as JSON |

## Examples

```bash
scuv uninstall 3.12             # Remove Python 3.12
scuv uninstall 3.11.8           # Remove specific version

# Remove Python and all environments using it
scuv uninstall 3.12 --cascade

# Remove without confirmation prompt
scuv uninstall 3.12 --cascade --force
```

### Uninstall a Python Version and All Associated Environments

Recommended workflow for a full cleanup:

```bash
# 1) Optional: preview which environments would be removed
scuv list --python-version 3.12

# 2) Remove Python 3.12 and all environments using it
scuv uninstall 3.12 --cascade

# 3) Verify cleanup
scuv list --pythons
scuv doctor
```

For non-interactive scripts, skip the confirmation prompt:

```bash
scuv uninstall 3.12 --cascade --force
```

If the target version is not installed, check available versions first:

```bash
scuv list --pythons
```

## Cascade Removal

The `--cascade` flag automatically removes all virtual environments that use the target Python version before uninstalling it. This replaces the manual multi-step workflow.

```bash
scuv uninstall 3.12 --cascade
# Finding environments using Python 3.12...
# Found 2 environments using Python 3.12:
#   - myproject
#   - webapp
# Remove these environments and uninstall Python 3.12? [y/N]
# Removing myproject...
# Removing webapp...
# Uninstalling Python 3.12...
# ✓ Python 3.12 uninstalled
```

With `--force`, the confirmation prompt is skipped:

```bash
scuv uninstall 3.12 --cascade --force
```

With `--json`, the output includes the list of removed environments:

```bash
scuv uninstall 3.12 --cascade --json
# {
#   "status": "success",
#   "command": "uninstall",
#   "data": {
#     "version": "3.12",
#     "removed_envs": ["myproject", "webapp"]
#   }
# }
```

> **Note:** Without `--cascade`, uninstalling a Python version does **not** remove virtual environments that were created with it. Those environments will become broken. Use `--cascade` to handle this automatically, or follow the manual workflow below.

## Manual Uninstall Workflow

If you prefer manual control (without `--cascade`):

### Step 1: Identify affected environments

```bash
# List environments filtered by Python version
scuv list --python-version 3.12
# Output:
#   myproject      3.12.1
#   webapp         3.12.1

# Or use JSON for scripting
scuv list --json
```

### Step 2: Handle affected environments

```bash
# Option A: Remove the environment entirely
scuv remove myproject --force

# Option B: Recreate with a different Python version
scuv remove myproject --force
scuv create myproject 3.13

# Option C: Keep it (will be broken until you reinstall that Python)
# Do nothing — scuv doctor can detect and help fix it later
```

### Step 3: Uninstall the Python version

```bash
scuv uninstall 3.12
```

### Step 4: Verify

```bash
# Confirm Python is removed
scuv list --pythons

# Check for broken environments
scuv doctor
# If any issues found:
scuv doctor --fix
```

## Recovery

If you uninstalled a Python version without cleaning up environments first:

```bash
# Detect broken environments
scuv doctor -v
# Output:
#   ⚠ Environment 'myproject': Python symlink broken

# Option 1: Reinstall the Python version
scuv install 3.12
scuv doctor --fix

# Option 2: Recreate affected environments with a new version
scuv remove myproject --force
scuv create myproject 3.13
```
