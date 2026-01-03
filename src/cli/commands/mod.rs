//! CLI command handlers

mod activate;
mod completions;
mod create;
mod deactivate;
mod init;
mod install;
mod list;
mod remove;
mod resolve;
mod use_env;

pub use activate::execute as activate;
pub use completions::execute as completions;
pub use create::execute as create;
pub use deactivate::execute as deactivate;
pub use init::execute as init;
pub use install::execute as install;
pub use list::execute as list;
pub use remove::execute as remove;
pub use resolve::execute as resolve;
pub use use_env::execute as use_env;
