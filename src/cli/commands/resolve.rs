//! Resolve command

use crate::core::VersionService;
use crate::error::Result;

/// Execute the resolve command
pub fn execute() -> Result<()> {
    if let Some(env_name) = VersionService::resolve_current() {
        println!("{env_name}");
    }
    Ok(())
}
