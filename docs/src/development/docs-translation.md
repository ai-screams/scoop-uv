# Documentation Translation (mdBook)

> For translating CLI strings (the ~115 keys in `locales/app.yml`
> consumed by the Rust binary), see [translation](translation.md)
> instead. This page covers user-documentation translation only.

scuv's user documentation lives in `docs/src/*.md` and is the
single English source of truth. Translations are layered on top
via [gettext](https://www.gnu.org/software/gettext/) `.po` files
under `docs/po/`, processed by the
[mdbook-i18n-helpers](https://github.com/google/mdbook-i18n-helpers)
preprocessor at render time. Untranslated strings automatically
fall back to English, so a partial translation is always
deployable.

## URL layout

- English (canonical): `https://ai-screams.github.io/scoop-uv/`
- Korean: `https://ai-screams.github.io/scoop-uv/ko/`

The locale switcher in the top-right of every page jumps between
the same page on each side.

## Prerequisites

```bash
# macOS
brew install gettext        # msginit / msgmerge / msgfmt
cargo install mdbook --version 0.5.3 --locked
cargo install mdbook-i18n-helpers --version 0.4.0 --locked

# Linux
sudo apt-get install -y gettext
# Then the same `cargo install` lines as above.
```

The `cargo install` step needs Rust **1.88 or newer** (helpers'
upstream MSRV). The project's `rust-toolchain.toml` pins 1.85
for the CRATE itself; for the docs tooling, use `cargo +stable
install ...` from outside the repo, or just rely on whatever
stable toolchain is on `ubuntu-latest` in CI.

## Workflow: updating an existing translation

When you edit a page in `docs/src/`, the existing translations
need to learn about the new / changed strings.

```bash
cd docs

# 1. Re-extract template (overwrites docs/po/messages.pot).
MDBOOK_OUTPUT='{"xgettext": {}}' mdbook build -d po

# 2. Merge the new template into your locale's .po file.
#    --backup=none avoids leaving a stray ko.po~ around.
msgmerge --update --backup=none po/ko.po po/messages.pot

# 3. Open po/ko.po in your editor. Look for:
#    - new empty `msgstr ""` entries  →  add translations
#    - "#, fuzzy" markers              →  review and remove flag
```

`docs/po/messages.pot` is in `.gitignore` — it's regenerated on
every CI run, committing it would create churn. The locale `.po`
files (`docs/po/ko.po`, etc.) ARE committed; they're the
translation memory.

## Workflow: previewing a translated build

```bash
cd docs

# English (root)
mdbook build -d book
open book/index.html

# Korean (book/ko subdir)
MDBOOK_BOOK__LANGUAGE=ko mdbook build -d book/ko
open book/ko/index.html
```

`mdbook serve` works too if you want live reload:

```bash
MDBOOK_BOOK__LANGUAGE=ko mdbook serve -d book/ko
```

## Workflow: adding a brand-new locale (e.g. `ja`)

```bash
cd docs

# Initialise an empty translation file.
msginit -i po/messages.pot -l ja -o po/ja.po --no-translator

# Translate msgids in po/ja.po (untranslated ones fall back to English).

# Add a CI build step for the new locale in
# .github/workflows/docs.yml, e.g.:
#
#   - name: Build Japanese (book/ja)
#     working-directory: docs
#     env:
#       MDBOOK_BOOK__LANGUAGE: ja
#     run: mdbook build -d book/ja
#
# Then expand the locale switcher in docs/theme/head.hbs to
# include the new locale link.
```

## CI guard: stale translations

The `Verify ko translations are in sync` step in
`.github/workflows/docs.yml` regenerates the pot template from
the current English source, runs `msgmerge --update` against
the committed `ko.po`, and fails the build if the result
differs. This means English edits land with their corresponding
`.po` updates in the same PR, or they don't land at all.

To fix a CI failure on this step:

```bash
cd docs
MDBOOK_OUTPUT='{"xgettext": {}}' mdbook build -d po
msgmerge --update --backup=none po/ko.po po/messages.pot
git add po/ko.po
git commit --amend  # or as a separate fixup commit
```

## Style guidelines

The same casual-tone guidelines from [translation](translation.md)
apply here. Don't try to translate code samples or CLI commands
literally — only the prose around them. Code-block content is
shown verbatim regardless of locale; mdbook-i18n-helpers
intentionally does not interpolate translations inside fenced
code blocks for unmarked code, though it does extract code
comments (`# Install Python`) as separate msgids so you can
translate those if it helps readability.

## Known limitations

- Search index is built per-locale. Korean search results only
  hit Korean pages, English only hits English. This is the
  intended mdBook behaviour.
- The locale switcher uses a JS-injected DOM element; users with
  JS disabled won't see it. They can navigate via direct URL
  (`/scoop-uv/ko/...`) or browser bookmarks.
- mdbook-i18n-helpers' `xgettext` doesn't extract HTML tables'
  cell-by-cell content as separate msgids when the table is
  written in pipe-syntax — it extracts the whole table as a
  single block. For now this is acceptable; for fine-grained
  table translation we'd need to switch to per-row HTML tables
  in the source.
