# uninstall

Remove an installed Python version.

## Usage

```bash
scoop uninstall <version>
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

## Examples

```bash
scoop uninstall 3.12             # Remove Python 3.12
scoop uninstall 3.11.8           # Remove specific version

# Remove Python and all environments using it
scoop uninstall 3.12 --cascade

# Remove without confirmation prompt
scoop uninstall 3.12 --cascade --force
```

## Cascade Removal

The `--cascade` flag automatically removes all virtual environments that use the target Python version before uninstalling it. This replaces the manual multi-step workflow.

```bash
scoop uninstall 3.12 --cascade
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
scoop uninstall 3.12 --cascade --force
```

With `--json`, the output includes the list of removed environments:

```bash
scoop uninstall 3.12 --cascade --json
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
scoop list --python-version 3.12
# Output:
#   myproject      3.12.1
#   webapp         3.12.1

# Or use JSON for scripting
scoop list --json
```

### Step 2: Handle affected environments

```bash
# Option A: Remove the environment entirely
scoop remove myproject --force

# Option B: Recreate with a different Python version
scoop remove myproject --force
scoop create myproject 3.13

# Option C: Keep it (will be broken until you reinstall that Python)
# Do nothing — scoop doctor can detect and help fix it later
```

### Step 3: Uninstall the Python version

```bash
scoop uninstall 3.12
```

### Step 4: Verify

```bash
# Confirm Python is removed
scoop list --pythons

# Check for broken environments
scoop doctor
# If any issues found:
scoop doctor --fix
```

## Recovery

If you uninstalled a Python version without cleaning up environments first:

```bash
# Detect broken environments
scoop doctor -v
# Output:
#   ⚠ Environment 'myproject': Python symlink broken

# Option 1: Reinstall the Python version
scoop install 3.12
scoop doctor --fix

# Option 2: Recreate affected environments with a new version
scoop remove myproject --force
scoop create myproject 3.13
```
