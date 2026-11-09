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
    /// Force overwrite existing environments (also allows EOL Python)
    pub force: bool,
    /// Allow an end-of-life source Python without overwriting anything
    pub allow_eol: bool,
    /// Dry run mode (no actual changes)
    pub dry_run: bool,
    /// New name for the environment (if renaming)
    pub rename_to: Option<String>,
    /// Fail on first package error (strict mode)
    pub strict: bool,
    /// Delete the original environment once every package has migrated
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

impl MigrationResult {
    /// Returns the exit code based on migration result.
    ///
    /// Returns `Success` if all packages were migrated successfully,
    /// `PartialSuccess` if some failed to install or to parse.
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

/// The refusal for an end-of-life source Python without --allow-eol or --force.
fn eol_error(version: &str) -> ScoopError {
    ScoopError::MigrationFailed {
        reason: format!("Python {version} is end-of-life. Use --allow-eol to migrate anyway."),
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
                // target_path checks the new name itself. The status
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
                } else if !options.allow_eol && super::common::is_python_eol(&source.python_version)
                {
                    Err(eol_error(&source.python_version))
                } else {
                    Ok(())
                }
            }
            EnvironmentStatus::PythonEol { version } => {
                if options.force || options.allow_eol {
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
    /// would refuse: an invalid name, or an existing env without `force`.
    /// Touches nothing, so the dry run checks the same things.
    fn target_path(&self, name: &str, force: bool) -> Result<PathBuf> {
        // Validate at the trust boundary: for `--rename` / `--auto-rename` the
        // target name comes straight from CLI args and would otherwise reach
        // the filesystem unchecked, allowing path traversal (e.g. `../../x`).
        // Every other entry point already validates; this covers the migrate path.
        crate::validate::validate_env_name(name)?;

        let target_path = paths::virtualenv_path(name)?;
        if target_path.exists() && !force {
            return Err(ScoopError::VirtualenvExists {
                name: name.to_string(),
            });
        }
        Ok(target_path)
    }

    /// Creates the target scuv environment at a path from `target_path`,
    /// replacing an existing one (`target_path` allowed it only with `force`).
    fn create_target_env(&self, target_path: &Path, python_version: &str) -> Result<()> {
        if target_path.exists() {
            fs::remove_dir_all(target_path)?;
        }

        // Ensure parent directory exists
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Create the virtual environment
        self.uv.create_venv(target_path, python_version)
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

        // The dry run checks what the real run checks, so a preview never
        // reports success for a migration the real run refuses.
        self.validate_source(source, options)?;
        let target_path = self.target_path(&target_name, options.force)?;

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
        self.create_target_env(&target_path, &source.python_version)?;

        // Set up rollback guard
        let mut rollback = RollbackGuard::new(target_path.clone());

        // Install packages
        let mut failed = if options.skip_packages {
            Vec::new()
        } else {
            self.install_packages(&target_path, &packages, options.strict)?
        };
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
    fn target_path_rejects_path_traversal_name() {
        let migrator = Migrator {
            uv: UvClient::with_path(PathBuf::from("/mock/uv")),
            extractor: PackageExtractor::new(),
        };
        // Names that escape the virtualenvs dir must be rejected before any
        // filesystem access (validation runs before uv is invoked).
        for evil in ["../../etc/evil", "..", "foo/bar", "/abs", ".hidden"] {
            let result = migrator.target_path(evil, false);
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
        // 3.10 is the newest EOL minor, 3.11 the oldest supported one.
        let mut eol = source.clone();
        eol.python_version = "3.10.14".to_string();
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
        let mut supported = source.clone();
        supported.python_version = "3.11.0".to_string();
        assert!(
            migrator
                .validate_source(&supported, &renamed("test-pyenv"))
                .is_ok()
        );
    }

    /// `allow_eol` admits an EOL source, including one a conflict hid behind
    /// a rename, but never an unrenamed conflict. Fails if `allow_eol` is
    /// ignored in either EOL branch, or starts overriding a name conflict.
    #[test]
    fn validate_source_allow_eol_admits_eol_but_not_conflicts() {
        let migrator = Migrator {
            uv: UvClient::with_path(PathBuf::from("/mock/uv")),
            extractor: PackageExtractor::new(),
        };
        let allow = MigrateOptions {
            allow_eol: true,
            ..MigrateOptions::default()
        };
        let eol = mock_source(
            "test",
            EnvironmentStatus::PythonEol {
                version: "3.10.14".to_string(),
            },
        );
        assert!(migrator.validate_source(&eol, &allow).is_ok());

        let mut conflict = mock_source(
            "test",
            EnvironmentStatus::NameConflict {
                existing: PathBuf::from("/home/u/.scuv/virtualenvs/test"),
            },
        );
        conflict.python_version = "3.10.14".to_string();
        assert!(matches!(
            migrator.validate_source(&conflict, &allow),
            Err(ScoopError::MigrationNameConflict { .. })
        ));
        let renamed = MigrateOptions {
            rename_to: Some("test-pyenv".to_string()),
            ..allow
        };
        assert!(migrator.validate_source(&conflict, &renamed).is_ok());
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

    /// `migrate()` against the fake uv: `SCUV_HOME` is a fresh directory,
    /// the source is a directory whose `bin/pip` prints `freeze`, and the
    /// migrator calls the fake by path.
    #[cfg(unix)]
    mod with_fake_uv {
        use super::*;
        use crate::test_utils::{FakeUv, env_guard};
        use serial_test::serial;

        struct Fixture {
            uv: FakeUv,
            home: tempfile::TempDir,
            source: SourceEnvironment,
        }

        impl Fixture {
            fn migrator(&self) -> Migrator {
                Migrator::with_uv(UvClient::with_path(self.uv.exe()))
            }
            fn target(&self, name: &str) -> PathBuf {
                self.home.path().join("virtualenvs").join(name)
            }
        }

        fn run(
            freeze: &str,
            status: EnvironmentStatus,
            extra: &[(&'static str, Option<&str>)],
            f: impl FnOnce(&Fixture),
        ) {
            use std::os::unix::fs::PermissionsExt;
            let uv = FakeUv::new(&[]);
            let home = tempfile::TempDir::new().unwrap();
            let src = home.path().join("pyenv-web");
            fs::create_dir_all(src.join("bin")).unwrap();
            let pip = src.join("bin").join("pip");
            fs::write(&pip, format!("#!/bin/sh\nprintf '{freeze}'\n")).unwrap();
            fs::set_permissions(&pip, fs::Permissions::from_mode(0o755)).unwrap();
            let home_str = home.path().display().to_string();
            let mut vars: Vec<(&'static str, Option<&str>)> =
                vec![("SCUV_HOME", Some(home_str.as_str()))];
            vars.extend_from_slice(extra);
            let _env = env_guard(&vars);
            let source = SourceEnvironment {
                name: "web".to_string(),
                python_version: "3.12.0".to_string(),
                path: src,
                source_type: SourceType::Pyenv,
                size_bytes: None,
                status,
            };
            f(&Fixture { uv, home, source });
        }

        fn dry_run() -> MigrateOptions {
            MigrateOptions {
                dry_run: true,
                ..Default::default()
            }
        }

        /// The dry run refuses what the real run refuses.
        #[test]
        #[serial]
        fn dry_run_refuses_a_corrupted_source() {
            let status = EnvironmentStatus::Corrupted {
                reason: "broken".to_string(),
            };
            run("six==1.16.0\n", status, &[], |fx| {
                let err = fx.migrator().migrate(&fx.source, &dry_run()).unwrap_err();
                assert!(
                    matches!(err, ScoopError::CorruptedEnvironment { .. }),
                    "{err}"
                );
            });
        }

        #[test]
        #[serial]
        fn dry_run_refuses_an_eol_python_without_allow_eol() {
            let status = EnvironmentStatus::PythonEol {
                version: "3.8.0".to_string(),
            };
            run("six==1.16.0\n", status, &[], |fx| {
                let err = fx.migrator().migrate(&fx.source, &dry_run()).unwrap_err();
                assert!(matches!(err, ScoopError::MigrationFailed { .. }), "{err}");
            });
        }

        #[test]
        #[serial]
        fn dry_run_refuses_an_existing_target() {
            run("six==1.16.0\n", EnvironmentStatus::Ready, &[], |fx| {
                fs::create_dir_all(fx.target("web")).unwrap();
                let err = fx.migrator().migrate(&fx.source, &dry_run()).unwrap_err();
                assert!(matches!(err, ScoopError::VirtualenvExists { .. }), "{err}");
            });
        }

        #[test]
        #[serial]
        fn dry_run_refuses_an_invalid_new_name() {
            run("six==1.16.0\n", EnvironmentStatus::Ready, &[], |fx| {
                let options = MigrateOptions {
                    rename_to: Some("../x".to_string()),
                    ..dry_run()
                };
                let err = fx.migrator().migrate(&fx.source, &options).unwrap_err();
                assert!(matches!(err, ScoopError::InvalidEnvName { .. }), "{err}");
            });
        }

        /// A freeze line that does not parse is reported by the real run as
        /// it is by the dry run, without counting against migrated packages.
        #[test]
        #[serial]
        fn real_run_reports_unparsed_freeze_lines() {
            run(
                "six==1.16.0\\nnot-a-pin\\n",
                EnvironmentStatus::Ready,
                &[],
                |fx| {
                    let preview = fx.migrator().migrate(&fx.source, &dry_run()).unwrap();
                    let result = fx
                        .migrator()
                        .migrate(&fx.source, &MigrateOptions::default())
                        .unwrap();
                    assert_eq!(result.packages_failed, ["not-a-pin"]);
                    assert_eq!(result.packages_failed, preview.packages_failed);
                    assert_eq!(result.packages_migrated, 1);
                },
            );
        }

        fn delete_source() -> MigrateOptions {
            MigrateOptions {
                delete_source: true,
                ..Default::default()
            }
        }

        #[test]
        #[serial]
        fn delete_source_removes_a_fully_migrated_source() {
            run("six==1.16.0\\n", EnvironmentStatus::Ready, &[], |fx| {
                let result = fx.migrator().migrate(&fx.source, &delete_source()).unwrap();
                assert!(result.source_deleted);
                assert!(!fx.source.path.exists());
            });
        }

        /// `--delete-source` keeps a source whose packages did not all move:
        /// an unparsed freeze line, or a package that failed to install.
        #[test]
        #[serial]
        fn delete_source_keeps_a_source_with_an_unparsed_line() {
            run(
                "six==1.16.0\\nnot-a-pin\\n",
                EnvironmentStatus::Ready,
                &[],
                |fx| {
                    let result = fx.migrator().migrate(&fx.source, &delete_source()).unwrap();
                    assert!(!result.source_deleted);
                    assert!(fx.source.path.exists());
                },
            );
        }

        #[test]
        #[serial]
        fn delete_source_keeps_a_source_with_a_failed_install() {
            let fails = [("FAKE_UV_PIP_INSTALL_FAILS", Some("bad==1.0"))];
            run(
                "six==1.16.0\\nbad==1.0\\n",
                EnvironmentStatus::Ready,
                &fails,
                |fx| {
                    let result = fx.migrator().migrate(&fx.source, &delete_source()).unwrap();
                    assert_eq!(result.packages_failed, ["bad==1.0"]);
                    // Two found, one failed: `-` here, not `+`.
                    assert_eq!(result.packages_migrated, 1);
                    assert!(!result.source_deleted);
                    assert!(fx.source.path.exists());
                },
            );
        }

        /// `actual_python_version` is what the created env's pyvenv.cfg
        /// records (current uv writes the minor for its minor links), and
        /// the source's version when the file has none.
        #[test]
        #[serial]
        fn actual_python_version_comes_from_the_target_pyvenv_cfg() {
            let minor = [("FAKE_UV_VENV_VERSION_INFO", Some("3.12"))];
            run("six==1.16.0\\n", EnvironmentStatus::Ready, &minor, |fx| {
                let result = fx
                    .migrator()
                    .migrate(&fx.source, &MigrateOptions::default())
                    .unwrap();
                assert_eq!(result.python_version, "3.12.0");
                assert_eq!(result.actual_python_version, "3.12");
            });
        }

        #[test]
        #[serial]
        fn actual_python_version_falls_back_to_the_source() {
            run("six==1.16.0\\n", EnvironmentStatus::Ready, &[], |fx| {
                let result = fx
                    .migrator()
                    .migrate(&fx.source, &MigrateOptions::default())
                    .unwrap();
                assert_eq!(result.actual_python_version, "3.12.0");
            });
        }
    }
}
