# export

Write a JSON snapshot of an environment so another machine (or another
teammate) can recreate it with [`scuv import`](import.md). Each package
records where it came from, so an editable, local-path or git package is put
back from the same place instead of from the registry.

## Usage

```bash
scuv export <name> [-o <PATH>]
```

## Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `name` | Yes | Name of the environment to export |

## Options

| Option | Description |
|--------|-------------|
| `-o`, `--output <PATH>` | Write to this file instead of stdout |

When `-o` is omitted, the JSON document is written to **stdout** and status
messages stay on stderr. That keeps the command pipe-friendly:

```bash
scuv export myenv > myenv.json
scuv export myenv | jq '.packages | length'
```

## Schema

The exported file is versioned (`scoop_export_version`) so a format change
is detected cleanly rather than silently mis-parsed. `scuv export` writes
version `2`; `scuv import` reads `1` and `2`.

```json
{
  "scoop_export_version": "2",
  "environment": {
    "name": "myproject",
    "python": "3.12",
    "created_at": "2026-05-29T12:34:56.762998+00:00"
  },
  "packages": [
    {
      "name": "mylib",
      "version": "0.3.0",
      "source": { "url": "file:///home/me/src/mylib", "dir_info": { "editable": true } }
    },
    { "name": "six", "version": "1.17.0", "source": null },
    {
      "name": "tool",
      "version": "1.2.0",
      "source": {
        "url": "https://github.com/org/tool.git",
        "vcs_info": { "vcs": "git", "commit_id": "8013f4a4…" }
      }
    }
  ]
}
```

Field notes:
- `packages` lists every installed package, dependencies included, sorted
  by name.
- `environment.python` is the version recorded in the env's metadata, which
  comes from `version_info` in the env's `pyvenv.cfg`. Current uv writes the
  minor version there (`3.12`); envs made with `--python-path` pointing at an
  interpreter uv does not manage record its full version (e.g. `3.14.8`).
- `environment.created_at` is RFC 3339 and may be absent for hand-authored or
  pre-metadata exports.
- `packages` is what `uv pip list` reports for the env, versions pinned
  exactly.
- `source` is the package's [PEP 610](https://packaging.python.org/en/latest/specifications/direct-url/)
  origin, as its installer recorded it in `direct_url.json`: `dir_info` for a
  local directory (`"editable": true` for `uv pip install -e`), `vcs_info`
  for a VCS checkout (pinned to `commit_id`), `archive_info` for a wheel or
  sdist URL (with its hashes, when recorded). `null` means no direct origin
  was recorded, which is usually a registry install; it does not say which
  index. An editable install made with an old pip (`setup.py develop`, an
  `.egg-link`) records no origin; scuv writes a `dir_info` one for it from
  the project location uv reports.
- scuv reads origins by running the env's own Python (3.8 or later) with
  `importlib.metadata`. The env's site hooks run as part of that, as they do
  whenever the env's Python starts.

A local `file://` source is an absolute path on the machine that exported
it. Importing on another machine needs the same path there; scuv checks
before creating anything and names the package whose path is missing. The
export does not contain the source code itself.

When the project lives somewhere else on the new machine, change that
package's `source.url` to the new place of the directory it names, written
the way export writes it: `file://` and the absolute path with forward
slashes, with `#`, `?` and `%` written as `%23`, `%3F` and `%25`
(`file:///home/me/code/mylib`, or `file:///C:/code/mylib` on Windows; a
space can stay as typed). An editable package cannot come from a path with
a `#` at all (see [`scuv import`](import.md#behaviour)). A `subdirectory`
in the same `source` stays relative to that directory. Then import again. scuv reports one
missing path at a time, so repeat for each local package it names.

Version 1 files (scuv 0.21 and older) have no `source`; importing one
reinstalls every package from the registry, as before. scuv 0.21 and older
refuse version 2 files.

> **Note:** The export schema intentionally does **not** include the `last_used`
> timestamp. `last_used` is local usage telemetry — it describes how
> you've been using *this* env, not what an importer needs to recreate
> it elsewhere. Imported envs start fresh with no `last_used` and the
> field populates the first time the new env is activated locally.

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Export written successfully (or printed to stdout) |
| 1 | Env not found, listing its packages with uv failed, reading their origins failed (`PACKAGE_ORIGINS_UNREADABLE`), or writing the destination file failed |
| 3 | The env has no readable metadata, so there is no Python version to record |

scuv writes nothing when listing the packages or reading their origins
fails; it never exports an empty package list, or a package without the
origin it has, in place of one it could not read. A shell redirect
(`scuv export myenv > myenv.json`) still creates or truncates the file
before scuv runs.

## See Also

- [`scuv import`](import.md) — reverse operation
- [`scuv sync`](sync.md) — declarative manifest with looser pinning
