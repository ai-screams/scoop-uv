//! Completions command

use clap::CommandFactory;
use clap_complete::{Shell, generate};

use crate::cli::{Cli, ShellType};
use crate::error::Result;

/// Execute the completions command
pub fn execute(shell: ShellType) -> Result<()> {
    let mut cmd = Cli::command();

    let shell = match shell {
        ShellType::Bash => Shell::Bash,
        ShellType::Zsh => Shell::Zsh,
        ShellType::Fish => Shell::Fish,
        ShellType::Powershell => Shell::PowerShell,
    };

    generate(shell, &mut cmd, "uvenv", &mut std::io::stdout());

    Ok(())
}
