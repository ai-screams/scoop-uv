//! Activate command

use crate::cli::ShellType;
use crate::core::VirtualenvService;
use crate::error::Result;
use crate::paths;
use crate::shell::{detect_shell, print_activate_script};
use crate::validate;

/// Execute the activate command
/// Outputs shell script to be eval'd
pub fn execute(name: &str, shell: Option<ShellType>) -> Result<()> {
    // Security: Validate input before any processing
    // This is defense-in-depth against command injection via malicious .scoop-version files
    validate::validate_env_name(name)?;

    let service = VirtualenvService::auto()?;

    // Verify environment exists
    let venv_path = service.get_path(name)?;
    let bin_path = paths::virtualenv_bin(name)?;

    // Detect shell or use specified
    let shell_type = shell.unwrap_or_else(detect_shell);

    // Output activation script for eval
    print_activate_script(shell_type, &venv_path, &bin_path, name);

    Ok(())
}
