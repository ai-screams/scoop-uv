//! scoop - Python virtual environment manager powered by uv

use clap::Parser;
use color_eyre::eyre::Result;

use scoop::cli::{Cli, Commands};
use scoop::output::Output;

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
        Commands::List => scoop::cli::commands::list(&output),
        Commands::Create {
            name,
            python,
            force,
        } => scoop::cli::commands::create(&output, &name, &python, force),
        Commands::Use {
            name,
            global,
            no_link,
        } => scoop::cli::commands::use_env(&output, &name, global, no_link),
        Commands::Remove { name, force } => scoop::cli::commands::remove(&output, &name, force),
        Commands::Install { version } => scoop::cli::commands::install(&output, &version),
        Commands::Init { shell } => scoop::cli::commands::init(shell),
        Commands::Completions { shell } => scoop::cli::commands::completions(shell),
        Commands::Resolve => scoop::cli::commands::resolve(),
        Commands::Activate { name } => scoop::cli::commands::activate(&name),
        Commands::Deactivate => scoop::cli::commands::deactivate(),
    };

    // Handle errors
    if let Err(e) = result {
        output.error(&e.to_string());
        std::process::exit(1);
    }

    Ok(())
}
