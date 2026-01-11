//! scoop - Python virtual environment manager powered by uv
//!
//! Provides pyenv-style workflow for managing Python virtual environments
//! using uv as the backend for blazing fast operations.

// Initialize i18n - must be before any module declarations
rust_i18n::i18n!("locales", fallback = "en");

pub mod cli;
pub mod config;
pub mod core;
pub mod error;
pub mod i18n;
pub mod output;
pub mod paths;
pub mod shell;
pub mod uv;
pub mod validate;

#[cfg(test)]
pub mod test_utils;

pub use error::{Result, ScoopError};
