//! Install command

use crate::error::{Result, ScoopError};
use crate::output::Output;
use crate::uv::UvClient;

/// Execute the install command
pub fn execute(output: &Output, version: Option<&str>, latest: bool, stable: bool) -> Result<()> {
    // Validate conflicting options
    let target = determine_target(version, latest, stable)?;

    let uv = UvClient::new()?;

    output.info(&format!("Installing Python {target}..."));

    uv.install_python(&target)?;

    output.success(&format!("Installed Python {target}"));

    Ok(())
}

/// Determine the Python version to install based on options
fn determine_target(version: Option<&str>, latest: bool, stable: bool) -> Result<String> {
    // Check for conflicting options
    if stable && latest {
        return Err(ScoopError::InvalidArgument {
            message: "Cannot use both --stable and --latest".to_string(),
        });
    }

    if let Some(ver) = version {
        if stable {
            return Err(ScoopError::InvalidArgument {
                message: format!("Cannot use --stable with version '{ver}'"),
            });
        }
        if latest {
            return Err(ScoopError::InvalidArgument {
                message: format!("Cannot use --latest with version '{ver}'"),
            });
        }
        return Ok(ver.to_string());
    }

    // No version specified
    if stable {
        // Use oldest fully-supported Python (currently 3.10)
        // This is the oldest version with active security support
        Ok("3.10".to_string())
    } else {
        // Default to latest (--latest flag or no flag)
        // uv uses "3" to mean "latest Python 3.x"
        Ok("3".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determine_target_version() {
        assert_eq!(
            determine_target(Some("3.12"), false, false).unwrap(),
            "3.12"
        );
    }

    #[test]
    fn test_determine_target_latest() {
        assert_eq!(determine_target(None, true, false).unwrap(), "3");
    }

    #[test]
    fn test_determine_target_stable() {
        assert_eq!(determine_target(None, false, true).unwrap(), "3.10");
    }

    #[test]
    fn test_determine_target_default() {
        assert_eq!(determine_target(None, false, false).unwrap(), "3");
    }

    #[test]
    fn test_determine_target_conflict_stable_latest() {
        assert!(determine_target(None, true, true).is_err());
    }

    #[test]
    fn test_determine_target_conflict_stable_version() {
        assert!(determine_target(Some("3.12"), false, true).is_err());
    }

    #[test]
    fn test_determine_target_conflict_latest_version() {
        assert!(determine_target(Some("3.12"), true, false).is_err());
    }
}
