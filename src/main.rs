//! scoop - Python virtual environment manager powered by uv

use clap::Parser;
use color_eyre::eyre::Result;

use scoop_uv::cli::{Cli, Commands};
use scoop_uv::output::Output;

fn main() -> Result<()> {
    // Initialize error handling
    color_eyre::install()?;

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::WARN.into()),
        )
        .init();

    // Parse CLI arguments
    let cli = Cli::parse();

    // Execute command
    let result = match cli.command {
        Commands::List {
            pythons,
            bare,
            json,
        } => {
            let output = Output::new(0, cli.quiet, cli.no_color, json);
            scoop_uv::cli::commands::list(&output, pythons, bare)
        }
        Commands::Create {
            name,
            python,
            force,
        } => {
            let output = Output::new(0, cli.quiet, cli.no_color, false);
            scoop_uv::cli::commands::create(&output, &name, &python, force)
        }
        Commands::Doctor { verbose, json } => {
            let output = Output::new(verbose, cli.quiet, cli.no_color, json);
            scoop_uv::cli::commands::doctor(&output)
        }
        Commands::Use {
            name,
            global,
            link,
            no_link: _, // explicit option, same as default (no symlink)
        } => {
            let output = Output::new(0, cli.quiet, cli.no_color, false);
            scoop_uv::cli::commands::use_env(&output, &name, global, link)
        }
        Commands::Remove { name, force } => {
            let output = Output::new(0, cli.quiet, cli.no_color, false);
            scoop_uv::cli::commands::remove(&output, &name, force)
        }
        Commands::Install {
            python_version,
            latest,
            stable,
        } => {
            let output = Output::new(0, cli.quiet, cli.no_color, false);
            scoop_uv::cli::commands::install(&output, python_version.as_deref(), latest, stable)
        }
        Commands::Uninstall { python_version } => {
            let output = Output::new(0, cli.quiet, cli.no_color, false);
            scoop_uv::cli::commands::uninstall(&output, &python_version)
        }
        Commands::Init { shell } => scoop_uv::cli::commands::init(shell),
        Commands::Completions { shell } => scoop_uv::cli::commands::completions(shell),
        Commands::Resolve => scoop_uv::cli::commands::resolve(),
        Commands::Activate { name } => scoop_uv::cli::commands::activate(&name),
        Commands::Deactivate => scoop_uv::cli::commands::deactivate(),
    };

    // Handle errors
    if let Err(e) = result {
        let output = Output::new(0, cli.quiet, cli.no_color, false);
        output.error(&e.to_string());
        std::process::exit(1);
    }

    Ok(())
}
