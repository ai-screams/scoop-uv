# Maintenance Plan

Last updated: 2026-02-16

## Goal
Keep operational memory docs aligned with current code behavior and release state.

## Completed
- Synced Context7-focused documentation updates into `main` (PR #80).
- Pruned stale local/remote branches; kept only active release branch.
- Refreshed memory references (`MEMORY.md`, `.serena/memories/*`, `AGENTS.md`).

## Next
1. Merge/release flow:
   - Track open release PR (`chore: release v0.8.1`).
2. CI hardening:
   - Improve Docker Trivy job resilience for `slim` image pull timing.
   - Add guard around SARIF upload when result file is missing.
3. Memory hygiene:
   - Re-validate memory files after each release-plz merge.
   - Keep command examples in memory aligned with `src/cli/mod.rs` and docs.

## Validation Checklist
- `mdbook build docs`
- `gh pr checks <PR_NUMBER>`
- Spot-check facts against:
  - `Cargo.toml`
  - `src/cli/mod.rs`
  - `src/core/version.rs`
  - `src/core/metadata.rs`
  - `.github/workflows/*.yml`
