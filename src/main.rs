//! scoop - Python virtual environment manager powered by uv

use clap::Parser;
use color_eyre::eyre::Result;

use scoop_uv::cli::{Cli, Commands, MigrateCommand, SelfCommand};
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
            sort,
            json,
        } => {
            let output = Output::new(0, cli.quiet, cli.no_color, json);
            scoop_uv::cli::commands::list(&output, pythons, bare, python_version.as_deref(), sort)
        }
        Commands::Create {
            name,
            python,
            python_path,
            force,
            install_python,
            json,
        } => {
            let output = Output::new(0, cli.quiet, cli.no_color, json);
            scoop_uv::cli::commands::create(
                &output,
                &name,
                &python,
                python_path.as_deref(),
                force,
                install_python,
            )
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
            // Subcommand carries its own --json flag (list / all / @env).
            // Without threading it into Output here, output.json_success()
            // would no-op in production (only tests built Output directly
            // with json=true), so the new emit_migrate_all_json_outcome
            // helper would never fire from the CLI. Bug fix is bundled
            // with Inc 4 because the rest of the JSON path depends on it.
            let json = match &command {
                Some(MigrateCommand::List { json, .. })
                | Some(MigrateCommand::All { json, .. })
                | Some(MigrateCommand::Env { json, .. }) => *json,
                None => false,
            };
            let output = Output::new(0, cli.quiet, cli.no_color, json);
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
        Commands::Self_ { command } => match command {
            SelfCommand::Update {
                force,
                version,
                no_verify,
                json,
            } => {
                let output = Output::new(0, cli.quiet, cli.no_color, json);
                scoop_uv::cli::commands::self_update(&output, force, version.as_deref(), no_verify)
            }
        },
        Commands::Status { json } => {
            let output = Output::new(0, cli.quiet, cli.no_color, json);
            scoop_uv::cli::commands::status(&output)
        }
        Commands::Run { env, command } => {
            let output = Output::new(0, cli.quiet, cli.no_color, false);
            scoop_uv::cli::commands::run(&output, &env, &command)
        }
        Commands::Sync {
            with,
            dry_run,
            json,
        } => {
            let output = Output::new(0, cli.quiet, cli.no_color, json);
            scoop_uv::cli::commands::sync(&output, &with, dry_run)
        }
        Commands::Clone {
            src,
            dst,
            no_packages,
            force,
            json,
        } => {
            let out = Output::new(0, cli.quiet, cli.no_color, json);
            scoop_uv::cli::commands::clone(&out, &src, &dst, no_packages, force)
        }
        Commands::Export { name, output } => {
            // Stdout is the schema itself; status messages go to stderr only.
            let out = Output::new(0, cli.quiet, cli.no_color, false);
            scoop_uv::cli::commands::export(&out, &name, output.as_deref())
        }
        Commands::Import {
            path,
            name,
            force,
            json,
        } => {
            let out = Output::new(0, cli.quiet, cli.no_color, json);
            scoop_uv::cli::commands::import(&out, &path, name.as_deref(), force)
        }
        Commands::Which { exe, env, json } => {
            let output = Output::new(0, cli.quiet, cli.no_color, json);
            scoop_uv::cli::commands::which(&output, &exe, env.as_deref())
        }
        Commands::Prune { json } => {
            let output = Output::new(0, cli.quiet, cli.no_color, json);
            scoop_uv::cli::commands::prune(&output)
        }
        Commands::Gc {
            yes,
            aggressive,
            older_than,
            json,
        } => {
            let output = Output::new(0, cli.quiet, cli.no_color, json);
            scoop_uv::cli::commands::gc(&output, yes, aggressive, older_than.as_deref())
        }
        Commands::Man { output_dir, json } => {
            let output = Output::new(0, cli.quiet, cli.no_color, json);
            scoop_uv::cli::commands::man(&output, output_dir.as_deref())
        }
        Commands::Verify { name, strict, json } => {
            let output = Output::new(0, cli.quiet, cli.no_color, json);
            scoop_uv::cli::commands::verify(&output, name.as_deref(), strict)
        }
        Commands::Diff {
            env_a,
            env_b,
            packages_only,
            metadata_only,
            strict,
            json,
        } => {
            let output = Output::new(0, cli.quiet, cli.no_color, json);
            let mode = match (packages_only, metadata_only) {
                (true, false) => scoop_uv::cli::commands::DiffMode::PackagesOnly,
                (false, true) => scoop_uv::cli::commands::DiffMode::MetadataOnly,
                // (false, false) is the default; (true, true) is blocked by clap.
                _ => scoop_uv::cli::commands::DiffMode::All,
            };
            scoop_uv::cli::commands::diff(
                &output,
                &scoop_uv::cli::commands::DiffOpts {
                    env_a,
                    env_b,
                    mode,
                    strict,
                },
            )
        }
    };

    // Handle errors via the policy layer in `src/error/exit.rs`:
    //   - `render_policy()` decides whether to print the global `error:`
    //     prefix (Default) or stay quiet because the command already
    //     rendered its report (Quiet — e.g. `verify --strict`).
    //   - `exit_code()` decides the process exit code (1 / 2 / 3) so CI
    //     scripts can distinguish source-discovery failures (migrate, exit 3)
    //     from generic operational errors (exit 1).
    if let Err(e) = result {
        let output = Output::new(0, cli.quiet, cli.no_color, false);
        if matches!(
            e.render_policy(),
            scoop_uv::error::ErrorRenderPolicy::Default
        ) {
            output.error(&e.to_string());
            if let Some(suggestion) = e.suggestion() {
                eprintln!("{suggestion}");
            }
        }
        std::process::exit(i32::from(e.exit_code()));
    }

    Ok(())
}
