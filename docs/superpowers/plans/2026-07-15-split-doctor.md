# Split doctor.rs into a doctor/ module — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Split `src/core/doctor.rs` (1055 prod lines) into a `doctor/` module — one small file per health check owning its diagnosis, auto-fix, and tests — with zero change to observable behavior or the public API.

**Architecture:** This is a **relocation refactor, not TDD**. New behavior is not added, so the regression signal is the *existing* test suite: after each move, `cargo test doctor` must stay green. The one logic change is converting `Doctor`'s string-id fix dispatch (`try_fix` matching `result.id`) into a polymorphic `Check::fix()` trait method — a pure mechanical transposition that yields identical fix results. The module's public paths (`crate::core::doctor::{Doctor, CheckResult, CheckStatus, Check}`) are preserved via `pub use` in `doctor/mod.rs`, so no external caller changes.

**Tech Stack:** Rust 2024 (MSRV 1.85), rstest, serial_test, tempfile. Test discipline: `env_guard` (src/test_utils.rs) + `#[serial]` for env-mutating tests; the root-skip permission probe from the 0.15.1 CI fix must be preserved verbatim.

## Global Constraints

- Branch: `refactor/split-doctor` (already created; spec committed there). Commit per task; NEVER create a PR or push `--force` without explicit user go-ahead.
- Public API frozen: `crate::core::doctor::{Doctor, CheckResult, CheckStatus, Check}` must all still resolve. `src/core/mod.rs:3` stays `pub mod doctor;` unchanged. No edits to `src/cli/commands/doctor.rs` or `src/output/mod.rs`.
- Behavior frozen: each `run()` body moves verbatim; `--json` output shape and every `CheckResult { id, name, status, suggestion, details }` field unchanged. No message/suggestion/i18n edits.
- Gates every task: `cargo test doctor`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo fmt --check` — all clean before commit.
- No IO port/adapter abstraction, no mock rewrite (YAGNI, per spec). Checks keep calling `std::fs`/`Command`/`std::env`/`dirs`/`paths`/`UvClient` directly.
- `.docs/` is never committed; this plan and the spec live under `docs/superpowers/` (tracked).
- Spec: `docs/superpowers/specs/2026-07-15-split-doctor-design.md`.

## File structure (end state)

```
src/core/doctor/
├── mod.rs          # `mod` decls + `pub use` re-exports (Doctor, CheckResult, CheckStatus, Check)
├── types.rs        # CheckStatus, CheckResult (+impl), Check trait (+ default fix)
├── engine.rs       # Doctor struct + new()/run_all()/run_and_fix()  (+ engine tests)
└── checks/
    ├── mod.rs      # `mod` decls + `pub(super) fn default_checks() -> Vec<Box<dyn Check>>`
    ├── uv.rs           # UvCheck (+ install_hint) + tests
    ├── home.rs         # HomeCheck (+ fix override) + tests
    ├── virtualenv.rs   # VirtualenvCheck + tests
    ├── symlink.rs      # SymlinkCheck (+ fix override) + tests
    ├── shell.rs        # ShellCheck + tests
    ├── version.rs      # VersionCheck + classify_version_entry + resolve_local_version_file_for_doctor + tests
    └── legacy.rs       # LegacyCheck + check_legacy_remnants + tests
```

---

### Task 1: Scaffold — relocate the file unchanged into `doctor/mod.rs`

**Files:**
- Create dir: `src/core/doctor/`
- Move: `src/core/doctor.rs` → `src/core/doctor/mod.rs` (content byte-identical)

**Interfaces:**
- Produces: the `doctor/` directory module. `pub mod doctor;` in `src/core/mod.rs` now resolves to `doctor/mod.rs`. All items (`Doctor`, `CheckResult`, `CheckStatus`, `Check`, checks, helpers) stay exactly where they were, so every existing path resolves unchanged.

- [ ] **Step 1: Move the file with git (preserves history)**

```bash
cd /Users/hanyul/Works/AiScream/scoop-uv
mkdir -p src/core/doctor
git mv src/core/doctor.rs src/core/doctor/mod.rs
```

- [ ] **Step 2: Verify nothing else references a `doctor.rs` path**

Run: `rg -n 'doctor\.rs|include!.*doctor' src/ --type rust`
Expected: no hits (the module is referenced as `mod doctor`, not by file path).

- [ ] **Step 3: Build + test + lint**

Run: `cargo test doctor && cargo clippy --all-targets --all-features -- -D warnings && cargo fmt --check`
Expected: all green — this is a pure file relocation, the compiler sees identical code.

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "refactor(doctor): relocate doctor.rs to doctor/mod.rs (no code change)"
```

