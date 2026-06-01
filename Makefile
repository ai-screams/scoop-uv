# Makefile for scoop development
# Docker test environment management

.PHONY: help \
	docker-build docker-build-slim docker-build-all \
	docker-shell docker-shell-slim docker-shell-zsh docker-shell-fish \
	test test-unit test-integration test-all \
	bench docker-clean docker-prune \
	ci-build ci-test

# ============================================================
# Variables
# ============================================================
COMPOSE := docker compose -f docker/docker-compose.yml
IMAGE := ghcr.io/ai-screams/scoop

# ============================================================
# Help
# ============================================================
help:
	@echo "scoop Development Commands"
	@echo ""
	@echo "Build:"
	@echo "  docker-build       Build full image (all tools)"
	@echo "  docker-build-slim  Build slim image (pyenv only)"
	@echo "  docker-build-all   Build both images"
	@echo ""
	@echo "Run:"
	@echo "  docker-shell       Interactive shell (full)"
	@echo "  docker-shell-slim  Interactive shell (slim)"
	@echo "  docker-shell-zsh   Interactive zsh shell"
	@echo "  docker-shell-fish  Interactive fish shell"
	@echo ""
	@echo "Test:"
	@echo "  test               Run all tests (local)"
	@echo "  test-unit          Run unit tests only (local)"
	@echo "  test-integration   Run integration tests (Docker)"
	@echo "  test-all           Run unit + integration tests"
	@echo ""
	@echo "Benchmark:"
	@echo "  bench              Run benchmarks (Docker)"
	@echo ""
	@echo "Cleanup:"
	@echo "  docker-clean       Remove containers and volumes"
	@echo "  docker-prune       Deep clean (images too)"
	@echo ""
	@echo "CI:"
	@echo "  ci-build           Build slim image for CI"
	@echo "  ci-test            Run tests in CI mode"

# ============================================================
# Docker Build
# ============================================================
docker-build:
	$(COMPOSE) build full

docker-build-slim:
	$(COMPOSE) build slim

docker-build-all: docker-build docker-build-slim

# ============================================================
# Docker Shell
# ============================================================
docker-shell:
	$(COMPOSE) run --rm full bash -l

docker-shell-slim:
	$(COMPOSE) run --rm slim bash -l

docker-shell-zsh:
	$(COMPOSE) run --rm full zsh -l

docker-shell-fish:
	$(COMPOSE) run --rm full fish -l

# Drop into a per-source image for ad-hoc debugging
docker-shell-pyenv:
	$(COMPOSE) run --rm pyenv-test bash -l

docker-shell-conda:
	$(COMPOSE) run --rm conda-test bash -l

docker-shell-venvwrapper:
	$(COMPOSE) run --rm venvwrapper-test bash -l

# ============================================================
# Test
# ============================================================
test:
	cargo test

test-unit:
	cargo test --lib

test-integration:
	$(COMPOSE) run --rm slim cargo test --features integration-test

# Per-source integration tests (matrix-friendly for CI parity)
test-integration-pyenv:
	$(COMPOSE) run --rm pyenv-test cargo test --features integration-test

test-integration-conda:
	$(COMPOSE) run --rm conda-test cargo test --features integration-test

test-integration-venvwrapper:
	$(COMPOSE) run --rm venvwrapper-test cargo test --features integration-test

# Runs all three source-tool integration suites sequentially. Use the
# CI matrix workflow (.github/workflows/integration-test.yml) for the
# parallel version — running all three in a single Docker host serially
# is fine for local debugging.
test-integration-all: test-integration-pyenv test-integration-conda test-integration-venvwrapper

test-all: test-unit test-integration

# ============================================================
# Benchmark
# ------------------------------------------------------------
# `bench` runs all suites in Docker (reproducible).
# `bench-save` / `bench-compare` use the host toolchain because
# Criterion stores baselines under target/criterion/ and reusing the
# host's target/ avoids the volume-mount roundtrip on macOS.
# ============================================================
BENCH_BASELINE ?= main

bench:
	$(COMPOSE) --profile bench run --rm bench

# Save the current bench results as a named baseline (default: "main").
# Run on the main branch after a release; Criterion writes to
# target/criterion/<bench>/<baseline>/.
bench-save:
	cargo bench --bench parsing --bench validation --bench path_lookup \
		-- --save-baseline $(BENCH_BASELINE)

# Compare current results against a saved baseline. Exits non-zero on
# the per-bench regression threshold (Criterion default: 5% noise).
bench-compare:
	cargo bench --bench parsing --bench validation --bench path_lookup \
		-- --baseline $(BENCH_BASELINE)

# ============================================================
# Cleanup
# ============================================================
docker-clean:
	$(COMPOSE) down --volumes --remove-orphans

docker-prune: docker-clean
	docker image rm $(IMAGE):latest $(IMAGE):slim 2>/dev/null || true
	docker volume prune -f

# ============================================================
# CI Helpers
# ============================================================
ci-build:
	docker buildx build \
		--file docker/Dockerfile \
		--target slim \
		--cache-from type=gha \
		--cache-to type=gha,mode=max \
		--tag $(IMAGE):slim \
		.

ci-test:
	docker run --rm \
		-v $(PWD):/workspace \
		$(IMAGE):slim \
		cargo test --features integration-test

# ============================================================
# Development shortcuts
# ============================================================
.PHONY: c cb cr fmt clippy

c:
	cargo check

cb:
	cargo build

cr:
	cargo run --

fmt:
	cargo fmt

clippy:
	cargo clippy --all-targets --all-features -- -D warnings
