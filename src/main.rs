//! scuv - Python virtual environment manager powered by uv

use clap::{CommandFactory, FromArgMatches};
use color_eyre::eyre::Result;

use scoop_uv::cli::color;
use scoop_uv::cli::{Cli, Commands, SelfCommand};
use scoop_uv::output::{Colors, Output};

fn main() -> Result<()> {
    restore_default_sigpipe();

    // Initialize i18n (must be early, before any translated output)
    scoop_uv::i18n::init();

    let cli = parse_cli();
    let colors = Colors::detect(cli.color_choice().force());
    install_panic_hook(colors)?;
    init_logging(colors);

    // One Output for the whole run: quiet and colors are global, json and
    // verbosity come from the subcommand (`Commands::json` is exhaustive).
    let quiet = cli.quiet;
    let output = Output::new(cli.command.verbosity(), quiet, colors, cli.command.json());

    if let Err(e) = dispatch(cli.command, &output) {
        exit_with(&e, quiet, colors);
    }
    Ok(())
}

/// Lets a closed stdout end the process the way it ends `cat` or `ls`.
///
/// The Rust runtime ignores SIGPIPE, so a write to a pipe whose reader has
/// gone (`scuv info myenv | true`, `scuv lang --list | false`) fails with
/// EPIPE and `println!` panics: exit 101 and a crash report on stderr.
/// With the default action the kernel stops the process at that write,
/// silently. The shell wrappers read scuv's whole output through `$(...)`,
/// so they never close the pipe early.
#[cfg(unix)]
fn restore_default_sigpipe() {
    // SAFETY: runs first thing in `main`, before any thread exists, and only
    // sets a signal disposition to its default.
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }
}

#[cfg(not(unix))]
fn restore_default_sigpipe() {}

/// Parses the command line. clap prints help and parse errors before `Cli`
/// exists, so it gets the color choice from a scan of the raw arguments
/// (#205); everything after parsing uses the parsed flags instead, since the
/// scan cannot tell `scuv run env --color x` apart from scuv's own.
fn parse_cli() -> Cli {
    let early = color::choice_from_args(std::env::args_os());
    let matches = Cli::command().color(early.into()).get_matches();
    Cli::from_arg_matches(&matches)
        .unwrap_or_else(|e| e.format(&mut Cli::command().color(early.into())).exit())
}

/// Installs the panic/error report hook; it writes to stderr, so it follows
/// stderr's color decision.
fn install_panic_hook(colors: Colors) -> Result<()> {
    let mut hook = color_eyre::config::HookBuilder::default();
    if !colors.stderr {
        hook = hook.theme(color_eyre::config::Theme::new());
    }
    hook.install()
}

/// Initializes logging. fmt() writes to stdout by default, which the shell
/// wrappers eval: a warning there became a "command" in the user's shell.
/// Send it to stderr, colored by the same decision as the rest.
fn init_logging(colors: Colors) {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_ansi(colors.stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::WARN.into()),
        )
        .init();
}

