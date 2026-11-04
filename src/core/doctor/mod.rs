//! Doctor diagnostic module for scuv installation health checks.
//!
//! This module provides functionality to diagnose the scuv installation
//! and report any issues with suggested fixes.

mod checks;
mod engine;
mod types;

pub(crate) use checks::apply_uv_install;
pub use engine::Doctor;
pub use types::{Check, CheckResult, CheckStatus};
