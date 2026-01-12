//! Single environment migration
//!
//! Handles migration of individual environments with status validation.

use rust_i18n::t;

use crate::core::migrate::{EnvironmentStatus, MigrateOptions, MigrationResult, Migrator};
use crate::error::{Result, ScoopError};
use crate::output::Output;

use super::conflict::{
    ConflictResolution, generate_unique_name, prompt_conflict_resolution, prompt_rename,
};
use super::scan::find_environment_by_name;
use super::types::MigrateExecuteOptions;

/// Print migration result in human-readable format.
pub fn print_migration_result(output: &Output, result: &MigrationResult, dry_run: bool) {
    if dry_run {
        output.info("");
        output.info(&t!("migrate.dry_run_header"));
        output.info(&t!(
            "migrate.to",
            path = crate::paths::abbreviate_home(&result.path)
        ));
        output.info(&t!("migrate.python", version = &result.python_version));
        output.info(&t!("migrate.packages", count = result.packages_migrated));

        if !result.packages_failed.is_empty() {
            output.warn(&t!(
                "migrate.failed_packages",
                count = result.packages_failed.len()
            ));
            for pkg in &result.packages_failed {
                output.info(&format!("    - {}", pkg));
            }
        }

        output.info("");
        output.info(&t!("migrate.dry_run_hint"));
    } else {
        output.success(&t!("migrate.success", name = &result.name));
        output.info(&t!(
            "create.path",
            path = crate::paths::abbreviate_home(&result.path)
        ));
        output.info(&t!("migrate.python", version = &result.python_version));
        output.info(&t!("migrate.packages", count = result.packages_migrated));

        if !result.packages_failed.is_empty() {
            output.warn(&t!(
                "migrate.failed_packages",
                count = result.packages_failed.len()
            ));
            for pkg in &result.packages_failed {
                output.info(&format!("    - {}", pkg));
            }
        }

        output.info("");
        output.info(&t!("migrate.activate_hint", name = &result.name));
    }
}

