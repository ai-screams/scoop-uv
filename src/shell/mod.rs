//! Shell integration module

use crate::cli::ShellType;
use std::path::Path;

pub mod bash;
pub mod common;
pub mod fish;
pub mod zsh;

/// Detect current shell from environment variables
pub fn detect_shell() -> ShellType {
    if std::env::var("FISH_VERSION").is_ok() {
        ShellType::Fish
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
        _ => println!("unset SCOOP_VERSION"),
    }
}

/// Print export SCOOP_VERSION script for the given shell
pub fn print_export_scoop_version(shell: ShellType, value: &str) {
    match shell {
        ShellType::Fish => println!("set -gx SCOOP_VERSION '{}'", value),
        _ => println!("export SCOOP_VERSION=\"{}\"", value),
    }
}
