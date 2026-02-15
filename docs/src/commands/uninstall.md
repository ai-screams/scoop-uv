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

## Examples

```bash
scoop uninstall 3.12             # Remove Python 3.12
scoop uninstall 3.11.8           # Remove specific version
```

> **Important:** Uninstalling a Python version does **not** remove virtual environments that were created with it. Those environments will become broken (their Python symlink becomes invalid). See the workflow below to handle this safely.

## Complete Uninstall Workflow

### Step 1: Identify affected environments

Before uninstalling, check which environments use the target Python version:

```bash
# List all environments with their Python versions
scoop list
# Output:
#   myproject      3.12.1  ← uses 3.12
#   webapp         3.12.1  ← uses 3.12
#   ml-env         3.11.8

# Or use JSON for scripting
scoop list --json
# Filter by version in your script:
# jq '.data.environments[] | select(.python_version | startswith("3.12"))'
```

### Step 2: Handle affected environments

For each environment using the target Python version, choose one option:

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
