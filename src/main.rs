//! uvenv - Python virtual environment manager powered by uv

use clap::Parser;
use color_eyre::eyre::Result;

use uvenv::cli::{Cli, Commands};
use uvenv::output::Output;

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
        Commands::List => uvenv::cli::commands::list(&output),
        Commands::Create {
            name,
            python,
            force,
        } => uvenv::cli::commands::create(&output, &name, &python, force),
        Commands::Use {
            name,
            global,
            no_link,
        } => uvenv::cli::commands::use_env(&output, &name, global, no_link),
        Commands::Remove { name, force } => uvenv::cli::commands::remove(&output, &name, force),
        Commands::Install { version } => uvenv::cli::commands::install(&output, &version),
        Commands::Init { shell } => uvenv::cli::commands::init(shell),
        Commands::Completions { shell } => uvenv::cli::commands::completions(shell),
        Commands::Resolve => uvenv::cli::commands::resolve(),
        Commands::Activate { name } => uvenv::cli::commands::activate(&name),
        Commands::Deactivate => uvenv::cli::commands::deactivate(),
    };

    // Handle errors
    if let Err(e) = result {
        output.error(&e.to_string());
        std::process::exit(1);
    }

    Ok(())
}
