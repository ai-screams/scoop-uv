//! Handler for the `scoop info` command

use std::path::Path;
use std::process::Command;

use crate::core::{VirtualenvService, get_active_env};
use crate::error::{Result, ScoopError};
use crate::output::{EnvInfoData, Output, PackageInfo, PackagesInfo, format_size};
use crate::paths::{abbreviate_home, calculate_dir_size};

const DEFAULT_PACKAGE_LIMIT: usize = 5;

/// Get installed packages using pip list
fn get_packages(venv_path: &Path) -> Vec<(String, String)> {
    let pip_path = venv_path.join("bin").join("pip");

    if !pip_path.exists() {
        return Vec::new();
    }

    let output = Command::new(&pip_path)
        .args(["list", "--format=json"])
        .output();

    match output {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            serde_json::from_str::<Vec<serde_json::Value>>(&stdout)
                .unwrap_or_default()
                .into_iter()
                .filter_map(|p| {
                    Some((
                        p.get("name")?.as_str()?.to_string(),
                        p.get("version")?.as_str()?.to_string(),
                    ))
                })
                .collect()
        }
        _ => Vec::new(),
    }
}

/// Execute the info command
pub fn execute(output: &Output, name: &str, all_packages: bool, no_size: bool) -> Result<()> {
    let service = VirtualenvService::auto()?;

    if !service.exists(name)? {
        return Err(ScoopError::VirtualenvNotFound {
            name: name.to_string(),
        });
    }

    let path = service.get_path(name)?;
    let metadata = service.read_metadata(&path);
    let is_active = get_active_env().as_deref() == Some(name);

    // Calculate size (unless --no-size)
    let (size_bytes, size_display) = if no_size {
        (None, None)
    } else {
        calculate_dir_size(&path)
            .inspect_err(|e| tracing::debug!("Size calculation failed: {}", e))
            .map(|b| (Some(b), Some(format_size(b))))
            .unwrap_or((None, None))
    };

    // Get packages
    let packages = get_packages(&path);
    let limit = if all_packages {
        usize::MAX
    } else {
        DEFAULT_PACKAGE_LIMIT
    };
    let truncated = packages.len() > limit;
    let remaining = packages.len().saturating_sub(limit);

    // JSON output
    if output.is_json() {
        let data = EnvInfoData {
            name: name.to_string(),
            python: metadata.as_ref().map(|m| m.python_version.clone()),
            path: path.display().to_string(),
            active: is_active,
            created_at: metadata.as_ref().map(|m| m.created_at.to_rfc3339()),
            size_bytes,
            size_display,
            packages: PackagesInfo {
                total: packages.len(),
                items: packages
                    .iter()
                    .take(limit)
                    .map(|(n, v)| PackageInfo {
                        name: n.clone(),
                        version: v.clone(),
                    })
                    .collect(),
                truncated,
            },
        };
        output.json_success("info", data);
        return Ok(());
    }

    // Human-readable output
    let w = 12; // label width
    let python = metadata
        .as_ref()
        .map(|m| m.python_version.as_str())
        .unwrap_or("-");
    let created = metadata
        .as_ref()
        .map(|m| m.created_at.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_else(|| "-".to_string());

    println!("{:w$}{}", "Name:", name);
    println!("{:w$}{}", "Python:", python);
    println!("{:w$}{}", "Path:", abbreviate_home(&path));
    println!("{:w$}{}", "Active:", if is_active { "yes" } else { "no" });
    println!("{:w$}{}", "Created:", created);

    if let Some(size) = size_display {
        println!("{:w$}{}", "Size:", size);
    }

    println!("{:w$}{}", "Packages:", packages.len());
    let indent = " ".repeat(w);
    for (name, ver) in packages.iter().take(limit) {
        println!("{}{}=={}", indent, name, ver);
    }
    if truncated {
        println!("{}... ({} more)", indent, remaining);
    }

    Ok(())
}
