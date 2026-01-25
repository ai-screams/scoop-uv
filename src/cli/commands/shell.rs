//! Shell command - set shell-specific environment

use rust_i18n::t;

use crate::cli::ShellType;
use crate::core::VirtualenvService;
use crate::error::{Result, ScoopError};
use crate::output::Output;
use crate::paths;
use crate::shell::{
    detect_shell, print_activate_script, print_deactivate_script, print_export_scoop_version,
    print_unset_scoop_version,
};
use crate::validate::validate_env_name;

/// Execute the shell command - outputs shell code for eval
pub fn execute(
    output: &Output,
    name: Option<&str>,
    unset: bool,
    shell: Option<ShellType>,
) -> Result<()> {
    // Detect shell or use specified
    let shell_type = shell.unwrap_or_else(detect_shell);

    // Handle --unset
    if unset {
        print_unset_scoop_version(shell_type);

        if !output.is_json() && !output.is_quiet() {
            eprintln!("{}", t!("shell.unset"));
        }
        return Ok(());
    }

    // Name is required when not using --unset
    let name = name.ok_or_else(|| ScoopError::InvalidArgument {
        message: t!("error.shell_missing_name").to_string(),
    })?;

    // Handle "system" special value
    if name.eq_ignore_ascii_case("system") {
        print_export_scoop_version(shell_type, "system");
        // Also output deactivate script
        print_deactivate_script(shell_type);

        if !output.is_json() && !output.is_quiet() {
            eprintln!("{}", t!("shell.system"));
        }
        return Ok(());
    }

    // Validate environment name and check existence
    validate_env_name(name)?;
    let service = VirtualenvService::auto()?;
    let venv_path = service.get_path(name)?;
    let bin_path = paths::virtualenv_bin(name)?;

    // Output shell script for eval
    print_export_scoop_version(shell_type, name);
    print_activate_script(shell_type, &venv_path, &bin_path, name);

    if !output.is_json() && !output.is_quiet() {
        eprintln!("{}", t!("shell.set", name = name));
    }

    Ok(())
}
