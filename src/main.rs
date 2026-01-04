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

    // Create output handler
    let output = Output::new(cli.verbose, cli.quiet, cli.no_color, cli.json);

    // Execute command
    let result = match cli.command {
        Commands::List => scoop_uv::cli::commands::list(&output),
        Commands::Create {
            name,
            python,
            force,
        } => scoop_uv::cli::commands::create(&output, &name, &python, force),
        Commands::Use {
            name,
            global,
            link,
            no_link: _, // explicit option, same as default (no symlink)
        } => scoop_uv::cli::commands::use_env(&output, &name, global, link),
        Commands::Remove { name, force } => scoop_uv::cli::commands::remove(&output, &name, force),
        Commands::Install { python_version } => {
            scoop_uv::cli::commands::install(&output, &python_version)
        }
        Commands::Uninstall { python_version } => {
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
        output.error(&e.to_string());
        std::process::exit(1);
    }

    Ok(())
}
