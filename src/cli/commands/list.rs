//! List command

use crate::core::VirtualenvService;
use crate::error::Result;
use crate::output::Output;

/// Execute the list command
pub fn execute(output: &Output) -> Result<()> {
    let service = VirtualenvService::auto()?;
    let envs = service.list()?;

    if envs.is_empty() {
        output.info("No virtual environments found");
        output.info("Create one with: scoop create <name> --python <version>");
        return Ok(());
    }

    for env in envs {
        let version_str = env
            .python_version
            .map(|v| format!(" (Python {v})"))
            .unwrap_or_default();

        output.println(&format!("{}{}", env.name, version_str));
    }

    Ok(())
}
