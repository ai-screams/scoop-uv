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
```

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
    "created_at": "2026-05-29T12:34:56+00:00"
  }
}
```

Fields are omitted (`skip_serializing_if`) when not applicable to the state.

## Examples

```bash
scoop status                       # human-readable
scoop status --json                # machine-readable
```
