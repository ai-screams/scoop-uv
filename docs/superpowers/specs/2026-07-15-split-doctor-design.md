# Split `doctor.rs` into a `doctor/` module — Design

**Status**: Approved (2026-07-15 brainstorm). Implementation plan to follow.

## Goal

Split `src/core/doctor.rs` (production 1055 lines) into a focused `doctor/` module so each
health check owns its diagnosis, auto-fix, and tests in one small file — without changing any
observable behavior or the module's public API.

## Scope decisions (user-confirmed)

1. **Module split + boundary cleanup**, NOT full dependency inversion. Checks keep calling IO
   (`std::fs`, `Command`, `std::env`, `dirs`, `paths`, `UvClient`) directly. No `FileSystem`/
   `ProcessRunner` ports — that boilerplate is unjustified at this tool's scale (YAGNI), and the
   existing tempdir + `env_guard` + `#[serial]` tests exercise real IO well.
2. **`fix` becomes a `Check` trait method.** Add `fn fix(&self, result: &CheckResult, output:
   &Output) -> Option<CheckResult> { None }` to the trait. Only `HomeCheck` and `SymlinkCheck`
   override it. The engine's string-id dispatch (`"home"` → `fix_home`, `"symlink"` →
   `fix_symlink`) is removed in favor of a polymorphic `check.fix(...)` call.

## Target layout

```
src/core/doctor/
├── mod.rs          # pub use re-exports: Doctor, CheckResult, CheckStatus, Check
├── types.rs        # CheckStatus, CheckResult (+ impl), Check trait (+ default fix)
├── engine.rs       # Doctor: new(), run_all(), run_and_fix()
└── checks/
    ├── mod.rs      # declares check modules + `pub(super) fn default_checks() -> Vec<Box<dyn Check>>` (single registration point)
    ├── uv.rs           # UvCheck (+ install_hint helper) + tests
    ├── home.rs         # HomeCheck + fix() override + tests
    ├── virtualenv.rs   # VirtualenvCheck + tests
    ├── symlink.rs      # SymlinkCheck + fix() override + tests
    ├── shell.rs        # ShellCheck + tests
    ├── version.rs      # VersionCheck + classify_version_entry, resolve_local_version_file_for_doctor + tests
    └── legacy.rs       # LegacyCheck + check_legacy_remnants + tests
```

Current → new mapping (line ranges from the pre-split file, for reference):
- `CheckStatus` (19), `CheckResult` (30–103), `Check` trait (113–124) → `types.rs`
- `Doctor` struct + `new`/`run_all`/`run_and_fix`/`try_fix` (133–425) → `engine.rs`
  (`try_fix` + `fix_home`/`fix_symlink` dissolve: `fix_home`→`home.rs::fix`, `fix_symlink`→
  `symlink.rs::fix`, `try_fix` replaced by `check.fix()` in `run_and_fix`)
- `UvCheck` (427–491) → `checks/uv.rs`
- `HomeCheck` (492–542) + `fix_home` logic → `checks/home.rs`
- `VirtualenvCheck` (543–631) → `checks/virtualenv.rs`
- `SymlinkCheck` (632–708) + `fix_symlink` logic → `checks/symlink.rs`
- `ShellCheck` (709–850) → `checks/shell.rs`
- `classify_version_entry` (851), `resolve_local_version_file_for_doctor` (881),
  `VersionCheck` (896–987) → `checks/version.rs`
- `check_legacy_remnants` (988), `LegacyCheck` (1036–) → `checks/legacy.rs`

## Core principles

1. **Public API unchanged.** `doctor/mod.rs` re-exports `Doctor`, `CheckResult`, `CheckStatus`,
   `Check` so every `crate::core::doctor::X` path still resolves. Callers (`cli/commands/doctor.rs`,
   any `--json` output code) are not edited. Verified by: no changes outside `src/core/doctor.rs`
   → `src/core/doctor/**` and `src/core/mod.rs`'s `mod doctor;` line (unchanged).
2. **Behavior unchanged.** Pure move. Each `run()` body is transplanted verbatim; the only logic
   change is the `fix` dispatch mechanism (string-id → trait method), which produces identical
   fix results for the same inputs. The full test suite is the proof.
3. **Visibility & registration.** Each check struct is `pub(super)` inside `checks/` so
   `checks::default_checks()` can construct them; `Doctor::new()` calls `checks::default_checks()`
   (adding a check later touches only `checks/mod.rs`). Free-fn helpers move next to their sole
   caller and stay file-private. The `Check` trait's new `fix` default must keep the trait
   object-safe (it does — `&self`, concrete params, no generics/`Self` return).
4. **Tests move with their code.** Each check's `#[cfg(test)] mod tests` goes to its file (retains
   private access). Engine/`fix`/trait-dispatch tests → `engine.rs`. The `env_guard` + `#[serial]`
   discipline and the root-skip permission probe (from the 0.15.1 CI fix) are preserved exactly.

## Implementation order (each step independently testable)

Incremental, compile-green at every step:
1. Create `doctor/` dir, move the whole current file to `doctor/mod.rs` unchanged; confirm build
   + `cargo test doctor` green (pure relocation, `mod doctor;` still works via `mod.rs`).
2. Extract `types.rs` (CheckStatus/CheckResult/Check) + add `fix` default to the trait; `mod.rs`
   `pub use`s them. Green.
3. Extract `engine.rs` (Doctor); replace `try_fix` string dispatch with `check.fix()`. Green.
4. Extract `checks/` one file at a time (uv → home+fix → virtualenv → symlink+fix → shell →
   version → legacy), moving each check's tests alongside. Green after each.
5. Final: `mod.rs` reduced to re-exports + `mod` declarations; full gates + Docker integration.

## Testing / verification

- Per step: `cargo test doctor`, `cargo clippy --all-targets --all-features -- -D warnings`,
  `cargo fmt --check`.
- Final: full `cargo test` (898+ unit + 45 CLI + 2 i18n + 25 doctest), and — because the
  0.15.1 unwritable-dir test only fails under root — the Docker integration path must pass in CI
  (run as root); the root-skip probe stays intact.
- Doctor's `--json` output shape and every `CheckResult { id, name, status, suggestion, details }`
  field are unchanged (no serialization touched).

## Out of scope

- No IO port/adapter abstraction, no mock-based test rewrite.
- No new checks, no changed suggestions/messages, no i18n changes.
- Other large files (migrate/batch.rs, gc/mod.rs) — separate future work per the backlog.
