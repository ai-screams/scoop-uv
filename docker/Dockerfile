# syntax=docker/dockerfile:1.10
# ============================================================
# Multi-stage Dockerfile for scuv test environment
# Layout:
#   base → pyenv → python → scuv-build → shells → scuv-test-base
#                                                    ├── pyenv-test
#                                                    ├── conda-test
#                                                    ├── venvwrapper-test
#                                                    ├── slim   (alias-ish)
#                                                    └── full   (all fixtures)
#
# Best practices:
#   - BuildKit cache mounts (apt, pyenv, cargo registry/git)
#   - CARGO_HOME / RUSTUP_HOME stay at /usr/local/* (rust: image default) so
#     compose can persist them as named volumes for `make docker-shell`
#   - Miniforge (MIT, conda-forge default) instead of Miniconda — no
#     Anaconda commercial-licensing concerns and a smaller installer
#   - Per-source leaf stages let CI matrix-build just one variant
# ============================================================

# ============================================================
# Build Arguments — single source of truth
# ============================================================
ARG RUST_VERSION=1.85
ARG PYTHON_311_VERSION=3.11.14
ARG PYTHON_312_VERSION=3.12.12
ARG UV_VERSION=0.11.28
# Miniforge version — pinned for reproducible builds. Bump when a new
# release adds a feature we need; verify the tag exists at
# https://github.com/conda-forge/miniforge/releases.
ARG MINIFORGE_VERSION=26.3.2-2

# ============================================================
# Stage: base — system dependencies and tools
# ============================================================
FROM rust:${RUST_VERSION}-bookworm AS base

# Prevent interactive prompts. Note: CARGO_HOME and RUSTUP_HOME are
# intentionally NOT overridden here — the official rust: image already
# sets them to /usr/local/cargo and /usr/local/rustup, which are the
# paths compose persists as named volumes.
ENV DEBIAN_FRONTEND=noninteractive \
    PYENV_ROOT="/root/.pyenv" \
    LANG=C.UTF-8 \
    LC_ALL=C.UTF-8

