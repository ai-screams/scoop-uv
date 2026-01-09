//! CLI command handlers

mod activate;
mod completions;
mod create;
mod deactivate;
mod doctor;
mod info;
mod init;
mod install;
mod list;
mod remove;
mod resolve;
mod uninstall;
mod use_env;

pub use activate::execute as activate;
pub use completions::execute as completions;
pub use create::execute as create;
pub use deactivate::execute as deactivate;
pub use doctor::execute as doctor;
pub use info::execute as info;
pub use init::execute as init;
pub use install::execute as install;
pub use list::execute as list;
pub use remove::execute as remove;
pub use resolve::execute as resolve;
pub use uninstall::execute as uninstall;
pub use use_env::execute as use_env;