---

### Task 2: Extract `types.rs` and add the default `Check::fix`

**Files:**
- Create: `src/core/doctor/types.rs`
- Modify: `src/core/doctor/mod.rs` (remove the moved items; add `mod types;` + `pub use`)

**Interfaces:**
- Consumes: nothing new.
- Produces: `types.rs` exporting `pub enum CheckStatus`, `pub struct CheckResult` (+ its impl), and `pub trait Check` **now with a default `fix`**:
  ```rust
  fn fix(&self, _result: &CheckResult, _output: &crate::output::Output) -> Option<CheckResult> {
      None
  }
  ```
  Re-exported from `mod.rs` so `crate::core::doctor::{CheckStatus, CheckResult, Check}` still resolve.

- [ ] **Step 1: Create `types.rs` with the moved type items + the new `fix` default**

Move these VERBATIM from `mod.rs` into a new `src/core/doctor/types.rs`:
- `pub enum CheckStatus { ... }` (currently ~line 19)
- `pub struct CheckResult { ... }` + its full `impl CheckResult { ... }` block (ok/warn/error/with_suggestion/with_details/is_ok/is_warning/is_error)
- `pub trait Check { ... }` — and ADD the `fix` default method shown in Interfaces above, right after `fn run(&self) -> Vec<CheckResult>;`.

Prepend the imports `types.rs` needs (only what these items reference): none beyond `crate::output::Output` used in the new `fix` signature — reference it fully-qualified as written, so no `use` line is required.

- [ ] **Step 2: Trim `mod.rs` and wire the module**

In `src/core/doctor/mod.rs`: delete the three items just moved. At the top of the file add:
```rust
mod types;
pub use types::{Check, CheckResult, CheckStatus};
```

- [ ] **Step 3: Build + test + lint**

