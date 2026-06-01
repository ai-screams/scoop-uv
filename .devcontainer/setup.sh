#!/usr/bin/env bash
# scoop-uv devcontainer setup — runs in `onCreateCommand` (inside Codespace
# prebuild or first local create). Every step is idempotent so this script
# is safe to re-run on container rebuild.
set -euo pipefail

echo "==> rustup components (pinned by rust-toolchain.toml)"
# rust-toolchain.toml already lists rustfmt + clippy. Add rust-src and
# rust-analyzer so IDE features work out of the box on Codespaces.
rustup component add rust-src rust-analyzer rustfmt clippy

echo "==> cargo tools (locked)"
# Each install is gated with `|| true` so a previously-installed binary
# at the same version doesn't fail the script. `--locked` keeps these
# reproducible — the tools' own Cargo.lock files are used, not ours.
cargo install --locked cargo-nextest 2>/dev/null || true
cargo install --locked cargo-llvm-cov 2>/dev/null || true
cargo install --locked cargo-mutants 2>/dev/null || true

echo "==> uv (scoop-uv's backend dependency)"
# The CLI under development wraps uv; install via the official script so
# the version tracks upstream's latest stable. MIN_VERSION in
# src/uv/version.rs is the floor we enforce against.
curl -LsSf https://astral.sh/uv/install.sh | sh

echo "==> warm cargo build (populates target/ in the prebuild snapshot)"
# Failures here aren't fatal — if Cargo.lock is mid-edit on the user's
# branch the prebuild can still succeed and the first `cargo test` in
# the Codespace will rebuild as needed.
cargo build 2>/dev/null || true

echo "==> done. Codespace is ready."
