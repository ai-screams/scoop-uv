//! scoop - Python virtual environment manager powered by uv

use clap::Parser;
use color_eyre::eyre::Result;

use scoop_uv::cli::{Cli, Commands};
use scoop_uv::output::Output;

fn main() -> Result<()> {
    // Initialize i18n (must be early, before any translated output)
    scoop_uv::i18n::init();

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
            python_version,
            json,
        } => {
            let output = Output::new(0, cli.quiet, cli.no_color, json);
            scoop_uv::cli::commands::list(&output, pythons, bare, python_version.as_deref())
        }
        Commands::Create {
            name,
            python,
            python_path,
            force,
            json,
        } => {
            let output = Output::new(0, cli.quiet, cli.no_color, json);
            scoop_uv::cli::commands::create(&output, &name, &python, python_path.as_deref(), force)
        }
        Commands::Doctor { verbose, json, fix } => {
            let output = Output::new(verbose, cli.quiet, cli.no_color, json);
            scoop_uv::cli::commands::doctor(&output, fix)
        }
        Commands::Info {
            name,
            json,
            all_packages,
            no_size,
        } => {
            let output = Output::new(0, cli.quiet, cli.no_color, json);
            scoop_uv::cli::commands::info(&output, &name, all_packages, no_size)
        }
        Commands::Use {
            name,
            unset,
            global,
            link,
            no_link: _, // explicit option, same as default (no symlink)
            json,
        } => {
            let output = Output::new(0, cli.quiet, cli.no_color, json);
            scoop_uv::cli::commands::use_env(&output, name.as_deref(), unset, global, link)
        }
        Commands::Remove { name, force, json } => {
            let output = Output::new(0, cli.quiet, cli.no_color, json);
            scoop_uv::cli::commands::remove(&output, &name, force)
        }
        Commands::Install {
            python_version,
            latest,
            stable,
            json,
        } => {
            let output = Output::new(0, cli.quiet, cli.no_color, json);
            scoop_uv::cli::commands::install(&output, python_version.as_deref(), latest, stable)
        }
        Commands::Uninstall {
            python_version,
            cascade,
            force,
            json,
        } => {
            let output = Output::new(0, cli.quiet, cli.no_color, json);
            scoop_uv::cli::commands::uninstall(&output, &python_version, cascade, force)
        }
        Commands::Init { shell } => scoop_uv::cli::commands::init(shell),
        Commands::Completions { shell } => scoop_uv::cli::commands::completions(shell),
        Commands::Resolve => scoop_uv::cli::commands::resolve(),
        Commands::Activate { name, shell } => scoop_uv::cli::commands::activate(&name, shell),
        Commands::Deactivate { shell } => scoop_uv::cli::commands::deactivate(shell),
        Commands::Shell { name, unset, shell } => {
            let output = Output::new(0, cli.quiet, cli.no_color, false);
            scoop_uv::cli::commands::shell(&output, name.as_deref(), unset, shell)
        }
        Commands::Migrate { command } => {
            let output = Output::new(0, cli.quiet, cli.no_color, false);
            scoop_uv::cli::commands::migrate(&output, command)
        }
        Commands::Lang {
            lang,
            list,
            reset,
            json,
        } => {
            let output = Output::new(0, cli.quiet, cli.no_color, json);
            scoop_uv::cli::commands::lang(&output, lang.as_deref(), list, reset)
        }
    };

    // Handle errors
    if let Err(e) = result {
        let output = Output::new(0, cli.quiet, cli.no_color, false);
        output.error(&e.to_string());

        // Print suggestion hint if available
        if let Some(suggestion) = e.suggestion() {
            eprintln!("{suggestion}");
        }

        std::process::exit(1);
    }

    Ok(())
}
