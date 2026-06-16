# self update

Reinstall scoop from crates.io to update (or pin) the installed version.

## Usage

```bash
scoop self update [OPTIONS]
```

## Options

| Option | Description |
|--------|-------------|
| `--force` | Reinstall even if already on the latest version |
| `--version <VERSION>` | Install a specific version instead of the latest |
| `--no-verify` | Skip the post-update `scoop doctor` verification |
| `--json` | Output result as JSON |

## Behavior

1. Queries crates.io (via `cargo search`) for the latest `scoop-uv` version.
2. Runs `cargo install --force --locked scoop-uv --version <target>`.
3. Unless `--no-verify` is set, runs `scoop doctor` with the freshly installed binary and reports the outcome.

Use `--force` to reinstall the current version (useful for repairing a broken install). Use `--version` to pin to a specific release.

## Examples

```bash
# Update to the latest release
scoop self update

# Pin to a specific version
scoop self update --version 0.14.0

# Force reinstall of the current version
scoop self update --force

# Machine-readable output
scoop self update --json
```

### JSON Output

```json
{
  "status": "success",
  "command": "self update",
  "data": {
    "from": "0.14.0",
    "to": "0.14.1",
    "skipped": false,
    "verify": { "status": "passed" }
  }
}
```

The `verify.status` field is one of `skipped`, `passed`, `warned`, `errored`, or `launch_failed`.

## See also

- [`scoop doctor`](doctor.md) — diagnose the current installation
