//! CLI command handlers

mod activate;
mod clone;
mod completions;
mod create;
mod deactivate;
mod doctor;
mod duration;
mod export;
mod gc;
mod import;
mod info;
mod init;
mod install;
mod lang;
mod list;
mod man;
mod migrate;
mod prune;
mod remove;
mod resolve;
mod run;
mod self_update;
mod shell;
mod status;
mod sync;
mod uninstall;
mod use_env;
mod verify;
mod which;

// `duration` is module-private; Step 5's gc handler imports
// `parse_duration` via `use super::duration::parse_duration;` so the
// helper isn't pinned in the crate's public surface area.

pub use activate::execute as activate;
pub use clone::execute as clone;
pub use completions::execute as completions;
pub use create::execute as create;
pub use deactivate::execute as deactivate;
pub use doctor::execute as doctor;
pub use export::execute as export;
pub use gc::execute as gc;
pub use import::execute as import;
pub use info::execute as info;
pub use init::execute as init;
pub use install::execute as install;
pub use lang::execute as lang;
pub use list::execute as list;
pub use man::execute as man;
pub use migrate::execute as migrate;
pub use prune::execute as prune;
pub use remove::execute as remove;
pub use resolve::execute as resolve;
pub use run::execute as run;
pub use self_update::execute as self_update;
pub use shell::execute as shell;
pub use status::execute as status;
pub use sync::execute as sync;
pub use uninstall::execute as uninstall;
pub use use_env::execute as use_env;
pub use verify::execute as verify;
pub use which::execute as which;
