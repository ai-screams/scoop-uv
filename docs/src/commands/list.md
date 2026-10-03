# list

List all virtual environments or installed Python versions.

**Aliases:** `ls`

## Usage

```bash
scuv list [options]
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
populates when an env is actually activated: `scuv activate`,
shell-hook auto-activation triggered by `scuv use`, `scuv run`, or
`scuv shell`. `scuv use` on its own only writes the version file
and does not touch metadata; the touch fires when the shell wrapper
sources the activate script afterwards.

`--sort` is mutually exclusive with `--pythons` (which lists Python
installations, not environments).

## Examples

```bash
scuv list                           # List all virtualenvs
scuv list --pythons                 # List installed Python versions
scuv list --bare                    # Names only, one per line
scuv list --json                    # JSON output

# Filter by Python version
scuv list --python-version 3.12     # Show only 3.12.x environments
scuv list --python-version 3        # Show all Python 3.x environments
scuv list --python-version 3.12.1   # Matches only envs that recorded 3.12.1

# Sort
scuv list --sort created            # Newest envs first
scuv list --sort last-used          # Recently active envs first
```

Each row shows the name, the Python version the env recorded, and its
path. The last row is the `system` Python found on `PATH`; it appears
in `--bare` and `--json` output too, and an active env is marked `*`:

```bash
$ scuv list
  myproject      3.12    ~/.scuv/virtualenvs/myproject
  other          3.11    ~/.scuv/virtualenvs/other
  webapp         3.12    ~/.scuv/virtualenvs/webapp
  webapp-mirror  3.11    ~/.scuv/virtualenvs/webapp-mirror
  system         3.13.x  /usr/bin/python3 (system)

$ scuv list --bare
myproject
other
webapp
webapp-mirror
system
```

Envs created by current uv record the minor version (`3.12`), because
that is what uv writes to `pyvenv.cfg`.

## List Python Versions with Associated Environments

Use this workflow to see both sides of the mapping:

```bash
# 1) List installed Python versions managed by scuv/uv
scuv list --pythons

# 2) List all virtual environments with their Python versions
scuv list

# 3) Show environments associated with a specific Python version
scuv list --python-version 3.12
```

For scripting, combine `--bare` with per-version filtering. `--pythons
--bare` prints full versions (`3.12.14`) while envs record the minor
version (`3.12`), so cut each version down to `major.minor` first:

```bash
for v in $(scuv list --pythons --bare | cut -d. -f1,2 | sort -uV); do
  echo "== Python $v =="
  scuv list --python-version "$v" --bare
done
```

The `system` row matches the filter for its own version, so it shows up
under that version's heading.

You can also use `--json` for machine-readable output:

```bash
scuv list --pythons --json
scuv list --json
```

## Version Filtering

The `--python-version` option uses prefix matching to filter environments:

```bash
scuv list --python-version 3.12
# Output:
#   myproject  3.12  ~/.scuv/virtualenvs/myproject
#   webapp     3.12  ~/.scuv/virtualenvs/webapp
# (environments using 3.11 or 3.13 are not shown)
```

This is useful for identifying environments before uninstalling a Python version:

```bash
# See which environments will be affected
scuv list --python-version 3.12

# Then uninstall with cascade
scuv uninstall 3.12 --cascade
```

> **Note:** `--python-version` cannot be combined with `--pythons` (which lists Python installations, not environments).

## Empty Results

- If no Python versions are installed, `scuv list --pythons` shows no entries.
- If no environments exist, `scuv list` still shows the `system` row when
  a Python is on `PATH`. Only when there is none either does it print
  `No environments yet` and a `scuv create` hint (to stderr).
- If no environments match a filter, `scuv list --python-version <VERSION>`
  prints `No environments using Python <VERSION>` (to stderr; `--bare`
  prints nothing).
