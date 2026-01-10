//! List command

use std::collections::BTreeSet;

use owo_colors::OwoColorize;

use crate::core::{VirtualenvService, get_active_env};
use crate::error::Result;
use crate::output::{ListEnvsData, ListPythonsData, Output, PythonInfo, VirtualenvInfo};
use crate::paths::abbreviate_home;
use crate::uv::UvClient;
use crate::validate::PythonVersion;

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
    let active_env = get_active_env();

    // JSON output
    if output.is_json() {
        let virtualenvs: Vec<VirtualenvInfo> = envs
            .iter()
            .map(|env| VirtualenvInfo {
                name: env.name.clone(),
                python: env.python_version.clone(),
                path: env.path.display().to_string(),
                active: active_env.as_ref() == Some(&env.name),
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
        // Calculate column widths for alignment
        let max_name_len = envs.iter().map(|e| e.name.len()).max().unwrap_or(0);
        let max_ver_len = envs
            .iter()
            .filter_map(|e| e.python_version.as_ref())
            .map(|v| v.len())
            .max()
            .unwrap_or(1); // At least 1 for "-"

        // Output with marker, name, version, and path
        for env in envs {
            let is_active = active_env.as_ref() == Some(&env.name);
            let marker = if is_active { "*" } else { " " };
            let version = env.python_version.as_deref().unwrap_or("-");
            let path = abbreviate_home(&env.path);

            if output.use_color() && is_active {
                // Active environment in green
                println!(
                    "{} {:<name_w$}  {:<ver_w$}  {}",
                    marker.green(),
                    env.name.green(),
                    version,
                    path,
                    name_w = max_name_len,
                    ver_w = max_ver_len
                );
            } else {
                // Normal output
                println!(
                    "{} {:<name_w$}  {:<ver_w$}  {}",
                    marker,
                    env.name,
                    version,
                    path,
                    name_w = max_name_len,
                    ver_w = max_ver_len
                );
            }
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
        // Output unique, sorted versions for shell completion
        // This eliminates the need for `| sort -u` in shell scripts
        let versions: BTreeSet<PythonVersion> = pythons
            .iter()
            .filter_map(|p| PythonVersion::parse(&p.version))
            .collect();

        for version in versions {
            println!("{version}");
        }
    } else {
        // Calculate max version length for alignment
        let max_ver_len = pythons.iter().map(|p| p.version.len()).max().unwrap_or(0);

        // Normal output with path info
        for python in pythons {
            let path_str = python
                .path
                .map(|p| format!("({})", p.display()))
                .unwrap_or_default();

            println!(
                "{:<width$}  {}",
                python.version,
                path_str,
                width = max_ver_len
            );
        }
    }

    Ok(())
}
