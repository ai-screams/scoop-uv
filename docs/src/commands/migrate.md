# migrate

Migrate virtual environments from other tools (pyenv-virtualenv, virtualenvwrapper, conda).

## Usage

```bash
# List migratable environments
scoop migrate list

# Migrate a single environment
scoop migrate @env <name>

# Migrate all environments
scoop migrate all
```

## Subcommands

| Subcommand | Description |
|------------|-------------|
| `list` | List environments available for migration |
| `@env <name>` | Migrate a single environment by name |
| `all` | Migrate all discovered environments |

## Supported Sources

| Source | Detection |
|--------|-----------|
| **pyenv-virtualenv** | `~/.pyenv/versions/` (non-system virtualenvs) |
| **virtualenvwrapper** | `$WORKON_HOME` or `~/.virtualenvs/` |
| **conda** | `conda info --envs` |

## Options

| Option | Description |
|--------|-------------|
| `--json` | Output as JSON |
| `--quiet` | Suppress output |
| `--no-color` | Disable colored output |

## Examples

### List Migratable Environments

```bash
$ scoop migrate list
📦 Migratable Environments

  pyenv-virtualenv:
    • myproject (Python 3.12.0)
    • webapp (Python 3.11.8)

  conda:
    • ml-env (Python 3.10.4)
```

### Migrate Single Environment

```bash
$ scoop migrate @env myproject
✓ Migrated 'myproject' from pyenv-virtualenv
  Source: ~/.pyenv/versions/myproject
  Target: ~/.scoop/virtualenvs/myproject
```

### Migrate All

```bash
$ scoop migrate all
✓ Migrated 3 environments
  • myproject (pyenv-virtualenv)
  • webapp (pyenv-virtualenv)
  • ml-env (conda)
```

### JSON Output

```bash
$ scoop migrate list --json
{
  "status": "success",
  "data": {
    "environments": [
      {
        "name": "myproject",
        "source": "pyenv",
        "python_version": "3.12.0"
      }
    ]
  }
}
```

## Migration Process

1. **Discovery**: Scans configured source paths for virtual environments
2. **Extraction**: Identifies Python version and installed packages
3. **Recreation**: Creates new scoop environment with same Python version
4. **Package Install**: Reinstalls packages using `uv pip install`
5. **Cleanup**: Originals are preserved by default; `--delete-source` removes them after successful migration

## Notes

- Original environments are preserved by default; use `--delete-source` to remove sources after migration
- Package versions are preserved where possible
- Migration creates fresh environments using `uv` for improved performance
