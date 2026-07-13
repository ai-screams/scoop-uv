# run

Run a command inside a virtualenv without activating it in the parent shell —
useful for CI, one-shot scripts, and editor integrations.

## Usage

```bash
scuv run <env> [--] <command> [args...]
```

The `--` separator is optional but recommended when `<command>` accepts flags
that might collide with `scuv`'s own flags.

## Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `env` | Yes | Name of the virtualenv |
| `command` | Yes | Program (and arguments) to execute |

## Environment Wiring

The spawned child sees the same vars that `scuv activate` would set:

| Variable | Value |
|----------|-------|
| `VIRTUAL_ENV` | Absolute path of the env |
| `SCUV_ACTIVE` | `<env>` |
| `PATH` | Env's `bin/` prepended to inherited `PATH` |
| `PYTHONHOME` | Removed |

A bare program name (no `/` or `\`) is looked up inside the env's `bin/`
first — so `scuv run env -- python` always picks the env's interpreter, not
a system one. An explicit path (`/usr/bin/python3`) is used verbatim.

## Exit Codes

`scuv run` exits with the child's exit code. On Unix, a child killed by a
signal exits as `128 + signum` (matching what bash exposes via `$?`).

## Examples

```bash
scuv run myenv -- python script.py
scuv run myenv -- pip install requests
scuv run myenv -- pytest -vv tests/
scuv run myenv -- which python   # absolute path inside myenv
```