/// Migrate a single environment.
///
/// Handles conflict resolution, status validation, and actual migration.
pub fn migrate_environment(
    output: &Output,
    name: &str,
    opts: &MigrateExecuteOptions,
) -> Result<()> {
    let source = find_environment_by_name(name, opts.source_filter)?;

    if !opts.json {
        // Show environment info
        output.info(&format!(
            "Source: {} ({}, Python {})",
            name, source.source_type, source.python_version
        ));
        output.info(&t!(
            "migrate.source_path",
            path = crate::paths::abbreviate_home(&source.path)
        ));

        if let Some(size_bytes) = source.size_bytes {
            let size_mb = size_bytes as f64 / 1_048_576.0;
            output.info(&t!("migrate.size", size = format!("{:.1}", size_mb)));
        }
    }

    // Determine final name (may be renamed)
    let mut final_name = opts.rename.clone().unwrap_or_else(|| name.to_string());
    let mut effective_force = opts.force;

    // Check status
    match &source.status {
        EnvironmentStatus::Ready => {}
        EnvironmentStatus::NameConflict { existing } => {
            if opts.auto_rename {
                // Auto-rename: generate unique name
                final_name = generate_unique_name(name)?;
                if !opts.json {
                    output.info(&t!("migrate.auto_rename", name = &final_name));
                }
            } else if opts.rename.is_some() {
                // User provided explicit rename, check if that conflicts too
                let renamed_path = crate::paths::virtualenv_path(&final_name)?;
                if renamed_path.exists() && !opts.force {
                    return Err(ScoopError::MigrationNameConflict {
                        name: final_name.clone(),
                        existing: renamed_path,
                    });
                }
            } else if !opts.force {
                // Interactive conflict resolution (if not json and not yes)
                if !opts.json && !opts.yes {
                    let resolution = prompt_conflict_resolution(output, name, existing)?;
                    match resolution {
                        ConflictResolution::Overwrite => {
                            effective_force = true;
                            if !opts.json {
                                output.warn(&t!("migrate.will_overwrite"));
                            }
                        }
                        ConflictResolution::Rename => {
                            final_name = prompt_rename(name)?;
                            if !opts.json {
                                output.info(&t!("migrate.will_migrate_as", name = &final_name));
                            }
                        }
                        ConflictResolution::Skip => {
                            if !opts.json {
                                output.info(&t!("migrate.skipped"));
                            }
                            return Ok(());
                        }
                    }
                } else {
                    // Non-interactive mode: error out
                    if !opts.json {
                        output.warn(&t!("migrate.name_exists", name = name));
                        output.info(&t!("migrate.use_flags"));
                    }
                    return Err(ScoopError::MigrationNameConflict {
                        name: name.to_string(),
                        existing: existing.clone(),
                    });
                }
            } else if !opts.json {
                output.warn(&t!("migrate.overwriting"));
            }
        }
        EnvironmentStatus::PythonEol { version } => {
            if !opts.force {
                if !opts.json {
                    output.warn(&t!("migrate.eol_warning", version = version));
                    output.info(&t!("migrate.eol_force_hint"));
                }
                return Err(ScoopError::MigrationFailed {
                    reason: format!("Python {} is EOL", version),
                });
            }
            if !opts.json {
                output.warn(&t!("migrate.eol_proceeding", version = version));
            }
        }
        EnvironmentStatus::Corrupted { reason } => {
            if !opts.json {
                output.error(&t!("migrate.corrupted", reason = reason));
            }
            return Err(ScoopError::CorruptedEnvironment {
                name: name.to_string(),
                reason: reason.clone(),
            });
        }
    }

    // Create migrator and options
    let migrator = Migrator::new()?;
    let options = MigrateOptions {
        dry_run: opts.dry_run,
        force: effective_force,
        skip_packages: false,
        rename_to: if final_name != name {
            Some(final_name.clone())
        } else {
            None
        },
        strict: opts.strict,
        delete_source: opts.delete_source,
        auto_install_python: false,
    };

    if !opts.json {
        if opts.dry_run {
            output.info(&t!("migrate.simulating"));
        } else {
            output.info(&t!("migrate.migrating"));
        }
    }

    // Perform migration
    let result = migrator.migrate(&source, &options)?;

    // JSON output
    if opts.json {
        output.json_success("migrate", &result);
        return Ok(());
    }

    // Report results
    print_migration_result(output, &result, opts.dry_run);

    Ok(())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::migrate::MigrationResult;
    use crate::test_utils::{
        create_corrupted_pyenv_env, create_mock_pyenv_env, with_full_migrate_env,
        with_isolated_migrate_env,
    };
    use serial_test::serial;
    use std::path::PathBuf;

    // =========================================================================
    // MigrationResult JSON Serialization Tests
    // =========================================================================

    fn create_test_result(
        name: &str,
        dry_run: bool,
        packages_migrated: usize,
        packages_failed: Vec<String>,
    ) -> MigrationResult {
        MigrationResult {
            name: name.to_string(),
            python_version: "3.12.0".to_string(),
            packages_migrated,
            packages_failed,
            dry_run,
            path: PathBuf::from(format!("/home/test/.scoop/virtualenvs/{}", name)),
            source_deleted: false,
            actual_python_version: "3.12.0".to_string(),
        }
    }

    /// True roundtrip test: serialize → deserialize → compare
    /// Catches Serialize/Deserialize mismatches (e.g., serde rename typos, type incompatibilities)
    #[test]
    fn migration_result_json_roundtrip() {
        let original = create_test_result("testenv", false, 10, vec![]);

        // Serialize to JSON
        let json = serde_json::to_string(&original).unwrap();

        // Deserialize back to struct (TRUE roundtrip)
        let restored: MigrationResult = serde_json::from_str(&json).unwrap();

        // Compare original and restored
        assert_eq!(original, restored);
    }

    /// Roundtrip with failed packages - tests Vec<String> serialization
    #[test]
    fn migration_result_roundtrip_with_failed_packages() {
        let failed = vec!["broken-pkg".to_string(), "bad-dep".to_string()];
        let original = create_test_result("failenv", false, 8, failed);

        let json = serde_json::to_string(&original).unwrap();
        let restored: MigrationResult = serde_json::from_str(&json).unwrap();

        assert_eq!(original, restored);
        assert_eq!(restored.packages_failed.len(), 2);
        assert_eq!(restored.packages_failed[0], "broken-pkg");
    }

    /// Roundtrip with dry_run flag - tests boolean field
    #[test]
    fn migration_result_roundtrip_dry_run_flag() {
        let dry_original = create_test_result("dryenv", true, 5, vec![]);
        let actual_original = create_test_result("actualenv", false, 5, vec![]);

        // Dry run
        let dry_json = serde_json::to_string(&dry_original).unwrap();
        let dry_restored: MigrationResult = serde_json::from_str(&dry_json).unwrap();
        assert_eq!(dry_original, dry_restored);
        assert!(dry_restored.dry_run);

        // Actual run
        let actual_json = serde_json::to_string(&actual_original).unwrap();
        let actual_restored: MigrationResult = serde_json::from_str(&actual_json).unwrap();
        assert_eq!(actual_original, actual_restored);
        assert!(!actual_restored.dry_run);
    }

    /// API contract test: verifies all expected JSON fields are present
    /// This catches accidental field removal or renaming
    #[test]
    fn migration_result_json_api_contract() {
        let result = create_test_result("apitest", false, 10, vec![]);
        let json = serde_json::to_string(&result).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // These field names are part of the public JSON API
        // If any of these fail, it means a breaking API change occurred
        let required_fields = [
            "name",
            "python_version",
            "packages_migrated",
            "packages_failed",
            "dry_run",
            "path",
            "source_deleted",
            "actual_python_version",
        ];

        for field in required_fields {
            assert!(
                parsed.get(field).is_some(),
                "Missing required JSON field: {}",
                field
            );
        }
    }

    // =========================================================================
    // Security & Edge Case Tests
    // =========================================================================

    /// Boundary value test: packages_migrated = 0 and usize::MAX
    #[test]
    fn migration_result_roundtrip_boundary_values() {
        // Zero packages
        let zero = MigrationResult {
            name: "zero".to_string(),
            python_version: "3.12.0".to_string(),
            packages_migrated: 0,
            packages_failed: vec![],
            dry_run: false,
            path: PathBuf::from("/test/zero"),
            source_deleted: false,
            actual_python_version: "3.12.0".to_string(),
        };

        let json = serde_json::to_string(&zero).unwrap();
        let restored: MigrationResult = serde_json::from_str(&json).unwrap();
        assert_eq!(zero, restored);

        // Maximum value (usize::MAX)
        let max = MigrationResult {
            name: "max".to_string(),
            python_version: "3.12.0".to_string(),
            packages_migrated: usize::MAX,
            packages_failed: vec![],
            dry_run: false,
            path: PathBuf::from("/test/max"),
            source_deleted: false,
            actual_python_version: "3.12.0".to_string(),
        };

        let json = serde_json::to_string(&max).unwrap();
        let restored: MigrationResult = serde_json::from_str(&json).unwrap();
        assert_eq!(max, restored);
        assert_eq!(restored.packages_migrated, usize::MAX);
    }

    /// Path traversal attempt in name - verifies data integrity
    /// Note: Actual path validation should happen at a different layer
    #[test]
    fn migration_result_roundtrip_path_traversal_in_name() {
        let malicious_names = [
            "../../../etc/passwd",
            "..\\..\\..\\windows\\system32",
            "env/../../../root",
            "normal/../../attack",
        ];

        for name in malicious_names {
            let result = MigrationResult {
                name: name.to_string(),
                python_version: "3.12.0".to_string(),
                packages_migrated: 1,
                packages_failed: vec![],
                dry_run: false,
                path: PathBuf::from("/test/path"),
                source_deleted: false,
                actual_python_version: "3.12.0".to_string(),
            };

            let json = serde_json::to_string(&result).unwrap();
            let restored: MigrationResult = serde_json::from_str(&json).unwrap();

            // Data integrity: malicious name is preserved as-is (not sanitized at this layer)
            assert_eq!(result, restored, "Roundtrip failed for: {}", name);
        }
    }

    /// Large packages_failed list - verifies no performance/memory issues
    #[test]
    fn migration_result_roundtrip_many_failed_packages() {
        let many_packages: Vec<String> =
            (0..1000).map(|i| format!("failed-package-{}", i)).collect();

        let result = MigrationResult {
            name: "large".to_string(),
            python_version: "3.12.0".to_string(),
            packages_migrated: 500,
            packages_failed: many_packages.clone(),
            dry_run: false,
            path: PathBuf::from("/test/large"),
            source_deleted: false,
            actual_python_version: "3.12.0".to_string(),
        };

        let json = serde_json::to_string(&result).unwrap();
        let restored: MigrationResult = serde_json::from_str(&json).unwrap();

        assert_eq!(result, restored);
        assert_eq!(restored.packages_failed.len(), 1000);
    }

    /// Special characters in package names
    #[test]
    fn migration_result_roundtrip_special_chars_in_packages() {
        let special_packages = vec![
            "pkg-with-dash".to_string(),
            "pkg_with_underscore".to_string(),
            "pkg.with.dots".to_string(),
            "pkg[version]".to_string(),
            "pkg>=1.0.0".to_string(),
            "pkg==1.0.0; python_version>='3.8'".to_string(),
            "pkg\twith\ttabs".to_string(),
            "한글패키지".to_string(),
        ];

        let result = MigrationResult {
            name: "special".to_string(),
            python_version: "3.12.0".to_string(),
            packages_migrated: 10,
            packages_failed: special_packages.clone(),
            dry_run: false,
            path: PathBuf::from("/test/special"),
            source_deleted: false,
            actual_python_version: "3.12.0".to_string(),
        };

        let json = serde_json::to_string(&result).unwrap();
        let restored: MigrationResult = serde_json::from_str(&json).unwrap();

        assert_eq!(result, restored);
        for (orig, rest) in special_packages.iter().zip(restored.packages_failed.iter()) {
            assert_eq!(orig, rest);
        }
    }

    /// Empty strings in various fields
    #[test]
    fn migration_result_roundtrip_empty_strings() {
        let result = MigrationResult {
            name: String::new(),
            python_version: String::new(),
            packages_migrated: 0,
            packages_failed: vec![String::new(), String::new()],
            dry_run: false,
            path: PathBuf::from(""),
            source_deleted: false,
            actual_python_version: String::new(),
        };

        let json = serde_json::to_string(&result).unwrap();
        let restored: MigrationResult = serde_json::from_str(&json).unwrap();

        assert_eq!(result, restored);
        assert!(restored.name.is_empty());
        assert!(restored.python_version.is_empty());
        assert_eq!(restored.packages_failed.len(), 2);
    }

    // =========================================================================
    // migrate_environment Error Path Tests
    // =========================================================================

    #[test]
    #[serial]
    fn migrate_environment_not_found_returns_error() {
        with_isolated_migrate_env(|| {
            let output = Output::new(0, false, true, false);
            let opts = MigrateExecuteOptions::default();

            let result = migrate_environment(&output, "nonexistent_env_12345", &opts);
            assert!(result.is_err());
        });
    }

    #[test]
    #[serial]
    fn migrate_environment_with_pyenv_filter_not_found() {
        with_isolated_migrate_env(|| {
            let output = Output::new(0, false, true, false);
            let opts = MigrateExecuteOptions {
                source_filter: Some(crate::cli::MigrateSource::Pyenv),
                ..Default::default()
            };

            let result = migrate_environment(&output, "nonexistent", &opts);
            assert!(result.is_err());
        });
    }

    #[test]
    #[serial]
    fn migrate_environment_corrupted_returns_error() {
        with_full_migrate_env(|_scoop, pyenv| {
            // Create a corrupted environment (no bin/python)
            create_corrupted_pyenv_env(pyenv.path(), "corrupted_env", "3.12.0");

            let output = Output::new(0, false, true, false);
            let opts = MigrateExecuteOptions {
                source_filter: Some(crate::cli::MigrateSource::Pyenv),
                json: false,
                ..Default::default()
            };

            let result = migrate_environment(&output, "corrupted_env", &opts);
            assert!(result.is_err());

            // Check error type
            let err = result.unwrap_err();
            assert!(
                matches!(err, ScoopError::CorruptedEnvironment { .. }),
                "Expected CorruptedEnvironment error, got {:?}",
                err
            );
        });
    }

    #[test]
    #[serial]
    fn migrate_environment_json_corrupted_returns_error() {
        with_full_migrate_env(|_scoop, pyenv| {
            create_corrupted_pyenv_env(pyenv.path(), "corrupted_json", "3.12.0");

            let output = Output::new(0, false, true, true);
            let opts = MigrateExecuteOptions {
                source_filter: Some(crate::cli::MigrateSource::Pyenv),
                json: true,
                ..Default::default()
            };

            let result = migrate_environment(&output, "corrupted_json", &opts);
            assert!(result.is_err());
        });
    }

    // =========================================================================
    // MigrateExecuteOptions Tests
    // =========================================================================

    #[test]
    fn migrate_execute_options_default_values() {
        let opts = MigrateExecuteOptions::default();

        assert!(!opts.dry_run);
        assert!(!opts.force);
        assert!(!opts.yes);
        assert!(!opts.json);
        assert!(!opts.strict);
        assert!(!opts.delete_source);
        assert!(opts.rename.is_none());
        assert!(!opts.auto_rename);
        assert!(opts.source_filter.is_none());
    }

    #[test]
    fn migrate_execute_options_with_rename() {
        let opts = MigrateExecuteOptions {
            rename: Some("new_name".to_string()),
            ..Default::default()
        };

        assert_eq!(opts.rename, Some("new_name".to_string()));
    }

    #[test]
    fn migrate_execute_options_clone() {
        let opts = MigrateExecuteOptions {
            dry_run: true,
            force: true,
            rename: Some("cloned".to_string()),
            ..Default::default()
        };

        let cloned = opts.clone();
        assert_eq!(cloned.dry_run, opts.dry_run);
        assert_eq!(cloned.force, opts.force);
        assert_eq!(cloned.rename, opts.rename);
    }

    // =========================================================================
    // EOL Version Handling Tests
    // =========================================================================

    // Note: Testing actual EOL handling requires a mock environment with
    // an EOL Python version. The discovery logic marks Python 2.x as EOL.

    #[test]
    #[serial]
    fn migrate_environment_eol_without_force_fails() {
        with_full_migrate_env(|_scoop, pyenv| {
            // Create environment with Python 2.7 (EOL)
            create_mock_pyenv_env(pyenv.path(), "py27_env", "2.7.18");

            let output = Output::new(0, false, true, false);
            let opts = MigrateExecuteOptions {
                source_filter: Some(crate::cli::MigrateSource::Pyenv),
                force: false,
                ..Default::default()
            };

            let result = migrate_environment(&output, "py27_env", &opts);
            // Should fail because Python 2.7 is EOL
            assert!(result.is_err());
        });
    }
}
