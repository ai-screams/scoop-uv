//! Init command

use crate::cli::ShellType;
use crate::error::Result;
use crate::shell;

/// Execute the init command
pub fn execute(shell: ShellType) -> Result<()> {
    let script = match shell {
        ShellType::Bash => shell::bash::init_script(),
        ShellType::Zsh => shell::zsh::init_script(),
        ShellType::Fish => todo!("Fish shell support not yet implemented"),
        ShellType::Powershell => todo!("PowerShell support not yet implemented"),
    };

    print!("{script}");
    Ok(())
}
