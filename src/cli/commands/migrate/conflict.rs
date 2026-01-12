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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::with_temp_scoop_home;
    use serial_test::serial;
    use std::fs;

    /// Happy path: 충돌 없을 때 {name}-pyenv 반환
    #[test]
    #[serial]
    fn test_generate_unique_name_no_conflict() {
        with_temp_scoop_home(|_temp| {
            let result = generate_unique_name("myenv");
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "myenv-pyenv");
        });
    }

    /// First collision: {name}-pyenv 존재 시 {name}-1 반환
    #[test]
    #[serial]
    fn test_generate_unique_name_first_collision() {
        with_temp_scoop_home(|temp| {
            // myenv-pyenv가 이미 존재하도록 생성
            let existing = temp.path().join("virtualenvs").join("myenv-pyenv");
            fs::create_dir_all(&existing).expect("Failed to create existing dir");

            let result = generate_unique_name("myenv");
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "myenv-1");
        });
    }

    /// Multiple collisions: {name}-pyenv, {name}-1, {name}-2 존재 시 {name}-3 반환
    #[test]
    #[serial]
    fn test_generate_unique_name_multiple_collisions() {
        with_temp_scoop_home(|temp| {
            let venvs_dir = temp.path().join("virtualenvs");
            fs::create_dir_all(&venvs_dir).unwrap();

            // myenv-pyenv, myenv-1, myenv-2 생성
            fs::create_dir_all(venvs_dir.join("myenv-pyenv")).unwrap();
            fs::create_dir_all(venvs_dir.join("myenv-1")).unwrap();
            fs::create_dir_all(venvs_dir.join("myenv-2")).unwrap();

            let result = generate_unique_name("myenv");
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "myenv-3");
        });
    }

    /// Max attempts exceeded: 100개 모두 존재 시 에러 반환
    #[test]
    #[serial]
    fn test_generate_unique_name_max_attempts_exceeded() {
        with_temp_scoop_home(|temp| {
            let venvs_dir = temp.path().join("virtualenvs");
            fs::create_dir_all(&venvs_dir).unwrap();

            // myenv-pyenv 생성
            fs::create_dir_all(venvs_dir.join("myenv-pyenv")).unwrap();

            // myenv-1 ~ myenv-99 생성 (MAX_RENAME_ATTEMPTS=100, 범위는 1..100이므로 1~99)
            for i in 1..MAX_RENAME_ATTEMPTS {
                fs::create_dir_all(venvs_dir.join(format!("myenv-{}", i))).unwrap();
            }

            let result = generate_unique_name("myenv");
            assert!(result.is_err());

            let err = result.unwrap_err();
            let err_msg = err.to_string();
            assert!(
                err_msg.contains("myenv") || err_msg.contains("unique name"),
                "Error should mention the base name or unique name: {}",
                err_msg
            );
        });
    }

    /// Edge case: 빈 이름 (실제로는 validate에서 걸리지만 함수 자체 테스트)
    #[test]
    #[serial]
    fn test_generate_unique_name_empty_base() {
        with_temp_scoop_home(|_temp| {
            // 빈 이름은 "-pyenv"가 된다
            let result = generate_unique_name("");
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "-pyenv");
        });
    }

    /// Edge case: 특수 문자가 포함된 이름
    #[test]
    #[serial]
    fn test_generate_unique_name_with_special_chars() {
        with_temp_scoop_home(|_temp| {
            let result = generate_unique_name("my-env_test");
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "my-env_test-pyenv");
        });
    }

    /// ConflictResolution enum 테스트
    #[test]
    fn test_conflict_resolution_enum_variants() {
        // Debug, Clone, Copy, PartialEq, Eq 트레잇 검증
        let overwrite = ConflictResolution::Overwrite;
        let rename = ConflictResolution::Rename;
        let skip = ConflictResolution::Skip;

        // Clone 동작 확인
        let cloned = overwrite;
        assert_eq!(cloned, overwrite);

        // PartialEq 동작 확인
        assert_ne!(overwrite, rename);
        assert_ne!(rename, skip);
        assert_ne!(overwrite, skip);

        // Debug 동작 확인
        let debug_str = format!("{:?}", overwrite);
        assert!(debug_str.contains("Overwrite"));
    }

    /// MAX_RENAME_ATTEMPTS 상수 값 검증
    #[test]
    fn test_max_rename_attempts_constant() {
        assert_eq!(MAX_RENAME_ATTEMPTS, 100);
    }
}
