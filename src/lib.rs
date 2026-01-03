//! uvenv - Python virtual environment manager powered by uv
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

pub use error::{Result, UvenvError};
