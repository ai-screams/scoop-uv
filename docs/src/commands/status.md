# status

Summarise the current environment in one shot — designed to be fast (no
package listing, no directory size walk). Use [`scoop info`](info.md) for the
heavier per-env view.

## Usage

```bash
scoop status [--json]
```

## States

`status` resolves to one of four states:

| State | Trigger |
|-------|---------|
| `active` | `$SCOOP_ACTIVE` is set (shell-activated) |
| `configured` | A `.scoop-version` file or `~/.scoop/version` selects an env |
| `system` | The configured env is the literal name `system` |
| `none` | Nothing resolved |

`$SCOOP_ACTIVE` wins over version files because it reflects what the shell
actually activated.

## Human Output

For a real env (`active` / `configured`):

```
Name:     myenv
Source:   scoop_active_env
Python:   3.12.1
Path:     ~/.scoop/virtualenvs/myenv
Created:  2026-05-29 12:34:56
Last used:3 hours ago
```

The `Last used:` row reads `never` for envs that have metadata but
have not yet been activated (fresh `scoop create`, or envs whose
metadata predates the field). It's omitted entirely when there's no
metadata at all — that way "we don't know" doesn't get conflated with
"definitely never used".

For `system`: a single line indicating system Python is in use.

For `none`: a hint pointing to `scoop use <name>`.

## JSON Output

```json
{
  "status": "success",
  "command": "status",
  "data": {
    "state": "active",
    "name": "myenv",
    "source": "scoop_active_env",
    "path": "/Users/me/.scoop/virtualenvs/myenv",
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
scoop status                       # human-readable
scoop status --json                # machine-readable
```
