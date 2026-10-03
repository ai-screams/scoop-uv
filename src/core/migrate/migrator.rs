//! Migration orchestration

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::metadata::Metadata;
use crate::error::{MigrationExitCode, Result, ScoopError};
use crate::paths;
use crate::uv::PythonInfo;
use crate::uv::UvClient;

use super::extractor::{ExtractionResult, PackageExtractor};
use super::source::{EnvironmentStatus, SourceEnvironment};

/// Result of Python version availability check
#[derive(Debug)]
pub enum PythonAvailability {
    /// Exact version is installed
    Available(PythonInfo),
    /// Compatible version available (e.g., 3.9 instead of 3.9.1)
    Compatible {
        requested: String,
        available: PythonInfo,
    },
    /// Not available, but can be installed by uv
    CanInstall { version: String },
    /// Not available at all
    Unavailable { reason: String },
}

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
    /// Fail on first package error (strict mode)
    pub strict: bool,
    /// Delete original environment after successful migration
    pub delete_source: bool,
    /// Automatically install Python if missing
    pub auto_install_python: bool,
}

/// Result of a migration operation
#[derive(Debug, Serialize, Deserialize, PartialEq)]
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
    /// Whether the source environment was deleted
    pub source_deleted: bool,
    /// Actual Python version used (may differ from requested if compatible version used)
    pub actual_python_version: String,
}

impl MigrationResult {
    /// Returns the exit code based on migration result.
    ///
    /// Returns `Success` if all packages were migrated successfully,
    /// `PartialSuccess` if some packages failed to install.
    pub fn exit_code(&self) -> MigrationExitCode {
        if self.packages_failed.is_empty() {
            MigrationExitCode::Success
        } else {
            MigrationExitCode::PartialSuccess
        }
    }
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

/// Orchestrates migration from source to scuv
pub struct Migrator {
    uv: UvClient,
    extractor: PackageExtractor,
}

/// The refusal for an end-of-life source Python without --force.
fn eol_error(version: &str) -> ScoopError {
    ScoopError::MigrationFailed {
        reason: format!("Python {version} is end-of-life. Use --force to migrate anyway."),
    }
}

impl Migrator {
    /// Creates a new migrator.
    ///
    /// # Errors
    ///
    /// Returns [`ScoopError::UvNotFound`] if uv is not installed or not in PATH.
    pub fn new() -> Result<Self> {
        Ok(Self {
            uv: UvClient::new()?,
            extractor: PackageExtractor::new(),
        })
    }

