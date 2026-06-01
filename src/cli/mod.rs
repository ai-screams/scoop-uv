//! CLI module

pub mod commands;

use std::path::PathBuf;

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

    /// Suppress all output
    #[arg(short, long, global = true)]
    pub quiet: bool,

    /// Disable colored output
    #[arg(long, global = true, env = "NO_COLOR")]
    pub no_color: bool,
}

/// Source type for migration
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum MigrateSource {
    /// pyenv-virtualenv
    Pyenv,
    /// virtualenvwrapper
    Virtualenvwrapper,
    /// conda
    Conda,
}

impl std::fmt::Display for MigrateSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pyenv => write!(f, "pyenv"),
            Self::Virtualenvwrapper => write!(f, "virtualenvwrapper"),
            Self::Conda => write!(f, "conda"),
        }
    }
}

/// Migrate subcommands
#[derive(Subcommand, Debug)]
pub enum MigrateCommand {
    /// List environments available for migration
    List {
        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Filter by source tool (pyenv, virtualenvwrapper, conda)
        #[arg(long, value_enum)]
        source: Option<MigrateSource>,
    },
    /// Migrate all environments at once
    All {
        /// Preview migration without making changes
        #[arg(short = 'n', long)]
        dry_run: bool,

        /// Include EOL Python versions and overwrite conflicts
        #[arg(short, long)]
        force: bool,

        /// Skip confirmation prompts
        #[arg(short, long)]
        yes: bool,

        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Fail migration if any package fails to install
        #[arg(long)]
        strict: bool,

        /// Delete original environments after successful migration
        #[arg(long)]
        delete_source: bool,

        /// Filter by source tool (pyenv, virtualenvwrapper, conda)
        #[arg(long, value_enum)]
        source: Option<MigrateSource>,
    },
    /// Migrate a specific environment
    #[command(name = "@env")]
    Env {
        /// Name of the environment to migrate
        name: String,

        /// Preview migration without making changes
        #[arg(short = 'n', long)]
        dry_run: bool,

        /// Overwrite if environment already exists in scoop
        #[arg(short, long)]
        force: bool,

        /// Skip confirmation prompts
        #[arg(short, long)]
        yes: bool,

        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Fail migration if any package fails to install
        #[arg(long)]
        strict: bool,

        /// Migrate with a different name
        #[arg(long, value_name = "NEW_NAME")]
        rename: Option<String>,

        /// Auto-rename on conflict (uses {name}-<source> pattern)
        #[arg(long, conflicts_with = "force")]
        auto_rename: bool,

        /// Delete original environment after successful migration
        #[arg(long)]
        delete_source: bool,

        /// Source tool (pyenv, virtualenvwrapper, conda)
        #[arg(long, value_enum)]
        source: Option<MigrateSource>,
    },
}

