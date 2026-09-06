# status

Summarise the current environment in one shot — designed to be fast (no
package listing, no directory size walk). Use [`scuv info`](info.md) for the
heavier per-env view.

## Usage

```bash
scuv status [--json]
```

## States

`status` resolves to one of four states:

| State | Trigger |
|-------|---------|
| `active` | `$SCUV_ACTIVE` is set (shell-activated) |
| `configured` | A `.scuv-version` file or `~/.scuv/version` selects an env |
| `system` | The configured env is the literal name `system` |
| `none` | Nothing resolved |

`$SCUV_ACTIVE` wins over version files because it reflects what the shell
actually activated.

## Source

`source` names where the environment came from, which is not always a file:

| Value | Meaning |
|-------|---------|
| `scuv_active_env` | `$SCUV_ACTIVE` — what the shell activated |
| `env_var` | `SCUV_VERSION` (or the legacy `SCOOP_VERSION`) |
| `version_file` | `.scuv-version` (local or a parent) or `~/.scuv/version` |

Resolution order is `$SCUV_ACTIVE` → `SCUV_VERSION` → version files, so
`env_var` outranks every file. Before 0.15.3 an environment selected by
`SCUV_VERSION` was also reported as `version_file`, which could mislead a
script trying to work out what to change.

## Human Output

For a real env (`active` / `configured`):

```
Name:     myenv
Source:   scuv_active_env
Python:   3.12.1
Path:     ~/.scuv/virtualenvs/myenv
Created:  2026-05-29 12:34:56
Last used:3 hours ago
```

The `Last used:` row reads `never` for envs that have metadata but
have not yet been activated (fresh `scuv create`, or envs whose
metadata predates the field). It's omitted entirely when there's no
metadata at all — that way "we don't know" doesn't get conflated with
"definitely never used".

For `system`: a single line indicating system Python is in use.

For `none`: a hint pointing to `scuv use <name>`.

## JSON Output

```json
{
  "status": "success",
  "command": "status",
  "data": {
    "state": "active",
    "name": "myenv",
    "source": "scuv_active_env",
    "path": "/Users/me/.scuv/virtualenvs/myenv",
    "python": "3.12.1",
    "created_at": "2026-05-29T12:34:56+00:00",
    "last_used": "2026-06-02T09:00:00+00:00"
  }
}
```

Fields are omitted (`skip_serializing_if`) when not applicable to the
state. `last_used` is RFC 3339 and absent in two distinct cases:

* **No metadata at all** (legacy env / metadata file removed) —
  the timestamp is *unknown*. Human output omits the `Last used:`
  row entirely.
* **Metadata present but never activated** since the field landed —
  the timestamp is *known to be never*. Human output renders
  `Last used: never`.

JSON consumers therefore should NOT collapse "missing" to "never";
combine the absence of `last_used` with the presence of `created_at`
to tell the two cases apart.

## Examples

```bash
scuv status                       # human-readable
scuv status --json                # machine-readable
```
