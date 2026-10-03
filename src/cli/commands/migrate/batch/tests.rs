//! End-to-end tests for `migrate_all_environments`.

use super::*;
use crate::cli::MigrateSource;
use crate::output::Colors;
use crate::test_utils::{
    create_corrupted_pyenv_env, create_mock_pyenv_env, with_full_migrate_env,
    with_isolated_migrate_env,
};
use serial_test::serial;

#[test]
#[serial]
fn migrate_all_environments_empty_when_no_sources() {
    with_isolated_migrate_env(|| {
        let output = Output::new(0, false, Colors::NONE, false);
        let opts = MigrateExecuteOptions {
            yes: true,
            ..Default::default()
        };

        // No source tools detected → MigrationSourcesNotFound (exit 3).
        // Previously this returned Ok(()) silently.
        let result = migrate_all_environments(&output, &opts);
        assert!(matches!(
            result,
            Err(ScoopError::MigrationSourcesNotFound { .. })
        ));
    });
}

#[test]
#[serial]
fn migrate_all_environments_json_empty_no_sources() {
    with_isolated_migrate_env(|| {
        let output = Output::new(0, true, Colors::NONE, true);
        let opts = MigrateExecuteOptions {
            json: true,
            ..Default::default()
        };

        // Same exit-3 contract under --json (no tool present).
        let result = migrate_all_environments(&output, &opts);
        assert!(matches!(
            result,
            Err(ScoopError::MigrationSourcesNotFound { .. })
        ));
    });
}

#[test]
#[serial]
fn migrate_all_environments_empty_with_tools_present() {
    // pyenv root exists but has zero envs → Ok(()) (no exit 3).
    with_full_migrate_env(|_scoop, _pyenv| {
        let output = Output::new(0, false, Colors::NONE, false);
        let opts = MigrateExecuteOptions {
            source_filter: Some(MigrateSource::Pyenv),
            yes: true,
            ..Default::default()
        };

        let result = migrate_all_environments(&output, &opts);
        assert!(result.is_ok(), "tool present + no envs must be Ok(())");
    });
}

#[test]
#[serial]
fn migrate_all_environments_with_source_filter() {
    with_isolated_migrate_env(|| {
        let output = Output::new(0, false, Colors::NONE, false);
        let opts = MigrateExecuteOptions {
            source_filter: Some(MigrateSource::Pyenv),
            yes: true,
            ..Default::default()
        };

        // Filtering to a source tool that's also absent → exit 3 path.
        let result = migrate_all_environments(&output, &opts);
        assert!(matches!(
            result,
            Err(ScoopError::MigrationSourcesNotFound { requested: Some(_) })
        ));
    });
}

#[test]
#[serial]
fn migrate_all_environments_with_corrupted_envs() {
    with_full_migrate_env(|_scoop, pyenv| {
        create_corrupted_pyenv_env(pyenv.path(), "corrupted_batch", "3.12.0");

        let output = Output::new(0, false, Colors::NONE, false);
        let opts = MigrateExecuteOptions {
            source_filter: Some(MigrateSource::Pyenv),
            yes: true,
            ..Default::default()
        };

        // Corrupted is skipped, no migratable, no conflicts → Ok(()).
        let result = migrate_all_environments(&output, &opts);
        assert!(result.is_ok());
    });
}

#[test]
#[serial]
fn migrate_all_environments_dry_run_propagates_pip_failure() {
    // create_mock_pyenv_env builds a directory skeleton WITHOUT a real
    // pip binary, so the migrator's package-extraction step fails.
    // Pre-Inc4 this was silently swallowed (Ok regardless). Now it
    // correctly surfaces as MigrationBatchFailed so users (and CI)
    // learn the dry-run would fail in production.
    with_full_migrate_env(|_scoop, pyenv| {
        create_mock_pyenv_env(pyenv.path(), "dryrun_env", "3.12.0");

        let output = Output::new(0, false, Colors::NONE, false);
        let opts = MigrateExecuteOptions {
            source_filter: Some(MigrateSource::Pyenv),
            dry_run: true,
            yes: true,
            ..Default::default()
        };

        let result = migrate_all_environments(&output, &opts);
        assert!(matches!(
            result,
            Err(ScoopError::MigrationBatchFailed {
                failed_count: 1,
                conflict_count: 0,
            })
        ));
    });
}

#[test]
#[serial]
fn migrate_all_environments_json_emits_failure_envelope() {
    // Mock env has no pip → migrator returns extraction error → JSON
    // failure envelope is emitted on stdout BEFORE the Err returns.
    // The Quiet render policy on MigrationBatchFailed then prevents
    // main.rs from appending the generic `error:` prefix.
    with_full_migrate_env(|_scoop, pyenv| {
        create_mock_pyenv_env(pyenv.path(), "json_batch", "3.12.0");
        create_corrupted_pyenv_env(pyenv.path(), "json_corrupted", "3.11.0");

        let output = Output::new(0, true, Colors::NONE, true);
        let opts = MigrateExecuteOptions {
            source_filter: Some(MigrateSource::Pyenv),
            json: true,
            yes: true,
            ..Default::default()
        };

        let result = migrate_all_environments(&output, &opts);
        assert!(matches!(
            result,
            Err(ScoopError::MigrationBatchFailed { .. })
        ));
    });
}
