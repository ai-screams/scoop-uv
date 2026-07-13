# man

Generate Unix man pages from scuv's `clap::Command` tree. Because they're rendered from the live CLI definition, the man pages always reflect the actual `--help` text — no separate documentation to keep in sync.

## Usage

```bash
# Print the top-level scuv.1 to stdout
scuv man

# Preview with `man -l`
scuv man | man -l -

# Write scuv.1 + scuv-<sub>.1 (one per subcommand) into a directory
scuv man /tmp/scuv-man
```

## Arguments

| Argument | Description |
|----------|-------------|
| `[DIR]` | Write `scuv.1` + `scuv-<sub>.1` files into this directory. Omit to print the top-level page to stdout. |

## Options

| Option | Description |
|--------|-------------|
| `--json` | Output as JSON (only meaningful with `DIR`) |

## Packager usage

Distro packagers can wire this into their build recipe:

```bash
# In your build script
mkdir -p $PKG_DIR/usr/share/man/man1
./scuv man $PKG_DIR/usr/share/man/man1
gzip -9 $PKG_DIR/usr/share/man/man1/*.1
```

Hidden subcommands (`activate`, `deactivate`, `resolve` — internal to the shell wrapper) are intentionally **not** rendered: they're not user-facing.

## See also

- [`completions`](completions.md) — shell completion scripts (also generated from `clap::Command`)
