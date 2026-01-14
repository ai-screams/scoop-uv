How do I uninstall scoop completely?
To remove scoop-uv from your system, you need to delete the source folder and remove the shell integration:

1. Delete the data folder:

[Bash]

rm -rf ~/.scoop

2. Remove the Shell Hook: Open your shell configuration file (e.g., ~/.bashrc, ~/.zshrc, or your PowerShell profile) and remove the line that looks like: eval "$(scoop init zsh)"

3. Restart your terminal.
