# syntax=docker/dockerfile:1.7
# ============================================================
# Multi-stage Dockerfile for scoop test environment
# Best Practices: layer caching, non-root user, BuildKit cache mounts
# ============================================================

# ============================================================
# Stage 1: base - System dependencies and tools
# ============================================================
FROM rust:1.85-bookworm AS base

# Prevent interactive prompts
ENV DEBIAN_FRONTEND=noninteractive \
    # Rust environment
    CARGO_HOME="/root/.cargo" \
    RUSTUP_HOME="/root/.rustup" \
    # pyenv environment
    PYENV_ROOT="/root/.pyenv" \
    # Locale
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
# Stage 2: pyenv - Install pyenv and pyenv-virtualenv
# ============================================================
FROM base AS pyenv

# Clone pyenv (shallow clone for speed)
RUN git clone --depth 1 https://github.com/pyenv/pyenv.git "$PYENV_ROOT" \
    && git clone --depth 1 https://github.com/pyenv/pyenv-virtualenv.git \
        "$PYENV_ROOT/plugins/pyenv-virtualenv"

# Add pyenv to PATH
ENV PATH="$PYENV_ROOT/bin:$PYENV_ROOT/shims:$PATH"

# Verify installation
RUN pyenv --version

# ============================================================
# Stage 3: python - Install Python versions (SLOW - cache this!)
# ============================================================
FROM pyenv AS python

# Install Python versions with cache mount for build artifacts
# NOTE: This layer takes ~10-20 minutes, so we cache it aggressively
RUN --mount=type=cache,target=/root/.pyenv/cache,sharing=locked \
    pyenv install 3.11.9 \
    && pyenv install 3.12.4 \
    && pyenv global 3.12.4

# Verify Python installation
RUN python --version && pip --version

# ============================================================
# Stage 4: full - Complete test environment
# ============================================================
FROM python AS full

# Install uv (version pinned)
ENV UV_VERSION=0.5.14
RUN curl -LsSf "https://astral.sh/uv/${UV_VERSION}/install.sh" | sh
ENV PATH="/root/.local/bin:$PATH"

# Install Miniconda
ENV CONDA_DIR="/opt/conda"
RUN curl -fsSL -o /tmp/miniconda.sh \
    "https://repo.anaconda.com/miniconda/Miniconda3-py312_24.7.1-0-Linux-x86_64.sh" \
    && bash /tmp/miniconda.sh -b -p "$CONDA_DIR" \
    && rm /tmp/miniconda.sh
ENV PATH="$CONDA_DIR/bin:$PATH"

# Install virtualenvwrapper
ENV WORKON_HOME="/root/.virtualenvs"
RUN --mount=type=cache,target=/root/.cache/pip,sharing=locked \
    pip install virtualenvwrapper \
    && mkdir -p "$WORKON_HOME"

# Create test fixtures
# pyenv virtualenv
RUN pyenv virtualenv 3.12.4 test-pyenv-312 \
    && pyenv virtualenv 3.11.9 test-pyenv-311

# conda env
RUN conda create -n test-conda-312 python=3.12 -y

# virtualenvwrapper env (requires bash)
RUN bash -c "source $(which virtualenvwrapper.sh) && \
    mkvirtualenv -p python3.12 test-venvwrapper-312"

# Copy entrypoint and shell config
COPY docker/scripts/entrypoint.sh /entrypoint.sh
COPY docker/scripts/bashrc.sh /root/.bashrc
RUN chmod +x /entrypoint.sh

WORKDIR /workspace

ENTRYPOINT ["/entrypoint.sh"]
CMD ["bash", "-l"]

# ============================================================
# Stage 5: slim - Minimal CI environment
# ============================================================
FROM python AS slim

# Install uv only
ENV UV_VERSION=0.5.14
RUN curl -LsSf "https://astral.sh/uv/${UV_VERSION}/install.sh" | sh
ENV PATH="/root/.local/bin:$PATH"

# Minimal test fixture
RUN pyenv virtualenv 3.12.4 test-pyenv-312

# Copy entrypoint
COPY docker/scripts/entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

WORKDIR /workspace

ENTRYPOINT ["/entrypoint.sh"]
CMD ["bash", "-l"]
