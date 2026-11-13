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
fn migrate_all_environments_dry_run_propagates_unreadable_packages() {
    // create_mock_pyenv_env's `bin/python` is an empty file that uv cannot
    // query, so reading the env's packages (`uv pip list`) fails.
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

/// `migrate all --allow-eol` plans an EOL env instead of skipping it. The
/// mock env's `bin/python` cannot be queried, so reading its packages fails
/// and the dry run then fails on that env. Fails if the
/// batch plans with `--force` alone: the EOL env is skipped and the batch
/// succeeds with nothing to migrate.
#[test]
#[serial]
fn migrate_all_environments_allow_eol_plans_eol_envs() {
    with_full_migrate_env(|_scoop, pyenv| {
        create_mock_pyenv_env(pyenv.path(), "old_env", "3.10.14");

        let output = Output::new(0, false, Colors::NONE, false);
        let opts = MigrateExecuteOptions {
            source_filter: Some(MigrateSource::Pyenv),
            dry_run: true,
            yes: true,
            allow_eol: true,
            ..Default::default()
        };

        let result = migrate_all_environments(&output, &opts);
        assert!(
            matches!(
                result,
                Err(ScoopError::MigrationBatchFailed {
                    failed_count: 1,
                    conflict_count: 0,
                })
            ),
            "{result:?}"
        );
    });
}

#[test]
#[serial]
fn migrate_all_environments_json_emits_failure_envelope() {
    // Mock env's `bin/python` cannot be queried → reading its packages
    // fails → JSON failure envelope is emitted on stdout BEFORE the Err returns.
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

/// A source env named with a reserved word cannot migrate under that name.
/// It fails the batch before any plan even when it is the only env, and
/// `--force` does not replace the scuv env that already has the name. What
/// fails this: the empty-batch branch ignoring the invalid names (the batch
/// then reports nothing to migrate and succeeds).
#[test]
#[serial]
fn migrate_all_environments_fails_a_reserved_name_without_touching_the_target() {
    with_full_migrate_env(|scoop, pyenv| {
        create_mock_pyenv_env(pyenv.path(), "gc", "3.12.0");
        let existing = scoop.path().join("virtualenvs").join("gc");
        std::fs::create_dir_all(&existing).unwrap();
        std::fs::write(existing.join("keep"), "").unwrap();

        let output = Output::new(0, true, Colors::NONE, false);
        for dry_run in [true, false] {
            let opts = MigrateExecuteOptions {
                source_filter: Some(MigrateSource::Pyenv),
                dry_run,
                force: true,
                yes: true,
                ..Default::default()
            };
            let result = migrate_all_environments(&output, &opts);
            assert!(
                matches!(
                    result,
                    Err(ScoopError::MigrationBatchFailed {
                        failed_count: 1,
                        conflict_count: 0,
                    })
                ),
                "dry_run={dry_run}: {result:?}"
            );
            assert!(existing.join("keep").exists(), "dry_run={dry_run}");
        }
    });
}

/// Declining the plan does not turn the names already reported as failed
/// into a success. What fails this: returning Ok on cancel regardless.
#[test]
#[serial]
fn migrate_all_declined_plan_still_fails_reserved_names() {
    with_full_migrate_env(|scoop, pyenv| {
        create_mock_pyenv_env(pyenv.path(), "gc", "3.12.0");
        create_mock_pyenv_env(pyenv.path(), "web", "3.12.0");

        let output = Output::new(0, true, Colors::NONE, false);
        let opts = MigrateExecuteOptions {
            source_filter: Some(MigrateSource::Pyenv),
            ..Default::default()
        };
        let mut asked = 0;
        let result = migrate_all_with(&output, &opts, |count| {
            asked += 1;
            assert_eq!(count, 1, "only `web` is planned");
            Ok(false)
        });
        assert_eq!(asked, 1);
        assert!(
            matches!(
                result,
                Err(ScoopError::MigrationBatchFailed {
                    failed_count: 1,
                    conflict_count: 0,
                })
            ),
            "{result:?}"
        );
        assert!(!scoop.path().join("virtualenvs").join("web").exists());
    });
}

/// Without invalid names a declined plan is still a clean cancel.
#[test]
#[serial]
fn migrate_all_declined_plan_is_ok_without_failures() {
    with_full_migrate_env(|_scoop, pyenv| {
        create_mock_pyenv_env(pyenv.path(), "web", "3.12.0");

        let output = Output::new(0, true, Colors::NONE, false);
        let opts = MigrateExecuteOptions {
            source_filter: Some(MigrateSource::Pyenv),
            ..Default::default()
        };
        assert!(migrate_all_with(&output, &opts, |_| Ok(false)).is_ok());
    });
}

/// What fails this: dropping the sort (the invalid name would stay last).
#[test]
fn merge_failures_keeps_the_list_sorted_by_name() {
    use crate::core::migrate::SourceType;
    let failure = |name: &str| super::super::types::MigrateFailure {
        name: name.to_string(),
        source_type: SourceType::Pyenv,
        error_code: "X",
        error: String::new(),
    };
    let mut failed = vec![failure("aaa"), failure("zzz")];
    merge_failures(&mut failed, vec![failure("gc")]);
    let names: Vec<_> = failed.iter().map(|f| f.name.as_str()).collect();
    assert_eq!(names, ["aaa", "gc", "zzz"]);
}

/// The batch question: yes and no are answers; a question that could not
/// be asked is an error naming `--yes`. What fails this: an unavailable
/// prompt read as "no" (the batch would quietly do nothing).
#[test]
fn batch_confirm_maps_each_answer() {
    use crate::cli::confirm::answering;
    assert!(answering(Some(true), || confirm(3)).unwrap());
    assert!(!answering(Some(false), || confirm(3)).unwrap());
    let err = answering(None, || confirm(3)).unwrap_err();
    assert!(
        matches!(err, ScoopError::ConfirmationUnavailable { ref flag } if flag == "--yes"),
        "{err:?}"
    );
}
