//! Create command

use rust_i18n::t;

use crate::core::VirtualenvService;
use crate::error::Result;
use crate::output::{CreateData, Output};
use crate::paths;

/// Execute the create command
pub fn execute(output: &Output, name: &str, python: &str, force: bool) -> Result<()> {
    let service = VirtualenvService::auto()?;

    // Check if exists and handle force
    if service.exists(name)? {
        if force {
            output.info(&t!("create.removing_existing", name = name));
            service.delete(name)?;
        } else {
            return Err(crate::error::ScoopError::VirtualenvExists {
                name: name.to_string(),
            });
        }
    }

    output.info(&t!("create.creating", name = name, python = python));

    let path = service.create(name, python)?;

    // JSON output
    if output.is_json() {
        output.json_success(
            "create",
            CreateData {
                name: name.to_string(),
                python: python.to_string(),
                path: path.display().to_string(),
            },
        );
        return Ok(());
    }

    output.success(&t!("create.success", name = name));
    output.info(&t!("create.path", path = paths::abbreviate_home(&path)));
    output.info(&t!("create.activate_hint", name = name));

    Ok(())
}
