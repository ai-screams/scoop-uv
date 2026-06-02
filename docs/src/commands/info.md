# info

Show detailed information about a virtual environment — heavier
sibling of [`scoop status`](status.md). Reads metadata, walks the
directory for size, and shells out to the venv's own `pip` for a
package list.

## Usage

```bash
scoop info <name>
```

## Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `name`   | Yes      | Name of the virtualenv |

## Options

| Option           | Description |
|------------------|-------------|
| `--all-packages` | Show the full installed-package list (default: top 5) |
| `--no-size`      | Skip the directory-size walk |
| `--json`         | Output as JSON |

## Human Output

```
Name:        myproject
Python:      3.12.1
Path:        ~/.scoop/virtualenvs/myproject
Active:      yes
Created:     2026-05-29 12:34:56
Last used:   3 hours ago
Size:        45 MB
Packages:    8
              requests==2.31.0
              ...
```

The `Last used:` row reads `never` for envs whose metadata exists but
have never been activated (`scoop activate` / `scoop run` /
`scoop shell` is what touches it), and is omitted entirely when there
is no on-disk metadata at all.

## JSON Output

```bash
scoop info myproject --json
```

```json
{
  "status": "success",
  "command": "info",
  "data": {
    "name": "myproject",
    "python": "3.12.1",
    "path": "/Users/me/.scoop/virtualenvs/myproject",
    "active": true,
    "created_at": "2026-05-29T12:34:56+00:00",
    "last_used": "2026-06-02T09:00:00+00:00",
    "size_bytes": 47185920,
    "size_display": "45 MB",
    "packages": { "total": 8, "items": [{"name": "requests", "version": "2.31.0"}], "truncated": true }
  }
}
```

`last_used` (RFC 3339) is omitted when the env has never been
activated. `size_bytes` / `size_display` are omitted under `--no-size`.

## Examples

```bash
scoop info myproject              # Default top-5 packages
scoop info myproject --all-packages
scoop info myproject --no-size    # Skip directory-size walk
scoop info myproject --json
```
