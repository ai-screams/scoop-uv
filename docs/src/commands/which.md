# which

Print the full path to an executable inside a scoop environment, the same way
`pyenv which` resolves binaries.

## Usage

```bash
scoop which <exe> [--env <name>] [--json]
```

## Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `exe` | Yes | Executable name to locate (e.g. `python`, `pip`, `pytest`) |

## Options

| Option | Description |
|--------|-------------|
| `--env <name>` | Look in this environment instead of the active one |
| `--json` | Output as JSON |

## Resolution Order

1. `--env <name>` if provided
2. `$SCOOP_ACTIVE` (set by `scoop activate` / `scoop shell`)
3. `.scoop-version` (local → parents → global)

If none of those resolve to a real virtualenv (e.g. system Python or no
configuration), the command fails with `No active environment`.

On Windows, the lookup also probes `.exe`, `.bat`, and `.cmd` extensions.

## Examples

```bash
scoop which python                 # active env's python
scoop which pytest --env myenv     # explicit env
scoop which python --json          # JSON: { exe, env, path }
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Path printed to stdout |
| 1 | No active env / env missing / executable not in env's bin/ |
