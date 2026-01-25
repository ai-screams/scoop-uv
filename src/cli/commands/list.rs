//! List command

use std::collections::BTreeSet;

use owo_colors::OwoColorize;
use rust_i18n::t;

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
    use crate::core::VersionService;

    let service = VirtualenvService::auto()?;
    let envs = service.list()?;
    let active_env = get_active_env();

    // Check if "system" is the resolved version
    let resolved = VersionService::resolve_current();
    let system_active = resolved.as_deref() == Some("system");

    // Get system Python info
    let system_python = get_system_python_info();

    // JSON output
    if output.is_json() {
        let mut virtualenvs: Vec<VirtualenvInfo> = envs
            .iter()
            .map(|env| VirtualenvInfo {
                name: env.name.clone(),
                python: env.python_version.clone(),
                path: env.path.display().to_string(),
                active: active_env.as_ref() == Some(&env.name),
            })
            .collect();

        // Add system Python to JSON output
        if let Some((version, path)) = &system_python {
            virtualenvs.push(VirtualenvInfo {
                name: "system".to_string(),
                python: Some(version.clone()),
                path: path.clone(),
                active: system_active,
            });
        }

        let total = virtualenvs.len();
        output.json_success("list", ListEnvsData { virtualenvs, total });
        return Ok(());
    }

    if envs.is_empty() && system_python.is_none() {
        if !bare {
            output.info(&t!("list.no_envs"));
            output.info(&t!("list.no_envs_hint"));
        }
        return Ok(());
    }

    if bare {
        // Output names only, one per line (for completion)
        for env in &envs {
            println!("{}", env.name);
        }
        // Add system to bare output
        if system_python.is_some() {
            println!("system");
        }
    } else {
        // Calculate column widths for alignment (include "system" in calculation)
        let mut max_name_len = envs.iter().map(|e| e.name.len()).max().unwrap_or(0);
        if system_python.is_some() {
            max_name_len = max_name_len.max(6); // "system".len() == 6
        }

        let mut max_ver_len = envs
            .iter()
            .filter_map(|e| e.python_version.as_ref())
            .map(|v| v.len())
            .max()
            .unwrap_or(1);
        if let Some((version, _)) = &system_python {
            max_ver_len = max_ver_len.max(version.len());
        }

        // Output with marker, name, version, and path
        for env in &envs {
            let is_active = active_env.as_ref() == Some(&env.name);
            let marker = if is_active { "*" } else { " " };
            let version = env.python_version.as_deref().unwrap_or("-");
            let path = abbreviate_home(&env.path);

            if output.use_color() && is_active {
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

        // Add system Python at the end
        if let Some((version, path)) = system_python {
            let marker = if system_active { "*" } else { " " };
            let display_path = format!("{} (system)", path);

            if output.use_color() && system_active {
                println!(
                    "{} {:<name_w$}  {:<ver_w$}  {}",
                    marker.green(),
                    "system".green(),
                    version,
                    display_path.dimmed(),
                    name_w = max_name_len,
                    ver_w = max_ver_len
                );
            } else {
                println!(
                    "{} {:<name_w$}  {:<ver_w$}  {}",
                    marker,
                    "system",
                    version,
                    display_path,
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
            output.info(&t!("list.no_pythons"));
            output.info(&t!("list.no_pythons_hint"));
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

/// Get system Python version and path
///
/// Returns `(version, path)` tuple if system Python is found.
fn get_system_python_info() -> Option<(String, String)> {
    use std::process::Command;

    // Try python3 first, then python - reuse the output to avoid double process calls
    let (python_cmd, version_output) = {
        let output = Command::new("python3").arg("--version").output().ok();
        match output {
            Some(ref out) if out.status.success() => ("python3", output),
            _ => (
                "python",
                Command::new("python").arg("--version").output().ok(),
            ),
        }
    };

    let version_output = version_output?;

    if !version_output.status.success() {
        return None;
    }

    let version_str = String::from_utf8_lossy(&version_output.stdout);
    // "Python 3.12.1" -> "3.12.1"
    let version = version_str
        .trim()
        .strip_prefix("Python ")
        .unwrap_or(version_str.trim())
        .to_string();

    // Get path using 'which' on Unix
    let path_output = Command::new("which").arg(python_cmd).output().ok()?;

    if !path_output.status.success() {
        return None;
    }

    let path = String::from_utf8_lossy(&path_output.stdout)
        .trim()
        .to_string();

    Some((version, path))
}
