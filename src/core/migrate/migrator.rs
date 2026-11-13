//! Migration orchestration
//!
//! A migration reads the source's packages with their origins
//! ([`read_installed_packages_with`], the reader clone and export use),
//! plans them ([`prepare_install_with`]), decides what it may delete
//! ([`super::deletion`]), and only then creates, installs, verifies and,
//! under `--delete-source`, removes the source.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use rust_i18n::t;
use serde::{Deserialize, Serialize};

use crate::core::metadata::Metadata;
use crate::core::packages::{
    InstallPlan, InstalledPackage, Origin, OriginIdentity, canonical_name, prepare_install_with,
    read_installed_packages_with,
};
use crate::error::{Result, ScoopError};
use crate::paths;
use crate::uv::UvClient;

use super::deletion::{
    Deletion, DeletionKind, Survivor, check_deletions, interpreter_paths, overlaps, would_be_within,
};
use super::source::{EnvironmentStatus, SourceEnvironment, SourceType};

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
    /// Fail on the first package error instead of migrating the rest
    pub strict: bool,
    /// Delete the original environment once every package has migrated
    pub delete_source: bool,
}

/// A package migrate deliberately does not install.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExcludedPackage {
    pub name: String,
    pub version: String,
    /// Why: `installer` for pip, which scuv envs do not carry.
    pub reason: String,
}

