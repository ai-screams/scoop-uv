# list

List all virtual environments or installed Python versions.

**Aliases:** `ls`

## Usage

```bash
scoop list [options]
```

## Options

| Option | Description |
|--------|-------------|
| `--pythons` | Show Python versions instead of virtualenvs |
| `--python-version <VERSION>` | Filter environments by Python version (e.g., `3.12`) |
| `--sort <MODE>` | Sort order: `name` (default), `created`, `last-used` |
| `--bare` | Output names only (for scripting); hidden from `--help` |
| `--json` | Output as JSON |

## Sort

`--sort` reorders the output without changing what's shown:

| Mode        | Order                              | Tie-break          |
|-------------|------------------------------------|--------------------|
| `name`      | Alphabetical (default, back-compat)| —                  |
| `created`   | Newest `created_at` first          | Name (asc)         |
| `last-used` | Most recently activated first      | Name (asc)         |

Envs missing the relevant timestamp (`created_at` / `last_used`) sort
to the **end** of the list, with name-order tie-break — so legacy or
never-activated envs don't bury the interesting ones. `last_used`
populates when an env is actually activated: `scoop activate`,
shell-hook auto-activation triggered by `scoop use`, `scoop run`, or
`scoop shell`. `scoop use` on its own only writes the version file
and does not touch metadata; the touch fires when the shell wrapper
sources the activate script afterwards.

`--sort` is mutually exclusive with `--pythons` (which lists Python
installations, not environments).

## Examples

```bash
scoop list                           # List all virtualenvs
scoop list --pythons                 # List installed Python versions
scoop list --bare                    # Names only, one per line
scoop list --json                    # JSON output

# Filter by Python version
scoop list --python-version 3.12     # Show only 3.12.x environments
scoop list --python-version 3        # Show all Python 3.x environments
scoop list --python-version 3.12.1   # Exact version match

# Sort
scoop list --sort created            # Newest envs first
scoop list --sort last-used          # Recently active envs first
```

## List Python Versions with Associated Environments

Use this workflow to see both sides of the mapping:

```bash
# 1) List installed Python versions managed by scoop/uv
scoop list --pythons

# 2) List all virtual environments with their Python versions
scoop list

# 3) Show environments associated with a specific Python version
scoop list --python-version 3.12
```

For scripting, combine `--bare` with per-version filtering:

```bash
for v in $(scoop list --pythons --bare); do
  echo "== Python $v =="
  scoop list --python-version "$v" --bare
done
```

You can also use `--json` for machine-readable output:

```bash
scoop list --pythons --json
scoop list --json
```

## Version Filtering

The `--python-version` option uses prefix matching to filter environments:

```bash
scoop list --python-version 3.12
# Output:
#   myproject      3.12.1
#   webapp         3.12.0
# (environments using 3.11 or 3.13 are not shown)
```

This is useful for identifying environments before uninstalling a Python version:

```bash
# See which environments will be affected
scoop list --python-version 3.12

# Then uninstall with cascade
scoop uninstall 3.12 --cascade
```

> **Note:** `--python-version` cannot be combined with `--pythons` (which lists Python installations, not environments).

## Empty Results

- If no Python versions are installed, `scoop list --pythons` shows no entries.
- If no environments exist, `scoop list` shows no entries.
- If no environments match a filter, `scoop list --python-version <VERSION>` shows no entries.