/// Self-management subcommands (under `scoop self ...`).
#[derive(Subcommand, Debug)]
pub enum SelfCommand {
    /// Reinstall scoop from crates.io to the latest version
    // Disable clap's auto `--version` flag here so `--version <VER>` can be
    // a real arg meaning "install this specific version". `scoop --version`
    // and the auto flag on other subcommands are unaffected.
    #[command(disable_version_flag = true)]
    Update {
        /// Reinstall even if already on the latest version
        #[arg(long)]
        force: bool,

        /// Install a specific version instead of the latest
        #[arg(long, value_name = "VERSION")]
        version: Option<String>,

        /// Skip the post-install environment verification (`scoop doctor`)
        #[arg(long)]
        no_verify: bool,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

/// Available commands
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// List all virtual environments
    #[command(alias = "ls")]
    List {
        /// Show installed Python versions instead of virtualenvs
        #[arg(long)]
        pythons: bool,

        /// Output names only, one per line (for scripting/completion)
        #[arg(long, hide = true)]
        bare: bool,

        /// Filter environments by Python version (e.g., 3.12)
        #[arg(long, value_name = "VERSION", conflicts_with = "pythons")]
        python_version: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Set local environment for current directory
    Use {
        /// Name of the virtual environment (or "system" for system Python)
        name: Option<String>,

        /// Remove version file (unset local/global setting)
        #[arg(long)]
        unset: bool,

        /// Create .venv symlink to the virtual environment
        #[arg(long, conflicts_with = "no_link")]
        link: bool,

        /// Set as global default
        #[arg(short, long)]
        global: bool,

        /// Do not create .venv symlink (default behavior, explicit)
        #[arg(long, conflicts_with = "link")]
        no_link: bool,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Create a new virtual environment
    Create {
        /// Name of the virtual environment
        name: String,

        /// Python version or specifier (e.g., 3.12, cpython@3.12, pypy@3.10)
        #[arg(default_value = "3")]
        python: String,

        /// Path to a specific Python interpreter to use instead of a version
        #[arg(long = "python-path", value_name = "PATH")]
        python_path: Option<PathBuf>,

        /// Overwrite existing environment
        #[arg(short, long)]
        force: bool,

        /// Install the requested Python version first if it is not already available
        #[arg(long, conflicts_with = "python_path")]
        install_python: bool,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Remove a virtual environment
    #[command(alias = "rm", alias = "delete")]
    Remove {
        /// Name of the virtual environment
        name: String,

        /// Skip confirmation
        #[arg(short, long)]
        force: bool,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Install a Python version
    Install {
        /// Python version to install (e.g., 3.12, 3.13)
        #[arg(name = "VERSION")]
        python_version: Option<String>,

        /// Install latest stable Python (default if no version specified)
        #[arg(long)]
        latest: bool,

        /// Install oldest fully-supported Python (more stable)
        #[arg(long)]
        stable: bool,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Uninstall a Python version
    Uninstall {
        /// Python version to uninstall
        #[arg(name = "VERSION")]
        python_version: String,

        /// Also remove all virtual environments using this Python version
        #[arg(long)]
        cascade: bool,

        /// Skip confirmation for cascade removal
        #[arg(short, long, requires = "cascade")]
        force: bool,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Diagnose installation issues
    Doctor {
        /// Increase verbosity (can be repeated)
        #[arg(short, long, action = ArgAction::Count)]
        verbose: u8,

        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Automatically fix issues where possible
        #[arg(long)]
        fix: bool,
    },

    /// Show detailed information about a virtual environment
    Info {
        /// Name of the virtual environment
        name: String,

        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Show all installed packages (default: top 5 only)
        #[arg(long)]
        all_packages: bool,

        /// Skip directory size calculation (faster)
        #[arg(long)]
        no_size: bool,
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

        /// Target shell type (auto-detected if not specified)
        #[arg(long, value_enum)]
        shell: Option<ShellType>,
    },

    /// Output deactivation script for eval
    #[command(hide = true)]
    Deactivate {
        /// Target shell type (auto-detected if not specified)
        #[arg(long, value_enum)]
        shell: Option<ShellType>,
    },

    /// Set environment for current shell session only (highest priority)
    Shell {
        /// Name of the virtual environment (or "system" for system Python)
        name: Option<String>,

        /// Clear shell-specific environment setting
        #[arg(long)]
        unset: bool,

        /// Target shell type (auto-detected if not specified)
        #[arg(long, value_enum)]
        shell: Option<ShellType>,
    },

    /// Migrate environments from other tools (pyenv, virtualenvwrapper)
    Migrate {
        /// Subcommand or environment name to migrate
        #[command(subcommand)]
        command: Option<MigrateCommand>,
    },

    /// Set or show language preference
    Lang {
        /// Language code to set (e.g., ko, en)
        lang: Option<String>,

        /// List supported languages
        #[arg(long)]
        list: bool,

        /// Reset to system default
        #[arg(long)]
        reset: bool,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Manage scoop itself (update, etc.)
    #[command(name = "self")]
    Self_ {
        #[command(subcommand)]
        command: SelfCommand,
    },

    /// Show the current environment status
    Status {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Clone an environment (optionally without copying packages)
    Clone {
        /// Name of the source environment
        src: String,

        /// Name of the new (destination) environment
        dst: String,

        /// Skip copying packages — create an empty env with the same Python
        #[arg(long)]
        no_packages: bool,

        /// Overwrite destination if it already exists
        #[arg(short, long)]
        force: bool,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Export an environment as a portable JSON file
    Export {
        /// Name of the environment to export
        name: String,

        /// Write to this path instead of stdout
        #[arg(short = 'o', long = "output", value_name = "PATH")]
        output: Option<PathBuf>,
    },

    /// Import an environment from a `scoop export` JSON file (use `-` for stdin)
    Import {
        /// Path to the export JSON, or `-` to read from stdin
        path: String,

        /// Override the environment name from the file
        #[arg(long, value_name = "NAME")]
        name: Option<String>,

        /// Overwrite an existing environment with the same name
        #[arg(short, long)]
        force: bool,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Sync an environment from `.scoop.toml`
    Sync {
        /// Additional package group(s) to install on top of `default`
        #[arg(long = "with", value_name = "GROUP", action = clap::ArgAction::Append)]
        with: Vec<String>,

        /// Preview the plan without creating env or installing packages
        #[arg(long)]
        dry_run: bool,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Run a command inside an environment without activating it
    Run {
        /// Name of the virtual environment
        env: String,

        /// Command and arguments to execute (use -- to separate, e.g. `scoop run myenv -- python x.py`)
        #[arg(trailing_var_arg = true, allow_hyphen_values = true, num_args = 1..)]
        command: Vec<String>,
    },

    /// Print the full path to an executable in an environment
    Which {
        /// Name of the executable to locate (e.g., python, pip)
        exe: String,

        /// Look in this environment instead of the active one
        #[arg(long, value_name = "NAME")]
        env: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Prune the uv cache (delete unused download/wheel cache entries)
    Prune {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
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
