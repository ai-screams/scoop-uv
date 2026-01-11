#!/bin/bash
# Shell configuration for scoop test container

# ============================================================
# pyenv
# ============================================================
export PYENV_ROOT="/root/.pyenv"
export PATH="$PYENV_ROOT/bin:$PYENV_ROOT/shims:$PATH"
eval "$(pyenv init -)"
eval "$(pyenv virtualenv-init -)" 2>/dev/null || true

# ============================================================
# conda (if installed)
# ============================================================
if [ -d "/opt/conda" ]; then
    eval "$(/opt/conda/bin/conda shell.bash hook)"
fi

# ============================================================
# virtualenvwrapper (if installed)
# ============================================================
export WORKON_HOME="/root/.virtualenvs"
if command -v virtualenvwrapper.sh &> /dev/null; then
    source "$(which virtualenvwrapper.sh)"
fi

# ============================================================
# Rust
# ============================================================
export PATH="/root/.cargo/bin:$PATH"

# ============================================================
# uv
# ============================================================
export PATH="/root/.local/bin:$PATH"

# ============================================================
# Development settings
# ============================================================
export CARGO_TERM_COLOR=always
export RUST_BACKTRACE=1

# ============================================================
# Prompt
# ============================================================
PS1='\[\033[1;36m\][scoop-test]\[\033[0m\] \w $ '

# ============================================================
# Aliases
# ============================================================
alias ll='ls -la'
alias t='cargo test'
alias tb='cargo test --lib'
alias c='cargo check'
alias cb='cargo build'
alias cr='cargo run --'
