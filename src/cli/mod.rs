//! CLI module

pub mod commands;

use clap::{ArgAction, Parser, Subcommand};

/// scoop - Python virtual environment manager powered by uv
#[derive(Parser, Debug)]
#[command(name = "scoop")]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    /// Subcommand to run
    #[command(subcommand)]
    pub command: Commands,

    /// Increase verbosity (can be repeated)
    #[arg(short, long, action = ArgAction::Count, global = true)]
    pub verbose: u8,

    /// Suppress all output
    #[arg(short, long, global = true)]
    pub quiet: bool,

    /// Disable colored output
    #[arg(long, global = true, env = "NO_COLOR")]
    pub no_color: bool,

    /// Output as JSON
    #[arg(long, global = true)]
    pub json: bool,
}

/// Available commands
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// List all virtual environments
    #[command(alias = "ls")]
    List,

    /// Create a new virtual environment
    Create {
        /// Name of the virtual environment
        name: String,

        /// Python version to use (e.g., 3.12, 3.11.8)
        #[arg(default_value = "3")]
        python: String,

        /// Overwrite existing environment
        #[arg(short, long)]
        force: bool,
    },

    /// Set local environment for current directory
    Use {
        /// Name of the virtual environment
        name: String,

        /// Set as global default
        #[arg(short, long)]
        global: bool,

        /// Create .venv symlink to the virtual environment
        #[arg(long, conflicts_with = "no_link")]
        link: bool,

        /// Do not create .venv symlink (default behavior, explicit)
        #[arg(long, conflicts_with = "link")]
        no_link: bool,
    },

    /// Remove a virtual environment
    #[command(alias = "rm", alias = "delete")]
    Remove {
        /// Name of the virtual environment
        name: String,

        /// Skip confirmation
        #[arg(short, long)]
        force: bool,
    },

    /// Install a Python version
    Install {
        /// Python version to install (e.g., 3.12, 3.13, latest, stable)
        #[arg(name = "VERSION")]
        python_version: String,
    },

    /// Uninstall a Python version
    Uninstall {
        /// Python version to uninstall
        #[arg(name = "VERSION")]
        python_version: String,
    },

    /// Output shell initialization script
    Init {
        /// Shell to generate script for
        #[arg(value_enum)]
        shell: ShellType,
    },

    /// Output shell completion script
    Completions {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: ShellType,
    },

    /// Resolve and print current environment name
    #[command(hide = true)]
    Resolve,

    /// Output activation script for eval
    #[command(hide = true)]
    Activate {
        /// Name of the virtual environment
        name: String,
    },

    /// Output deactivation script for eval
    #[command(hide = true)]
    Deactivate,
}

/// Supported shell types
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum ShellType {
    /// Bash shell
    Bash,
    /// Zsh shell
    Zsh,
    /// Fish shell
    Fish,
    /// PowerShell
    #[value(alias = "pwsh")]
    Powershell,
}
