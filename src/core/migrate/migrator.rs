//! Migration orchestration

use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::core::metadata::Metadata;
use crate::error::{Result, ScoopError};
use crate::paths;
use crate::uv::UvClient;

use super::extractor::{ExtractionResult, PackageExtractor};
use super::source::{EnvironmentStatus, SourceEnvironment};

/// Options for migration
#[derive(Debug, Clone, Default)]
pub struct MigrateOptions {
    /// Skip package installation (structure only)
    pub skip_packages: bool,
    /// Force overwrite existing environments
    pub force: bool,
    /// Dry run mode (no actual changes)
    pub dry_run: bool,
    /// New name for the environment (if renaming)
    pub rename_to: Option<String>,
}

/// Result of a migration operation
#[derive(Debug, Serialize)]
pub struct MigrationResult {
    /// Name of the migrated environment
    pub name: String,
    /// Python version used
    pub python_version: String,
    /// Number of packages migrated
    pub packages_migrated: usize,
    /// Packages that failed to install
    pub packages_failed: Vec<String>,
    /// Whether this was a dry run
    pub dry_run: bool,
    /// Path to the new environment
    pub path: PathBuf,
}

/// Guard for rollback on failure
struct RollbackGuard {
    path: Option<PathBuf>,
}

impl RollbackGuard {
    fn new(path: PathBuf) -> Self {
        Self { path: Some(path) }
    }

    fn disarm(&mut self) {
        self.path = None;
    }
}

impl Drop for RollbackGuard {
    fn drop(&mut self) {
        if let Some(path) = &self.path {
            let _ = fs::remove_dir_all(path);
        }
    }
}

/// Orchestrates migration from source to scoop
pub struct Migrator {
    uv: UvClient,
    extractor: PackageExtractor,
}

impl Default for Migrator {
    fn default() -> Self {
        Self::new()
    }
}

impl Migrator {
    /// Creates a new migrator.
    pub fn new() -> Self {
        Self {
            uv: UvClient::new().expect("uv not found"),
            extractor: PackageExtractor::new(),
        }
    }

    /// Creates a migrator with a specific UvClient.
    pub fn with_uv(uv: UvClient) -> Self {
        Self {
            uv,
            extractor: PackageExtractor::new(),
        }
    }

    /// Validates that the source environment can be migrated.
    fn validate_source(&self, source: &SourceEnvironment, options: &MigrateOptions) -> Result<()> {
        match &source.status {
            EnvironmentStatus::Ready => Ok(()),
            EnvironmentStatus::NameConflict { existing } => {
                if options.force {
                    Ok(())
                } else {
                    Err(ScoopError::MigrationNameConflict {
                        name: source.name.clone(),
                        existing: existing.clone(),
                    })
                }
            }
            EnvironmentStatus::PythonEol { version } => {
                if options.force {
                    Ok(())
                } else {
                    Err(ScoopError::MigrationFailed {
                        reason: format!(
                            "Python {} is end-of-life. Use --force to migrate anyway.",
                            version
                        ),
                    })
                }
            }
            EnvironmentStatus::Corrupted { reason } => Err(ScoopError::CorruptedEnvironment {
                name: source.name.clone(),
                reason: reason.clone(),
            }),
        }
    }

    /// Extracts packages from the source environment.
    fn extract_packages(&self, source: &SourceEnvironment) -> Result<ExtractionResult> {
        self.extractor.extract(&source.path)
    }

    /// Creates the target scoop environment.
    fn create_target_env(&self, name: &str, python_version: &str, force: bool) -> Result<PathBuf> {
        let target_path = paths::virtualenv_path(name)?;

        if target_path.exists() {
            if force {
                fs::remove_dir_all(&target_path)?;
            } else {
                return Err(ScoopError::VirtualenvExists {
                    name: name.to_string(),
                });
            }
        }

        // Ensure parent directory exists
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Create the virtual environment
        self.uv.create_venv(&target_path, python_version)?;

        Ok(target_path)
    }

    /// Installs packages into the target environment.
    fn install_packages(
        &self,
        target_path: &Path,
        packages: &ExtractionResult,
    ) -> Result<Vec<String>> {
        let mut failed = Vec::new();

        // Install regular packages in one batch
        let regular_specs: Vec<String> = packages
            .regular_packages()
            .iter()
            .map(|p| p.to_requirement())
            .collect();

        if !regular_specs.is_empty() {
            if let Err(e) = self.uv.pip_install(target_path, &regular_specs) {
                // Try installing packages one by one to identify failures
                for spec in &regular_specs {
                    if self
                        .uv
                        .pip_install(target_path, std::slice::from_ref(spec))
                        .is_err()
                    {
                        failed.push(spec.clone());
                    }
                }

                // If all failed, propagate the original error
                if failed.len() == regular_specs.len() {
                    return Err(e);
                }
            }
        }

        // Editable packages need special handling - we skip them for now
        // since the source paths may not be valid in the new environment
        for editable in packages.editable_packages() {
            failed.push(format!(
                "{} (editable - skipped)",
                editable.to_requirement()
            ));
        }

        Ok(failed)
    }

