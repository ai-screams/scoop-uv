//! Handler for the `scuv info` command

use chrono::Utc;

use crate::core::{VirtualenvService, get_active_env, list_installed_packages};
use crate::error::{Result, ScoopError};
use crate::output::{EnvInfoData, Output, PackagesInfo, format_last_used_value, format_size};
use crate::paths::{abbreviate_home, calculate_dir_size};

const DEFAULT_PACKAGE_LIMIT: usize = 5;

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

    // Get packages with truncation
    let packages = list_installed_packages(&path);
    let limit = if all_packages {
        usize::MAX
    } else {
        DEFAULT_PACKAGE_LIMIT
    };
    let packages_info = PackagesInfo::new(&packages, limit);

    let last_used_ts = metadata.as_ref().and_then(|m| m.last_used);

    // JSON output
    if output.is_json() {
        let data = EnvInfoData {
            name: name.to_string(),
            python: metadata.as_ref().map(|m| m.python_version.clone()),
            path: path.display().to_string(),
            active: is_active,
            created_at: metadata.as_ref().map(|m| m.created_at.to_rfc3339()),
            last_used: last_used_ts.map(|t| t.to_rfc3339()),
            size_bytes,
            size_display,
            packages: packages_info,
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
    // Shared three-state contract — see [`format_last_used_value`] for
    // the "hide vs never vs N units ago" rules.
    if let Some(label) = format_last_used_value(metadata.is_some(), last_used_ts, Utc::now()) {
        println!("{:w$}{}", "Last used:", label);
    }

    if let Some(size) = size_display {
        println!("{:w$}{}", "Size:", size);
    }

    println!("{:w$}{}", "Packages:", packages_info.total);
    let indent = " ".repeat(w);
    for pkg in &packages_info.items {
        println!("{}{}=={}", indent, pkg.name, pkg.version);
    }
    if packages_info.truncated {
        println!("{}... ({} more)", indent, packages_info.remaining());
    }

    Ok(())
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::with_temp_scoop_home;
    use serial_test::serial;

    // =========================================================================
    // execute Error Path Tests
    // =========================================================================

    #[test]
    #[serial]
    fn execute_nonexistent_env_returns_error() {
        with_temp_scoop_home(|temp_dir| {
            // Create virtualenvs directory (required by VirtualenvService)
            std::fs::create_dir_all(temp_dir.path().join("virtualenvs")).unwrap();

            let output = Output::new(0, false, false, false);
            let result = execute(&output, "nonexistent", false, false);

            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(matches!(err, ScoopError::VirtualenvNotFound { .. }));
        });
    }

    #[test]
    #[serial]
    fn execute_with_all_packages_flag() {
        with_temp_scoop_home(|temp_dir| {
            std::fs::create_dir_all(temp_dir.path().join("virtualenvs")).unwrap();

            let output = Output::new(0, false, false, false);
            // all_packages flag should not cause panic even with nonexistent env
            let result = execute(&output, "nonexistent", true, false);

            assert!(result.is_err());
        });
    }

    #[test]
    #[serial]
    fn execute_with_no_size_flag() {
        with_temp_scoop_home(|temp_dir| {
            std::fs::create_dir_all(temp_dir.path().join("virtualenvs")).unwrap();

            let output = Output::new(0, false, false, false);
            // no_size flag should not cause panic
            let result = execute(&output, "nonexistent", false, true);

            assert!(result.is_err());
        });
    }

    // =========================================================================
    // Package Limit Logic Tests
    // =========================================================================

    #[test]
    fn package_limit_with_all_packages_is_usize_max() {
        let all_packages = true;
        let limit = if all_packages {
            usize::MAX
        } else {
            DEFAULT_PACKAGE_LIMIT
        };
        assert_eq!(limit, usize::MAX);
    }

    #[test]
    fn package_limit_without_all_packages_is_default() {
        let all_packages = false;
        let limit = if all_packages {
            usize::MAX
        } else {
            DEFAULT_PACKAGE_LIMIT
        };
        assert_eq!(limit, DEFAULT_PACKAGE_LIMIT);
    }
}
