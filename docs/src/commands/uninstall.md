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

The `--cascade` flag also removes the virtual environments that lose their Python when it is uninstalled. This replaces the manual multi-step workflow.

Which environments count as using it is read from each environment's
`pyvenv.cfg` (`home`), not from the version it records:

- An environment linked to an install being removed (`cpython-3.12.14-…`)
  is removed.
- An environment linked to uv's minor-version link (`cpython-3.12-…`, what
  current uv creates) is removed only if no other install of the same build
  is left to take that link over. With CPython 3.12.13 still installed,
  `uninstall 3.12.14 --cascade` keeps it, and uv points it at 3.12.13. A
  remaining free-threaded 3.12 does not count: it is a different build
  with its own link.
- A plain version removes the default builds only, as uv does: an
  environment on a free-threaded install (`cpython-3.12.13+freethreaded-…`)
  is not affected by `uninstall 3.12.13`. From uv 0.9.1, a version that
  names the patch leaves that patch's pre-releases: `uninstall 3.14.0` does
  not take 3.14.0rc1, while `uninstall 3.14` takes both. Older uv takes
  the pre-releases with the patch, and the cascade follows whichever uv is
  installed.
- An environment on a Python uv does not manage (Homebrew, a
  `--python-path` interpreter) is never removed.
- An environment whose `pyvenv.cfg` is missing, unreadable or has no
  `home` line is never removed either: which Python it uses cannot be
  told. It is named in a warning before anything is uninstalled (and in
  `unverified_envs` under `--json`).

Environments are removed only after uv has uninstalled the Python, and
only those whose interpreter is actually gone by then: if the uninstall
fails, or uv kept a link alive, the environment is left in place. Each
one is checked again just before removal: an environment that was
replaced in the meantime (another directory under the name, or another
`home`) is kept with a warning, and one that cannot be checked (an I/O
or permission error) is kept and counted as not removed. If one
environment cannot be removed, the others still are; it is reported with
a warning, and the command exits with status 1 although the Python is
gone.

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
# • Uninstalling Python 3.12...
# • Removing 'myproject'...
# • Removing 'webapp'...
# • Removed 2 environment(s)
# ✓ Python 3.12 uninstalled
```

With `--force`, the confirmation prompt is skipped:

```bash
scuv uninstall 3.12 --cascade --force
```

With `--json`, the output includes the list of removed environments. When
any could not be removed, `status` is `"error"` (code
`UNINSTALL_CASCADE_INCOMPLETE`) and `data` adds `failed_envs`, with `name`
and `error` for each. Environments left alone because their interpreter
could not be read are listed in `unverified_envs`, with `name` and
`reason`:

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
