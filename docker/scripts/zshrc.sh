# scoop zsh config

# Completion system (required for scoop)
autoload -Uz compinit && compinit

# ============================================================
# pyenv
# ============================================================
export PYENV_ROOT="$HOME/.pyenv"
export PATH="$PYENV_ROOT/bin:$PYENV_ROOT/shims:$PATH"
eval "$(pyenv init -)"
eval "$(pyenv virtualenv-init -)" 2>/dev/null || true

# ============================================================
# conda (if installed)
# ============================================================
if [ -d "/opt/conda" ]; then
    eval "$(/opt/conda/bin/conda shell.zsh hook)" 2>/dev/null || true
fi

# ============================================================
# virtualenvwrapper (if installed)
# ============================================================
export WORKON_HOME="/root/.virtualenvs"
if command -v virtualenvwrapper.sh &> /dev/null; then
    source "$(which virtualenvwrapper.sh)" 2>/dev/null || true
fi

# ============================================================
# Rust
# ============================================================
export PATH="/usr/local/cargo/bin:/root/.cargo/bin:$PATH"

# ============================================================
# uv
# ============================================================
export PATH="/root/.local/bin:$PATH"

# ============================================================
# scoop (workspace build takes precedence over /usr/local/bin)
# ============================================================
if [ -d "/workspace/target/release" ]; then
    export PATH="/workspace/target/release:$PATH"
    # Initialize shell integration if available
    if [ -x "/workspace/target/release/scoop" ]; then
        eval "$(/workspace/target/release/scoop init zsh)" 2>/dev/null || true
    fi
fi

# ============================================================
# Development settings
# ============================================================
export CARGO_TERM_COLOR=always
export RUST_BACKTRACE=1

# ============================================================
# Prompt
# ============================================================
PROMPT="%F{cyan}[scoop-test]%f %~ $ "

# ============================================================
# Aliases
# ============================================================
alias ll='ls -la'
alias t='cargo test'
alias tb='cargo test --lib'
alias c='cargo check'
alias cb='cargo build'
alias cr='cargo run --'
