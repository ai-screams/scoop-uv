# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

> Note: the per-module AGENTS.md files are untracked via `.git/info/exclude` — update them on disk, never try to commit them. (CLAUDE.md itself is tracked since v0.15.0.)

## Project Overview

**scuv** is a centralized Python virtual environment manager using [uv](https://github.com/astral-sh/uv) as backend. It combines pyenv-virtualenv's workflow with uv's speed.

**Minimum uv version**: 0.5.19 (first release with `uv python list --output-format=json`; enforced by `scuv doctor`, pinned in a CI smoke job, single source of truth is `MIN_VERSION` in `src/uv/version.rs`).

- **Language**: Rust (Edition 2024, MSRV 1.89)
- **License**: MIT OR Apache-2.0
- **Version**: scuv 0.16.1 (command renamed `scoop` → `scuv` in 0.15.0; crate/repo stay `scoop-uv`)
- **Tests**: 1018 passed (948 unit + 44 integration + 2 i18n + 24 doctest), 0 clippy warnings — these drift; `cargo test` is the source of truth
- **Doc drift guard**: `python3 scripts/check-doc-references.py` (CI Lint job) verifies MSRV, version samples, reserved names and key counts in README/CONTRIBUTING/llms.txt/llms-full.txt/docs against the code. Run it after editing any of those.
- **CI/CD design**: `docs/src/development/ci-cd.md` documents what each of the 13 workflows guards, the cross-cutting decisions (concurrency, cache keys, gate-vs-track), the failure modes that shaped them, and the known gaps.
- **Test tooling**: rstest (table tests), proptest, cargo-mutants (mutation), cargo-fuzz (nightly `fuzz/` workspace); see `.docs/dev/testing-strategy.md`
- **i18n**: English, Korean, Japanese, Portuguese-BR, Spanish (rust-i18n)
- **Shells**: bash, zsh, fish, PowerShell

## Rename & Legacy Compatibility (since v0.15.0)

- Legacy names (`SCOOP_*` env, `~/.scoop`, `.scoop-version`, `.scoop.toml`) stopped being read in v0.16.0; the 0.15.x read-only fallbacks, the `scoop` shell forwarder and the `deprecation.*` i18n keys are gone. The doctor `legacy` check stays as a warn-only diagnostic (no fallback reads) so an incomplete upgrade is not silent. Tests named `*ignores_legacy*`, `doctor_registers_legacy_check` and `init_script_never_defines_scoop` (all four shells) pin that no fallback comes back and the diagnostic stays.
- **PowerShell must NEVER define a `scoop` function/alias** (would shadow scoop.sh, the Windows package manager — the reason for the rename). Enforced by `init_script_never_defines_scoop` test.
- fish init/shell idiom is `scuv init fish | source` — `eval (...)` does NOT work in fish (splits multi-line output). The same holds inside the fish wrapper and hook: `activate`/`deactivate`/`shell` output is piped to `source` with an explicit `--shell fish`, because fish does not export `FISH_VERSION` and `detect_shell` would otherwise print bash syntax. `tests/cli.rs::fish_wrapper_and_hook_source_multiline_scripts` runs the real thing when a `fish` binary is installed; the CI Test and MSRV jobs install one and set `SCUV_REQUIRE_FISH` so a missing fish fails there instead of skipping (coverage and mutants run without fish and skip).
- Deliberately KEPT legacy identifiers (on-disk/serialized format compat): `.scoop-metadata.json`, export-schema field `scoop_export_version`. Do not "fix" these.

## Collaboration Rules (MUST FOLLOW)

- **NEVER create or merge a PR on your own.** Do not run `gh pr create`, `gh pr merge`, or any equivalent unless the user has **explicitly told you, in that specific request, that you may create and/or merge the PR**.
- Committing, pushing a branch, running CI, and reporting status are allowed without asking — but **creating a PR** and **merging a PR** each require an explicit, current go-ahead from the user.
- A general instruction like "fix this", "handle it", "해결해줘", or "가장 타당하게 진행해줘" does **NOT** authorize PR creation or merge. When in doubt, stop and ask.
- Closing/reopening PRs, force-push, and other externally visible or hard-to-reverse git actions also require explicit confirmation.

### Merging

- Squash external PRs with `gh pr merge --squash --match-head-commit <sha> --subject "<PR title>"` and no `(#N)` in the subject: `cliff.toml`'s GitHub remote link already adds the PR number, and a `(#N)` in the message doubles it in CHANGELOG (#190). Internal and release PRs use merge commits (precedent #183, #189, #194).
- First-time fork PRs: every workflow sits in `action_required` and `gh pr checks` says "no checks reported" — approve the runs (`gh api -X POST repos/ai-screams/scoop-uv/actions/runs/<id>/approve`) or run the gates in a scratch worktree (`git fetch origin pull/<n>/head:refs/pr/<n>`). CI Lint has no whitespace hook (prek is local-only), so also run `git diff --check` on external diffs.
- Contributors table: comment `@all-contributors please add @<login> for translation` on the PR. The bot's PR carries `[skip ci]`, so the required checks never run and it stays BLOCKED — push one empty commit to its `all-contributors/add-<login>` branch.

## Build & Development Commands

```bash
# Build
cargo build
cargo build --release

# Test
cargo test
cargo test <test_name>           # Single test

# Lint & Format
cargo fmt                        # Format code
cargo fmt --check                # Check formatting
cargo clippy --all-targets --all-features -- -D warnings

# Run
cargo run -- --help
cargo run -- list
```

### Pre-commit (prek)

```bash
prek install                     # Install hooks (first time)
prek run --all-files             # Run all checks
prek run cargo-fmt cargo-clippy  # Run specific hooks
```

### Testing Gotchas

- **After editing `locales/app.yml`, run `touch src/lib.rs` before `cargo test`** — rust_i18n's proc-macro isn't cargo-tracked; a yml-only edit reuses the stale binary and reports false-green.
- Env-var tests MUST use `env_guard` (src/test_utils.rs) + `#[serial]` (and control `HOME` when dirs are inspected) — the dev machine has a real `~/.scoop`. Never nest `env_guard` inside `with_temp_scoop_home`: both take `ENV_LOCK` and the test deadlocks. A test that needs an isolated home plus other variables uses `env_guard` alone with `SCUV_HOME` pointed at a `TempDir` (see `resolve_env_ignores_legacy_scoop_version`).
- PR CI runs `cargo-mutants --in-diff`: new `Check`-trait impls and thin wrappers need direct dispatch tests or the Mutants gate fails.
- `.cargo/mutants.toml` `exclude_re` matches the full mutant description, not the function name — a bare `"foo"` silently drops every mutant in `foo`, including ones the tests kill. Exclude the exact description (`"delete match arm \\[major\\] in foo"`); verify the delta with `cargo mutants --config <alt>.toml --list --file '<glob>'` + `comm`.
- A green `Mutants (diff)` proves little on a test-only PR — no production lines changed — and `Mutants (full)` is skipped on PRs. Verify mutation claims locally with `cargo mutants --file '<glob>'`.
- `--in-diff` scopes mutants to the *enclosing function*, not the changed lines — touching one line in an untested function surfaces its pre-existing gaps as new failures. Check whether the missed mutant is one your change could have caused before treating it as a regression.

## Dependabot & Release Automation

- Dependabot does not read `rust-version`: it will raise a dep past the MSRV, and resolution then fails before anything compiles (every job dies on one error — #173). Block those in `.github/dependabot.yml` `ignore`; entries there are debt markers to drop when the MSRV catches up.
- Dependabot-triggered runs get the **Dependabot** secret store, not Actions'. A secret needed by both (e.g. `CODECOV_TOKEN`) must be registered twice: `gh secret set NAME --app dependabot`.
- release-plz bumps `Cargo.toml` and, with `dependencies_update = true`, runs `cargo update` on `Cargo.lock` (the 0.16.0 release PR moved ~20 crates); a step in `release-plz.yml` then commits `check-doc-references.py --fix` onto the release branch, so the release PR carries an extra `docs: sync version samples` commit. That is expected, not drift.
- That `cargo update` supersedes open Dependabot PRs — Dependabot closes them as "no longer updatable" once the release merges. Before merging a release PR, check each bumped crate's `rust_version` against the MSRV: `curl -s -A scuv https://crates.io/api/v1/crates/<name>/<ver> | jq .version.rust_version`.
- release-plz regenerates the release PR's changelog section on every push to `main` and drops hand-written bullets (`4a24eab`); `cliff.toml` renders commit subjects only, so a `BREAKING CHANGE:` footer never reaches CHANGELOG. For a breaking release, push the BREAKING bullets onto the `release-plz-*` branch right before merging it, and merge before anything else lands on `main`.
- A `feat` on `main` opens the next minor release PR at once (`features_always_increment_minor`). Before merging that PR, check whether the version was promised anywhere (`git grep 'DEPRECATION(' src; grep -c 'v<ver>' locales/app.yml`).
- `--fix` rewrites `docs/po/ko.po` alongside the four doc files, because two of them live in the mdBook and changing them moves the gettext msgids. Without it the release automation fixes the Lint gate and breaks the docs gate on the same commit. Entry-scoped and version-string-only: api.md's msgstr reads `**scuv 버전:**`, so matching on the English `scuv <version>` pattern alone would leave the Korean page on the previous release.

## MSRV Policy

**Policy**: dependency-driven · **Current MSRV**: 1.89 · **Test Matrix**: `[msrv, stable]`

The MSRV rises only when a dependency forces it — never on a schedule. Both bumps so
far were forced (1.85 → 1.88 by `let`-chains and `ignore` 0.4.30; 1.88 → 1.89 by
`serde-saphyr` 1.2). It therefore trails stable by a wide margin: 1.89 against stable
1.98.1 as of 2026-09. This was previously labelled "N-1 (current stable + 1 previous)",
which the numbers never supported.

1.89 because `rust-i18n` 4.2.2 depends on `serde-saphyr ^1.2`, and the only release
in that range (1.2.0) declares `rust-version = 1.89`. No pin escapes it — taking
`rust-i18n` 4.2.2 and holding MSRV 1.88 are mutually exclusive. The earlier 1.88 floor
came from `let`-chains and `ignore` 0.4.30. Edition 2024's own hard floor is 1.85 —
going below that means changing the edition, not just the MSRV. `rust-toolchain.toml`
auto-selects 1.89 here, so `cargo test` already runs on MSRV; `rustup override set
stable` checks the other half of the matrix.

### Bumping it

Verify first (`cargo msrv verify`, `cargo tree --duplicates`). The version string is
spread wider than it looks — `git grep '1\.<old>'` is the real checklist. The 1.88 → 1.89
bump touched 24 files:

- [ ] Declarations: `Cargo.toml` `rust-version`, `rust-toolchain.toml` `channel`,
      `.clippy.toml` `msrv`, `.github/workflows/ci.yml` (`dtolnay/rust-toolchain@<ver>`),
      `docker/Dockerfile` `ARG RUST_VERSION`. The `msrv` job's *name* is
      deliberately unversioned — it is the status-check context, and renaming it
      would strand any branch rule that requires it.
- [ ] Prose stating the current MSRV: `CLAUDE.md`, `README.md`, `CONTRIBUTING.md`,
      `llms.txt`, `llms-full.txt`, `docs/src/**`, `context7.json`,
      `.devcontainer/devcontainer.json`, `docker/docker-compose.yml`, `fuzz/**`,
      and comments in `docs.yml` / `fuzz.yml` / `msrv-check.yml` / `release-plz.yml`
- [ ] `CHANGELOG.md`: a new `[Unreleased]` entry with the reason — do NOT rewrite the
      historical entries that record earlier bumps
- [ ] README badge auto-updates from `Cargo.toml` — nothing to do

Three traps a blanket `sed` walks into: the `CHANGELOG.md` history above; the
CONTRIBUTING "Bumping MSRV" guide, whose worked example must stay one step *ahead* of
the current MSRV; and the 1.88s that are **not ours** — `mdbook-i18n-helpers`' own
upstream floor (`docs.yml`, `docs/src/development/docs-translation.md`). Verify with
`git grep -n '1\.<old>' -- . ':!Cargo.lock' ':!CHANGELOG.md' ':!docs/po/ko.po'` and
`python3 scripts/check-doc-references.py`.

Editing `docs/src/**` invalidates `docs/po/ko.po`; regenerate it (see the i18n section)
before the next `v*` tag, or `docs.yml` fails the release.

CI enforces it: `ci.yml` tests both toolchains, and `msrv-check.yml` runs
`cargo-msrv` whenever `Cargo.toml`/`Cargo.lock` changes.

## Architecture

### Module Structure

```
src/
├── cli/           # CLI parsing (clap)
│   ├── mod.rs     # Cli struct, Commands enum, ShellType
│   └── commands/  # Subcommand handlers (execute functions)
│       ├── use_env/   # Use command modular (normal, system, unset, symlink)
│       └── migrate/   # Migration subcommands
├── core/          # Domain logic
│   ├── version.rs       # Version file resolution (.scuv-version)
│   ├── metadata.rs      # Virtualenv metadata (JSON; last_used since 0.13)
│   ├── manifest.rs      # .scuv.toml manifest parsing (scuv sync)
│   ├── export_schema.rs # Portable env export/import schema (scuv export/import)
│   ├── virtualenv/      # Virtualenv entity (mod.rs + tests.rs)
│   ├── doctor/          # Health check (engine.rs, types.rs, checks/)
│   └── migrate/         # Migration from pyenv/conda/virtualenvwrapper
├── shell/         # Shell integration (bash, zsh, fish, powershell)
│   ├── bash.rs, zsh.rs, fish.rs, powershell.rs  # Shell-specific scripts
│   └── common.rs  # Shared utilities (version check, 4-shell macros)
├── uv/            # uv CLI wrapper (client.rs) + version policy (version.rs)
├── output/        # Terminal UI & JSON output
│   └── time.rs    # English fuzzy-age formatter for last_used display
├── error/         # ScoopError module (code, display, exit codes, migrate, suggestion; i18n Display)
├── paths.rs       # Path utilities (scoop_home, virtualenvs_dir)
├── validate.rs    # Validation logic
├── i18n.rs        # Internationalization (locale detection, t! macro)
└── config.rs      # Config management (~/.scuv/config.json)

locales/
└── app.yml        # Translation strings (en, ko, ja, pt-BR, es)
```

Per-module deep dives live in untracked `AGENTS.md` files (src/, src/core/, src/shell/, src/uv/, locales/, tests/, ...).

### Key Patterns

**Shell Integration**: CLI outputs shell code to stdout, shell function uses `eval` to execute (pyenv pattern).

```bash
# User runs: scuv activate myenv
# CLI outputs: export VIRTUAL_ENV="..." export PATH="..."
# Shell wrapper: eval "$(command scuv activate myenv)"
```

**Version File Priority**: `SCUV_VERSION` env > `.scuv-version` (local + parent walk) > `~/.scuv/version`

**Shell Type Detection**: Auto-detects via `FISH_VERSION`, `PSModulePath`, `ZSH_VERSION` (in priority order). Override with `--shell` option.

## CLI Commands

> **Tip:** 19 subcommands take `--json`; `grep -B12 'json: bool' src/cli/mod.rs` lists them. Not on: activate, completions, deactivate, export (already emits JSON), init, migrate (its subcommands have it, the parent doesn't), resolve, run, shell.

| Command | Aliases | Description |
|---------|---------|-------------|
| `scuv list` | `ls` | List virtualenvs or Python versions (`--sort name|created|last-used`) |
| `scuv create <NAME> [VER]` | - | Create virtualenv (`--install-python` for lazy install) |
| `scuv use <NAME>` | - | Set + activate environment |
| `scuv remove <NAME>` | `rm`, `delete` | Remove virtualenv |
| `scuv clone <SRC> <DST>` | - | Duplicate an environment |
| `scuv diff <A> <B>` | - | Compare two environments (Python, packages, metadata) |
| `scuv export <NAME>` | - | Snapshot an env as portable JSON |
| `scuv import <FILE>` | - | Recreate an env from an export file |
| `scuv sync` | - | Apply `.scuv.toml` (create env + install packages) |
| `scuv install [VER]` | - | Install Python version |
| `scuv uninstall <VER>` | - | Uninstall Python version |
| `scuv doctor` | - | Diagnose installation |
| `scuv info <NAME>` | - | Show virtualenv details |
| `scuv status` | - | Summarise the currently active environment |
| `scuv which <EXE>` | - | Resolve an executable inside the active env |
| `scuv run <ENV> -- <CMD>` | - | Run a command inside an env without activating |
| `scuv init <SHELL>` | - | Shell init script |
| `scuv completions <SHELL>` | - | Completion script |
| `scuv lang [CODE]` | - | Get/set language (en, ko, ja, pt-BR, es) |
| `scuv migrate list` | - | List migratable environments |
| `scuv migrate @env <NAME>` | - | Migrate single environment |
| `scuv migrate all` | - | Migrate all environments (parallel via rayon) |
| `scuv gc` | - | GC orphan virtualenvs (default dry-run; `--yes` removes, `--aggressive` also for Pythons, `--older-than <n>d/w/y` flags stale envs by `last_used`) |
| `scuv prune` | - | Prune the uv cache (`uv cache prune` wrapper) |
| `scuv verify [NAME]` | - | Per-env health diagnosis (6 checks; `--strict` for CI gates) |
| `scuv man [DIR]` | - | Generate man pages (stdout or `scuv.1` + `scuv-<sub>.1` files in DIR) |
| `scuv self update` | - | Update scuv itself from crates.io |
| `scuv use system` | - | Use system Python (deactivate) |
| `scuv use --unset` | - | Remove local/global version file |
| `scuv shell <NAME>` | - | Set shell-specific env (eval required) |
| `scuv shell --unset` | - | Clear shell-specific setting |
| `scuv resolve` | - | *(hidden)* Print the current environment name |
| `scuv activate <NAME>` | - | *(hidden)* Output activation script (eval required) |
| `scuv deactivate` | - | *(hidden)* Output deactivation script (eval required) |

The last three carry `#[command(hide = true)]` — the shell wrappers call them via `eval`, they are absent from `--help`, and they deliberately have no page under `docs/src/commands/`.

### Global Options

| Option | Description |
|--------|-------------|
| `--quiet` | Minimal output |
| `--no-color` | Disable colors |

### Common Options

| Option | Availability | Description |
|--------|--------------|-------------|
| `--json` | Most commands | JSON output (for scripts) |
| `-h`, `--help` | All commands | Show help |
| `-V`, `--version` | All commands | Show version |

## Naming Conventions

| Item | Rule | Example |
|------|------|---------|
| CLI commands | lowercase | `scuv create` |
| Environment variables | SCREAMING_SNAKE | `SCUV_HOME` |
| Error types | PascalCase | `ScoopError` |
| Version file | dot-prefix | `.scuv-version` |

### Environment Name Rules

- Regex: `^[a-zA-Z][a-zA-Z0-9_-]*$`
- Must start with letter (not number, to distinguish from version strings like "3.12")
- Reserved words: 28 entries in `RESERVED_NAMES` (src/validate.rs) — read it rather than trusting a copy here; it grows with every new subcommand

## Documentation Style

### Doc Comment Rules

- `///` for items (functions, structs), `//!` for modules
- Summary line: verb-first, period-end (e.g., `/// Returns the...`)
- Required sections for `pub` items:
  - `# Examples` - Always include usage examples
  - `# Errors` - When returning `Result`
  - `# Panics` - When panic is possible
  - `# Safety` - For `unsafe fn`

### Doctest

```rust
/// Creates a new environment.
///
/// # Examples
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let env = VirtualEnv::create("myenv", "3.12")?;
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns [`ScoopError::InvalidName`] if the name is invalid.
pub fn create(name: &str, version: &str) -> Result<VirtualEnv, ScoopError>
```

- Use `#` prefix for hidden setup code
- Use `?` operator instead of `unwrap()`
- Attributes: `no_run`, `should_panic`, `compile_fail`

## Design Documents

`.docs/` contains internal design documents (git excluded):

| Category | Path | Description |
|----------|------|-------------|
| ADR | `adr/0001-architecture.md` | Architecture decisions |
| Plan | `plan/mvp.md` | MVP feature scope |
| Spec | `spec/naming.md` | Naming conventions |
| Spec | `spec/cli-options.md` | CLI options spec |
| Dev | `dev/code-quality.md` | Linting/formatting setup |
| Dev | `dev/documentation.md` | Rust documentation best practices |

### Document Naming Rules

- **ADR**: `NNNN-kebab-title.md` (e.g., `0001-architecture.md`)
- **Stable docs**: `kebab-case.md` (e.g., `doctor.md`, `cli-options.md`)
- **WIP docs**: `[type] YYYY-MM-DD-kebab-title.md`
  - Workflow types: `[brainstorm]`, `[research]`, `[impl]`, `[refactor]`
  - Conventional types: `[feat]`, `[fix]`, `[docs]`, `[chore]`, `[perf]`, `[test]`
  - e.g., `[research] 2026-01-10-migrate.md`, `[impl] 2026-01-10-migrate-phase1.md`
- **Forbidden**: spaces, Korean filenames, SCREAMING_SNAKE_CASE

### Frontmatter (Required)

All `.docs/` files must include YAML frontmatter:

```yaml
---
title: filename-without-extension
tags:
  - scuv           # Required for all files
  - [folder-tag]    # adr, design, spec, dev, plan, wip, done
  - [content-tags]  # Relevant keywords
---
```

## Error Handling

Use `thiserror` with manual `Display` impl for i18n support:

```rust
// ScoopError uses thiserror for Error trait, but Display is manual for i18n
#[derive(Error, Debug)]
pub enum ScoopError {
    #[error("")]  // Placeholder - Display impl handles actual message
    VirtualenvNotFound { name: String },
}

impl std::fmt::Display for ScoopError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::VirtualenvNotFound { name } => {
                write!(f, "{}", t!("error.virtualenv_not_found", name = name))
            }
            // ...
        }
    }
}
```

## Internationalization (i18n)

**Locale Priority**:
1. `SCUV_LANG` environment variable
2. `~/.scuv/config.json` setting
3. System locale (sys-locale)
4. Default: `en`

**Usage**:
```rust
use crate::i18n::t;

// In CLI output
println!("{}", t!("create.creating", name = name));

// In error messages (via Display impl)
t!("error.virtualenv_not_found", name = name)
```

**Translation file**: `locales/app.yml`
- 222 keys total (error.* 41, suggestion.* 16); parity across all 5 locales enforced by tests/i18n_completeness.rs
- Adding a locale touches 7 files: `locales/app.yml`, `SUPPORTED_LANGS` (src/i18n.rs), `LOCALES` (tests/i18n_completeness.rs), and the hand-written `scuv lang` completion lists in all four shells (`src/shell/{bash,zsh,fish,powershell}.rs`). Missing `LOCALES` fails silently — CI passes with that locale unverified. A missed completion list fails `lang_completion_list_matches_supported_langs` in that shell's module, and a doc that still enumerates the old list fails `check-doc-references.py`. Contributor guide: `docs/src/development/translation.md`.
- ko conventions: no semicolons in ko values; "scuv"(스커브) has no batchim — particles are 가/를/는/와/로 (never 이/을/은/과/으로). Hand-edit ko/ja, never blind-sed.
- `docs/po/ko.po`: regenerate via `MDBOOK_OUTPUT='{"xgettext": {}}' mdbook build -d po && msgmerge --update po/ko.po po/messages.pot`; CI (tag push) requires the committed file to round-trip byte-identical. Install the versions `docs.yml` pins (mdbook 0.5.3, mdbook-i18n-helpers 0.4.0) — latest produces a different `.pot`. `messages.pot` is untracked; only `ko.po` is committed.
  - Reproducing that guard locally also needs: restore `POT-Creation-Date`/`PO-Revision-Date` from the pre-merge copy (msgmerge rewrites both to "now" → phantom diff), and `msgcat --width=79` any hand-written msgstr (unwrapped lines are gettext-version-sensitive; CI's gettext may differ from Homebrew's). Done when two consecutive runs leave the file byte-identical with 0 fuzzy.
- The ko.po staleness guard runs in two places: `docs-check.yml` ("Documentation checks") on PRs that touch the docs paths it lists, and `docs.yml` (build + deploy) on `v*` tags. A PR outside those paths only gets `msgfmt --check` from the CI Lint job.
- `docs.yml`'s `deploy` job has no branch guard — a `workflow_dispatch` from any branch publishes that branch to production Pages. Verify on `main` only.

## Docker Development

```bash
# Interactive shells (auto-builds scuv on entry)
make docker-shell          # bash
make docker-shell-zsh      # zsh
make docker-shell-fish     # fish

# Run tests (integration tests run in Docker)
make test-integration    # Docker integration tests
make test-all            # unit + integration
```

**Features**:
- Auto-build on source change detection (`.rs`, `locales/*.yml`)
- Live reload for shell scripts (no image rebuild needed)
- Workspace scuv takes precedence over image-installed version
