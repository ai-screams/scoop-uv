//! Migration command implementation
//!
//! Migrates environments from pyenv, virtualenvwrapper, and conda to scoop.
//!
//! # Module Structure
//!
//! - `types`: Data structures for JSON serialization and CLI options
//! - `conflict`: Conflict resolution dialog and auto-rename logic
//! - `scan`: Environment discovery from various sources
//! - `list`: Environment listing display
//! - `single`: Single environment migration
//! - `batch`: Batch migration with progress tracking

mod batch;
mod conflict;
mod list;
mod scan;
mod single;
mod types;

use crate::cli::MigrateCommand;
use crate::error::Result;
use crate::output::Output;

use batch::migrate_all_environments;
use list::list_environments;
use single::migrate_environment;
use types::MigrateExecuteOptions;

/// Execute migrate command.
///
/// Dispatches to the appropriate subcommand handler:
/// - `list`: Show available environments
/// - `@env`: Migrate single environment
/// - `all`: Migrate all environments
pub fn execute(output: &Output, command: Option<MigrateCommand>) -> Result<()> {
    match command {
        Some(MigrateCommand::List { json, source }) => list_environments(output, json, source),
        Some(MigrateCommand::All {
            dry_run,
            force,
            yes,
            json,
            strict,
            delete_source,
            source,
        }) => {
            let opts = MigrateExecuteOptions {
                dry_run,
                force,
                yes,
                json,
                strict,
                delete_source,
                source_filter: source,
                ..Default::default()
            };
            migrate_all_environments(output, &opts)
        }
        Some(MigrateCommand::Env {
            name,
            dry_run,
            force,
            yes,
            json,
            strict,
            rename,
            auto_rename,
            delete_source,
            source,
        }) => {
            let opts = MigrateExecuteOptions {
                dry_run,
                force,
                yes,
                json,
                strict,
                rename,
                auto_rename,
                delete_source,
                source_filter: source,
            };
            migrate_environment(output, &name, &opts)
        }
        None => {
            // No subcommand - show help or list
            list_environments(output, output.is_json(), None)
        }
    }
}