Run: `cargo test doctor && cargo clippy --all-targets --all-features -- -D warnings && cargo fmt --check`
Expected: green. The default `fix` is unused so far (engine still dispatches via `try_fix`), so behavior is unchanged. If clippy warns the default `fix` is never used, that resolves in Task 3 when the engine calls it — but it should not warn (trait defaults aren't dead-code-flagged).

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "refactor(doctor): extract types.rs, add default Check::fix"
```

---

### Task 3: Extract `engine.rs` and convert fix dispatch to the trait method

**Files:**
- Create: `src/core/doctor/engine.rs`
- Modify: `src/core/doctor/mod.rs` (remove `Doctor`; add `mod engine;` + `pub use`)

**Interfaces:**
- Consumes: `Check`, `CheckResult`, `CheckStatus` from `types` (via `use super::types::...` or `use super::*`); the check structs still live in `mod.rs` at this point, so `engine.rs` references them via `use super::*`.
- Produces: `engine.rs` with `pub struct Doctor` and `Doctor::{new, run_all, run_and_fix}`. `try_fix`, `fix_home`, `fix_symlink` are REMOVED from `Doctor`. `HomeCheck` and `SymlinkCheck` gain a `fix()` override (moved from the deleted `fix_home`/`fix_symlink` bodies). `run_and_fix` calls `check.fix(&result, output)` instead of `self.try_fix(...)`.

- [ ] **Step 1: Move `fix_home`/`fix_symlink` bodies onto the checks as `Check::fix` overrides**

The checks still live in `mod.rs` for now. On `impl Check for HomeCheck`, add:
```rust
fn fix(&self, result: &CheckResult, output: &crate::output::Output) -> Option<CheckResult> {
    // ← paste the entire body of the old Doctor::fix_home here, verbatim.
    // It uses only paths::scoop_home() and std::fs (no `self`/Doctor state),
    // so it transplants unchanged.
}
```
Do the same for `impl Check for SymlinkCheck` using the old `Doctor::fix_symlink` body. (If `fix_symlink` references any `Doctor` helper, inline it — verify with `rg -n 'self\.' ` inside that body; the fix logic is self-contained.)

- [ ] **Step 2: Create `engine.rs` with `Doctor`, minus the fix internals**

Move `pub struct Doctor { checks: Vec<Box<dyn Check>> }`, `impl Doctor` (`new`, `run_all`, `run_and_fix`), and `impl Default for Doctor` into `src/core/doctor/engine.rs`. Head the file with `use super::*;` so it sees `Check`, `CheckResult`, and the check structs still in `mod.rs`. **Delete** `try_fix`, `fix_home`, `fix_symlink`. Change `run_and_fix`'s inner block from:
```rust
if let Some(fixed_result) = self.try_fix(&result, output) {
```
to:
```rust
if let Some(fixed_result) = check.fix(&result, output) {
```
(`check` is already the loop variable `for check in &self.checks`.)

- [ ] **Step 3: Wire the module in `mod.rs`**

Delete the moved `Doctor`/`impl`/`Default` from `mod.rs`. Add near the top:
```rust
mod engine;
pub use engine::Doctor;
```

- [ ] **Step 4: Build + test + lint**

Run: `cargo test doctor && cargo clippy --all-targets --all-features -- -D warnings && cargo fmt --check`
Expected: green. The fix tests (`fix_home_creates_missing_home_and_virtualenvs_dir`, `fix_home_ignores_non_not_found_results`, and any symlink-fix test) must still pass — they now exercise `HomeCheck::fix`/`SymlinkCheck::fix` through the same `run_and_fix` path. If a fix test called `Doctor::fix_home` directly, update it to construct `HomeCheck` and call `.fix(...)`; keep the assertions identical.

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "refactor(doctor): extract engine.rs, dispatch fix via Check::fix trait method"
```

---

### Task 4: Extract `checks/` group 1 — uv, home, virtualenv, symlink

**Files:**
- Create: `src/core/doctor/checks/mod.rs`, `checks/uv.rs`, `checks/home.rs`, `checks/virtualenv.rs`, `checks/symlink.rs`
- Modify: `src/core/doctor/mod.rs` (remove these 4 checks; add `mod checks;`), `src/core/doctor/engine.rs` (use `checks::default_checks()`)

**Interfaces:**
- Consumes: `Check`, `CheckResult`, `CheckStatus` from `types`.
- Produces: each check as a `pub(super) struct` with `impl Check for X` (and `fix` for home/symlink) in its own file; `checks/mod.rs` exposing `pub(super) fn default_checks() -> Vec<Box<dyn Check>>` that `engine::Doctor::new()` calls. After this task, `default_checks()` returns all 7 (the 3 still-in-`mod.rs` checks are referenced by `default_checks` via their `mod.rs` paths until Task 5 — see Step 4).

- [ ] **Step 1: Create the four check files**

For each of `UvCheck`, `HomeCheck`, `VirtualenvCheck`, `SymlinkCheck`: create `checks/<name>.rs` and move the `struct` + its `impl Check for ...` (including `home`/`symlink` `fix` overrides added in Task 3, and `UvCheck`'s inherent `impl UvCheck { fn install_hint... }` helper) VERBATIM. Head each file with `use super::super::types::{Check, CheckResult, CheckStatus};` plus whatever `std`/`crate` imports that check's body uses (`use crate::paths;`, `use std::process::Command;` for uv, etc. — copy from `mod.rs`'s existing `use` block, keeping only what the moved code references). Make each struct `pub(super)`.

Move each check's `#[cfg(test)] mod tests` (the tests naming that check: `home_check_*`, `fix_home_*`, `virtualenv_*`, `symlink_*`, `uv_check_*`) into the same file, wrapped in `#[cfg(test)] mod tests { use super::*; ... }`. Keep `#[serial]`, `env_guard`, and any `TempDirCwdGuard`/root-skip-probe usages exactly.

- [ ] **Step 2: Create `checks/mod.rs` with the registration factory**

```rust
mod uv;
mod home;
mod virtualenv;
mod symlink;
// group 2 (Task 5) adds: mod shell; mod version; mod legacy;

use super::types::Check;

/// The default set of checks, in display order. Single place to register a check.
pub(super) fn default_checks() -> Vec<Box<dyn Check>> {
    vec![
        Box::new(uv::UvCheck),
        Box::new(home::HomeCheck),
        Box::new(virtualenv::VirtualenvCheck),
        Box::new(symlink::SymlinkCheck),
        // group 2 (Task 5): shell::ShellCheck, version::VersionCheck, legacy::LegacyCheck
        // TEMPORARY until Task 5: the three below still live in mod.rs
        Box::new(super::ShellCheck),
        Box::new(super::VersionCheck),
        Box::new(super::LegacyCheck),
    ]
}
```
(The `super::ShellCheck` etc. references work because those structs are still defined in `mod.rs` this task; Task 5 moves them into `checks/` and drops the `super::` forms. To reference them via `super::`, they must be visible from `checks/` — if the compiler rejects the path, temporarily mark `ShellCheck`/`VersionCheck`/`LegacyCheck` `pub(crate)` in `mod.rs`; Task 5 removes that.)

- [ ] **Step 3: Point the engine at the factory**

In `engine.rs`, change `Doctor::new()`'s body from the inline `vec![Box::new(UvCheck), ...]` to:
```rust
Self { checks: super::checks::default_checks() }
```
Add `mod checks;` to `mod.rs`. Remove the 4 moved checks (and their tests) from `mod.rs`.

- [ ] **Step 4: Build + test + lint**

Run: `cargo test doctor && cargo clippy --all-targets --all-features -- -D warnings && cargo fmt --check`
Expected: green. Same 7 checks run in the same order (verify a `doctor` run still lists uv/home/virtualenv/symlink/shell/version/legacy — the registration-order test, if any, still passes).

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "refactor(doctor): extract uv/home/virtualenv/symlink checks + registration factory"
```

---

### Task 5: Extract `checks/` group 2 — shell, version, legacy (+ their free helpers)

**Files:**
- Create: `checks/shell.rs`, `checks/version.rs`, `checks/legacy.rs`
- Modify: `src/core/doctor/mod.rs` (remove these 3 checks + 3 free helpers), `checks/mod.rs` (finalize `default_checks`)

**Interfaces:**
- Consumes: `types` items.
- Produces: `ShellCheck`, `VersionCheck`, `LegacyCheck` each `pub(super)` in their own files. `classify_version_entry` + `resolve_local_version_file_for_doctor` → private in `version.rs`; `check_legacy_remnants` → private in `legacy.rs`. `checks/mod.rs::default_checks()` now references all 7 via `<module>::<Check>` (no more `super::` temporaries).

- [ ] **Step 1: Create the three files with checks + their sole-caller helpers**

- `checks/shell.rs`: move `struct ShellCheck` + `impl Check for ShellCheck` + all `shell_check_*` tests. Head with `use super::super::types::{Check, CheckResult, CheckStatus};` + the check's own imports (`use std::env;`, `use crate::paths;`, etc.).
- `checks/version.rs`: move `struct VersionCheck` + `impl Check for VersionCheck`, plus the two free helpers `classify_version_entry` and `resolve_local_version_file_for_doctor` (they're only called by VersionCheck) as private `fn`s, plus all `version_check_*` / `version_check_local_read_error_*` tests.
- `checks/legacy.rs`: move `struct LegacyCheck` + `impl Check for LegacyCheck`, plus `check_legacy_remnants` (only called by LegacyCheck) as a private `fn`, plus all `legacy_check_*` tests. Preserve the `SCOOP_NO_AUTO` entry and its comment, and the `env_guard` `#[serial]` discipline verbatim.

- [ ] **Step 2: Finalize `checks/mod.rs`**

```rust
mod uv;
mod home;
mod virtualenv;
mod symlink;
mod shell;
mod version;
mod legacy;

use super::types::Check;

pub(super) fn default_checks() -> Vec<Box<dyn Check>> {
    vec![
        Box::new(uv::UvCheck),
        Box::new(home::HomeCheck),
        Box::new(virtualenv::VirtualenvCheck),
        Box::new(symlink::SymlinkCheck),
        Box::new(shell::ShellCheck),
        Box::new(version::VersionCheck),
        Box::new(legacy::LegacyCheck),
    ]
}
```
Remove the 3 checks, 3 helpers, and any temporary `pub(crate)` markers from `mod.rs`.

- [ ] **Step 3: Build + test + lint**

Run: `cargo test doctor && cargo clippy --all-targets --all-features -- -D warnings && cargo fmt --check`
Expected: green. `mod.rs` now holds only `mod`/`pub use` lines.

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "refactor(doctor): extract shell/version/legacy checks + their helpers"
```

---

### Task 6: Final verification — mod.rs is re-exports only, full gates, Docker

**Files:**
- Verify/Modify: `src/core/doctor/mod.rs` (should be re-exports + `mod` decls only)

**Interfaces:**
- Consumes: all prior tasks.
- Produces: the finished module. No new interface.

- [ ] **Step 1: Confirm `mod.rs` is minimal**

Run: `cat src/core/doctor/mod.rs`
Expected: only `mod types; mod engine; mod checks;` and `pub use types::{Check, CheckResult, CheckStatus}; pub use engine::Doctor;` (plus the top-of-file `//!` doc comment, kept). If any check/helper/type body remains, move it per Tasks 2–5.

- [ ] **Step 2: Confirm public API unchanged**

Run: `rg -n 'core::doctor::' src/ --type rust`
Expected: `cli/commands/doctor.rs` and `output/mod.rs` still import `{CheckResult, Doctor, CheckStatus}` and compile — unchanged files (`git diff --stat origin/main -- src/cli src/output` shows no doctor-related edits).

- [ ] **Step 3: Full local gates (forced rebuild not needed — no yml touched)**

Run: `cargo test && cargo clippy --all-targets --all-features -- -D warnings && cargo fmt --check`
Expected: full suite green (898+ unit incl. all doctor tests now in their new files, 45 CLI, 2 i18n, 25 doctest), 0 warnings.

- [ ] **Step 4: Docker integration (root env — the root-skip probe must hold)**

Run (if a Docker daemon is available): `make test-integration`
Expected: pass. If no daemon locally, note it — CI runs this on the PR; the `test_unset_local_fails_fast_on_unwritable_dir` root-skip probe (unrelated to doctor but in the same suite) already handles root.

- [ ] **Step 5: Commit any final cleanup**

```bash
git add -A
git commit -m "refactor(doctor): finish module split — mod.rs is re-exports only" --allow-empty
```

---

## Self-review notes

- **Spec coverage:** every spec item mapped — module split (Tasks 1–5), fix-as-trait (Task 3), API-frozen via `pub use` (Tasks 2–3, verified Task 6 Step 2), behavior-frozen via existing suite (gates each task), tests-move-with-code (Tasks 4–5), helpers-next-to-caller (Task 5), root-skip probe preserved (Global Constraints + Task 6 Step 4).
- **Relocation, not TDD:** intentional — the plan's "failing test first" is replaced by "existing suite stays green," which is the correct regression signal for a pure move. Called out in Architecture.
- **Known sharp edge (Task 4 Step 2):** the temporary `super::ShellCheck` references + possible `pub(crate)` marker are transient scaffolding removed in Task 5; flagged inline so a reviewer doesn't mistake it for the end state.
- **Type consistency:** `Check::fix(&self, result: &CheckResult, output: &crate::output::Output) -> Option<CheckResult>` is identical in types.rs (default), home.rs/symlink.rs (override), and engine.rs (call site). `default_checks() -> Vec<Box<dyn Check>>` identical in checks/mod.rs and engine.rs call site.
