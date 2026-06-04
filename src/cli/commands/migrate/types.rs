//! Data types for migration CLI commands
//!
//! Contains all data structures used for JSON serialization and internal state.

use std::path::PathBuf;

use serde::Serialize;

use crate::cli::MigrateSource;
use crate::core::migrate::{MigrationResult, SourceEnvironment, SourceType};

/// JSON output for migrate list command
#[derive(Debug, Serialize)]
pub struct MigrateListData {
    pub source: String,
    pub environments: Vec<SourceEnvironment>,
    pub summary: MigrateListSummary,
}

/// Summary statistics for migrate list
#[derive(Debug, Serialize)]
pub struct MigrateListSummary {
    pub total: usize,
    pub ready: usize,
    pub conflict: usize,
    pub eol: usize,
    pub corrupted: usize,
}

/// JSON output for migrate all command.
///
/// `conflicts` (new in 0.14) is *additive*: name-conflicting envs
/// without `--force` still appear in `skipped` for backward
/// compatibility, but they ALSO appear here so consumers can branch
/// on the failure class without parsing the localized reason string.
/// `summary.total == migrated.len() + failed.len() + skipped.len()`
/// holds; `conflicts.len() <= skipped.len()`.
#[derive(Debug, Serialize)]
pub struct MigrateAllData {
    pub migrated: Vec<MigrationResult>,
    pub failed: Vec<MigrateFailure>,
    pub skipped: Vec<MigrateSkipped>,
    pub conflicts: Vec<MigrationConflictDetail>,
    pub summary: MigrateAllSummary,
}

/// Failed migration info.
///
/// `source_type` and `error_code` were added in 0.14 so JSON consumers
/// can branch on the failure class (which source tool, which error
/// variant) without parsing the localized `error` message.
#[derive(Debug, Serialize)]
pub struct MigrateFailure {
    pub name: String,
    pub source_type: SourceType,
    pub error_code: &'static str,
    pub error: String,
}

/// Preflight name conflict (env exists in scoop home and `--force` not set).
///
/// Distinct from `MigrateSkipped` so consumers can count conflicts
/// separately and recommend `--force` programmatically. The same env
/// also appears in `skipped` for shape backward compatibility — see
/// the `MigrateAllData` doc comment.
#[derive(Debug, Serialize)]
pub struct MigrationConflictDetail {
    pub name: String,
    pub source_type: SourceType,
    pub existing: PathBuf,
}

/// Skipped environment info
#[derive(Debug, Serialize)]
pub struct MigrateSkipped {
    pub name: String,
    pub reason: String,
}

/// Summary for migrate all
#[derive(Debug, Serialize)]
pub struct MigrateAllSummary {
    pub total: usize,
    pub success: usize,
    pub failed: usize,
    pub skipped: usize,
}

/// Options for CLI migrate execution.
///
/// This struct consolidates the many boolean flags used by migrate commands
/// to improve readability and maintainability.
#[derive(Debug, Clone, Default)]
pub struct MigrateExecuteOptions {
    /// Preview migration without making changes
    pub dry_run: bool,
    /// Force overwrite existing environments
    pub force: bool,
    /// Skip confirmation prompts
    pub yes: bool,
    /// Output as JSON
    pub json: bool,
    /// Fail on first package error
    pub strict: bool,
    /// Delete original environment after successful migration
    pub delete_source: bool,
    /// Migrate with a different name
    pub rename: Option<String>,
    /// Auto-rename on conflict
    pub auto_rename: bool,
    /// Filter by source tool
    pub source_filter: Option<MigrateSource>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Default 트레잇 구현 검증
    /// 모든 boolean 필드는 false, Option 필드는 None이어야 한다
    #[test]
    fn test_migrate_execute_options_default_values() {
        let opts = MigrateExecuteOptions::default();

        assert!(!opts.dry_run, "dry_run should default to false");
        assert!(!opts.force, "force should default to false");
        assert!(!opts.yes, "yes should default to false");
        assert!(!opts.json, "json should default to false");
        assert!(!opts.strict, "strict should default to false");
        assert!(!opts.delete_source, "delete_source should default to false");
        assert!(opts.rename.is_none(), "rename should default to None");
        assert!(!opts.auto_rename, "auto_rename should default to false");
        assert!(
            opts.source_filter.is_none(),
            "source_filter should default to None"
        );
    }

    /// MigrateListData 구조체 테스트
    #[test]
    fn test_migrate_list_data_serialize() {
        let data = MigrateListData {
            source: "pyenv".to_string(),
            environments: vec![],
            summary: MigrateListSummary {
                total: 5,
                ready: 3,
                conflict: 1,
                eol: 1,
                corrupted: 0,
            },
        };

        let json = serde_json::to_string(&data).expect("should serialize");
        assert!(json.contains("\"source\":\"pyenv\""));
        assert!(json.contains("\"total\":5"));
        assert!(json.contains("\"ready\":3"));
    }

    /// MigrateFailure 구조체 테스트 — Inc 4 additive fields preserved.
    #[test]
    fn test_migrate_failure_serialize() {
        let failure = MigrateFailure {
            name: "broken-env".to_string(),
            source_type: SourceType::Pyenv,
            error_code: "MIGRATE_FAILED",
            error: "Python version not found".to_string(),
        };

        let json = serde_json::to_string(&failure).expect("should serialize");
        assert!(json.contains("\"name\":\"broken-env\""));
        assert!(json.contains("\"error\":\"Python version not found\""));
        // Inc 4 additive keys — JSON consumers must be able to branch
        // on these without parsing the localized message.
        assert!(json.contains("\"source_type\":\"pyenv\""));
        assert!(json.contains("\"error_code\":\"MIGRATE_FAILED\""));
    }

    /// MigrationConflictDetail 구조체 테스트 — preflight conflict surfacing.
    #[test]
    fn test_migration_conflict_detail_serialize() {
        let conflict = MigrationConflictDetail {
            name: "dup".to_string(),
            source_type: SourceType::Conda,
            existing: PathBuf::from("/home/user/.scoop/virtualenvs/dup"),
        };

        let json = serde_json::to_string(&conflict).expect("should serialize");
        assert!(json.contains("\"name\":\"dup\""));
        assert!(json.contains("\"source_type\":\"conda\""));
        assert!(json.contains("/home/user/.scoop/virtualenvs/dup"));
    }

    /// MigrateSkipped 구조체 테스트
    #[test]
    fn test_migrate_skipped_serialize() {
        let skipped = MigrateSkipped {
            name: "old-env".to_string(),
            reason: "EOL Python version".to_string(),
        };

        let json = serde_json::to_string(&skipped).expect("should serialize");
        assert!(json.contains("\"name\":\"old-env\""));
        assert!(json.contains("\"reason\":\"EOL Python version\""));
    }

    /// MigrateAllSummary 구조체 테스트
    #[test]
    fn test_migrate_all_summary_serialize() {
        let summary = MigrateAllSummary {
            total: 10,
            success: 7,
            failed: 2,
            skipped: 1,
        };

        let json = serde_json::to_string(&summary).expect("should serialize");
        assert!(json.contains("\"total\":10"));
        assert!(json.contains("\"success\":7"));
        assert!(json.contains("\"failed\":2"));
        assert!(json.contains("\"skipped\":1"));
    }
}
