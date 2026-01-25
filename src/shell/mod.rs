//! Shell integration module

use crate::cli::ShellType;
use std::path::Path;

pub mod bash;
pub mod common;
pub mod fish;
pub mod powershell;
pub mod zsh;

/// Detect current shell from environment variables
pub fn detect_shell() -> ShellType {
    // Check Fish first (has unique env var)
    if std::env::var("FISH_VERSION").is_ok() {
        ShellType::Fish
    // Check PowerShell (PSModulePath exists in pwsh, but also check it's not Fish)
    } else if std::env::var("PSModulePath").is_ok() {
        ShellType::Powershell
    } else if std::env::var("ZSH_VERSION").is_ok() {
        ShellType::Zsh
    } else {
        ShellType::Bash
    }
}

/// Print activation script for the given shell
pub fn print_activate_script(shell: ShellType, venv_path: &Path, bin_path: &Path, name: &str) {
    match shell {
        ShellType::Fish => {
            // Save original PATH only on first activation
            println!(
                r#"if not set -q _SCOOP_OLD_PATH
    set -gx _SCOOP_OLD_PATH $PATH
end
if set -q PYTHONHOME
    set -gx _SCOOP_OLD_PYTHONHOME $PYTHONHOME
end"#
            );
            println!("set -gx VIRTUAL_ENV '{}'", venv_path.display());
            println!("set -gx PATH '{}' $PATH", bin_path.display());
            println!("set -gx SCOOP_ACTIVE '{}'", name);
            println!("set -e PYTHONHOME");
        }
        ShellType::Powershell => {
            // Save original PATH only on first activation
            println!(
                r#"if (-not $env:_SCOOP_OLD_PATH) {{
    $env:_SCOOP_OLD_PATH = $env:PATH
}}
if ($env:PYTHONHOME) {{
    $env:_SCOOP_OLD_PYTHONHOME = $env:PYTHONHOME
}}"#
            );
            // Use [IO.Path]::PathSeparator for cross-platform
            // Escape single quotes in paths by doubling them (PowerShell string escape)
            let venv_escaped = venv_path.display().to_string().replace('\'', "''");
            let bin_escaped = bin_path.display().to_string().replace('\'', "''");
            let name_escaped = name.replace('\'', "''");
            println!("$env:VIRTUAL_ENV = '{}'", venv_escaped);
            println!(
                "$env:PATH = '{}' + [IO.Path]::PathSeparator + $env:PATH",
                bin_escaped
            );
            println!("$env:SCOOP_ACTIVE = '{}'", name_escaped);
            println!("Remove-Item Env:\\PYTHONHOME -ErrorAction SilentlyContinue");
        }
        _ => {
            // Save original PATH only on first activation
            println!(
                r#"if [ -z "$_SCOOP_OLD_PATH" ]; then
    _SCOOP_OLD_PATH="$PATH"
    export _SCOOP_OLD_PATH
fi
if [ -n "$PYTHONHOME" ]; then
    _SCOOP_OLD_PYTHONHOME="$PYTHONHOME"
    export _SCOOP_OLD_PYTHONHOME
fi"#
            );
            println!("export VIRTUAL_ENV=\"{}\"", venv_path.display());
            println!("export PATH=\"{}:$PATH\"", bin_path.display());
            println!("export SCOOP_ACTIVE=\"{}\"", name);
            println!("unset PYTHONHOME");
        }
    }
}

pub fn print_deactivate_script(shell: ShellType) {
    match shell {
        ShellType::Fish => {
            println!(
                r#"if set -q VIRTUAL_ENV
    # Restore original PATH
    if set -q _SCOOP_OLD_PATH
        set PATH $_SCOOP_OLD_PATH
        set -e _SCOOP_OLD_PATH
    end
    # Restore PYTHONHOME if it was saved
    if set -q _SCOOP_OLD_PYTHONHOME
        set -gx PYTHONHOME $_SCOOP_OLD_PYTHONHOME
        set -e _SCOOP_OLD_PYTHONHOME
    end
    set -e VIRTUAL_ENV
    set -e SCOOP_ACTIVE
end"#
            );
        }
        ShellType::Powershell => {
            println!(
                r#"if ($env:VIRTUAL_ENV) {{
    # Restore original PATH
    if ($env:_SCOOP_OLD_PATH) {{
        $env:PATH = $env:_SCOOP_OLD_PATH
        Remove-Item Env:\\_SCOOP_OLD_PATH -ErrorAction SilentlyContinue
    }}
    # Restore PYTHONHOME if it was saved
    if ($env:_SCOOP_OLD_PYTHONHOME) {{
        $env:PYTHONHOME = $env:_SCOOP_OLD_PYTHONHOME
        Remove-Item Env:\\_SCOOP_OLD_PYTHONHOME -ErrorAction SilentlyContinue
    }}
    Remove-Item Env:\\VIRTUAL_ENV -ErrorAction SilentlyContinue
    Remove-Item Env:\\SCOOP_ACTIVE -ErrorAction SilentlyContinue
}}"#
            );
        }
        _ => {
            println!(
                r#"if [ -n "$VIRTUAL_ENV" ]; then
    # Restore original PATH
    if [ -n "$_SCOOP_OLD_PATH" ]; then
        PATH="$_SCOOP_OLD_PATH"
        export PATH
        unset _SCOOP_OLD_PATH
    fi
    # Restore PYTHONHOME if it was saved
    if [ -n "$_SCOOP_OLD_PYTHONHOME" ]; then
        PYTHONHOME="$_SCOOP_OLD_PYTHONHOME"
        export PYTHONHOME
        unset _SCOOP_OLD_PYTHONHOME
    fi
    unset VIRTUAL_ENV
    unset SCOOP_ACTIVE
fi"#
            );
        }
    }
}

/// Print unset SCOOP_VERSION script for the given shell
pub fn print_unset_scoop_version(shell: ShellType) {
    match shell {
        ShellType::Fish => println!("set -e SCOOP_VERSION"),
        ShellType::Powershell => {
            println!("Remove-Item Env:\\SCOOP_VERSION -ErrorAction SilentlyContinue")
        }
        _ => println!("unset SCOOP_VERSION"),
    }
}

/// Print export SCOOP_VERSION script for the given shell
pub fn print_export_scoop_version(shell: ShellType, value: &str) {
    match shell {
        ShellType::Fish => {
            // Fish: escape single quotes by replacing ' with \'
            let escaped = value.replace('\'', "\\'");
            println!("set -gx SCOOP_VERSION '{}'", escaped);
        }
        ShellType::Powershell => {
            // PowerShell: escape single quotes by doubling them
            let escaped = value.replace('\'', "''");
            println!("$env:SCOOP_VERSION = '{}'", escaped);
        }
        _ => println!("export SCOOP_VERSION=\"{}\"", value),
    }
}