# Install system dependencies with cache mount
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    apt-get update && apt-get install -y --no-install-recommends \
    # Python build dependencies
    build-essential libssl-dev zlib1g-dev \
    libbz2-dev libreadline-dev libsqlite3-dev \
    libncursesw5-dev tk-dev libxml2-dev \
    libxmlsec1-dev libffi-dev liblzma-dev \
    # Shells
    zsh fish \
    # Tools
    git curl wget ca-certificates \
    procps htop \
    && rm -rf /var/lib/apt/lists/*

# ============================================================
# Stage: pyenv — install pyenv and pyenv-virtualenv
# ============================================================
FROM base AS pyenv

# Shallow-clone keeps the layer small.
RUN git clone --depth 1 https://github.com/pyenv/pyenv.git "$PYENV_ROOT" \
    && git clone --depth 1 https://github.com/pyenv/pyenv-virtualenv.git \
        "$PYENV_ROOT/plugins/pyenv-virtualenv"

ENV PATH="$PYENV_ROOT/bin:$PYENV_ROOT/shims:$PATH"

RUN pyenv --version

# ============================================================
# Stage: python — install Python versions (SLOW — cache this!)
# ============================================================
FROM pyenv AS python

ARG PYTHON_311_VERSION
ARG PYTHON_312_VERSION

# 10-20 minutes from cold; the cache mount on /root/.pyenv/cache holds
# the downloaded tarballs so a rebuild after Python version change is fast.
RUN --mount=type=cache,target=/root/.pyenv/cache,sharing=locked \
    pyenv install ${PYTHON_311_VERSION} && \
    pyenv install ${PYTHON_312_VERSION} && \
    pyenv global ${PYTHON_312_VERSION}

RUN python --version && pip --version

# ============================================================
# Stage: scuv-build — compile the scuv binary
# ============================================================
FROM python AS scuv-build

RUN rustup default stable

COPY Cargo.toml Cargo.lock /build/scuv/
COPY src /build/scuv/src
# `[[bench]]` entries in Cargo.toml reference files in benches/. Cargo
# parses the manifest before compiling anything, so the bench sources
# must exist on disk even when we only `cargo build` the binary.
COPY benches /build/scuv/benches

WORKDIR /build/scuv

# Cache cargo registry, git, and target inside one RUN so the binary
# can be cp'd out before the cache mount unmounts.
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,target=/build/scuv/target,sharing=locked \
    cargo build --release \
    && cp target/release/scuv /usr/local/bin/scuv

RUN scuv --version

# ============================================================
# Stage: shells — zsh and fish base configs
# ============================================================
FROM scuv-build AS shells

# Basic zsh configuration
RUN echo '# scuv zsh config\n\
\n\
# Completion system (required for scuv)\n\
autoload -Uz compinit && compinit\n\
\n\
export PYENV_ROOT="$HOME/.pyenv"\n\
export PATH="$PYENV_ROOT/bin:$PYENV_ROOT/shims:$PATH"\n\
eval "$(pyenv init -)"\n\
eval "$(pyenv virtualenv-init -)"\n\
eval "$(scuv init zsh)"\n\
\n\
# Prompt\n\
PROMPT="%F{cyan}[scuv-test]%f %~ $ "\n\
' > /root/.zshrc

# Basic fish configuration
RUN mkdir -p /root/.config/fish \
    && echo '# scuv fish config\n\
set -gx PYENV_ROOT $HOME/.pyenv\n\
set -gx PATH $PYENV_ROOT/bin $PYENV_ROOT/shims $PATH\n\
pyenv init - | source\n\
pyenv virtualenv-init - | source\n\
scuv init fish | source\n\
\n\
# Prompt\n\
function fish_prompt\n\
    echo -n (set_color cyan)"[scuv-test]"(set_color normal)" "(prompt_pwd)" \$ "\n\
end\n\
' > /root/.config/fish/config.fish

# ============================================================
# Stage: scuv-test-base — common test scaffolding
# (uv, entrypoint, workdir; NO per-source fixtures)
#
# Per-source leaf stages extend this so each variant only carries the
# tooling it actually tests. CI matrix-builds just the leaf it needs.
# ============================================================
FROM shells AS scuv-test-base

ARG UV_VERSION
RUN curl -LsSf "https://astral.sh/uv/${UV_VERSION}/install.sh" | sh
ENV PATH="/root/.local/bin:$PATH"

COPY docker/scripts/entrypoint.sh /entrypoint.sh
COPY docker/scripts/bashrc.sh /root/.bashrc
RUN chmod +x /entrypoint.sh

WORKDIR /workspace

ENTRYPOINT ["/entrypoint.sh"]
CMD ["bash", "-l"]

# ============================================================
# Leaf stage: pyenv-test — pyenv-virtualenv fixture only
# ============================================================
FROM scuv-test-base AS pyenv-test

ARG PYTHON_312_VERSION
RUN pyenv virtualenv ${PYTHON_312_VERSION} test-pyenv-312

# ============================================================
# Leaf stage: conda-test — Miniforge + conda env fixture
#
# Why Miniforge over Miniconda:
#   - MIT-licensed installer (Miniconda's installer + default channel
#     carry Anaconda commercial-use restrictions as of 2024+)
#   - conda-forge as the default channel — community-curated, larger
#     package set, no anaconda.com terms
#   - Multi-arch (amd64 + arm64) without per-arch URL juggling
# ============================================================
FROM scuv-test-base AS conda-test

ARG MINIFORGE_VERSION
ENV CONDA_DIR="/opt/conda"
RUN ARCH=$(uname -m) && \
    curl -fsSL -o /tmp/miniforge.sh \
      "https://github.com/conda-forge/miniforge/releases/download/${MINIFORGE_VERSION}/Miniforge3-Linux-${ARCH}.sh" \
    && bash /tmp/miniforge.sh -b -p "$CONDA_DIR" \
    && rm /tmp/miniforge.sh \
    && "$CONDA_DIR/bin/conda" clean -afy
ENV PATH="$CONDA_DIR/bin:$PATH"

RUN conda create -n test-conda-312 python=3.12 -y && conda clean -afy

# ============================================================
# Leaf stage: venvwrapper-test — virtualenvwrapper fixture
# ============================================================
FROM scuv-test-base AS venvwrapper-test

ENV WORKON_HOME="/root/.virtualenvs"
RUN --mount=type=cache,target=/root/.cache/pip,sharing=locked \
    pip install virtualenvwrapper && mkdir -p "$WORKON_HOME"

RUN bash -c "source $(which virtualenvwrapper.sh) && \
    mkvirtualenv -p python3.12 test-venvwrapper-312"

# ============================================================
# Stage: slim — minimal CI environment (pyenv only)
# ============================================================
FROM pyenv-test AS slim
# Alias-ish: identical to pyenv-test today. Kept as a separate target so
# CI / Makefile references like `--target slim` continue to work and we
# have a stable name for "minimal".

# ============================================================
# Stage: full — complete test environment (all fixtures, both Pythons)
# ============================================================
FROM scuv-test-base AS full

ARG PYTHON_311_VERSION
ARG PYTHON_312_VERSION

# pyenv fixtures (both major versions for cross-version smoke)
RUN pyenv virtualenv ${PYTHON_312_VERSION} test-pyenv-312 && \
    pyenv virtualenv ${PYTHON_311_VERSION} test-pyenv-311

# conda (Miniforge — see conda-test stage for rationale)
ARG MINIFORGE_VERSION
ENV CONDA_DIR="/opt/conda"
RUN ARCH=$(uname -m) && \
    curl -fsSL -o /tmp/miniforge.sh \
      "https://github.com/conda-forge/miniforge/releases/download/${MINIFORGE_VERSION}/Miniforge3-Linux-${ARCH}.sh" \
    && bash /tmp/miniforge.sh -b -p "$CONDA_DIR" \
    && rm /tmp/miniforge.sh \
    && "$CONDA_DIR/bin/conda" clean -afy
ENV PATH="$CONDA_DIR/bin:$PATH"
RUN conda create -n test-conda-312 python=3.12 -y && conda clean -afy

# virtualenvwrapper
ENV WORKON_HOME="/root/.virtualenvs"
RUN --mount=type=cache,target=/root/.cache/pip,sharing=locked \
    pip install virtualenvwrapper && mkdir -p "$WORKON_HOME"
RUN bash -c "source $(which virtualenvwrapper.sh) && \
    mkvirtualenv -p python3.12 test-venvwrapper-312"
