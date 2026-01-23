//! Deactivate command

use crate::cli::ShellType;
use crate::error::Result;
use crate::shell::{detect_shell, print_deactivate_script};

/// Execute the deactivate command
/// Outputs shell script to be eval'd
pub fn execute(shell: Option<ShellType>) -> Result<()> {
    // Detect shell or use specified
    let shell_type = shell.unwrap_or_else(detect_shell);

    // Output deactivation script for eval
    print_deactivate_script(shell_type);

    Ok(())
}
