# Translation Guide

This document provides guidelines for contributing translations to scuv.

## Current Status

For the latest translation status, see:

- **[Issue #42: i18n Translation Tracking](https://github.com/ai-screams/scoop-uv/issues/42)**
- Run `scuv lang --list` to see currently supported languages

---

## Contribution Process

### Step 1: Fork and Clone

```bash
git clone https://github.com/YOUR_USERNAME/scoop-uv.git
cd scoop-uv
```

### Step 2: Add Translations

Edit `locales/app.yml` and add your language to **every key**:

```yaml
create.success:
  en: "Created '%{name}' environment"
  ko: "'%{name}' 환경 생성됨"
  pt-BR: "Ambiente '%{name}' criado"
  { lang }: "Your translation here"  # Add your language code and translation
```

**Important:**

- Add translations to all 226 keys
- Keep placeholder syntax exactly: `%{name}`, `%{version}`, etc.
- Preserve special characters: `→`, quotes, backticks

The key count grows between releases. For the current number:

```bash
grep -c '^[a-z_][a-zA-Z0-9_.]*:$' locales/app.yml
```

### Step 3: Register Language

Edit `src/i18n.rs` and add your language to `SUPPORTED_LANGS`:

```rust
pub const SUPPORTED_LANGS: &[(&str, &str)] = &[
    ("en", "English"),
    ("ko", "한국어"),
    ("ja", "日本語"),
    ("pt-BR", "Português (Brasil)"),
    ("{lang}", "Your Language Name"),  // Add your language
];
```

**Language Code Format:**

- Use [BCP 47](https://en.wikipedia.org/wiki/IETF_language_tag) format
- Simple languages: `ja`, `fr`, `es`, `de`, `it`
- Regional variants: `pt-BR`, `zh-CN`, `zh-TW`, `es-MX`

**Three more places list the supported locales.** The first one matters most.

**1. `tests/i18n_completeness.rs`** — add your code to the `LOCALES` const:

```rust
const LOCALES: &[&str] = &["en", "ko", "ja", "pt-BR", "{lang}"];
```

This is the CI gate that checks every key exists in every locale. If your
language is missing from this list, CI passes while your translation goes
completely unverified. It is the only step in this guide that fails silently
— everything else tells you what is wrong.

**2. Shell completions** — the `scuv lang` candidate lists are hand-written
in two shells:

- `src/shell/fish.rs` — the `complete -c scuv ... from lang` lines
- `src/shell/zsh.rs` — the `langs=(...)` array

bash and PowerShell do not enumerate locales, so there is nothing to change
there.

**3. Locale loops in tests (optional)** — `src/error/mod.rs` and
`src/error/suggestion.rs` iterate the supported locales. Adding yours gives
your translation unit-level coverage. Skip it if you would rather not touch
Rust, and a maintainer can add it during review.

### Step 4: Test Locally

`rust-i18n`'s proc macro is not tracked by cargo, so editing `locales/app.yml`
on its own does not trigger a rebuild. `cargo test` reuses the stale binary and
reports a false pass. Always `touch src/lib.rs` first.

```bash
# Build and test (the touch is required — see the note above)
touch src/lib.rs
cargo build
cargo test

# Test your language (replace {lang} with your language code)
SCUV_LANG={lang} ./target/debug/scuv --help
SCUV_LANG={lang} ./target/debug/scuv lang
```

### Step 5: Create Pull Request

**Required files in PR:**

- [ ] `locales/app.yml` - All 226 keys translated
- [ ] `src/i18n.rs` - Language registered in SUPPORTED_LANGS
- [ ] `tests/i18n_completeness.rs` - Language added to LOCALES
- [ ] `src/shell/fish.rs`, `src/shell/zsh.rs` - Completion lists updated

**PR Title Format:**

```
docs(i18n): add {Language Name} translation
```

---

## Style Guidelines

### Philosophy: Your Language, Your Style

**We trust translators.** You know your language and community best.

- **Word choice is yours** — Pick terms that feel natural to native speakers
- **Creativity welcome** — Witty expressions are fine if they're clear and widely understood
- **Casual over formal** — scuv is a friendly CLI tool, not enterprise software

### General Principles

1. **Concise**: CLI messages should be short and clear
2. **Natural**: Use natural phrasing, not word-for-word translation
3. **Casual**: Friendly, approachable tone — like talking to a colleague
4. **Clear**: Wit is great, but clarity comes first

### Tone Examples

```
# Too formal (avoid)
"The environment has been successfully created."

# Too robotic (avoid)
"Environment creation: complete."

# Good - casual and clear
"Created 'myenv' — ready to go!"
"'myenv' is ready"
```

### Message Types

| Type     | English Example             | Guidance                            |
|----------|-----------------------------|-------------------------------------|
| Progress | "Installing..."             | Use progressive/ongoing form        |
| Success  | "Created 'myenv'"           | Completion — feel free to add flair |
| Error    | "Can't find 'myenv'"        | Clear and actionable                |
| Hint     | "→ Create: scuv create..." | Helpful, not lecturing              |

### Translator's Discretion

These decisions are **up to you**:

- **Vocabulary**: Choose words that resonate with your community
- **Idioms**: Use local expressions if they fit naturally
- **Humor**: Light wit is welcome (e.g., ice cream puns if appropriate)
- **Formality level**: Lean casual, but match your culture's CLI norms

**Only requirement**: The meaning must be clear to users.

### Technical Terms

For technical vocabulary:

1. **Check your community** — What do Python developers in your language use?
2. **Consistency** — Pick one term and stick with it throughout
3. **Loanwords OK** — If your community uses English terms (e.g., "install"), that's fine

**Tip:** Study existing translations in `locales/app.yml` for reference, but don't feel bound by them.

---

## Glossary

### Do NOT Translate

These terms should remain in English in all languages:

| Term                | Reason                           |
|---------------------|----------------------------------|
| `scuv`             | Brand name                       |
| `uv`                | Tool name                        |
| `pyenv`             | Tool name                        |
| `conda`             | Tool name                        |
| `virtualenv`        | Technical term                   |
| `virtualenvwrapper` | Tool name                        |
| `Python`            | Language name                    |
| `shell`             | Technical term (bash, zsh, fish) |
| `JSON`              | Format name                      |
| `PATH`              | Environment variable             |
| `pip`               | Tool name                        |

### Commands - Never Translate

All commands and code examples must stay in English:

```yaml
# WRONG - Command translated
hint: "→ Create: {translated_command} myenv 3.12"

# CORRECT - Only description translated
hint: "→ {translated_word}: scuv create myenv 3.12"
```

### Common Terms to Translate

These are core concepts you'll need to translate. Reference existing translations for consistency:

| English       | What to look for                               |
|---------------|------------------------------------------------|
| environment   | Your language's term for "environment"         |
| create        | Common verb for "make/create"                  |
| remove/delete | Common verb for "delete/remove"                |
| install       | Standard software installation term            |
| uninstall     | Standard software removal term                 |
| activate      | Term for "enable/turn on"                      |
| deactivate    | Term for "disable/turn off"                    |
| migrate       | IT term for migration (often kept as loanword) |
| version       | Your language's term for "version"             |
| path          | Your language's term for file path             |
| error         | Your language's term for "error"               |
| success       | Your language's term for "success"             |

**Tip:** Check how these terms are translated in existing translations for reference.

### Ice Cream Metaphor (README only)

scuv uses ice cream metaphors in documentation:

| Term    | Meaning             | Guidance                                         |
|---------|---------------------|--------------------------------------------------|
| scuv   | The tool            | Always keep as "scuv"                           |
| flavor  | virtualenv          | Translate if the metaphor works in your language |
| freezer | ~/.scuv/ directory | Translate if the metaphor works                  |

**Note:** The metaphor is mainly in README.md, not in CLI messages (`locales/app.yml`).

---

## File Structure

### locales/app.yml

```yaml
# Categories in order:
# 1. lang.*        - Language command messages
# 2. create.*      - Create command messages
# 3. remove.*      - Remove command messages
# 4. list.*        - List command messages
# 5. use.*         - Use command messages
# 6. install.*     - Install command messages
# 7. uninstall.*   - Uninstall command messages
# 8. migrate.*     - Migrate command messages
# 9. error.*       - Error messages
# 10. suggestion.* - Suggestion/hint messages
```

### src/i18n.rs

```rust
// Language detection priority:
// 1. SCUV_LANG environment variable
// 2. Config file (~/.scuv/config.json)
// 3. System locale
// 4. Default: "en"

pub const SUPPORTED_LANGS: &[(&str, &str)] = &[
    ("en", "English"),
    // ... existing languages
    // Add new languages here
];
```

---

## Common Mistakes

### 1. Missing `SUPPORTED_LANGS` Registration

**Symptom:** Translation exists but `scuv lang {code}` doesn't work

**Fix:** Add language to `src/i18n.rs` SUPPORTED_LANGS

### 2. Broken Placeholders

```yaml
# WRONG - Missing placeholder
error: "Cannot find environment"

# CORRECT - Placeholder preserved
error: "Cannot find '%{name}' environment"
```

### 3. Translating Commands

```yaml
# WRONG - Command translated
hint: "→ List: {translated} list"

# CORRECT - Only label translated
hint: "→ {Translated Label}: scuv list"
```

### 4. Inconsistent Key Coverage

All languages must have ALL keys. Missing keys fall back to English.

### 5. Missing `LOCALES` Registration

**Symptom:** CI is green, but nothing ever checked your locale

**Fix:** Add your code to the `LOCALES` const in `tests/i18n_completeness.rs`

### 6. Stale i18n Cache

**Symptom:** `cargo test` passes, but your new strings never show up

**Fix:** Run `touch src/lib.rs` before `cargo test`

---

## Testing Checklist

Before submitting PR:

- [ ] All 226 keys translated
- [ ] All placeholders preserved (`%{name}`, `%{version}`, etc.)
- [ ] Language registered in SUPPORTED_LANGS
- [ ] Language added to LOCALES in `tests/i18n_completeness.rs`
- [ ] Shell completion lists updated (fish, zsh)
- [ ] `cargo build` succeeds
- [ ] `touch src/lib.rs` run, then `cargo test` passes
- [ ] `SCUV_LANG={code} scuv lang` shows your language
- [ ] Messages display correctly in terminal

---

## Questions?

- Open an issue: [GitHub Issues](https://github.com/ai-screams/scoop-uv/issues)
- See existing translations for reference: `locales/app.yml`
