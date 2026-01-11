//! Remove command

use dialoguer::Confirm;
use rust_i18n::t;

use crate::core::VirtualenvService;
use crate::error::Result;
use crate::output::{Output, RemoveData};

/// Execute the remove command
pub fn execute(output: &Output, name: &str, force: bool) -> Result<()> {
    let service = VirtualenvService::auto()?;

    // Verify environment exists
    let path = service.get_path(name)?;

    // JSON mode always implies force (no interactive confirmation)
    if !force && !output.is_json() {
        // Show what will be deleted
        output.info(&t!(
            "remove.path",
            path = crate::paths::abbreviate_home(&path)
        ));

        let confirmed = Confirm::new()
            .with_prompt(t!("remove.confirm", name = name).to_string())
            .default(false)
            .interact()
            .unwrap_or(false);

        if !confirmed {
            output.info(&t!("remove.cancelled"));
            return Ok(());
        }
    }

    output.info(&t!("remove.removing", name = name));
    service.delete(name)?;

    // JSON output
    if output.is_json() {
        output.json_success(
            "remove",
            RemoveData {
                name: name.to_string(),
                path: path.display().to_string(),
            },
        );
        return Ok(());
    }

    output.success(&t!("remove.success", name = name));

    Ok(())
}