    /// Creates a migrator with a specific UvClient.
    #[cfg(test)]
    pub(crate) fn with_uv(uv: UvClient) -> Self {
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
                // The conflict is with the source's own name; under a new
                // name (--rename / --auto-rename) it no longer applies, and
                // create_target_env checks the new name itself. The status
                // records the conflict ahead of an EOL Python, so a renamed
                // env still answers to the EOL guard here.
                let renamed = options
                    .rename_to
                    .as_deref()
                    .is_some_and(|to| to != source.name);
                if options.force {
                    Ok(())
                } else if !renamed {
                    Err(ScoopError::MigrationNameConflict {
                        name: source.name.clone(),
                        existing: existing.clone(),
                    })
                } else if super::common::is_python_eol(&source.python_version) {
                    Err(eol_error(&source.python_version))
                } else {
                    Ok(())
                }
            }
            EnvironmentStatus::PythonEol { version } => {
                if options.force {
                    Ok(())
                } else {
                    Err(eol_error(version))
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

    /// Creates the target scuv environment.
    fn create_target_env(&self, name: &str, python_version: &str, force: bool) -> Result<PathBuf> {
        // Validate at the trust boundary: for `--rename` / `--auto-rename` the
        // target name comes straight from CLI args and would otherwise reach
        // the filesystem unchecked, allowing path traversal (e.g. `../../x`).
        // Every other entry point already validates; this covers the migrate path.
        crate::validate::validate_env_name(name)?;

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
    ///
    /// # Arguments
    ///
    /// * `target_path` - Path to the target virtual environment.
    /// * `packages` - Extracted packages to install.
    /// * `strict` - If true, fail immediately on first package error.
    fn install_packages(
        &self,
        target_path: &Path,
        packages: &ExtractionResult,
        strict: bool,
    ) -> Result<Vec<String>> {
        let mut failed = Vec::new();

        // Install regular packages in one batch
        let regular_specs: Vec<String> = packages
            .regular_packages()
            .iter()
            .map(|p| p.to_requirement())
            .collect();

        if !regular_specs.is_empty()
            && let Err(e) = self.uv.pip_install(target_path, &regular_specs)
        {
            // Try installing packages one by one to identify failures
            for spec in &regular_specs {
                if self
                    .uv
                    .pip_install(target_path, std::slice::from_ref(spec))
                    .is_err()
                {
                    if strict {
                        return Err(ScoopError::MigrationFailed {
                            reason: format!("Failed to install package: {}", spec),
                        });
                    }
                    failed.push(spec.clone());
                }
            }

            // If all failed, propagate the original error
            if failed.len() == regular_specs.len() {
                return Err(e);
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

    /// Deletes the source environment after successful migration.
    ///
    /// # Errors
    ///
    /// Returns an error if the source directory cannot be deleted.
    pub fn delete_source(&self, source: &SourceEnvironment) -> Result<()> {
        if !source.path.exists() {
            return Ok(()); // Already gone
        }

        fs::remove_dir_all(&source.path).map_err(|e| {
            ScoopError::Io(std::io::Error::new(
                e.kind(),
                format!(
                    "Failed to delete source at {}: {}",
                    source.path.display(),
                    e
                ),
            ))
        })
    }

    /// Writes metadata for the migrated environment.
    fn write_metadata(&self, target_path: &Path, name: &str, python_version: &str) -> Result<()> {
        let uv_version = self.uv.version().ok();
        let metadata = Metadata::new(name.to_string(), python_version.to_string(), uv_version);

        let metadata_path = target_path.join(Metadata::FILE_NAME);
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
                source_deleted: false,
                actual_python_version: source.python_version.clone(),
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
            self.install_packages(&target_path, &packages, options.strict)?
        };

        // Write metadata
        self.write_metadata(&target_path, &target_name, &source.python_version)?;

        // Success - disarm rollback
        rollback.disarm();

        // Delete source if requested
        let source_deleted = if options.delete_source {
            self.delete_source(source)?;
            true
        } else {
            false
        };

        let packages_migrated = packages.packages.len() - failed.len();

        Ok(MigrationResult {
            name: target_name,
            python_version: source.python_version.clone(),
            packages_migrated,
            packages_failed: failed,
            dry_run: false,
            path: target_path,
            source_deleted,
            actual_python_version: source.python_version.clone(),
        })
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
            size_bytes: Some(1024),
            status,
        }
    }

    /// `RollbackGuard` deletes a half-built environment when a migration
    /// fails, and `disarm` is what stops it after success. Both halves were
    /// untested, which matters more than usual here: this guard is one of the
    /// pieces in the `--force` batch race fixed in #168 — an armed guard
    /// dropping late can delete an environment another migration just built.
    #[test]
    fn rollback_guard_removes_the_directory_when_not_disarmed() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("half-built");
        fs::create_dir_all(path.join("bin")).unwrap();

        {
            let _guard = RollbackGuard::new(path.clone());
        } // dropped here, still armed

        assert!(!path.exists(), "an armed guard must clean up on drop");
    }

    #[test]
    fn rollback_guard_leaves_the_directory_once_disarmed() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("finished");
        fs::create_dir_all(path.join("bin")).unwrap();

        {
            let mut guard = RollbackGuard::new(path.clone());
            guard.disarm();
        }

        assert!(
            path.exists(),
            "a disarmed guard must leave the finished environment alone"
        );
    }

    /// `write_metadata` is what makes a migrated directory recognisable to
    /// the rest of scuv; replaced with a bare `Ok(())` the environment ends up
    /// without `.scoop-metadata.json` and `scuv list` cannot see it. The uv
    /// version lookup inside is `.ok()`, so an unusable UvClient is fine here.
    #[test]
    fn write_metadata_creates_a_readable_metadata_file() {
        let temp = tempfile::tempdir().unwrap();
        let target = temp.path().join("web");
        fs::create_dir_all(&target).unwrap();

        offline_migrator()
            .write_metadata(&target, "web", "3.12.0")
            .unwrap();

        let raw = fs::read_to_string(target.join(".scoop-metadata.json")).unwrap();
        assert!(raw.contains("\"web\""), "name should be recorded: {raw}");
        assert!(
            raw.contains("3.12.0"),
            "python version should be recorded: {raw}"
        );
    }

    /// A migrator with a UvClient that is never invoked — enough to reach
    /// the filesystem-only paths below without needing uv on PATH.
    fn offline_migrator() -> Migrator {
        Migrator::with_uv(crate::uv::UvClient::with_path(PathBuf::from(
            "/nonexistent/uv",
        )))
    }

    /// `delete_source` guards with `!source.path.exists()` and returns early
    /// when the directory is already gone. Dropping the `!` inverts it: a
    /// live source is left in place and a missing one is handed to
    /// `remove_dir_all`. Both halves are asserted here.
    #[test]
    fn delete_source_removes_existing_directory() {
        let temp = tempfile::tempdir().unwrap();
        let env_path = temp.path().join("web");
        fs::create_dir_all(env_path.join("bin")).unwrap();

        let source = SourceEnvironment {
            name: "web".to_string(),
            python_version: "3.12.0".to_string(),
            path: env_path.clone(),
            source_type: SourceType::Pyenv,
            size_bytes: None,
            status: EnvironmentStatus::Ready,
        };

        offline_migrator().delete_source(&source).unwrap();
        assert!(!env_path.exists(), "source directory should be gone");
    }

    #[test]
    fn delete_source_is_ok_when_already_gone() {
        let temp = tempfile::tempdir().unwrap();
        let source = SourceEnvironment {
            name: "web".to_string(),
            python_version: "3.12.0".to_string(),
            path: temp.path().join("never-created"),
            source_type: SourceType::Pyenv,
            size_bytes: None,
            status: EnvironmentStatus::Ready,
        };

        assert!(offline_migrator().delete_source(&source).is_ok());
    }

    #[test]
    fn create_target_env_rejects_path_traversal_name() {
        let migrator = Migrator {
            uv: UvClient::with_path(PathBuf::from("/mock/uv")),
            extractor: PackageExtractor::new(),
        };
        // Names that escape the virtualenvs dir must be rejected before any
        // filesystem access (validation runs before uv is invoked).
        for evil in ["../../etc/evil", "..", "foo/bar", "/abs", ".hidden"] {
            let result = migrator.create_target_env(evil, "3.12", false);
            assert!(
                matches!(result, Err(ScoopError::InvalidEnvName { .. })),
                "name {evil:?} should be rejected as invalid"
            );
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

    /// A name conflict is about the source's name, so migrating under a new
    /// name (--rename, --auto-rename) goes ahead; "renaming" to the same
    /// name does not, and neither does a renamed env on an EOL Python
    /// without --force. Fails if the conflict check ignores `rename_to` (the
    /// rename then always fails), accepts a rename to the same name, or lets
    /// a rename skip the EOL guard.
    #[test]
    fn validate_source_lets_a_renamed_conflict_through() {
        let migrator = Migrator {
            uv: UvClient::with_path(PathBuf::from("/mock/uv")),
            extractor: PackageExtractor::new(),
        };
        let source = mock_source(
            "test",
            EnvironmentStatus::NameConflict {
                existing: PathBuf::from("/home/u/.scuv/virtualenvs/test"),
            },
        );
        let renamed = |to: &str| MigrateOptions {
            rename_to: Some(to.to_string()),
            ..MigrateOptions::default()
        };
        assert!(
            migrator
                .validate_source(&source, &MigrateOptions::default())
                .is_err()
        );
        assert!(
            migrator
                .validate_source(&source, &renamed("test-pyenv"))
                .is_ok()
        );
        assert!(migrator.validate_source(&source, &renamed("test")).is_err());

        // The conflict hid an EOL Python: renaming must not skip that guard.
        let mut eol = source.clone();
        eol.python_version = "3.7.17".to_string();
        assert!(
            migrator
                .validate_source(&eol, &renamed("test-pyenv"))
                .is_err()
        );
        let forced = MigrateOptions {
            force: true,
            ..renamed("test-pyenv")
        };
        assert!(migrator.validate_source(&eol, &forced).is_ok());
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
