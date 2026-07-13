# Installation

## Prerequisites

| Dependency | Version | Install Command |
|------------|---------|-----------------|
| **uv** | 0.5.14 or newer | `curl -LsSf https://astral.sh/uv/install.sh \| sh` |
| **Rust** | 1.85+ | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |

## Install via Cargo

```bash
cargo install scoop-uv
```

The binary is installed to `~/.cargo/bin/scuv`.

## Upgrade

To upgrade scuv to the latest version:

```bash
cargo install scoop-uv
```

This overwrites the existing binary in `~/.cargo/bin/scuv`. Your virtual environments in `~/.scuv/` are preserved.

Verify the upgrade:

```bash
scuv --version
```

### Upgrading from scoop (≤ 0.14.x)

The CLI command was renamed in v0.15.0 (`scoop` → `scuv`). One-time migration:

```bash
scoop self update        # installs the new `scuv` binary
                         # (the "could not locate the freshly installed
                         #  `scoop` binary" warning is expected)
rm -f ~/.cargo/bin/scoop # remove the old binary if cargo left one behind
mv ~/.scoop ~/.scuv      # move your environments
```

Then update your shell rc file — replace `eval "$(scoop init <shell>)"` with
`eval "$(scuv init <shell>)"` (fish: `scuv init fish | source`), restart your
shell, and run `scuv doctor` to confirm nothing legacy is left over.

Legacy `SCOOP_*` env vars and `.scoop-version` / `.scoop.toml` files keep
working with a one-shot deprecation warning until v0.16.0. Don't skip the
`rm` step: a leftover old binary keeps running 0.14.x silently.

## Verify Installation

```bash
scuv --version
# scuv 0.15.0
```

## Troubleshooting

### `scuv: command not found`

Ensure `~/.cargo/bin` is in your PATH:

```bash
# Add to ~/.zshrc or ~/.bashrc
export PATH="$HOME/.cargo/bin:$PATH"
```

Then restart your terminal or run:

```bash
source ~/.zshrc  # or ~/.bashrc
```

### uv not found

scuv requires uv to be installed and available in PATH. Verify:

```bash
uv --version
```

If not installed, run:

```bash
curl -LsSf https://astral.sh/uv/install.sh | sh
```

## Next Steps

After installation, set up [Shell Integration](shell-integration.md) to enable auto-activation and tab completion.
