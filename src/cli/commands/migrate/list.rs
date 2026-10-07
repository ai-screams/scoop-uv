//! Environment listing display
//!
//! Displays discovered environments in human-readable or JSON format.

use rust_i18n::t;

use crate::cli::MigrateSource;
use crate::core::migrate::common::dir_size;
use crate::core::migrate::{EnvironmentStatus, SourceEnvironment, SourceType};
use crate::error::Result;
use crate::output::Output;

use super::scan::{no_envs_message, scan_all_environments, scanning_message};
use super::types::{MigrateListData, MigrateListSummary};

/// List environments available for migration.
///
/// Displays environments grouped by source type with status indicators:
/// - ✓ Ready to migrate
/// - ⚠ Has warnings (conflict, EOL)
/// - ✗ Corrupted
pub fn list_environments(
    output: &Output,
    json: bool,
    source_filter: Option<MigrateSource>,
) -> Result<()> {
    if !json {
        output.info(&scanning_message(source_filter, &rust_i18n::locale()));
    }

    let environments = with_sizes(scan_all_environments(source_filter));

    if json {
        let summary = summarize(&environments);
        output.json_success(
            "migrate list",
            MigrateListData {
                source: source_filter
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "all".to_string()),
                environments,
                summary,
            },
        );
        return Ok(());
    }

    if environments.is_empty() {
        output.info(&no_envs_message(source_filter, &rust_i18n::locale()));
        return Ok(());
    }

    output.success(&t!("migrate.found", count = environments.len()));
    println!();
    print_grouped(&environments);
    println!();
    output.info(&t!("migrate.hint_single"));
    output.info(&t!("migrate.hint_preview"));

    Ok(())
}

/// Counts the scanned environments by status.
fn summarize(environments: &[SourceEnvironment]) -> MigrateListSummary {
    let mut summary = MigrateListSummary {
        total: environments.len(),
        ready: 0,
        conflict: 0,
        eol: 0,
        corrupted: 0,
    };
    for env in environments {
        match &env.status {
            EnvironmentStatus::Ready => summary.ready += 1,
            EnvironmentStatus::NameConflict { .. } => summary.conflict += 1,
            EnvironmentStatus::PythonEol { .. } => summary.eol += 1,
            EnvironmentStatus::Corrupted { .. } => summary.corrupted += 1,
        }
    }
    summary
}

/// Prints the environments under a `[source]` header per source type (the
/// scan already sorts by source).
fn print_grouped(environments: &[SourceEnvironment]) {
    let mut current_source: Option<SourceType> = None;
    for env in environments {
        if current_source != Some(env.source_type) {
            if current_source.is_some() {
                println!();
            }
            println!("  [{}]", env.source_type);
            current_source = Some(env.source_type);
        }
        println!("{}", env_line(env));
    }
}

/// Fills each environment's on-disk size. Discovery leaves `size_bytes`
/// unset so that `migrate all` and `migrate @env` do not walk every tree;
/// `migrate list` shows the size, so it measures here.
fn with_sizes(mut environments: Vec<SourceEnvironment>) -> Vec<SourceEnvironment> {
    for env in &mut environments {
        env.size_bytes = Some(dir_size(&env.path));
    }
    environments
}

