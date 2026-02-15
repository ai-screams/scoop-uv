# Quick Start

This guide walks you through the basic scoop workflow.

## 1. Set Up Shell Integration

**Zsh** (macOS default):

```bash
echo 'eval "$(scoop init zsh)"' >> ~/.zshrc
source ~/.zshrc
```

**Bash**:

```bash
echo 'eval "$(scoop init bash)"' >> ~/.bashrc
source ~/.bashrc
```

**Fish**:

```fish
echo 'eval (scoop init fish)' >> ~/.config/fish/config.fish
source ~/.config/fish/config.fish
```

**PowerShell**:

```powershell
Add-Content -Path $PROFILE -Value 'Invoke-Expression (& scoop init powershell)'
. $PROFILE
```

## 2. Install Python

```bash
# Install latest Python
scoop install 3.12

# Verify installation
scoop list --pythons
```

## 3. Create a Virtual Environment

```bash
scoop create myproject 3.12
```

This creates a virtual environment at `~/.scoop/virtualenvs/myproject/`.

## 4. Use the Environment

```bash
cd ~/projects/myproject
scoop use myproject
```

This:
1. Creates `.scoop-version` file in the current directory
2. Activates the environment (prompt shows `(myproject)`)

## 5. Work With Your Environment

```bash
(myproject) $ pip install requests
(myproject) $ python -c "import requests; print(requests.__version__)"
```

## 6. Auto-Activation

Once configured, entering a directory with `.scoop-version` automatically activates the environment:

```bash
cd ~/projects/myproject
# (myproject) appears in prompt automatically
```

## Common Commands

| Task | Command |
|------|---------|
| List environments | `scoop list` |
| List Python versions | `scoop list --pythons` |
| Show environment info | `scoop info myproject` |
| Remove environment | `scoop remove myproject` |
| Check installation | `scoop doctor` |

## IDE Integration

Create a `.venv` symlink for IDE compatibility:

```bash
scoop use myproject --link
```

This creates `.venv` pointing to the scoop environment, recognized by VS Code, PyCharm, etc.

## Next Steps

- See [Shell Integration](shell-integration.md) for advanced configuration
- See [Commands](commands/README.md) for full command reference
