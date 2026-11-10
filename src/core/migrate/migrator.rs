//! Migration orchestration

use std::fs;
use std::path::{Path, PathBuf};

use rust_i18n::t;
use serde::{Deserialize, Serialize};

use crate::core::metadata::Metadata;
use crate::error::{Result, ScoopError};
use crate::paths;
use crate::uv::UvClient;

use super::extractor::{ExtractionResult, PackageExtractor};
use super::source::{EnvironmentStatus, SourceEnvironment};

/// Options for migration
#[derive(Debug, Clone, Default)]
pub struct MigrateOptions {
    /// Replace a scuv env that already has the target name. Says nothing
    /// about the source's Python: an end-of-life one also needs `allow_eol`.
    pub overwrite: bool,
    /// Allow an end-of-life source Python
    pub allow_eol: bool,
    /// Dry run mode (no actual changes)
    pub dry_run: bool,
    /// New name for the environment (if renaming)
    pub rename_to: Option<String>,
    /// Fail on first package error (strict mode)
    pub strict: bool,
    /// Delete the original environment once every package has migrated
    pub delete_source: bool,
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
    /// Packages that failed to install, and `pip freeze` lines that did
    /// not parse (those were never installed either)
    pub packages_failed: Vec<String>,
    /// Whether this was a dry run
    pub dry_run: bool,
    /// Path to the new environment
    pub path: PathBuf,
    /// Whether the source environment was deleted
    pub source_deleted: bool,
    /// The version the new env's pyvenv.cfg records (`3.12` for uv's minor
    /// links); the source's version for a dry run or when pyvenv.cfg has none
    pub actual_python_version: String,
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

/// The refusal for an end-of-life source Python without --allow-eol or --force.
fn eol_error(version: &str) -> ScoopError {
    ScoopError::MigrationFailed {
        reason: t!("migrate.eol_refused_hint", version = version).to_string(),
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
    pub(crate) fn validate_source(
        &self,
        source: &SourceEnvironment,
        options: &MigrateOptions,
    ) -> Result<()> {
        match &source.status {
            EnvironmentStatus::Ready => Ok(()),
            EnvironmentStatus::NameConflict { existing } => {
                // The conflict is with the source's own name; under a new
                // name (--rename / --auto-rename) it no longer applies, and
                // target_path checks the new name itself. The status
                // records the conflict ahead of an EOL Python, so a renamed
                // env still answers to the EOL guard here.
                let renamed = options
                    .rename_to
                    .as_deref()
                    .is_some_and(|to| to != source.name);
                // Overwriting or renaming settles the conflict, not the
                // Python: the EOL guard still applies either way.
                if !options.overwrite && !renamed {
                    Err(ScoopError::MigrationNameConflict {
                        name: source.name.clone(),
                        existing: existing.clone(),
                    })
                } else if !options.allow_eol && super::common::is_python_eol(&source.python_version)
                {
                    Err(eol_error(&source.python_version))
                } else {
                    Ok(())
                }
            }
            EnvironmentStatus::PythonEol { version } => {
                if options.allow_eol {
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

    /// Resolves the target env path, refusing what `create_target_env`
    /// would refuse: an invalid name, or an existing env without `overwrite`.
    /// Touches nothing, so the dry run checks the same things.
    fn target_path(&self, name: &str, overwrite: bool) -> Result<PathBuf> {
        // Validate at the trust boundary: for `--rename` / `--auto-rename` the
        // target name comes straight from CLI args and would otherwise reach
        // the filesystem unchecked, allowing path traversal (e.g. `../../x`).
        // Every other entry point already validates; this covers the migrate path.
        crate::validate::validate_env_name(name)?;

        let target_path = paths::virtualenv_path(name)?;
        if target_path.exists() && !overwrite {
            return Err(ScoopError::VirtualenvExists {
                name: name.to_string(),
            });
        }
        Ok(target_path)
    }

    /// Creates the target scuv environment at a path from `target_path`.
    ///
    /// Checks `overwrite` again before replacing an existing env: the target
    /// may have appeared since `target_path` looked (package extraction runs
    /// in between), and only `overwrite` may delete it.
    fn create_target_env(
        &self,
        name: &str,
        target_path: &Path,
        python_version: &str,
        overwrite: bool,
    ) -> Result<()> {
        if target_path.exists() {
            if !overwrite {
                return Err(ScoopError::VirtualenvExists {
                    name: name.to_string(),
                });
            }
            fs::remove_dir_all(target_path)?;
        }
        crate::core::create_venv_dir(&self.uv, target_path, python_version)
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
                            reason: t!("migrate.package_install_failed", spec = spec).to_string(),
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
        // Atomic, like every other metadata write.
        crate::core::write_metadata_file(target_path, &metadata)
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

        // The dry run checks what the real run checks, so a preview never
        // reports success for a migration the real run refuses.
        self.validate_source(source, options)?;
        let target_path = self.target_path(&target_name, options.overwrite)?;

        // Extract packages from source
        let packages = self.extract_packages(source)?;

        if options.dry_run {
            return Ok(MigrationResult {
                name: target_name,
                python_version: source.python_version.clone(),
                packages_migrated: packages.packages.len(),
                packages_failed: packages.failed,
                dry_run: true,
                path: target_path,
                source_deleted: false,
                actual_python_version: source.python_version.clone(),
            });
        }

        // Create target environment
        self.create_target_env(
            &target_name,
            &target_path,
            &source.python_version,
            options.overwrite,
        )?;

        // Set up rollback guard
        let mut rollback = RollbackGuard::new(target_path.clone());

        // Install packages
        let mut failed = self.install_packages(&target_path, &packages, options.strict)?;
        let packages_migrated = packages.packages.len() - failed.len();
        // Freeze lines that did not parse were never installed either.
        failed.extend(packages.failed);

        // Write metadata
        self.write_metadata(&target_path, &target_name, &source.python_version)?;

        // Success - disarm rollback
        rollback.disarm();

        // `--delete-source` removes only a source whose packages all moved:
        // anything in `failed` exists nowhere else.
        let source_deleted = if options.delete_source && failed.is_empty() {
            self.delete_source(source)?;
            true
        } else {
            false
        };

        // What the new env links to, as its pyvenv.cfg records it: current
        // uv writes `3.12` for its minor links and the full version otherwise.
        let actual_python_version = crate::core::parse_pyvenv_version(&target_path)
            .unwrap_or_else(|| source.python_version.clone());

        Ok(MigrationResult {
            name: target_name,
            python_version: source.python_version.clone(),
            packages_migrated,
            packages_failed: failed,
            dry_run: false,
            path: target_path,
            source_deleted,
            actual_python_version,
        })
    }
}

#[cfg(test)]
mod tests;
