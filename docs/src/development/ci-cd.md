# CI/CD

Eleven workflows guard this repository. This page explains what each one
protects, the decisions behind how they are wired, and the failure modes
that shaped them.

## Pipeline at a glance

| Workflow | Trigger | Guards against |
|----------|---------|----------------|
| `ci.yml` | PR, main | Unformatted code, clippy warnings, failing tests, MSRV drift, broken shell scripts, stale reference docs, malformed `.po` |
| `integration-test.yml` | PR, main | Migration breaking for pyenv / conda / virtualenvwrapper users |
| `coverage.yml` | PR, main | Untested code paths going unnoticed |
| `bench.yml` | PR, main | Performance regressions in parsing and validation |
| `mutants.yml` | PR (diff), weekly (full) | Tests that execute code without asserting on it |
| `security.yml` | PR, main, weekly | Vulnerable dependencies, disallowed licences, untrusted sources |
| `msrv-check.yml` | `Cargo.toml`/`Cargo.lock` changes | A declared MSRV that no longer compiles |
| `docker-build.yml` | `docker/**` changes, weekly | Broken published images, vulnerable image contents |
| `fuzz.yml` | Weekly | Parser crashes on hostile input |
| `docs.yml` | `v*` tags | Broken documentation site |
| `release-plz.yml` | main | Manual release mistakes |

## Cross-cutting decisions

### Cancel PR runs, never cancel main

Every workflow that runs on both uses the same concurrency block:

```yaml
concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: ${{ github.ref != 'refs/heads/main' }}
```

Pushing a fixup to a PR should abandon the previous run — nobody needs
results for a commit that no longer exists. A run on `main` is different:
it produces state that later runs depend on. `bench.yml` writes the
benchmark baseline to `gh-pages`, and a cancelled main run leaves the
next PR comparing against a stale baseline.

`release-plz.yml` uses the same block with `cancel-in-progress: false`: a
release must never be cancelled, but two releases must never run at once
either, and a group delivers both.

### The MSRV is verified twice, deliberately

`ci.yml` has an `msrv` job that builds and tests on 1.88. `msrv-check.yml`
separately runs `cargo msrv verify`. These answer different questions:

- **`ci.yml`** — does the code work on the version we claim?
- **`msrv-check.yml`** — is the version we claim still the *lowest* one
  that works?

The second catches the case where a dependency bump silently raises the
real floor while `Cargo.toml` still advertises the old one. It only runs
when `Cargo.toml` or `Cargo.lock` changes, because that is the only way
the answer can change.

`cargo-msrv` itself is pinned to `^0.18`: 0.19 requires rustc 1.91, which
this project's own toolchain pin cannot satisfy.

### Gate what is stable, track what is noisy

Not every measurement can be a gate. `bench.yml` splits its benchmarks by
how reproducible they are on shared runners:

| Group | Benches | Observed spread | Behaviour |
|-------|---------|-----------------|-----------|
| CPU | `parsing`, `validation` | ~1.4x | Fails the build past 150% |
| Filesystem | `path_lookup` | ~3.4x | Recorded, never fails |

`find_executable_in` calls `stat`. Across 38 runs of unchanged code on
`main` it produced anywhere from 574 to 1931 ns depending on which runner
the job landed on. No threshold both tolerates that and catches a real
regression, so the group is tracked on `gh-pages` for trend visibility
and `fail-on-alert: false` keeps it out of the gate.

This matters because a noisy gate is worse than no gate: it blocks
unrelated work and trains reviewers to ignore red.

### Mutation testing runs at two scopes

`cargo test` proves a line executed. It does not prove a test would
notice if that line were wrong. `cargo-mutants` injects deliberate
defects — flipping `>` to `>=`, replacing a return value — and reports
any that the suite fails to catch.

- **On PRs** (`--in-diff`): only the lines this PR changed. Fast enough
  to gate on.
