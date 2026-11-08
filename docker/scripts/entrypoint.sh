#!/bin/bash
# Container entrypoint script
# Initializes pyenv, conda, and virtualenvwrapper.
#
# Robustness note: third-party shell init scripts (pyenv-virtualenv-init,
# conda hooks, virtualenvwrapper) run internal commands that can return
# non-zero in fresh containers (no existing venvs, missing optional
# python modules, etc.). When the calling shell has `set -e`, any such
# internal failure aborts the whole entrypoint with exit 127/1 — even
# though the init "failure" is benign. We wrap every third-party `source`
# / `eval` block with `set +e` to neutralise that, and re-enable `set -e`
# for our own commands so a real bug here still surfaces.

set -e

# ============================================================
# Rust and uv initialization (official image uses /usr/local/cargo)
# ============================================================
export PATH="/usr/local/cargo/bin:/root/.cargo/bin:/root/.local/bin:$PATH"

# ============================================================
# pyenv initialization — lenient: third-party init may exit non-zero
# in fresh containers; that's not our bug.
# ============================================================
export PYENV_ROOT="/root/.pyenv"
export PATH="$PYENV_ROOT/bin:$PYENV_ROOT/shims:$PATH"

if command -v pyenv &>/dev/null; then
  set +e
  eval "$(pyenv init -)"
  eval "$(pyenv virtualenv-init -)" 2>/dev/null
  set -e
fi

# ============================================================
# conda initialization (full image only) — lenient for same reason.
# ============================================================
if [ -d "/opt/conda" ]; then
  set +e
  eval "$(/opt/conda/bin/conda shell.bash hook)" 2>/dev/null
  set -e
fi

# ============================================================
# virtualenvwrapper setup — env vars only, do NOT source the script.
#
# An earlier version sourced `$(which virtualenvwrapper.sh)` here, with
# `set +e` around it, and the venvwrapper integration CI shard died with
# exit 127 in 0.4s before the entrypoint printed anything. `which`
# finds pyenv's shim, a script that ends in `exec pyenv exec ...`:
# sourcing it replaces this shell, so neither `set +e` nor `|| true`
# can contain it, and the `exec "$@"` at the end never runs.
#
# Sourcing isn't actually needed here: `scuv migrate list`
# discovers virtualenvwrapper envs by enumerating $WORKON_HOME,
# which doesn't require the `mkvirtualenv`/`workon` shell functions
# to be loaded. Interactive shells load /root/.bashrc (and the
# compose-mounted .zshrc), which source it through `pyenv which`.
# ============================================================
export WORKON_HOME="/root/.virtualenvs"
mkdir -p "$WORKON_HOME"

# ============================================================
# scuv build (workspace version takes precedence)
#
# SCUV_ENTRYPOINT_BUILD=0 skips it: CI's integration run compiles the
# workspace itself (nextest builds the bin with the tests), and a second
# release build here only doubled the job's compile time.
# ============================================================
if [ "${SCUV_ENTRYPOINT_BUILD:-1}" != "0" ] && [ -d "/workspace/src" ]; then
  SCUV_BIN="/workspace/target/release/scuv"

  # Build scuv if binary doesn't exist or source is newer
  if [ ! -f "$SCUV_BIN" ] || \
     [ "$(find /workspace/src -name '*.rs' -newer "$SCUV_BIN" 2>/dev/null | head -1)" ] || \
     [ "$(find /workspace/locales -name '*.yml' -newer "$SCUV_BIN" 2>/dev/null | head -1)" ]; then
    echo "Building scuv..."
    cargo build --release --manifest-path=/workspace/Cargo.toml
  fi
fi

# ============================================================
# Execute command
# ============================================================
exec "$@"
