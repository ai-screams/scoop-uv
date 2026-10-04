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

Which environments count as using it:

- An environment that records the version you name, or a more specific
  one: `uninstall 3.12 --cascade` removes environments recording `3.12`
  or `3.12.1`.
- An environment that records only the minor version, as current uv writes
  (`3.12`), when you name a patch release (`uninstall 3.12.14 --cascade`):
  uv points it at the newest compatible 3.12.x left, so it is removed only
  if no other uv-managed 3.12.x remains. With 3.12.3 still installed it is
  left alone. Any remaining 3.12.x counts, whatever its build, so an env
  whose only other 3.12.x is a PyPy or free-threaded build is kept even
  though it may no longer run.

`--cascade` takes plain version numbers only (`3`, `3.12`, `3.12.14`). A
request such as `3.13t`, `3.12.0rc1` or `cpython@3.12` is refused before
anything is removed, because its numbers alone do not say which installs
uv would remove.

```bash
scuv uninstall 3.12 --cascade
# • Found 2 environment(s) using Python 3.12:
# •   - myproject
# •   - webapp
# Remove these environments and uninstall Python 3.12? [y/N]
# • Removing 'myproject'...
# • Removing 'webapp'...
# • Removed 2 environment(s)
# • Uninstalling Python 3.12...
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
#   myproject  3.12  ~/.scuv/virtualenvs/myproject
#   webapp     3.12  ~/.scuv/virtualenvs/webapp

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
# Output (excerpt):
# ✗ broken virtualenv: 'myproject' is corrupted
#   → scuv remove myproject && scuv create myproject <python-version>
# ✗ broken symlink: Python symlink in 'myproject' is broken
#   → scuv remove myproject && scuv create myproject <python-version>

# Option 1: Reinstall the Python version
scuv install 3.12
scuv doctor --fix

# Option 2: Recreate affected environments with a new version
scuv remove myproject --force
scuv create myproject 3.13
```