- **Weekly** (full): every candidate in scope. Tracked rather than gated
  for now — the step tolerates exit 2 (mutants missed) and nothing else,
  so a broken baseline or a timeout still fails loudly while the backlog
  in [Known gaps](#known-gaps) is worked through.

`.cargo/mutants.toml` carries the exclusions, each with a written
rationale. Most exclude code whose mutations can only be killed by
spawning a real `uv` or `python` — a gap we accept rather than a test
hole. Add exclusions there with the reason, not silently.

New `Check` trait implementations and thin wrappers need direct dispatch
tests, or the PR gate reports them as `MISSED`.

### One cache per toolchain, one writer per cache

`Swatinem/rust-cache` keys on `shared-key`, so jobs sharing a key share a
cache:

| `shared-key` | Jobs |
|--------------|------|
| `stable-ci` | `ci.yml` lint / test / shellcheck, `coverage.yml`, `mutants.yml` |
| `msrv` | `ci.yml` msrv |
| `msrv-verify` | `msrv-check.yml` |
| `bench` | `bench.yml` |

MSRV artifacts are kept separate on purpose: a different compiler
produces incompatible output, and sharing would mean both jobs
permanently invalidating each other.

Within a shared key there is one writer. `ci.yml` marks lint and
shellcheck `save-if: false` so only the test job — which produces the
richest artifacts, including test binaries — populates the cache; in
`mutants.yml` the incremental job writes and the weekly full job reads.

Note that `rust-cache` folds workflow-level `env` into its key hash. Two
jobs can declare the same `shared-key` and still land on different
caches if their workflows set different environment variables.

## Failure modes worth remembering

These cost real debugging time. They are recorded so the next person
recognises them faster.

### Criterion errors corrupt the benchmark parser output

`rust-cache` restores `target/` with the criterion tree present but its
`sample.json` baselines pruned out. Criterion then fails to load a
baseline it can see should exist, and writes the error to stdout
*mid-line*:

```
test clap_parse_create ... Criterion.rs ERROR: error: Failed to access file
".../base/sample.json": No such file or directory
bench:      41,347 ns/iter (+/- 2,364)
```

`benchmark-action`'s parser needs `test NAME ... bench: N ns/iter` on one
line. Split in two, it reports "no benchmark result" even though every
benchmark ran. A cold cache passes precisely because there is no tree to
half-load.

Each bench step therefore starts with `rm -rf target/criterion`. The gate
compares against `gh-pages` history and never against criterion's local
baseline, so removing it costs nothing.

### Two benchmark-action invocations collide on gh-pages

The first invocation fetches `gh-pages` into a local branch and commits
its entry onto it — even on PRs, where it simply never pushes. A second
invocation's identical fetch is then a non-fast-forward update and git
rejects it, failing the step before any comparison happens.
`skip-fetch-gh-pages: true` on the second step reuses what the first
fetched.

### `cargo bench | tee` swallows failures

Without `set -o pipefail` the step exits with `tee`'s status, so a
genuinely failing `cargo bench` reports success and resurfaces one step
later as a confusing parse error.

### Docs guards only run on release tags

`docs.yml` — the mdBook build and the `ko.po` staleness round-trip —
triggers only on `v*` tags. Nothing about the documentation site is
verified on a PR. Two checks were moved into the `ci.yml` Lint job to
close the worst of that gap:

- `scripts/check-doc-references.py` compares facts copied into
  `README.md`, `CONTRIBUTING.md`, `llms.txt`, `llms-full.txt` and the
  docs against the code that owns them — version and MSRV from
  `Cargo.toml`, reserved names from `src/validate.rs`, key count from
  `locales/app.yml`. Every check is doc-against-code; comparing the three
  `llms` files to each other would pass with all three wrong.
- `msgfmt --check` on `docs/po/*.po` catches structural damage such as a
  `msgstr` whose trailing newline no longer matches its `msgid`.

Editing any page under `docs/src/` still requires regenerating `ko.po` in
the same commit. See [Docs Translation](docs-translation.md).

## Secrets

| Secret | Used by | Purpose |
|--------|---------|---------|
| `RELEASE_PLZ_TOKEN` | `release-plz.yml` | Push release PRs and tags |
| `CARGO_REGISTRY_TOKEN` | `release-plz.yml` | Publish to crates.io |
| `GITHUB_TOKEN` | bench, docker | Push `gh-pages`, publish to ghcr.io |

Default workflow permissions are `read`. Workflows that need more declare
it explicitly — `bench.yml` needs `contents: write` for `gh-pages`,
`docker-build.yml` needs `packages: write` for the registry.

## Releases

`release-plz` reads Conventional Commits on `main` and prepares the
release; `release-plz.toml` holds the policy:

- `release_commits = "^(feat|fix|perf|refactor|revert)"` — documentation
  and chore commits do not trigger a release on their own.
- `features_always_increment_minor = true` — a `feat` bumps the minor
  version even in `0.x`, where cargo's default would only bump the patch.
- Changelog generation goes through `git-cliff` (`cliff.toml`).

Merging the release PR is what publishes: it creates the tag, the GitHub
release, and the crates.io upload — and the `v*` tag is also what deploys
the documentation site.

## Known gaps

Accurate as of the last revision of this page. Verify before relying on
any of these being fixed.

- **No coverage threshold.** Uploads work again, but there is no
  `codecov.yml`, so nothing sets a target percentage or a PR status
  policy. Coverage is reported and never enforced.
- **55 unreviewed mutation escapes.** The weekly full run now finishes,
  and the run it has been failing to complete already found 55 mutants no
  test kills — 46 of them under `src/core/migrate/**`. Roughly 23 are pure
  logic mutations (`&&` to `||`, `delete !`) that unit tests ought to
  catch; the rest are return-value mutations that may need a real `uv` or
  `conda` to observe, in which case they belong in `.cargo/mutants.toml`
  with a rationale. Until that triage happens the job tolerates exit 2.
- **Docs are only verified on release tags.** `docs.yml` still runs
  nowhere else, so an mdBook build failure or a stale `ko.po` surfaces at
  release time. The two cheapest checks were moved into the Lint job; the
  build itself was not.
- **The cache sits near its limit.** 8.34 GB of the 10 GB allowance, of
  which `v0-rust-*` is only about 1 GB — the bulk is BuildKit blobs from
  the Docker workflows. Nothing is failing yet; eviction is LRU.

### Recently closed

Left here because the reasoning is worth keeping, not because anything is
outstanding.

- Coverage uploads were rejected for months with
  `Token required because branch is protected` while the job reported
  success, because `fail_ci_if_error: false` hid it. The org allows
  tokenless uploads, but that path only covers fork PRs. Fixed by wiring
  `CODECOV_TOKEN` and letting a rejected upload fail the job.
- The weekly full mutation run had never once completed — 60 minutes
  killed it every time, and GitHub reports a timed-out job as *cancelled*,
  which reads as benign. Measured at 385/446 mutants in 60 minutes, so the
  timeout is now 90.
- `aquasecurity/trivy-action@master` and `astral-sh/setup-uv@v7` are now
  pinned to release tags. Dependabot could not follow either: it cannot
  bump a branch ref, and setup-uv stopped publishing moving major tags at
  v8.
- `stable-ci` had four writers and, because `rust-cache` hashes
  workflow-level `env`, was never actually shared with `coverage.yml` or
  `mutants.yml` at all. Each now names the key it really uses.
- `release-plz.yml` had no concurrency group. It has one now, with
  `cancel-in-progress: false` — which delivers the "must always complete"
  intent that omitting the block only half-achieved.
