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
