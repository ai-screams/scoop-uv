//! scoop - Python virtual environment manager powered by uv
//!
//! Provides pyenv-style workflow for managing Python virtual environments
//! using uv as the backend for blazing fast operations.

pub mod cli;
pub mod core;
pub mod error;
pub mod output;
pub mod paths;
pub mod shell;
pub mod uv;
pub mod validate;

#[cfg(test)]
pub mod test_utils;

pub use error::{Result, ScoopError};
