//! List command

use crate::core::VirtualenvService;
use crate::error::Result;
use crate::output::Output;
use crate::uv::UvClient;

/// Execute the list command
pub fn execute(output: &Output, pythons: bool, bare: bool) -> Result<()> {
    if pythons {
        list_pythons(output, bare)
    } else {
        list_virtualenvs(output, bare)
    }
}

/// List virtual environments
fn list_virtualenvs(output: &Output, bare: bool) -> Result<()> {
    let service = VirtualenvService::auto()?;
    let envs = service.list()?;

    if envs.is_empty() {
        if !bare {
            output.info("No virtual environments found");
            output.info("Create one with: scoop create <name> <version>");
        }
        return Ok(());
    }

    if bare {
        // Output names only, one per line (for completion)
        for env in envs {
            println!("{}", env.name);
        }
    } else {
        // Normal output with Python version info
        for env in envs {
            let version_str = env
                .python_version
                .map(|v| format!(" (Python {v})"))
                .unwrap_or_default();

            output.println(&format!("{}{}", env.name, version_str));
        }
    }

    Ok(())
}

/// List installed Python versions
fn list_pythons(output: &Output, bare: bool) -> Result<()> {
    let uv = UvClient::new()?;
    let pythons = uv.list_installed_pythons()?;

    if pythons.is_empty() {
        if !bare {
            output.info("No Python versions installed");
            output.info("Install one with: scoop install <version>");
        }
        return Ok(());
    }

    if bare {
        // Output versions only, one per line (for completion)
        for python in pythons {
            println!("{}", python.version);
        }
    } else {
        // Normal output with path info
        for python in pythons {
            let path_str = python
                .path
                .map(|p| format!(" ({})", p.display()))
                .unwrap_or_default();

            output.println(&format!("{}{}", python.version, path_str));
        }
    }

    Ok(())
}
