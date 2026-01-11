//! Conflict resolution for environment migration
//!
//! Handles name conflicts when a scoop environment with the same name already exists.

use dialoguer::{Input, Select};

use crate::error::{Result, ScoopError};
use crate::output::Output;

/// Maximum number of rename attempts before failing.
pub const MAX_RENAME_ATTEMPTS: usize = 100;

/// User choice for handling name conflicts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictResolution {
    /// Delete existing environment and migrate fresh
    Overwrite,
    /// Migrate with a different name
    Rename,
    /// Don't migrate this environment
    Skip,
}

/// Prompts user for conflict resolution choice.
///
/// Displays three options: Overwrite, Rename, or Skip.
/// Default selection is Skip (safest option).
///
/// # Examples
///
/// ```ignore
/// use std::path::Path;
/// use crate::cli::commands::migrate::conflict::{prompt_conflict_resolution, ConflictResolution};
///
/// let existing = Path::new("/home/user/.scoop/virtualenvs/myenv");
/// let resolution = prompt_conflict_resolution(&output, "myenv", existing)?;
///
/// match resolution {
///     ConflictResolution::Overwrite => println!("Will delete and recreate"),
///     ConflictResolution::Rename => println!("Will prompt for new name"),
///     ConflictResolution::Skip => println!("Skipping migration"),
/// }
/// ```
pub fn prompt_conflict_resolution(
    output: &Output,
    name: &str,
    existing: &std::path::Path,
) -> Result<ConflictResolution> {
    output.warn(&format!(
        "Name conflict: '{}' already exists at {}",
        name,
        existing.display()
    ));

    let options = &[
        "Overwrite - Delete existing and migrate fresh",
        "Rename - Migrate with a different name",
        "Skip - Don't migrate this environment",
    ];

    let selection = Select::new()
        .with_prompt("How would you like to resolve this conflict?")
        .items(options)
        .default(2) // Default to Skip (safest)
        .interact()
        .map_err(|e| ScoopError::Io(std::io::Error::other(format!("Dialog error: {}", e))))?;

    Ok(match selection {
        0 => ConflictResolution::Overwrite,
        1 => ConflictResolution::Rename,
        _ => ConflictResolution::Skip,
    })
}

/// Prompts for new environment name when renaming.
///
/// Suggests `{name}-pyenv` as default and validates the input.
///
/// # Examples
///
/// ```ignore
/// use crate::cli::commands::migrate::conflict::prompt_rename;
///
/// // User will see: "Enter new name for the environment [myenv-pyenv]:"
/// let new_name = prompt_rename("myenv")?;
/// println!("Will migrate as: {}", new_name);
/// ```
pub fn prompt_rename(name: &str) -> Result<String> {
    let suggested = format!("{}-pyenv", name);

    let new_name: String = Input::new()
        .with_prompt("Enter new name for the environment")
        .default(suggested)
        .validate_with(|input: &String| {
            crate::validate::validate_env_name(input)
                .map(|_| ())
                .map_err(|e| e.to_string())
        })
        .interact_text()
        .map_err(|e| ScoopError::Io(std::io::Error::other(format!("Dialog error: {}", e))))?;

    Ok(new_name)
}

/// Generates a unique name by appending suffixes.
///
/// Tries `{name}-pyenv` first, then `{name}-1`, `{name}-2`, etc.
/// up to [`MAX_RENAME_ATTEMPTS`].
///
/// # Examples
///
/// ```ignore
/// use scoop_uv::cli::commands::migrate::conflict::generate_unique_name;
///
/// // Setup: isolated SCOOP_HOME
/// let temp = tempfile::tempdir().unwrap();
/// std::fs::create_dir_all(temp.path().join("virtualenvs")).unwrap();
/// std::env::set_var("SCOOP_HOME", temp.path());
///
/// // If "myenv-pyenv" doesn't exist, returns "myenv-pyenv"
/// let unique = generate_unique_name("myenv").unwrap();
/// assert_eq!(unique, "myenv-pyenv");
///
/// // Create "myenv-pyenv" to test fallback
/// std::fs::create_dir(temp.path().join("virtualenvs/myenv-pyenv")).unwrap();
/// let unique2 = generate_unique_name("myenv").unwrap();
/// assert_eq!(unique2, "myenv-1");
/// std::env::remove_var("SCOOP_HOME");
/// ```
///
/// # Errors
///
/// Returns [`ScoopError::MigrationFailed`] if no unique name can be found
/// after [`MAX_RENAME_ATTEMPTS`] tries.
pub fn generate_unique_name(base_name: &str) -> Result<String> {
    // Try {name}-pyenv first
    let first_try = format!("{}-pyenv", base_name);
    if !crate::paths::virtualenv_path(&first_try)?.exists() {
        return Ok(first_try);
    }

    // Try numbered suffixes
    for i in 1..MAX_RENAME_ATTEMPTS {
        let candidate = format!("{}-{}", base_name, i);
        if !crate::paths::virtualenv_path(&candidate)?.exists() {
            return Ok(candidate);
        }
    }

    Err(ScoopError::MigrationFailed {
        reason: format!("Could not find unique name for '{}'", base_name),
    })
}