/// Result of a migration operation
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct MigrationResult {
    /// Name of the migrated environment
    pub name: String,
    /// Python version used
    pub python_version: String,
    /// Packages verified in the new env as they were in the source (for a
    /// dry run: packages planned)
    pub packages_migrated: usize,
    /// Packages that did not migrate, each with what went wrong
    pub packages_failed: Vec<String>,
    /// Packages left out on purpose (pip)
    #[serde(default)]
    pub packages_excluded: Vec<ExcludedPackage>,
    /// Packages conda installed, put back from the registry instead
    #[serde(default)]
    pub packages_from_conda: Vec<String>,
    /// Packages the new env has that the source did not (`name==version`):
    /// dependencies the install resolved
    #[serde(default)]
    pub packages_added: Vec<String>,
    /// Whether this was a dry run
    pub dry_run: bool,
    /// Path to the new environment
    pub path: PathBuf,
    /// Whether the source environment was deleted
    pub source_deleted: bool,
    /// Why `--delete-source` kept the source, when it did
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_kept_reason: Option<String>,
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

/// One environment, read, planned and checked, before anything changed.
#[derive(Debug)]
pub(crate) struct Prepared {
    pub(crate) source: SourceEnvironment,
    /// The source's path, absolute.
    pub(crate) source_path: PathBuf,
    pub(crate) target_name: String,
    /// The target's path, absolute.
    pub(crate) target_path: PathBuf,
    /// The target exists and will be replaced.
    pub(crate) replace: bool,
    /// What gets installed: after excluding pip and converting conda's.
    pub(crate) packages: Vec<InstalledPackage>,
    pub(crate) plan: InstallPlan,
    pub(crate) excluded: Vec<ExcludedPackage>,
    pub(crate) from_conda: Vec<String>,
    /// `--delete-source` asked for, on a source that may be deleted at all.
    pub(crate) delete_source: bool,
    /// Why the source will be kept, once that is known.
    pub(crate) kept_reason: Option<String>,
}

/// Orchestrates migration from source to scuv
pub struct Migrator {
    uv: UvClient,
}

/// The refusal for an end-of-life source Python without --allow-eol or --force.
fn eol_error(version: &str) -> ScoopError {
    ScoopError::MigrationFailed {
        reason: t!("migrate.eol_refused_hint", version = version).to_string(),
    }
}

/// Whether a version (`3.7.17`, `2.7`) is older than 3.8, the first
/// Python whose package origins can be read. An unparsable version is not
/// (the reader then reports what it finds).
fn below_origin_floor(version: &str) -> bool {
    let mut parts = version.split('.').map(str::parse::<u32>);
    match (parts.next(), parts.next()) {
        (Some(Ok(major)), Some(Ok(minor))) => (major, minor) < (3, 8),
        _ => false,
    }
}

/// `path` made absolute against the working directory, keeping `..` and
/// symlinks (unlike `canonicalize`), so the deletion checks walk it.
fn absolute(path: &Path) -> Result<PathBuf> {
    Ok(std::path::absolute(path)?)
}

/// An origin as verification compares it: [`Origin::identity`] with
/// archive hashes left out (the plan passes only the strongest, and uv
/// enforced it).
fn verified_identity(origin: Option<&Origin>) -> Option<OriginIdentity> {
    origin.map(|o| match o.identity() {
        OriginIdentity::Archive {
            url, subdirectory, ..
        } => OriginIdentity::Archive {
            url,
            subdirectory,
            hashes: BTreeMap::new(),
        },
        other => other,
    })
}

/// How a package shows in `packages_failed`: `name==version`, or
/// `name (<origin>)` for a direct one.
fn label(package: &InstalledPackage) -> String {
    match &package.origin {
        None => format!("{}=={}", package.name, package.version),
        Some(origin) => format!("{} ({})", package.name, origin.describe()),
    }
}

/// What must still work after deleting or replacing anything in `envs`:
/// each source and its interpreter (going with that source only), each
/// target's parent, the env directory, and every local package source.
pub(crate) fn survivors(envs: &[&Prepared], gone_with: &[usize]) -> Result<Vec<Survivor>> {
    let mut out = vec![Survivor {
        path: absolute(&paths::virtualenvs_dir()?)?,
        what: t!("migrate.survivor_env_dir").to_string(),
        gone_with_source_of: None,
    }];
    for (env, &index) in envs.iter().zip(gone_with) {
        let name = &env.source.name;
        out.push(Survivor {
            path: env.source_path.clone(),
            what: t!("migrate.survivor_source", name = name).to_string(),
            gone_with_source_of: Some(index),
        });
        for path in interpreter_paths(&env.source_path) {
            out.push(Survivor {
                path,
                what: t!("migrate.survivor_source_python", name = name).to_string(),
                gone_with_source_of: Some(index),
            });
        }
        if let Some(parent) = env.target_path.parent() {
            out.push(Survivor {
                path: parent.to_path_buf(),
                what: t!("migrate.survivor_target_dir", name = &env.target_name).to_string(),
                gone_with_source_of: None,
            });
        }
        for source in env.plan.local_sources() {
            out.push(Survivor {
                path: source.path.clone(),
                what: t!(
                    "migrate.survivor_package",
                    package = &source.package,
                    name = name
                )
                .to_string(),
                gone_with_source_of: None,
            });
        }
    }
    Ok(out)
}

/// The deletions the prepared envs would make, each owned by its index.
pub(crate) fn deletions(envs: &[&Prepared], owners: &[usize]) -> Vec<Deletion> {
    let mut out = Vec::new();
    for (env, &owner) in envs.iter().zip(owners) {
        if env.replace {
            out.push(Deletion {
                path: env.target_path.clone(),
                owner,
                kind: DeletionKind::Target,
            });
        }
        if env.delete_source && env.kept_reason.is_none() {
            out.push(Deletion {
                path: env.source_path.clone(),
                owner,
                kind: DeletionKind::Source,
            });
        }
    }
    out
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
        })
    }

    /// Creates a migrator with a specific UvClient.
    #[cfg(test)]
    pub(crate) fn with_uv(uv: UvClient) -> Self {
        Self { uv }
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

    /// Reads, plans and checks one environment without changing anything:
    /// the source is valid, its Python can report origins, the source and
    /// target do not overlap, every package can be put back (local sources
    /// exist and are projects, none lies in a target to be replaced,
    /// hashes can be enforced). A failure here fails the environment.
    ///
    /// # Errors
    ///
    /// As [`Migrator::validate_source`], [`read_installed_packages_with`]
    /// and [`prepare_install_with`]; [`ScoopError::MigrationFailed`] when
    /// the source and target overlap.
    pub(crate) fn prepare(
        &self,
        source: &SourceEnvironment,
        options: &MigrateOptions,
    ) -> Result<Prepared> {
        let target_name = options.rename_to.as_ref().unwrap_or(&source.name).clone();
        self.validate_source(source, options)?;
        let source_path = absolute(&source.path)?;
        // Checked before running uv: uv may not run against a very old
        // Python at all, and the reason is known from discovery.
        if below_origin_floor(&source.python_version) {
            return Err(ScoopError::PackageOriginsUnreadable {
                env: source_path.display().to_string(),
                reason: t!("packages.python_too_old", version = &source.python_version).to_string(),
            });
        }
        let target_path = absolute(&self.target_path(&target_name, options.overwrite)?)?;
        let replace = target_path.exists();
        let overlap = if replace {
            overlaps(&source_path, &target_path)?
        } else {
            would_be_within(&target_path, &source_path)?
        };
        if overlap {
            return Err(ScoopError::MigrationFailed {
                reason: t!(
                    "migrate.paths_overlap",
                    source = source_path.display(),
                    target = target_path.display()
                )
                .to_string(),
            });
        }

        let mut excluded = Vec::new();
        let mut from_conda = Vec::new();
        let mut packages = Vec::new();
        for mut package in read_installed_packages_with(&self.uv, &source_path)? {
            if canonical_name(&package.name) == "pip" {
                excluded.push(ExcludedPackage {
                    name: package.name,
                    version: package.version,
                    reason: "installer".to_string(),
                });
                continue;
            }
            // conda's builds record a direct_url.json pointing at the
            // build directory; the registry release is what can be put back.
            if package.installer.as_deref() == Some("conda") {
                package.origin = None;
                from_conda.push(package.name.clone());
            }
            packages.push(package);
        }

        let roots: Vec<&Path> = if replace {
            vec![target_path.as_path()]
        } else {
            Vec::new()
        };
        let plan = prepare_install_with(&self.uv, &packages, &source.name, &roots)?;

        let conda = source.source_type == SourceType::Conda;
        let kept_reason =
            (options.delete_source && conda).then(|| t!("migrate.kept_conda").to_string());
        Ok(Prepared {
            source: source.clone(),
            source_path,
            target_name,
            target_path,
            replace,
            packages,
            plan,
            excluded,
            from_conda,
            delete_source: options.delete_source,
            kept_reason,
        })
    }

    /// Clears the way for the target: an existing one is removed only
    /// under `overwrite` (it may have appeared since [`Migrator::prepare`]).
    /// Nothing is created yet, so a refusal here leaves that env alone.
    fn clear_target(&self, env: &Prepared, overwrite: bool) -> Result<()> {
        if env.target_path.exists() {
            if !overwrite {
                return Err(ScoopError::VirtualenvExists {
                    name: env.target_name.clone(),
                });
            }
            fs::remove_dir_all(&env.target_path)?;
        }
        Ok(())
    }

    /// Removes the target and creates it again, empty.
    fn recreate_target(&self, env: &Prepared) -> Result<()> {
        fs::remove_dir_all(&env.target_path)?;
        crate::core::create_venv_dir(&self.uv, &env.target_path, &env.source.python_version)
    }

    /// `uv pip install --no-deps <plan>`.
    fn install_no_deps(&self, env: &Prepared, plan: &InstallPlan) -> Result<()> {
        let mut args = vec!["--no-deps".to_string()];
        args.extend(plan.args().iter().cloned());
        self.uv.pip_install(&env.target_path, &args)
    }

    /// Installs the plan and returns the packages that did not install.
    ///
    /// One call first, with dependencies (as clone does). If it fails:
    /// under `strict` that error is returned; otherwise the target is
    /// recreated (a failed call can leave files: uv writes wheels in
    /// parallel without rollback), each package is tried alone with
    /// `--no-deps` to find the failures, and the target is recreated once
    /// more and gets every other package in one `--no-deps` call, so it
    /// ends with exactly those and no residue.
    fn install(&self, env: &Prepared, strict: bool) -> Result<Vec<InstalledPackage>> {
        if env.plan.packages() == 0 {
            return Ok(Vec::new());
        }
        let Err(first) = self.uv.pip_install(&env.target_path, env.plan.args()) else {
            return Ok(Vec::new());
        };
        if strict {
            return Err(first);
        }
        self.recreate_target(env)?;
        let from = &env.source.name;
        let mut failed = Vec::new();
        let mut good = Vec::new();
        for package in &env.packages {
            let single = InstallPlan::build(std::slice::from_ref(package), from)?;
            if self.install_no_deps(env, &single).is_ok() {
                good.push(package.clone());
            } else {
                failed.push(package.clone());
            }
        }
        if good.is_empty() {
            return Err(first);
        }
        self.recreate_target(env)?;
        self.install_no_deps(env, &InstallPlan::build(&good, from)?)?;
        Ok(failed)
    }

    /// Compares the new env with the plan, by name, version and origin.
    /// Returns (verified count, failures, added packages).
    fn verify(
        &self,
        env: &Prepared,
        failed: &[InstalledPackage],
    ) -> Result<(usize, Vec<String>, Vec<String>)> {
        let installed = read_installed_packages_with(&self.uv, &env.target_path)?;
        let by_name: BTreeMap<String, &InstalledPackage> = installed
            .iter()
            .map(|p| (canonical_name(&p.name), p))
            .collect();
        let failed_names: Vec<String> = failed.iter().map(|p| canonical_name(&p.name)).collect();
        let mut verified = 0;
        let mut failures: Vec<String> = failed
            .iter()
            .map(|p| t!("migrate.failed_install", package = label(p)).to_string())
            .collect();
        for planned in &env.packages {
            let key = canonical_name(&planned.name);
            if failed_names.contains(&key) {
                continue;
            }
            let reason = match by_name.get(&key) {
                None => Some(t!("migrate.failed_missing", package = label(planned)).to_string()),
                Some(got) if got.version != planned.version => Some(
                    t!(
                        "migrate.failed_version",
                        package = label(planned),
                        version = &got.version
                    )
                    .to_string(),
                ),
                Some(got)
                    if verified_identity(got.origin.as_ref())
                        != verified_identity(planned.origin.as_ref()) =>
                {
                    let from = got
                        .origin
                        .as_ref()
                        .map_or_else(|| t!("migrate.from_registry").to_string(), Origin::describe);
                    Some(
                        t!(
                            "migrate.failed_origin",
                            package = label(planned),
                            origin = from
                        )
                        .to_string(),
                    )
                }
                Some(_) => None,
            };
            match reason {
                Some(reason) => failures.push(reason),
                None => verified += 1,
            }
        }
        let planned: Vec<String> = env
            .packages
            .iter()
            .map(|p| canonical_name(&p.name))
            .collect();
        let added = installed
            .iter()
            .filter(|p| !planned.contains(&canonical_name(&p.name)))
            .map(|p| format!("{}=={}", p.name, p.version))
            .collect();
        Ok((verified, failures, added))
    }

    /// Deletes the source environment.
    ///
    /// # Errors
    ///
    /// Returns an error if the source directory cannot be deleted.
    pub fn delete_source(&self, source: &SourceEnvironment) -> Result<()> {
        let path = absolute(&source.path)?;
        if !path.exists() {
            return Ok(()); // Already gone
        }
        fs::remove_dir_all(&path).map_err(|e| {
            ScoopError::Io(std::io::Error::new(
                e.kind(),
                format!("Failed to delete source at {}: {}", path.display(), e),
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

    /// The result of a dry run of `env`: what would be planned.
    pub(crate) fn dry_run_result(env: &Prepared) -> MigrationResult {
        MigrationResult {
            name: env.target_name.clone(),
            python_version: env.source.python_version.clone(),
            packages_migrated: env.plan.packages(),
            packages_failed: Vec::new(),
            packages_excluded: env.excluded.clone(),
            packages_from_conda: env.from_conda.clone(),
            packages_added: Vec::new(),
            dry_run: true,
            path: env.target_path.clone(),
            source_deleted: false,
            source_kept_reason: env.kept_reason.clone(),
            actual_python_version: env.source.python_version.clone(),
        }
    }

    /// Prepares every environment of a batch before any runs, then decides
    /// the batch's deletions together. `others` are envs migrate found but
    /// will not migrate (skipped, conflicting, an invalid name): when the
    /// batch may delete or replace anything, their packages are read too,
    /// and their sources, Pythons and local package sources must survive.
    ///
    /// If any env, selected or not, cannot be read, its local sources are
    /// unknown and nothing in the batch is deleted: sources are kept, an
    /// env that would replace a target fails, and that holds while the
    /// batch runs (a target that appears later is not replaced either). A
    /// target replacement another env depends on fails its env; a source
    /// deletion one depends on is kept.
    ///
    /// # Errors
    ///
    /// Only setting up the checks (the env directory cannot be resolved);
    /// per-env failures are in [`BatchPlan::failed`].
    pub(crate) fn plan_batch(
        &self,
        sources: &[&SourceEnvironment],
        others: &[&SourceEnvironment],
        options: &MigrateOptions,
    ) -> Result<BatchPlan> {
        let mut ready = Vec::new();
        let mut failed = Vec::new();
        for source in sources {
            match self.prepare(source, options) {
                Ok(env) => ready.push(env),
                Err(e) => failed.push(((*source).clone(), e)),
            }
        }
        let mut extra = Vec::new();
        let mut unknown = !failed.is_empty();
        if options.delete_source || options.overwrite {
            for other in others {
                match self.observe(other) {
                    Ok(survivors) => extra.extend(survivors),
                    Err(_) => unknown = true,
                }
            }
        }
        if unknown {
            let (replacing, rest): (Vec<Prepared>, Vec<Prepared>) =
                ready.into_iter().partition(|env| env.replace);
            for env in replacing {
                let reason = t!(
                    "migrate.replace_unplanned",
                    target = env.target_path.display()
                )
                .to_string();
                failed.push((env.source, ScoopError::MigrationFailed { reason }));
            }
            ready = rest;
            for env in &mut ready {
                if env.delete_source && env.kept_reason.is_none() {
                    env.kept_reason = Some(t!("migrate.kept_unplanned").to_string());
                }
            }
        }
        // The blocked envs are among the survivors here, so a deletion
        // their sources depend on is already kept; removing them from
        // `ready` afterwards changes no decision.
        let mut blocked = decide_batch_deletions(&mut ready, &extra)?;
        blocked.sort_by_key(|(i, _)| std::cmp::Reverse(*i));
        for (i, error) in blocked {
            let env = ready.remove(i);
            failed.push((env.source, error));
        }
        Ok(BatchPlan {
            ready,
            failed,
            extra,
            may_replace: !unknown,
        })
    }

    /// What an env migrate will not migrate leaves that must survive the
    /// batch: its source, its Python, and its local package sources.
    ///
    /// # Errors
    ///
    /// Its packages cannot be read or planned.
    fn observe(&self, source: &SourceEnvironment) -> Result<Vec<Survivor>> {
        let path = absolute(&source.path)?;
        let name = &source.name;
        let mut out = vec![Survivor {
            path: path.clone(),
            what: t!("migrate.survivor_source", name = name).to_string(),
            gone_with_source_of: None,
        }];
        for python in interpreter_paths(&path) {
            out.push(Survivor {
                path: python,
                what: t!("migrate.survivor_source_python", name = name).to_string(),
                gone_with_source_of: None,
            });
        }
        let packages = read_installed_packages_with(&self.uv, &path)?;
        for local in InstallPlan::build(&packages, name)?.local_sources() {
            out.push(Survivor {
                path: local.path.clone(),
                what: t!(
                    "migrate.survivor_package",
                    package = &local.package,
                    name = name
                )
                .to_string(),
                gone_with_source_of: None,
            });
        }
        Ok(out)
    }

    /// Creates, installs, verifies and records `env` (prepared and checked).
    /// The source is not deleted here: see [`Migrator::finish_deletion`].
    ///
    /// # Errors
    ///
    /// Creation, installation (all packages failing, or any under
    /// `strict`), verification under `strict`, or writing metadata. The new
    /// target is removed on any error; the source is untouched.
    pub(crate) fn execute(
        &self,
        env: &Prepared,
        options: &MigrateOptions,
    ) -> Result<MigrationResult> {
        self.clear_target(env, options.overwrite)?;
        // Armed before creating: `uv venv` can fail after making the
        // directory (a write that fails part way), and that half-made env
        // must not be left for the next run to trip over.
        let mut rollback = RollbackGuard::new(env.target_path.clone());
        crate::core::create_venv_dir(&self.uv, &env.target_path, &env.source.python_version)?;

        let failed = self.install(env, options.strict)?;
        let (verified, failures, added) = self.verify(env, &failed)?;
        if options.strict && !failures.is_empty() {
            return Err(ScoopError::MigrationFailed {
                reason: t!("migrate.verify_failed", packages = failures.join(", ")).to_string(),
            });
        }

        self.write_metadata(
            &env.target_path,
            &env.target_name,
            &env.source.python_version,
        )?;
        rollback.disarm();

        // What the new env links to, as its pyvenv.cfg records it: current
        // uv writes `3.12` for its minor links and the full version otherwise.
        let actual_python_version = crate::core::parse_pyvenv_version(&env.target_path)
            .unwrap_or_else(|| env.source.python_version.clone());
        let kept_reason = match &env.kept_reason {
            Some(reason) => Some(reason.clone()),
            None if env.delete_source && !failures.is_empty() => {
                Some(t!("migrate.kept_failed").to_string())
            }
            None => None,
        };
        Ok(MigrationResult {
            name: env.target_name.clone(),
            python_version: env.source.python_version.clone(),
            packages_migrated: verified,
            packages_failed: failures,
            packages_excluded: env.excluded.clone(),
            packages_from_conda: env.from_conda.clone(),
            packages_added: added,
            dry_run: false,
            path: env.target_path.clone(),
            source_deleted: false,
            source_kept_reason: kept_reason,
            actual_python_version,
        })
    }

    /// Deletes the sources of `done` (envs that migrated under
    /// `--delete-source` with nothing failed and no reason to keep them),
    /// unless a survivor of any env in `all` reaches one; the created
    /// targets' interpreters are survivors now too. Updates each result.
    pub(crate) fn finish_deletion(
        &self,
        all: &[&Prepared],
        extra: &[Survivor],
        done: &mut [(usize, &mut MigrationResult)],
    ) -> Result<()> {
        let indices: Vec<usize> = (0..all.len()).collect();
        let mut survivors = survivors(all, &indices)?;
        survivors.extend_from_slice(extra);
        for (i, result) in done.iter() {
            for path in interpreter_paths(&result.path) {
                survivors.push(Survivor {
                    path,
                    what: t!("migrate.survivor_new_python", name = &all[*i].target_name)
                        .to_string(),
                    gone_with_source_of: None,
                });
            }
        }
        let pending: Vec<(usize, Deletion)> = done
            .iter()
            .filter(|(i, result)| all[*i].delete_source && result.source_kept_reason.is_none())
            .map(|(i, _)| {
                (
                    *i,
                    Deletion {
                        path: all[*i].source_path.clone(),
                        owner: *i,
                        kind: DeletionKind::Source,
                    },
                )
            })
            .collect();
        let only: Vec<Deletion> = pending.iter().map(|(_, d)| d.clone()).collect();
        let blocked = check_deletions(&only, &survivors);
        for (k, (i, _)) in pending.iter().enumerate() {
            let result = done
                .iter_mut()
                .find(|(j, _)| j == i)
                .map(|(_, r)| r)
                .expect("pending comes from done");
            if let Some(b) = blocked.iter().find(|b| b.deletion == k) {
                result.source_kept_reason =
                    Some(t!("migrate.kept_depends", what = &b.because).to_string());
                continue;
            }
            self.delete_source(&all[*i].source)?;
            result.source_deleted = true;
        }
        Ok(())
    }

    /// Runs a planned batch one environment at a time, in plan order. Before
    /// each target replacement the deletion check runs again, now with the
    /// interpreter of every target created so far (an earlier env may have
    /// had uv put its Python inside a directory a later env replaces).
    /// `on_start` and `on_done` report progress. Returns each migrated
    /// env's index into `plan.ready` with its result, and the failures.
    ///
    /// # Errors
    ///
    /// Only setting up the checks; per-env failures are returned.
    pub(crate) fn execute_batch(
        &self,
        plan: &BatchPlan,
        options: &MigrateOptions,
        mut on_start: impl FnMut(&Prepared),
        mut on_done: impl FnMut(&Prepared, std::result::Result<&MigrationResult, &ScoopError>),
    ) -> Result<BatchOutcome> {
        let all: Vec<&Prepared> = plan.ready.iter().collect();
        let indices: Vec<usize> = (0..all.len()).collect();
        let mut survivors = survivors(&all, &indices)?;
        survivors.extend_from_slice(&plan.extra);
        let mut done = Vec::new();
        let mut failed = Vec::new();
        for (i, env) in plan.ready.iter().enumerate() {
            on_start(env);
            let outcome = if options.dry_run {
                Ok(Self::dry_run_result(env))
            } else {
                self.replace_check(env, i, &survivors, plan.may_replace)
                    .and_then(|()| self.execute(env, options))
            };
            match outcome {
                Ok(result) => {
                    on_done(env, Ok(&result));
                    for path in interpreter_paths(&result.path) {
                        survivors.push(Survivor {
                            path,
                            what: t!("migrate.survivor_new_python", name = &env.target_name)
                                .to_string(),
                            gone_with_source_of: None,
                        });
                    }
                    done.push((i, result));
                }
                Err(e) => {
                    on_done(env, Err(&e));
                    failed.push((env.source.clone(), e));
                }
            }
        }
        Ok((done, failed))
    }

    /// The deletion check for `env`'s target replacement against
    /// `survivors`, right before it happens. Whether there is a target to
    /// replace is asked of the filesystem now, not of the plan: an earlier
    /// env may have created it since (uv putting a Python there).
    fn replace_check(
        &self,
        env: &Prepared,
        index: usize,
        survivors: &[Survivor],
        may_replace: bool,
    ) -> Result<()> {
        if !env.target_path.exists() {
            return Ok(());
        }
        if !may_replace {
            return Err(ScoopError::MigrationFailed {
                reason: t!(
                    "migrate.replace_unplanned",
                    target = env.target_path.display()
                )
                .to_string(),
            });
        }
        let deletion = Deletion {
            path: env.target_path.clone(),
            owner: index,
            kind: DeletionKind::Target,
        };
        match check_deletions(std::slice::from_ref(&deletion), survivors).first() {
            None => Ok(()),
            Some(b) => Err(ScoopError::MigrationFailed {
                reason: t!(
                    "migrate.replace_blocked",
                    target = env.target_path.display(),
                    what = &b.because
                )
                .to_string(),
            }),
        }
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
        let mut env = self.prepare(source, options)?;
        decide_deletions(std::slice::from_mut(&mut env))?;
        if options.dry_run {
            return Ok(Self::dry_run_result(&env));
        }
        self.replace_check(&env, 0, &survivors(&[&env], &[0])?, true)?;
        let mut result = self.execute(&env, options)?;
        // Deletes only under --delete-source with nothing kept.
        self.finish_deletion(&[&env], &[], &mut [(0, &mut result)])?;
        Ok(result)
    }
}

/// What [`Migrator::execute_batch`] returns: each migrated env's index into
/// [`BatchPlan::ready`] with its result, and the envs that failed.
pub(crate) type BatchOutcome = (
    Vec<(usize, MigrationResult)>,
    Vec<(SourceEnvironment, ScoopError)>,
);

/// A batch, prepared and checked: what runs, what already failed, what
/// the envs it does not migrate need to survive, and whether it may replace
/// anything at all.
#[derive(Debug)]
pub(crate) struct BatchPlan {
    pub(crate) ready: Vec<Prepared>,
    pub(crate) failed: Vec<(SourceEnvironment, ScoopError)>,
    pub(crate) extra: Vec<Survivor>,
    /// False once any env could not be read: its local sources are
    /// unknown, so no target is replaced, including one that appears
    /// while the batch runs.
    pub(crate) may_replace: bool,
}

/// Decides, before anything runs, which deletions the prepared envs may
/// make: a target replacement something depends on fails its env
/// (returned as an error for that index); a source deletion something
/// depends on is turned into "kept" with the reason.
///
/// # Errors
///
/// [`ScoopError::MigrationFailed`] naming the first env whose target
/// replacement is blocked (single migrate); the batch uses
/// [`decide_batch_deletions`] to keep going.
pub(crate) fn decide_deletions(envs: &mut [Prepared]) -> Result<()> {
    let blocked = decide_batch_deletions(envs, &[])?;
    match blocked.into_iter().next() {
        Some((_, error)) => Err(error),
        None => Ok(()),
    }
}

/// [`decide_deletions`] over a batch: returns the indices whose target
/// replacement is blocked, with the error each fails with.
pub(crate) fn decide_batch_deletions(
    envs: &mut [Prepared],
    extra: &[Survivor],
) -> Result<Vec<(usize, ScoopError)>> {
    let refs: Vec<&Prepared> = envs.iter().collect();
    let indices: Vec<usize> = (0..envs.len()).collect();
    let mut all_survivors = survivors(&refs, &indices)?;
    all_survivors.extend_from_slice(extra);
    let planned = deletions(&refs, &indices);
    let blocked = check_deletions(&planned, &all_survivors);
    let mut failed = Vec::new();
    for b in blocked {
        let deletion = &planned[b.deletion];
        match deletion.kind {
            DeletionKind::Target => failed.push((
                deletion.owner,
                ScoopError::MigrationFailed {
                    reason: t!(
                        "migrate.replace_blocked",
                        target = deletion.path.display(),
                        what = &b.because
                    )
                    .to_string(),
                },
            )),
            DeletionKind::Source => {
                envs[deletion.owner].kept_reason =
                    Some(t!("migrate.kept_depends", what = &b.because).to_string());
            }
        }
    }
    Ok(failed)
}

#[cfg(test)]
mod tests;
