//! Error types for scuv.
//!
//! The [`ScoopError`] enum + its `From` derives live here; per-concern
//! `impl` blocks are split across submodules to keep each file under
//! ~200 LOC of code:
//!
//! | Submodule          | Responsibility                                          |
//! |--------------------|---------------------------------------------------------|
//! | [`display`]        | i18n rendering ([`ScoopError::message_in`] + `Display`) |
//! | [`code`]           | stable JSON error codes ([`ScoopError::code`])          |
//! | [`suggestion`]     | locale-aware fix hints ([`ScoopError::suggestion_in`])  |
//! | [`migrate`]        | [`MigrationExitCode`] + per-variant exit mapping        |
//!
//! All public API stays at `crate::error::ScoopError::*` regardless of
//! which submodule defines the impl block.

use std::path::PathBuf;

use thiserror::Error;

mod code;
mod display;
mod exit;
mod migrate;
mod suggestion;

pub use exit::ErrorRenderPolicy;
pub use migrate::MigrationExitCode;

/// Result type alias using [`ScoopError`].
pub type Result<T> = std::result::Result<T, ScoopError>;

/// Main error type for scuv
#[derive(Error, Debug)]
pub enum ScoopError {
    /// Virtual environment not found
    VirtualenvNotFound { name: String },

    /// Virtual environment already exists
    VirtualenvExists { name: String },

    /// Invalid environment name
    InvalidEnvName { name: String, reason: String },

    /// Invalid Python version
    InvalidPythonVersion { version: String },

    /// uv not found
    UvNotFound,

    /// uv command failed
    UvCommandFailed { command: String, message: String },

    /// Path error
    PathError(String),

    /// Home directory not found
    HomeNotFound,

    /// IO error
    Io(#[from] std::io::Error),

    /// JSON error
    Json(#[from] serde_json::Error),

    /// Version file not found
    VersionFileNotFound { path: PathBuf },

    /// Shell not supported
    UnsupportedShell { shell: String },

    /// Python version not installed
    PythonNotInstalled { version: String },

    /// Python installation failed
    PythonInstallFailed { version: String, message: String },

    /// Python uninstallation failed
    PythonUninstallFailed { version: String, message: String },

    /// No Python versions available
    NoPythonVersions { pattern: String },

    /// Invalid argument combination
    InvalidArgument { message: String },

    /// pyenv not found
    PyenvNotFound,

    /// pyenv environment not found
    PyenvEnvNotFound { name: String },

    /// virtualenvwrapper environment not found
    VenvWrapperEnvNotFound { name: String },

    /// conda environment not found
    CondaEnvNotFound { name: String },

    /// Corrupted environment
    CorruptedEnvironment { name: String, reason: String },

    /// Package extraction failed
    PackageExtractionFailed { reason: String },

    /// Migration failed
    MigrationFailed { reason: String },

    /// Name conflict with existing scuv environment
    MigrationNameConflict { name: String, existing: PathBuf },

    /// Invalid Python path (not found, not executable, not a Python binary)
    InvalidPythonPath { path: PathBuf, reason: String },

    /// Cascade uninstall aborted by user
    CascadeAborted,

    /// `scuv self update` failed (search, install, or post-install verify).
    SelfUpdateFailed { message: String },

    /// No environment is currently active and none was specified.
    NoActiveEnvironment,

    /// Executable not found within an environment's bin directory.
    ExecutableNotFound { exe: String, env: String },

    /// `.scuv.toml` could not be located walking up from `start_dir`.
    ManifestNotFound { start_dir: PathBuf },

    /// Export file failed to parse / load (invalid JSON or schema mismatch).
    InvalidExportFile { path: PathBuf, reason: String },

    /// Export file's `scoop_export_version` is not one this binary supports.
    UnsupportedExportVersion { version: String, supported: String },

    /// `scuv verify --strict` exit signal: at least one env has a failing
    /// check. The report itself was already rendered; this just carries the
    /// non-zero exit semantic without leaking `std::process::exit` into
    /// library code (which would skip destructors and stdout flush).
    VerifyFailed { issues: usize },

    /// `paths::virtualenv_site_packages` could not locate the
    /// `site-packages` directory inside a virtualenv root. The fallback
    /// chain (pyvenv.cfg → glob → sysconfig subprocess) exhausted with
    /// no usable result, so the env is likely malformed.
    SitePackagesNotFound { venv: String },

    /// `scuv migrate all`/`migrate @env` ran but no supported source
    /// tool (pyenv, virtualenvwrapper, conda) was detected on the
    /// system. Carries the requested filter (or `None` for "any") so
    /// the rendered message can be specific.
    ///
    /// Distinct from "tools present but produced no envs" — that path
    /// stays `Ok(())`. Exits 3 (`SourceError`) so CI can detect missing
    /// source-tool setup without conflating with operational failures.
    MigrationSourcesNotFound { requested: Option<String> },

    /// `scuv migrate all` finished with at least one per-env failure
    /// or one preflight name conflict (without `--force`). The batch
    /// code has already rendered the full summary (human or JSON)
    /// before returning this Err, so render policy is `Quiet`.
    ///
    /// Carries counts only — the structured per-env detail lives in
    /// the already-emitted summary, not in the error variant.
    MigrationBatchFailed {
        failed_count: usize,
        conflict_count: usize,
    },

    /// `scuv diff --strict` exit signal: the two envs differ in at
    /// least one observable way (Python version, packages, or
    /// metadata). The diff command has already rendered its report
    /// (human table or JSON envelope) before returning this Err, so
    /// render policy is `Quiet`.
    ///
    /// Carries both env names so the error message is self-rendering
    /// without needing an external context.
    DiffMismatch {
        env_a: String,
        env_b: String,
        differences: usize,
    },
}

#[cfg(test)]
mod tests;
