# scoop fish config

# ============================================================
# pyenv
# ============================================================
set -gx PYENV_ROOT $HOME/.pyenv
set -gx PATH $PYENV_ROOT/bin $PYENV_ROOT/shims $PATH
pyenv init - | source
pyenv virtualenv-init - | source 2>/dev/null; or true

# ============================================================
# conda (if installed)
# ============================================================
if test -d /opt/conda
    eval (/opt/conda/bin/conda shell.fish hook) 2>/dev/null; or true
end

# ============================================================
# virtualenvwrapper (if installed)
# ============================================================
set -gx WORKON_HOME /root/.virtualenvs

# ============================================================
# Rust
# ============================================================
set -gx PATH /usr/local/cargo/bin /root/.cargo/bin $PATH

# ============================================================
# uv
# ============================================================
set -gx PATH /root/.local/bin $PATH

# ============================================================
# scoop (workspace build takes precedence over /usr/local/bin)
# ============================================================
if test -d /workspace/target/release
    set -gx PATH /workspace/target/release $PATH
    # Initialize shell integration if available
    if test -x /workspace/target/release/scoop
        /workspace/target/release/scoop init fish | source 2>/dev/null; or true
    end
end

# ============================================================
# Development settings
# ============================================================
set -gx CARGO_TERM_COLOR always
set -gx RUST_BACKTRACE 1

# ============================================================
# Prompt
# ============================================================
function fish_prompt
    echo -n (set_color cyan)"[scoop-test]"(set_color normal)" "(prompt_pwd)" \$ "
end

# ============================================================
# Aliases
# ============================================================
alias ll='ls -la'
alias t='cargo test'
alias tb='cargo test --lib'
alias c='cargo check'
alias cb='cargo build'
alias cr='cargo run --'
