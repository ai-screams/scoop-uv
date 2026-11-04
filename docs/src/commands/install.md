# install

Install a Python version.

## Usage

```bash
scuv install [version] [options]
```

## Arguments

| Argument | Required | Default | Description |
|----------|----------|---------|-------------|
| `version` | No | `3` | Python version (e.g., `3.12`, `3.11.8`) |

## Options

| Option | Description |
|--------|-------------|
| `--latest` | Ask uv for Python 3 (default) |
| `--stable` | Install the oldest Python still receiving security fixes (3.11) |
| `--json` | Output result as JSON |

## What gets installed where

The interpreter itself lands in uv's own directory
(`~/.local/share/uv/python/...`); scuv records nothing about it beyond what
`uv python list` reports.

Since uv 0.8.0 there is a second effect worth knowing about: `uv python
install` also links a **versioned executable onto your `PATH`** — typically
`~/.local/bin/python3.13` — so the interpreter is reachable without going
through a virtual environment at all.

```console
$ ls -l ~/.local/bin/python3.14
~/.local/bin/python3.14 -> ~/.local/share/uv/python/cpython-3.14.../bin/python3.14
```

That executable points at the bare interpreter: standard library only, and you
cannot install packages into it. Use a scuv environment for anything past a
quick `python3.14 -c '...'`.

uv itself accepts `--no-bin` to skip the link, but `scuv install` does not
currently forward flags to uv, so there is no way to opt out through scuv. Run
`uv python install --no-bin <version>` directly if you need that.

## Version Resolution

- No argument or `--latest`: asks uv for `3`. A Python 3 that uv already manages satisfies it; with none, uv downloads its default (3.15 since uv 0.13)
- `--stable`: installs Python 3.11, the oldest still receiving security fixes (until 2027-10)
- `3.12`: installs the latest 3.12.x patch when uv manages no 3.12 yet
- `3.12.3`: installs exact version

uv skips the download when a version it manages already matches the request, so
`scuv install 3.12` does not move an existing 3.12 to a newer patch. Use
`uv python install --upgrade 3.12` for that.

## Examples

```bash
scuv install                    # Python 3 (a uv-managed one counts)
scuv install --latest           # Same as above
scuv install --stable           # Install Python 3.11
scuv install 3.12               # Latest 3.12.x, unless uv already manages a 3.12
scuv install 3.12.3             # Install exact 3.12.3
```

> **Note:** Python versions are managed by [uv](https://github.com/astral-sh/uv).

## Python Discovery

You don't always need `scuv install`. When you run `scuv create`, uv searches for a matching Python in this order:

1. **uv-managed** — installed via `scuv install` or `uv python install`
2. **System PATH** — Homebrew, apt, pyenv, or any Python on your `PATH`
3. **Platform-specific** — Windows registry, Microsoft Store

```bash
# See all Python versions uv can find
uv python list

# Example output:
# cpython-3.13.1    /opt/homebrew/bin/python3.13     (system)
# cpython-3.12.8    ~/.local/share/uv/python/...     (managed)
# cpython-3.11.5    /usr/bin/python3.11               (system)

# Use system Python directly — no scuv install needed
scuv create myenv 3.13
```

If the requested version isn't found anywhere, `scuv create` will fail with an error. Use `scuv install <version>` to download it first.

> **See also:** [Python Management](../python-management.md) for custom Python paths, environment variables, and migration.