/// Runs the subcommand: one arm per command, each handing its arguments to
/// the handler in `scoop_uv::cli::commands`.
fn dispatch(command: Commands, output: &Output) -> scoop_uv::error::Result<()> {
    match command {
        Commands::List {
            pythons,
            bare,
            python_version,
            sort,
            ..
        } => scoop_uv::cli::commands::list(output, pythons, bare, python_version.as_deref(), sort),
        Commands::Create {
            name,
            python,
            python_path,
            force,
            install_python,
            ..
        } => scoop_uv::cli::commands::create(
            output,
            &name,
            &python,
            python_path.as_deref(),
            force,
            install_python,
        ),
        Commands::Doctor { fix, .. } => scoop_uv::cli::commands::doctor(output, fix),
        Commands::Info {
            name,
            all_packages,
            no_size,
            ..
        } => scoop_uv::cli::commands::info(output, &name, all_packages, no_size),
        Commands::Use {
            name,
            unset,
            global,
            link,
            no_link: _, // explicit option, same as default (no symlink)
            ..
        } => scoop_uv::cli::commands::use_env(output, name.as_deref(), unset, global, link),
        Commands::Remove { name, force, .. } => {
            scoop_uv::cli::commands::remove(output, &name, force)
        }
        Commands::Install {
            python_version,
            latest,
            stable,
            ..
        } => scoop_uv::cli::commands::install(output, python_version.as_deref(), latest, stable),
        Commands::Uninstall {
            python_version,
            cascade,
            force,
            ..
        } => scoop_uv::cli::commands::uninstall(output, &python_version, cascade, force),
        Commands::Init { shell } => scoop_uv::cli::commands::init(shell),
        Commands::Completions { shell } => scoop_uv::cli::commands::completions(shell),
        Commands::Resolve => scoop_uv::cli::commands::resolve(),
        Commands::Activate { name, shell } => scoop_uv::cli::commands::activate(&name, shell),
        Commands::Deactivate { shell } => scoop_uv::cli::commands::deactivate(shell),
        Commands::Shell { name, unset, shell } => {
            scoop_uv::cli::commands::shell(output, name.as_deref(), unset, shell)
        }
        Commands::Migrate { command } => {
            // The subcommand's --json (list / all / @env) reaches Output
            // through `Commands::json`; without it json_success() no-ops.
            scoop_uv::cli::commands::migrate(output, command)
        }
        Commands::Lang {
            lang, list, reset, ..
        } => scoop_uv::cli::commands::lang(output, lang.as_deref(), list, reset),
        Commands::Self_ { command } => match command {
            SelfCommand::Update {
                force,
                version,
                no_verify,
                ..
            } => scoop_uv::cli::commands::self_update(output, force, version.as_deref(), no_verify),
        },
        Commands::Status { .. } => scoop_uv::cli::commands::status(output),
        Commands::Run { env, command } => scoop_uv::cli::commands::run(output, &env, &command),
        Commands::Sync { with, dry_run, .. } => {
            scoop_uv::cli::commands::sync(output, &with, dry_run)
        }
        Commands::Clone {
            src,
            dst,
            no_packages,
            force,
            ..
        } => scoop_uv::cli::commands::clone(output, &src, &dst, no_packages, force),
        Commands::Export {
            name,
            output: out_path,
        } => {
            // Stdout is the schema itself; status messages go to stderr only.
            scoop_uv::cli::commands::export(output, &name, out_path.as_deref())
        }
        Commands::Import {
            path, name, force, ..
        } => scoop_uv::cli::commands::import(output, &path, name.as_deref(), force),
        Commands::Which { exe, env, .. } => {
            scoop_uv::cli::commands::which(output, &exe, env.as_deref())
        }
        Commands::Prune { .. } => scoop_uv::cli::commands::prune(output),
        Commands::Gc {
            yes,
            aggressive,
            older_than,
            ..
        } => scoop_uv::cli::commands::gc(output, yes, aggressive, older_than.as_deref()),
        Commands::Man { output_dir, .. } => {
            scoop_uv::cli::commands::man(output, output_dir.as_deref())
        }
        Commands::Verify { name, strict, .. } => {
            scoop_uv::cli::commands::verify(output, name.as_deref(), strict)
        }
        Commands::Diff {
            env_a,
            env_b,
            packages_only,
            metadata_only,
            strict,
            ..
        } => {
            let mode = scoop_uv::cli::commands::DiffMode::from_flags(packages_only, metadata_only);
            scoop_uv::cli::commands::diff(
                output,
                &scoop_uv::cli::commands::DiffOpts {
                    env_a,
                    env_b,
                    mode,
                    strict,
                },
            )
        }
    }
}

/// Reports a failed command and exits, via the policy layer in
/// `src/error/exit.rs`:
///   - `render_policy()` decides whether to print the global `error:` prefix
///     (Default) or stay quiet because the command already rendered its
///     report (Quiet — e.g. `verify --strict`).
///   - `exit_code()` decides the process exit code (1 / 2 / 3) so CI scripts
///     can distinguish source-discovery failures (migrate, exit 3) from
///     generic operational errors (exit 1).
fn exit_with(e: &scoop_uv::error::ScoopError, quiet: bool, colors: Colors) -> ! {
    // Errors print as text even under --json (existing behavior).
    let output = Output::new(0, quiet, colors, false);
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
