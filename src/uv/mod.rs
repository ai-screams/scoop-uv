//! uv client module

mod client;
pub(crate) mod install;
pub mod version;

pub use client::{PythonInfo, UvClient, UvPipListEntry};
