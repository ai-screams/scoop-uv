# lang

Get or set the display language for scuv CLI messages.

## Usage

```bash
# Show current language
scuv lang

# Set language
scuv lang <code>

# List supported languages
scuv lang --list

# Reset to system default
scuv lang --reset
```

## Arguments

| Argument | Description |
|----------|-------------|
| `<code>` | Language code to set (e.g., `en`, `ko`) |

## Options

| Option | Description |
|--------|-------------|
| `--list` | List all supported languages |
| `--reset` | Reset to system default language |
| `--json` | Output as JSON |

## Supported Languages

| Code | Language |
|------|----------|
| `en` | English (default) |
| `ko` | 한국어 (Korean) |
| `ja` | 日本語 (Japanese) |
| `pt-BR` | Português (Brazilian Portuguese) |

## Language Detection Priority

1. `SCUV_LANG` environment variable
2. `~/.scuv/config.json` setting
3. System locale (via `sys-locale`)
4. Default: `en`

## Examples

### Show Current Language

```bash
$ scuv lang
Current language: en (English)
```

### Set Korean

```bash
$ scuv lang ko
✓ Language set to Korean (한국어)
```

### List Languages

```bash
$ scuv lang --list
Supported languages:
  en - English
  ko - 한국어 (Korean)
  ja - 日本語 (Japanese)
  pt-BR - Português (Brazilian Portuguese)
```

### Reset to System Default

```bash
$ scuv lang --reset
✓ Language reset to system default
```

### JSON Output

```bash
$ scuv lang --json
{
  "status": "success",
  "command": "lang",
  "data": {
    "current": "ko",
    "name": "한국어",
    "source": "config"
  }
}
```

## Configuration

Language preference is stored in:

```
~/.scuv/config.json
```

```json
{"lang": "ko"}
```

## Environment Variable Override

```bash
# Temporarily use English regardless of config
SCUV_LANG=en scuv list

# Set for current session
export SCUV_LANG=ko
```

## Notes

- CLI help text (`--help`) remains in English (industry standard)
- JSON output keys remain in English (machine-readable)
- Error messages, success messages, and prompts are translated
