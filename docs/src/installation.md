# Installation

## Prerequisites

| Dependency | Version | Install Command |
|------------|---------|-----------------|
| **uv** | Latest | `curl -LsSf https://astral.sh/uv/install.sh \| sh` |
| **Rust** | 1.85+ | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |

## Install via Cargo

```bash
cargo install scoop-uv
```

The binary is installed to `~/.cargo/bin/scoop`.

## Verify Installation

```bash
scoop --version
# scoop 0.2.10
```

## Troubleshooting

### `scoop: command not found`

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

scoop requires uv to be installed and available in PATH. Verify:

```bash
uv --version
```

If not installed, run:

```bash
curl -LsSf https://astral.sh/uv/install.sh | sh
```

## Next Steps

After installation, set up [Shell Integration](shell-integration.md) to enable auto-activation and tab completion.
