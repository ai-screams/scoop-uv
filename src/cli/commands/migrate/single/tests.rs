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
    let many_packages: Vec<String> = (0..1000).map(|i| format!("failed-package-{}", i)).collect();

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
        let output = Output::new(0, false, crate::output::Colors::NONE, false);
        let opts = MigrateExecuteOptions::default();

        let result = migrate_environment(&output, "nonexistent_env_12345", &opts);
        assert!(result.is_err());
    });
}

#[test]
#[serial]
fn migrate_environment_with_pyenv_filter_not_found() {
    with_isolated_migrate_env(|| {
        let output = Output::new(0, false, crate::output::Colors::NONE, false);
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

        let output = Output::new(0, false, crate::output::Colors::NONE, false);
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

        let output = Output::new(0, false, crate::output::Colors::NONE, true);
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

        let output = Output::new(0, false, crate::output::Colors::NONE, false);
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

// =========================================================================
// resolve_name_conflict: the non-interactive branches, in priority order
// =========================================================================

fn conflict_opts() -> MigrateExecuteOptions {
    MigrateExecuteOptions {
        yes: true,
        ..Default::default()
    }
}

fn start(name: &str, force: bool) -> Target {
    Target {
        name: name.to_string(),
        force,
    }
}

/// Fails if `--auto-rename` stops picking a fresh name.
#[test]
#[serial]
fn name_conflict_auto_rename_picks_a_free_name() {
    crate::test_utils::with_temp_scoop_home(|_| {
        let opts = MigrateExecuteOptions {
            auto_rename: true,
            ..conflict_opts()
        };
        let out = quiet_output();
        let t = resolve_name_conflict(&out, "web", Path::new("/x"), &opts, start("web", false))
            .unwrap()
            .expect("auto-rename proceeds");
        assert_ne!(t.name, "web");
        assert!(!t.force);
    });
}

/// `--rename` to a name that is taken fails unless `--force`; to a free
/// one it proceeds. Fails if the renamed-path check is dropped.
#[test]
#[serial]
fn name_conflict_rename_must_itself_be_free() {
    crate::test_utils::with_temp_scoop_home(|home| {
        std::fs::create_dir_all(home.path().join("virtualenvs").join("taken")).unwrap();
        let out = quiet_output();
        let opts = MigrateExecuteOptions {
            rename: Some("taken".into()),
            ..conflict_opts()
        };
        let err = resolve_name_conflict(&out, "web", Path::new("/x"), &opts, start("taken", false));
        assert!(
            matches!(err, Err(ScoopError::MigrationNameConflict { ref name, .. }) if name == "taken")
        );

        let opts = MigrateExecuteOptions {
            rename: Some("free".into()),
            ..conflict_opts()
        };
        let t = resolve_name_conflict(&out, "web", Path::new("/x"), &opts, start("free", false))
            .unwrap()
            .unwrap();
        assert_eq!(t.name, "free");
    });
}

/// `--force` overwrites; without it a non-interactive run fails.
/// Fails if the `--force` branch or the json/yes guard is reordered.
#[test]
fn name_conflict_force_overwrites_else_noninteractive_fails() {
    let out = quiet_output();
    let forced = MigrateExecuteOptions {
        force: true,
        ..conflict_opts()
    };
    let t = resolve_name_conflict(&out, "web", Path::new("/x"), &forced, start("web", true))
        .unwrap()
        .unwrap();
    assert_eq!((t.name.as_str(), t.force), ("web", true));

    for opts in [
        conflict_opts(),
        MigrateExecuteOptions {
            json: true,
            ..Default::default()
        },
    ] {
        let err = resolve_name_conflict(&out, "web", Path::new("/x"), &opts, start("web", false));
        assert!(
            matches!(err, Err(ScoopError::MigrationNameConflict { ref name, .. }) if name == "web")
        );
    }
}

fn quiet_output() -> Output {
    Output::new(0, true, crate::output::Colors::NONE, false)
}
