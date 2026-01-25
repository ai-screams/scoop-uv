# shell

Set shell-specific environment (current shell session only).

Unlike `scoop use` which writes to a file, `scoop shell` sets the `SCOOP_VERSION` environment variable for the current shell session only.

## Usage

```bash
eval "$(scoop shell <name>)"    # Bash/Zsh
scoop shell <name> | source     # Fish
```

> **Note:** If you have shell integration set up (`scoop init`), the `eval` is automatic:
> ```bash
> scoop shell myenv    # Works directly
> ```

## Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `name` | No | Environment name or `system` |

## Options

| Option | Description |
|--------|-------------|
| `--unset` | Clear shell-specific environment |
| `--shell <SHELL>` | Target shell type (auto-detected if not specified) |

## Behavior

- Sets `SCOOP_VERSION` environment variable
- If `name` is an environment: also outputs activation script
- If `name` is `system`: also outputs deactivation script
- `--unset`: outputs `unset SCOOP_VERSION`

## Priority

`SCOOP_VERSION` has the **highest priority** in version resolution:

```
1. SCOOP_VERSION env var    <- scoop shell (highest)
2. .scoop-version file      <- scoop use
3. .python-version file     <- pyenv compatibility
4. ~/.scoop/version         <- scoop use --global
```

This means `scoop shell` **overrides** any file-based settings until:
- You run `scoop shell --unset`
- You close the terminal

## Examples

```bash
# Use a specific environment in this terminal
scoop shell myproject

# Use system Python in this terminal
scoop shell system

# Clear the shell setting (return to file-based resolution)
scoop shell --unset

# Explicit shell type
scoop shell --shell fish myenv
```

## Use Cases

### Temporary Testing

```bash
# Currently using myproject
scoop shell testenv        # Switch to testenv temporarily
python test.py             # Test something
scoop shell myproject      # Switch back
```

### Override Project Settings

```bash
cd ~/project               # Has .scoop-version = projectenv
scoop shell system         # Use system Python anyway
python --version           # System Python
scoop shell --unset        # Back to projectenv
```