/// One environment's line: status icon, name, Python, size and a hint.
fn env_line(env: &SourceEnvironment) -> String {
    let (status_icon, status_hint) = match &env.status {
        EnvironmentStatus::Ready => ("✓", "".to_string()),
        EnvironmentStatus::NameConflict { existing } => {
            ("⚠", format!(" (conflicts with {})", existing.display()))
        }
        EnvironmentStatus::PythonEol { version } => ("⚠", format!(" (Python {} is EOL)", version)),
        EnvironmentStatus::Corrupted { reason } => ("✗", format!(" ({})", reason)),
    };

    let size_str = match env.size_bytes {
        Some(bytes) => format!("{:>8.1} MB", bytes as f64 / 1_048_576.0),
        None => "       - MB".to_string(), // Not calculated
    };
    format!(
        "    {} {:<20} Python {:<10} {}{}",
        status_icon, env.name, env.python_version, size_str, status_hint
    )
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {

    /// Fails if a status is counted in the wrong bucket or `total` stops
    /// matching the scan.
    #[test]
    fn summarize_counts_each_status() {
        use std::path::PathBuf;
        let env = |status| SourceEnvironment {
            name: "e".into(),
            python_version: "3.12".into(),
            path: PathBuf::from("/e"),
            source_type: SourceType::Pyenv,
            size_bytes: None,
            status,
        };
        let s = summarize(&[
            env(EnvironmentStatus::Ready),
            env(EnvironmentStatus::Ready),
            env(EnvironmentStatus::NameConflict {
                existing: PathBuf::from("/x"),
            }),
            env(EnvironmentStatus::PythonEol {
                version: "2.7".into(),
            }),
            env(EnvironmentStatus::Corrupted { reason: "r".into() }),
        ]);
        assert_eq!(
            (s.total, s.ready, s.conflict, s.eol, s.corrupted),
            (5, 2, 1, 1, 1)
        );
    }
    use super::*;
    use crate::core::migrate::SourceEnvironment;
    use crate::test_utils::{
        create_mock_pyenv_env, with_full_migrate_env, with_isolated_migrate_env,
    };
    use serial_test::serial;
    use std::path::PathBuf;

    // =========================================================================
    // Test Helpers
    // =========================================================================

    /// Compute summary counts from environments.
    fn compute_summary(environments: &[SourceEnvironment]) -> (usize, usize, usize, usize) {
        let mut ready = 0;
        let mut conflict = 0;
        let mut eol = 0;
        let mut corrupted = 0;

        for env in environments {
            match &env.status {
                EnvironmentStatus::Ready => ready += 1,
                EnvironmentStatus::NameConflict { .. } => conflict += 1,
                EnvironmentStatus::PythonEol { .. } => eol += 1,
                EnvironmentStatus::Corrupted { .. } => corrupted += 1,
            }
        }

        (ready, conflict, eol, corrupted)
    }

    /// Get status icon and hint for an environment.
    fn status_display(status: &EnvironmentStatus) -> (&'static str, String) {
        match status {
            EnvironmentStatus::Ready => ("✓", "".to_string()),
            EnvironmentStatus::NameConflict { existing } => {
                ("⚠", format!(" (conflicts with {})", existing.display()))
            }
            EnvironmentStatus::PythonEol { version } => {
                ("⚠", format!(" (Python {} is EOL)", version))
            }
            EnvironmentStatus::Corrupted { reason } => ("✗", format!(" ({})", reason)),
        }
    }

    /// Format size for display.
    fn format_size_display(size_bytes: Option<u64>) -> String {
        match size_bytes {
            Some(bytes) => {
                let size_mb = bytes as f64 / 1_048_576.0;
                format!("{:>8.1} MB", size_mb)
            }
            None => "       - MB".to_string(),
        }
    }

    // =========================================================================
    // compute_summary Tests
    // =========================================================================

    #[test]
    fn compute_summary_empty_list_returns_all_zeros() {
        let envs: Vec<SourceEnvironment> = vec![];
        let (ready, conflict, eol, corrupted) = compute_summary(&envs);
        assert_eq!(ready, 0);
        assert_eq!(conflict, 0);
        assert_eq!(eol, 0);
        assert_eq!(corrupted, 0);
    }

    #[test]
    fn compute_summary_counts_ready_environments() {
        let envs = vec![
            SourceEnvironment {
                name: "env1".to_string(),
                python_version: "3.12.0".to_string(),
                path: PathBuf::from("/test/env1"),
                source_type: SourceType::Pyenv,
                size_bytes: None,
                status: EnvironmentStatus::Ready,
            },
            SourceEnvironment {
                name: "env2".to_string(),
                python_version: "3.11.0".to_string(),
                path: PathBuf::from("/test/env2"),
                source_type: SourceType::Pyenv,
                size_bytes: None,
                status: EnvironmentStatus::Ready,
            },
        ];

        let (ready, conflict, eol, corrupted) = compute_summary(&envs);
        assert_eq!(ready, 2);
        assert_eq!(conflict, 0);
        assert_eq!(eol, 0);
        assert_eq!(corrupted, 0);
    }

    #[test]
    fn compute_summary_counts_all_status_types() {
        let envs = vec![
            SourceEnvironment {
                name: "ready".to_string(),
                python_version: "3.12.0".to_string(),
                path: PathBuf::from("/test/ready"),
                source_type: SourceType::Pyenv,
                size_bytes: None,
                status: EnvironmentStatus::Ready,
            },
            SourceEnvironment {
                name: "conflict".to_string(),
                python_version: "3.12.0".to_string(),
                path: PathBuf::from("/test/conflict"),
                source_type: SourceType::Pyenv,
                size_bytes: None,
                status: EnvironmentStatus::NameConflict {
                    existing: PathBuf::from("/existing"),
                },
            },
            SourceEnvironment {
                name: "eol".to_string(),
                python_version: "2.7.18".to_string(),
                path: PathBuf::from("/test/eol"),
                source_type: SourceType::Pyenv,
                size_bytes: None,
                status: EnvironmentStatus::PythonEol {
                    version: "2.7.18".to_string(),
                },
            },
            SourceEnvironment {
                name: "corrupted".to_string(),
                python_version: "3.12.0".to_string(),
                path: PathBuf::from("/test/corrupted"),
                source_type: SourceType::Pyenv,
                size_bytes: None,
                status: EnvironmentStatus::Corrupted {
                    reason: "No python binary".to_string(),
                },
            },
        ];

        let (ready, conflict, eol, corrupted) = compute_summary(&envs);
        assert_eq!(ready, 1);
        assert_eq!(conflict, 1);
        assert_eq!(eol, 1);
        assert_eq!(corrupted, 1);
    }

    // =========================================================================
    // status_display Tests
    // =========================================================================

    #[test]
    fn status_display_ready_returns_checkmark() {
        let (icon, hint) = status_display(&EnvironmentStatus::Ready);
        assert_eq!(icon, "✓");
        assert!(hint.is_empty());
    }

    #[test]
    fn status_display_conflict_returns_warning_with_path() {
        let status = EnvironmentStatus::NameConflict {
            existing: PathBuf::from("/home/user/.scoop/virtualenvs/myenv"),
        };
        let (icon, hint) = status_display(&status);
        assert_eq!(icon, "⚠");
        assert!(hint.contains("conflicts with"));
        assert!(hint.contains("myenv"));
    }

    #[test]
    fn status_display_eol_returns_warning_with_version() {
        let status = EnvironmentStatus::PythonEol {
            version: "2.7.18".to_string(),
        };
        let (icon, hint) = status_display(&status);
        assert_eq!(icon, "⚠");
        assert!(hint.contains("2.7.18"));
        assert!(hint.contains("EOL"));
    }

    #[test]
    fn status_display_corrupted_returns_x_with_reason() {
        let status = EnvironmentStatus::Corrupted {
            reason: "Python binary not found".to_string(),
        };
        let (icon, hint) = status_display(&status);
        assert_eq!(icon, "✗");
        assert!(hint.contains("Python binary not found"));
    }

    // =========================================================================
    // format_size_display Tests
    // =========================================================================

    #[test]
    fn format_size_display_none_returns_dash() {
        let result = format_size_display(None);
        assert!(result.contains("-"));
        assert!(result.contains("MB"));
    }

    #[test]
    fn format_size_display_zero_returns_zero_mb() {
        let result = format_size_display(Some(0));
        assert!(result.contains("0.0"));
        assert!(result.contains("MB"));
    }

    #[test]
    fn format_size_display_one_mb() {
        let result = format_size_display(Some(1_048_576));
        assert!(result.contains("1.0"));
        assert!(result.contains("MB"));
    }

    #[test]
    fn format_size_display_large_size() {
        let result = format_size_display(Some(524_288_000)); // 500 MB
        assert!(result.contains("500.0"));
        assert!(result.contains("MB"));
    }

    #[test]
    fn format_size_display_fractional_mb() {
        let result = format_size_display(Some(1_572_864)); // 1.5 MB
        assert!(result.contains("1.5"));
    }

    // =========================================================================
    // list_environments Integration Tests
    // =========================================================================

    #[test]
    #[serial]
    fn list_environments_empty_when_no_sources() {
        with_isolated_migrate_env(|| {
            let output = Output::new(0, false, crate::output::Colors::NONE, false);
            let result = list_environments(&output, false, None);
            assert!(result.is_ok());
        });
    }

    #[test]
    #[serial]
    fn list_environments_json_mode_empty() {
        with_isolated_migrate_env(|| {
            let output = Output::new(0, true, crate::output::Colors::NONE, true);
            let result = list_environments(&output, true, None);
            assert!(result.is_ok());
        });
    }

    #[test]
    #[serial]
    fn list_environments_with_pyenv_filter() {
        with_isolated_migrate_env(|| {
            let output = Output::new(0, false, crate::output::Colors::NONE, false);
            let result = list_environments(&output, false, Some(MigrateSource::Pyenv));
            assert!(result.is_ok());
        });
    }

    #[test]
    #[serial]
    fn list_environments_discovers_mock_pyenv() {
        with_full_migrate_env(|_scoop, pyenv| {
            // Create mock pyenv environments
            create_mock_pyenv_env(pyenv.path(), "testenv1", "3.12.0");
            create_mock_pyenv_env(pyenv.path(), "testenv2", "3.11.0");

            let output = Output::new(0, false, crate::output::Colors::NONE, false);
            let result = list_environments(&output, false, Some(MigrateSource::Pyenv));
            assert!(result.is_ok());
        });
    }

    #[test]
    #[serial]
    fn list_environments_json_with_mock_pyenv() {
        with_full_migrate_env(|_scoop, pyenv| {
            create_mock_pyenv_env(pyenv.path(), "jsontest", "3.12.0");

            let output = Output::new(0, true, crate::output::Colors::NONE, true);
            let result = list_environments(&output, true, Some(MigrateSource::Pyenv));
            assert!(result.is_ok());
        });
    }
}
