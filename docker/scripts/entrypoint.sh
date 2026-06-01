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
# virtualenvwrapper initialization (full image only).
#
# virtualenvwrapper.sh sources Python helpers that can exit non-zero in
# headless containers (no $WORKON_HOME, missing optional virtualenv
# clones, etc.). Without `set +e` the entrypoint aborted with exit 127
# under the `set -e` umbrella, even though the init failure was benign —
# this was the original root cause of the venvwrapper CI shard hanging.
# ============================================================
export WORKON_HOME="/root/.virtualenvs"
mkdir -p "$WORKON_HOME"
if command -v virtualenvwrapper.sh &>/dev/null; then
  set +e
  # shellcheck disable=SC1090
  source "$(which virtualenvwrapper.sh)" 2>/dev/null
  set -e
fi

# ============================================================
# scoop build (workspace version takes precedence)
# ============================================================
if [ -d "/workspace/src" ]; then
  SCOOP_BIN="/workspace/target/release/scoop"

  # Build scoop if binary doesn't exist or source is newer
  if [ ! -f "$SCOOP_BIN" ] || \
     [ "$(find /workspace/src -name '*.rs' -newer "$SCOOP_BIN" 2>/dev/null | head -1)" ] || \
     [ "$(find /workspace/locales -name '*.yml' -newer "$SCOOP_BIN" 2>/dev/null | head -1)" ]; then
    echo "Building scoop..."
    cargo build --release --manifest-path=/workspace/Cargo.toml
  fi
fi

# ============================================================
# Execute command
# ============================================================
exec "$@"
