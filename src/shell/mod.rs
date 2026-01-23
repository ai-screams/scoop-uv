//! Shell integration module

use crate::cli::ShellType;
use std::path::Path;

pub mod bash;
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
            println!("set -gx VIRTUAL_ENV '{}'", venv_path.display());
            println!("set -gx PATH '{}' $PATH", bin_path.display());
            println!("set -gx SCOOP_ACTIVE '{}'", name);
            println!("set -e PYTHONHOME");
        }
        _ => {
            println!("export VIRTUAL_ENV=\"{}\"", venv_path.display());
            println!("export PATH=\"{}:$PATH\"", bin_path.display());
            println!("export SCOOP_ACTIVE=\"{}\"", name);
            println!("unset PYTHONHOME");
        }
    }
}

/// Print deactivation script for the given shell
pub fn print_deactivate_script(shell: ShellType) {
    match shell {
        ShellType::Fish => {
            println!(
                r#"if set -q VIRTUAL_ENV
    if set -l idx (contains -i "$VIRTUAL_ENV/bin" $PATH)
        set -e PATH[$idx]
    end
    set -e VIRTUAL_ENV
    set -e SCOOP_ACTIVE
end"#
            );
        }
        _ => {
            println!(
                r#"if [ -n "$VIRTUAL_ENV" ]; then
    PATH="${{PATH#$VIRTUAL_ENV/bin:}}"
    export PATH
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
        _ => println!("export SCOOP_VERSION={}", value),
    }
}
