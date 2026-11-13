# clone

Duplicate an environment — same Python version, same packages by default —
without going through an export/import roundtrip.

## Usage

```bash
scuv clone <SRC> <DST> [--no-packages] [--force] [--json]
```

## Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `src` | Yes | Name of the source environment |
| `dst` | Yes | Name for the new environment |

## Options

| Option | Description |
|--------|-------------|
| `--no-packages` | Skip package copy — create an empty env at the same Python version |
| `-f`, `--force` | Overwrite the destination if it already exists |
| `--json` | Output as JSON |

## Behaviour

1. Validates `<DST>` (rejects reserved names like `list`, `clone`, …).
2. Refuses a self-clone: the same name, or two names for one directory
   (`Web` and `web` on a case-insensitive filesystem).
3. Refuses an existing `<DST>` unless `--force`.
4. Resolves `<SRC>`'s recorded Python version from its metadata; surfaces a
   clear `CorruptedEnvironment` error when metadata is missing so you know
   recreate-from-scratch is the right next step.
5. Unless `--no-packages`, lists the source's installed packages with
   `uv pip list` and reads where each came from (its PEP 610
   `direct_url.json`, through the source's own Python 3.8+; see
   [`scuv export`](export.md#schema)). A failed listing, or a package whose
   origin cannot be read or matched (`PACKAGE_ORIGINS_UNREADABLE`), stops
   the clone instead of copying a guess.
6. Checks that every package can be put back: each local source path
   exists (a directory being that package's project), none lies inside
   `<DST>` or is reached through it when `--force`
   would delete it, and a hashed archive has a uv (0.11.29+) that checks
   the hash. See [`scuv import`](import.md#behaviour) for the error codes.
7. Only now does `--force` remove the existing `<DST>`, so a source that
   cannot be cloned leaves it in place.
8. Creates `<DST>` at the same Python version and installs the packages in
   one `uv pip install`, each from where it came: `name==version` from the
   registry, `--editable <path>` for an editable project (the same working
   tree, shared with the source), `name @ <url>` for a local directory or
   archive, `name @ git+<url>@<commit>` for a git checkout. If the install
   fails, scuv removes the `<DST>` it just created (best effort) and
   reports the install error.

`--force` is not transactional: once step 7 has removed the old `<DST>`, a
failure in step 8 does not bring it back.

## Examples

```bash
# Full copy
scuv clone myenv myenv-experiment

# Just the shell, no packages
scuv clone myenv myenv-clean --no-packages

# Replace an existing clone
scuv clone myenv myenv-experiment --force
```

## JSON Output

```json
{
  "status": "success",
  "command": "clone",
  "data": {
    "src": "myenv",
    "dst": "myenv-experiment",
    "python": "3.12",
    "path": "/Users/me/.scuv/virtualenvs/myenv-experiment",
    "packages_copied": 12,
    "packages_skipped": false
  }
}
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Cloned successfully |
| 1 | Invalid dst name, self-clone, src missing, dst exists without `--force`, a package origin that cannot be read or put back, or a uv failure (listing, creating, installing) |
| 3 | src is corrupted (no readable metadata) |

## See Also

- [`scuv export`](export.md) / [`scuv import`](import.md) — portable JSON
  for cross-machine duplication
- [`scuv create --force`](create.md) — recreate from scratch with a specific
  Python version
