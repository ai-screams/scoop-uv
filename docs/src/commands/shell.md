# shell

Set shell-specific environment (current shell session only).

Unlike `scuv use` which writes to a file, `scuv shell` sets the `SCUV_VERSION` environment variable for the current shell session only.

## Usage

```bash
eval "$(scuv shell <name>)"    # Bash/Zsh
scuv shell <name> | source       # Fish
```

> **Note:** If you have shell integration set up (`scuv init`), the `eval` is automatic:
> ```bash
> scuv shell myenv    # Works directly
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

- Sets `SCUV_VERSION` environment variable
- If `name` is an environment: also outputs activation script
- If `name` is `system`: also outputs deactivation script
- `--unset`: outputs `unset SCUV_VERSION`

## Priority

`SCUV_VERSION` has the **highest priority** in version resolution:

```
1. SCUV_VERSION env var    <- scuv shell (highest)
2. .scuv-version file      <- scuv use
3. ~/.scuv/version         <- scuv use --global
```

This means `scuv shell` **overrides** any file-based settings until:
- You run `scuv shell --unset`
- You close the terminal

## Examples

```bash
# Use a specific environment in this terminal
scuv shell myproject

# Use system Python in this terminal
scuv shell system

# Clear the shell setting (return to file-based resolution)
scuv shell --unset

# Explicit shell type
scuv shell --shell fish myenv
```

## Use Cases

### Temporary Testing

```bash
# Currently using myproject
scuv shell testenv        # Switch to testenv temporarily
python test.py             # Test something
scuv shell myproject      # Switch back
```

### Override Project Settings

```bash
cd ~/project               # Has .scuv-version = projectenv
scuv shell system         # Use system Python anyway
python --version           # System Python
scuv shell --unset        # Back to projectenv
```
