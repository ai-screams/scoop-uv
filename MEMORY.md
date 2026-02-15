# Project Memory

Last updated: 2026-02-16

## Current Baseline
- Crate: `scoop-uv`, CLI binary: `scoop`.
- Main branch release baseline: `v0.8.0` (`Cargo.toml` + `CHANGELOG.md`).
- Config path: `~/.scoop/config.json` (or `$SCOOP_HOME/config.json`).
- Environment storage: `~/.scoop/virtualenvs/`.
- Metadata file: `.scoop-metadata.json` (includes optional `python_path`).

## Resolution and Shell Model
- Priority: `SCOOP_VERSION` -> `.scoop-version` (current + parents) -> `~/.scoop/version`.
- `scoop use` writes version files (local/global) and optional `.venv` symlink.
- `scoop shell` sets/unsets `SCOOP_VERSION` for session-level override.
- Auto-activation hook behavior is shell-wrapper driven (bash/zsh/fish/powershell).

## Supported Surface
- Shells: bash, zsh, fish, powershell (`pwsh` alias in CLI).
- i18n: en, ko, ja, pt-BR.
- Hidden/internal commands exist: `resolve`, `activate`, `deactivate`.

## Documentation and Retrieval
- Context retrieval sources maintained in this repo:
  - `docs/src/llms.md`
  - `llms.txt`
  - `llms-full.txt`
  - `context7.json`
- User-facing Q&A workflows are maintained in `docs/src/faq.md` and command docs.

## Active Follow-up
- Open release PR exists for `v0.8.1` (`release-plz` branch flow).
- CI hardening follow-up: Docker Trivy slim pull race handling and SARIF upload guard.
