# import

Recreate an environment from a [`scuv export`](export.md) JSON file.

## Usage

```bash
scuv import <PATH> [--name <NEW_NAME>] [--force] [--json]
scuv import -    [--name <NEW_NAME>]                # read from stdin
```

## Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `path` | Yes | Path to the export JSON, or `-` to read from stdin |

## Options

| Option | Description |
|--------|-------------|
| `--name <NAME>` | Override the env name from the file (validated like `scuv create`) |
| `-f`, `--force` | Overwrite an existing environment with the same name |
| `--json` | Output as JSON |

## Behaviour

1. Reads `scoop_export_version` first, then the rest as that version (`1`
   or `2`). Any other version produces a clear `EXPORT_UNSUPPORTED_VERSION`
   error instead of trying to limp on.
2. Applies `--name` override (if any) and validates the resulting name.
3. Checks that every package can be put back where it came from, before
   anything is deleted, installed or created:
   - every local source (`file://` path) exists (`PACKAGE_SOURCE_MISSING`);
   - with `--force` over an existing env, no local source lives inside that
     env or is reached through it, symlinks included
     (`PACKAGE_SOURCE_INSIDE_TARGET`), since deleting the env would remove
     it;
   - an archive with a recorded hash needs uv 0.11.29 or later, the first
     release that checks a URL's `#sha256=` (`PACKAGE_HASH_UNENFORCED`).
   - an origin uv would not reproduce is refused
     (`PACKAGE_ORIGINS_UNREADABLE`): an archive whose recorded hashes
     include no sha256, sha384 or sha512, a local archive with a
     `subdirectory` (uv ignores it there), a local path with `..` (uv folds
     it away without following symlinks, so it could install from a path
     other than the one checked), an editable path with `#`, `[`, `]` or
     `$` (uv reads them as a fragment, extras and a variable), a
     `subdirectory` with anything but ASCII letters, digits and `-._~/+@,=`
     (uv does not decode the fragment it reads it from), a non-editable
     directory named `*.git` (uv would install its committed HEAD), a VCS
     other than git, and uv's own `path` and `git_lfs` selectors. A hash in an archive
     URL's `#fragment`, where uv records it, counts as recorded.
4. If the target env already exists: errors out unless `--force` is set, in
   which case the existing env is removed first.
5. Auto-installs the requested Python if it isn't already available via uv
   (matches the ergonomics of [`scuv sync`](sync.md)).
6. Creates the env, then installs every package in one `uv pip install`:
   `name==version` when no origin is recorded (all of a version 1 file),
   `--editable <path>` for an editable directory, `name @ file://<path>`
   for a directory (its `subdirectory` joined on, since uv ignores
   `#subdirectory=` for a local directory), `name @ <url>` for an archive,
   `name @ git+<url>@<commit>` for a git checkout.
   If that install fails, the env this import created is removed again and
   the install error is reported. An env `--force` removed in step 4 is not
   restored.

## Examples

```bash
# Plain file -> recreates with the schema's original name
scuv import myenv.json

# Pipe from a sibling machine
ssh other 'scuv export myenv' | scuv import -

# Rename on the fly + overwrite if it already exists
scuv import myenv.json --name myenv-2 --force

# Machine-readable summary for CI
scuv import myenv.json --json
```

## JSON Output

```json
{
  "status": "success",
  "command": "import",
  "data": {
    "name": "myenv",
    "python": "3.12",
    "packages_installed": 42,
    "source": "myenv.json"
  }
}
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Imported successfully |
| 1 | Invalid file, unsupported schema version, invalid name, env existed without `--force`, a package source that is missing or inside the env being replaced, a hashed archive on a uv that cannot check it, or a failed package install |

## See Also

- [`scuv export`](export.md) — produce the input file
- [`scuv sync`](sync.md) — for declarative `.scuv.toml`-driven workflows
