# info

Show detailed information about a virtual environment — heavier
sibling of [`scuv status`](status.md). Reads metadata, walks the
directory for size, and runs `uv pip list` against the env for a
package list.

## Usage

```bash
scuv info <name>
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
Name:       myproject
Python:     3.12
Path:       ~/.scuv/virtualenvs/myproject
Active:     no
Created:    2026-05-29 12:34:56
Last used:  9 minutes ago
Size:       8 MB
Packages:   10
            certifi==2026.7.22
            charset-normalizer==3.5.2
            idna==3.20
            markdown-it-py==4.2.0
            mdurl==0.1.2
            ... (5 more)
```

Packages are listed in name order. `... (N more)` closes a truncated list;
`--all-packages` prints every package instead.

The `Last used:` row reads `never` for envs whose metadata exists but
have never been activated (`scuv activate` / `scuv run` /
`scuv shell` is what touches it), and is omitted entirely when there
is no on-disk metadata at all.

## JSON Output

```bash
scuv info myproject --json
```

```json
{
  "status": "success",
  "command": "info",
  "data": {
    "name": "myproject",
    "python": "3.12",
    "path": "/Users/me/.scuv/virtualenvs/myproject",
    "active": false,
    "created_at": "2026-05-29T12:34:56.375271+00:00",
    "last_used": "2026-06-02T09:00:00.746201+00:00",
    "size_bytes": 8575712,
    "size_display": "8 MB",
    "packages": {
      "total": 10,
      "items": [
        { "name": "certifi", "version": "2026.7.22" },
        { "name": "charset-normalizer", "version": "3.5.2" },
        { "name": "idna", "version": "3.20" },
        { "name": "markdown-it-py", "version": "4.2.0" },
        { "name": "mdurl", "version": "0.1.2" }
      ],
      "truncated": true
    }
  }
}
```

`last_used` (RFC 3339) is omitted when the env has never been
activated. `size_bytes` / `size_display` are omitted under `--no-size`.

## Examples

```bash
scuv info myproject              # Default top-5 packages
scuv info myproject --all-packages
scuv info myproject --no-size    # Skip directory-size walk
scuv info myproject --json
```