    /// Writes metadata for the migrated environment.
    fn write_metadata(&self, target_path: &Path, name: &str, python_version: &str) -> Result<()> {
        let uv_version = self.uv.version().ok();
        let metadata = Metadata::new(name.to_string(), python_version.to_string(), uv_version);

        let metadata_path = target_path.join(".scoop-metadata.json");
        let content = serde_json::to_string_pretty(&metadata)?;
        fs::write(metadata_path, content)?;
        Ok(())
    }

    /// Migrates a single environment.
    ///
    /// # Errors
    ///
    /// Returns an error if migration fails.
    pub fn migrate(
        &self,
        source: &SourceEnvironment,
        options: &MigrateOptions,
    ) -> Result<MigrationResult> {
        // Determine target name
        let target_name = options.rename_to.as_ref().unwrap_or(&source.name).clone();

        // Dry run - just report what would happen
        if options.dry_run {
            let packages = self.extract_packages(source)?;
            let target_path = paths::virtualenv_path(&target_name)?;
            return Ok(MigrationResult {
                name: target_name,
                python_version: source.python_version.clone(),
                packages_migrated: packages.packages.len(),
                packages_failed: packages.failed.clone(),
                dry_run: true,
                path: target_path,
            });
        }

        // Validate source
        self.validate_source(source, options)?;

        // Extract packages from source
        let packages = if options.skip_packages {
            ExtractionResult {
                packages: Vec::new(),
                failed: Vec::new(),
                total_found: 0,
            }
        } else {
            self.extract_packages(source)?
        };

        // Create target environment
        let target_path =
            self.create_target_env(&target_name, &source.python_version, options.force)?;

        // Set up rollback guard
        let mut rollback = RollbackGuard::new(target_path.clone());

        // Install packages
        let failed = if options.skip_packages {
            Vec::new()
        } else {
            self.install_packages(&target_path, &packages)?
        };

        // Write metadata
        self.write_metadata(&target_path, &target_name, &source.python_version)?;

        // Success - disarm rollback
        rollback.disarm();

        let packages_migrated = packages.packages.len() - failed.len();

        Ok(MigrationResult {
            name: target_name,
            python_version: source.python_version.clone(),
            packages_migrated,
            packages_failed: failed,
            dry_run: false,
            path: target_path,
        })
    }

    /// Migrates multiple environments.
    ///
    /// # Errors
    ///
    /// Returns results for all environments, including failures.
    pub fn migrate_all(
        &self,
        sources: &[SourceEnvironment],
        options: &MigrateOptions,
    ) -> Vec<Result<MigrationResult>> {
        sources
            .iter()
            .map(|source| self.migrate(source, options))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::migrate::source::SourceType;

    fn mock_source(name: &str, status: EnvironmentStatus) -> SourceEnvironment {
        SourceEnvironment {
            name: name.to_string(),
            python_version: "3.12.0".to_string(),
            path: PathBuf::from("/mock/path"),
            source_type: SourceType::Pyenv,
            size_bytes: 1024,
            status,
        }
    }

    #[test]
    fn test_validate_source_ready() {
        let migrator = Migrator {
            uv: UvClient::with_path(PathBuf::from("/mock/uv")),
            extractor: PackageExtractor::new(),
        };
        let source = mock_source("test", EnvironmentStatus::Ready);
        let options = MigrateOptions::default();

        assert!(migrator.validate_source(&source, &options).is_ok());
    }

    #[test]
    fn test_validate_source_corrupted() {
        let migrator = Migrator {
            uv: UvClient::with_path(PathBuf::from("/mock/uv")),
            extractor: PackageExtractor::new(),
        };
        let source = mock_source(
            "test",
            EnvironmentStatus::Corrupted {
                reason: "broken".to_string(),
            },
        );
        let options = MigrateOptions::default();

        assert!(migrator.validate_source(&source, &options).is_err());
    }

    #[test]
    fn test_validate_source_name_conflict_with_force() {
        let migrator = Migrator {
            uv: UvClient::with_path(PathBuf::from("/mock/uv")),
            extractor: PackageExtractor::new(),
        };
        let source = mock_source(
            "test",
            EnvironmentStatus::NameConflict {
                existing: PathBuf::from("/existing"),
            },
        );
        let options = MigrateOptions {
            force: true,
            ..Default::default()
        };

        assert!(migrator.validate_source(&source, &options).is_ok());
    }

    #[test]
    fn test_validate_source_eol_without_force() {
        let migrator = Migrator {
            uv: UvClient::with_path(PathBuf::from("/mock/uv")),
            extractor: PackageExtractor::new(),
        };
        let source = mock_source(
            "test",
            EnvironmentStatus::PythonEol {
                version: "3.7.0".to_string(),
            },
        );
        let options = MigrateOptions::default();

        assert!(migrator.validate_source(&source, &options).is_err());
    }
}
