#!/bin/bash
# Container entrypoint script
# Initializes pyenv, conda, and virtualenvwrapper

set -e

# ============================================================
# Rust and uv initialization (official image uses /usr/local/cargo)
# ============================================================
export PATH="/usr/local/cargo/bin:/root/.cargo/bin:/root/.local/bin:$PATH"

# ============================================================
# pyenv initialization
# ============================================================
export PYENV_ROOT="/root/.pyenv"
export PATH="$PYENV_ROOT/bin:$PYENV_ROOT/shims:$PATH"

if command -v pyenv &>/dev/null; then
  eval "$(pyenv init -)"
  eval "$(pyenv virtualenv-init -)" 2>/dev/null || true
fi

# ============================================================
# conda initialization (full image only)
# ============================================================
if [ -d "/opt/conda" ]; then
  eval "$(/opt/conda/bin/conda shell.bash hook)" 2>/dev/null || true
fi

# ============================================================
# virtualenvwrapper initialization (full image only)
# ============================================================
export WORKON_HOME="/root/.virtualenvs"
if command -v virtualenvwrapper.sh &>/dev/null; then
  source "$(which virtualenvwrapper.sh)" 2>/dev/null || true
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
