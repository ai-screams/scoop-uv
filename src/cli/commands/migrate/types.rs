//! Data types for migration CLI commands
//!
//! Contains all data structures used for JSON serialization and internal state.

use serde::Serialize;

use crate::cli::MigrateSource;
use crate::core::migrate::{MigrationResult, SourceEnvironment};

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

/// JSON output for migrate all command
#[derive(Debug, Serialize)]
pub struct MigrateAllData {
    pub migrated: Vec<MigrationResult>,
    pub failed: Vec<MigrateFailure>,
    pub skipped: Vec<MigrateSkipped>,
    pub summary: MigrateAllSummary,
}

/// Failed migration info
#[derive(Debug, Serialize)]
pub struct MigrateFailure {
    pub name: String,
    pub error: String,
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

    /// Clone 트레잇 동작 검증
    #[test]
    fn test_migrate_execute_options_clone() {
        let original = MigrateExecuteOptions {
            dry_run: true,
            force: true,
            yes: false,
            json: true,
            strict: false,
            delete_source: true,
            rename: Some("new-name".to_string()),
            auto_rename: true,
            source_filter: Some(MigrateSource::Pyenv),
        };

        let cloned = original.clone();

        assert_eq!(cloned.dry_run, original.dry_run);
        assert_eq!(cloned.force, original.force);
        assert_eq!(cloned.yes, original.yes);
        assert_eq!(cloned.json, original.json);
        assert_eq!(cloned.strict, original.strict);
        assert_eq!(cloned.delete_source, original.delete_source);
        assert_eq!(cloned.rename, original.rename);
        assert_eq!(cloned.auto_rename, original.auto_rename);
    }

    /// Debug 트레잇 동작 검증
    #[test]
    fn test_migrate_execute_options_debug() {
        let opts = MigrateExecuteOptions {
            dry_run: true,
            rename: Some("test-env".to_string()),
            ..Default::default()
        };

        let debug_str = format!("{:?}", opts);

        assert!(debug_str.contains("dry_run: true"));
        assert!(debug_str.contains("rename: Some(\"test-env\")"));
        assert!(debug_str.contains("MigrateExecuteOptions"));
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

    /// MigrateFailure 구조체 테스트
    #[test]
    fn test_migrate_failure_serialize() {
        let failure = MigrateFailure {
            name: "broken-env".to_string(),
            error: "Python version not found".to_string(),
        };

        let json = serde_json::to_string(&failure).expect("should serialize");
        assert!(json.contains("\"name\":\"broken-env\""));
        assert!(json.contains("\"error\":\"Python version not found\""));
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
