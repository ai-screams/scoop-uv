# Translation Guide

This document provides guidelines for contributing translations to scoop.

## Current Status

For the latest translation status, see:

- **[Issue #42: i18n Translation Tracking](https://github.com/ai-screams/scoop-uv/issues/42)**
- Run `scoop lang --list` to see currently supported languages

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

- Add translations to ALL ~106 keys
- Keep placeholder syntax exactly: `%{name}`, `%{version}`, etc.
- Preserve special characters: `→`, quotes, backticks

### Step 3: Register Language

Edit `src/i18n.rs` and add your language to `SUPPORTED_LANGS`:

```rust
pub const SUPPORTED_LANGS: &[(&str, &str)] = &[
    ("en", "English"),
    ("ko", "한국어"),
    ("pt-BR", "Português (Brasil)"),
    ("{lang}", "Your Language Name"),  // Add your language
];
```

**Language Code Format:**

- Use [BCP 47](https://en.wikipedia.org/wiki/IETF_language_tag) format
- Simple languages: `ja`, `fr`, `es`, `de`, `it`
- Regional variants: `pt-BR`, `zh-CN`, `zh-TW`, `es-MX`

### Step 4: Test Locally

```bash
# Build and test
cargo build
cargo test

# Test your language (replace {lang} with your language code)
SCOOP_LANG={lang} ./target/debug/scoop --help
SCOOP_LANG={lang} ./target/debug/scoop lang
```

### Step 5: Create Pull Request

**Required files in PR:**

- [ ] `locales/app.yml` - All 106 keys translated
- [ ] `src/i18n.rs` - Language registered in SUPPORTED_LANGS

**PR Title Format:**

```
feat(i18n): add {Language Name} translation
```

---

## Style Guidelines

### Philosophy: Your Language, Your Style

**We trust translators.** You know your language and community best.

- **Word choice is yours** — Pick terms that feel natural to native speakers
- **Creativity welcome** — Witty expressions are fine if they're clear and widely understood
- **Casual over formal** — scoop is a friendly CLI tool, not enterprise software

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
| Hint     | "→ Create: scoop create..." | Helpful, not lecturing              |

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
| `scoop`             | Brand name                       |
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
hint: "→ {translated_word}: scoop create myenv 3.12"
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

scoop uses ice cream metaphors in documentation:

| Term    | Meaning             | Guidance                                         |
|---------|---------------------|--------------------------------------------------|
| scoop   | The tool            | Always keep as "scoop"                           |
| flavor  | virtualenv          | Translate if the metaphor works in your language |
| freezer | ~/.scoop/ directory | Translate if the metaphor works                  |

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
// 1. SCOOP_LANG environment variable
// 2. Config file (~/.config/scoop/config.toml)
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

**Symptom:** Translation exists but `scoop lang {code}` doesn't work

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
hint: "→ {Translated Label}: scoop list"
```

### 4. Inconsistent Key Coverage

All languages must have ALL keys. Missing keys fall back to English.

---

## Testing Checklist

Before submitting PR:

- [ ] All 106 keys translated
- [ ] All placeholders preserved (`%{name}`, `%{version}`, etc.)
- [ ] Language registered in SUPPORTED_LANGS
- [ ] `cargo build` succeeds
- [ ] `cargo test` passes
- [ ] `SCOOP_LANG={code} scoop lang` shows your language
- [ ] Messages display correctly in terminal

---

## Questions?

- Open an issue: [GitHub Issues](https://github.com/ai-screams/scoop-uv/issues)
- See existing translations for reference: `locales/app.yml`
