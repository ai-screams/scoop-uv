//! List command

use crate::core::VirtualenvService;
use crate::error::Result;
use crate::output::{ListEnvsData, ListPythonsData, Output, PythonInfo, VirtualenvInfo};
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

    // JSON output
    if output.is_json() {
        let virtualenvs: Vec<VirtualenvInfo> = envs
            .iter()
            .map(|env| VirtualenvInfo {
                name: env.name.clone(),
                python: env.python_version.clone(),
                path: env.path.display().to_string(),
                active: false, // TODO: detect active env
            })
            .collect();
        let total = virtualenvs.len();
        output.json_success("list", ListEnvsData { virtualenvs, total });
        return Ok(());
    }

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

    // JSON output
    if output.is_json() {
        let python_infos: Vec<PythonInfo> = pythons
            .iter()
            .map(|p| PythonInfo {
                version: p.version.clone(),
                path: p.path.as_ref().map(|path| path.display().to_string()),
            })
            .collect();
        let total = python_infos.len();
        output.json_success(
            "list",
            ListPythonsData {
                pythons: python_infos,
                total,
            },
        );
        return Ok(());
    }

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
