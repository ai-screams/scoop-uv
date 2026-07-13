# Quick Start

This guide walks you through the basic scuv workflow.

## 1. Set Up Shell Integration

**Zsh** (macOS default):

```bash
echo 'eval "$(scuv init zsh)"' >> ~/.zshrc
source ~/.zshrc
```

**Bash**:

```bash
echo 'eval "$(scuv init bash)"' >> ~/.bashrc
source ~/.bashrc
```

**Fish**:

```fish
echo 'scuv init fish | source' >> ~/.config/fish/config.fish
source ~/.config/fish/config.fish
```

**PowerShell**:

```powershell
Add-Content -Path $PROFILE -Value 'Invoke-Expression (& scuv init powershell)'
. $PROFILE
```

## 2. Install Python

```bash
# Install latest Python
scuv install 3.12

# Verify installation
scuv list --pythons
```

## 3. Create a Virtual Environment

```bash
scuv create myproject 3.12
```

This creates a virtual environment at `~/.scuv/virtualenvs/myproject/`.

If Python 3.12 isn't installed yet, add `--install-python` to install it on demand:

```bash
scuv create myproject 3.12 --install-python
```

## 4. Use the Environment

```bash
cd ~/projects/myproject
scuv use myproject
```

This:
1. Creates `.scuv-version` file in the current directory
2. Activates the environment (prompt shows `(myproject)`)

## 5. Work With Your Environment

```bash
(myproject) $ pip install -r requirements.txt

# If the file is in a different location:
(myproject) $ pip install -r path/to/requirements.txt

# Verify installed packages
(myproject) $ pip list
```

## 6. Auto-Activation

Once configured, entering a directory with `.scuv-version` automatically activates the environment:

```bash
cd ~/projects/myproject
# (myproject) appears in prompt automatically
```

## Common Commands

| Task | Command |
|------|---------|
| List environments | `scuv list` |
| List Python versions | `scuv list --pythons` |
| Show environment info | `scuv info myproject` |
| Remove environment | `scuv remove myproject` |
| Check installation | `scuv doctor` |

## IDE Integration

Create a `.venv` symlink for IDE compatibility:

```bash
scuv use myproject --link
```

This creates `.venv` pointing to the scuv environment, recognized by VS Code, PyCharm, etc.

## Next Steps

- See [Shell Integration](shell-integration.md) for advanced configuration
- See [Commands](commands/README.md) for full command reference
