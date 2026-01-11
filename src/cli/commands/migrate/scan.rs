//! Environment scanning and discovery
//!
//! Discovers environments from pyenv, virtualenvwrapper, and conda sources.

use crate::cli::MigrateSource;
use crate::core::migrate::{
    CondaDiscovery, EnvironmentSource, PyenvDiscovery, SourceEnvironment, SourceType,
    VenvWrapperDiscovery,
};
use crate::error::{Result, ScoopError};

/// Scan environments from all available sources or a specific source.
///
/// Results are sorted by source type (pyenv, virtualenvwrapper, conda),
/// then alphabetically by name.
///
/// # Examples
///
/// ```ignore
/// use scoop_uv::cli::commands::migrate::scan::scan_all_environments;
/// use scoop_uv::cli::MigrateSource;
///
/// // Scan all sources (returns empty if none installed)
/// let all_envs = scan_all_environments(None);
/// println!("Found {} environments", all_envs.len());
///
/// // Scan only pyenv
/// let pyenv_only = scan_all_environments(Some(MigrateSource::Pyenv));
/// for env in pyenv_only {
///     println!("{}: Python {}", env.name, env.python_version);
/// }
/// ```
pub fn scan_all_environments(source_filter: Option<MigrateSource>) -> Vec<SourceEnvironment> {
    let mut all_envs = Vec::new();

    // Scan sources based on filter
    let scan_pyenv = source_filter.is_none() || source_filter == Some(MigrateSource::Pyenv);
    let scan_venv =
        source_filter.is_none() || source_filter == Some(MigrateSource::Virtualenvwrapper);
    let scan_conda = source_filter.is_none() || source_filter == Some(MigrateSource::Conda);

    if scan_pyenv {
        if let Some(discovery) = PyenvDiscovery::default_root() {
            if let Ok(envs) = discovery.scan_environments() {
                all_envs.extend(envs);
            }
        }
    }

    if scan_venv {
        if let Some(discovery) = VenvWrapperDiscovery::default_root() {
            if let Ok(envs) = discovery.scan_environments() {
                all_envs.extend(envs);
            }
        }
    }

    if scan_conda {
        if let Some(discovery) = CondaDiscovery::default_roots() {
            if let Ok(envs) = discovery.scan_environments() {
                all_envs.extend(envs);
            }
        }
    }

    // Sort by source type, then by name
    all_envs.sort_by(|a, b| {
        let source_order = |s: &SourceType| match s {
            SourceType::Pyenv => 0,
            SourceType::VirtualenvWrapper => 1,
            SourceType::Conda => 2,
        };
        source_order(&a.source_type)
            .cmp(&source_order(&b.source_type))
            .then(a.name.cmp(&b.name))
    });

    all_envs
}

/// Find an environment by name, searching across sources.
///
/// Searches in order: pyenv, virtualenvwrapper, conda.
/// Returns the first match found.
///
/// # Examples
///
/// ```ignore
/// use scoop_uv::cli::commands::migrate::scan::find_environment_by_name;
/// use scoop_uv::cli::MigrateSource;
///
/// // Search for non-existent environment returns error
/// let result = find_environment_by_name("nonexistent-env-12345", None);
/// assert!(result.is_err());
///
/// // Search only in specific source
/// let result = find_environment_by_name("myproject", Some(MigrateSource::Pyenv));
/// // Returns Ok(env) if found, Err if not
/// ```
///
/// # Errors
///
/// Returns source-specific error if environment is not found:
/// - [`ScoopError::PyenvEnvNotFound`]
/// - [`ScoopError::VenvWrapperEnvNotFound`]
/// - [`ScoopError::CondaEnvNotFound`]
pub fn find_environment_by_name(
    name: &str,
    source_filter: Option<MigrateSource>,
) -> Result<SourceEnvironment> {
    // Try pyenv first
    if source_filter.is_none() || source_filter == Some(MigrateSource::Pyenv) {
        if let Some(discovery) = PyenvDiscovery::default_root() {
            if let Ok(env) = discovery.find_environment(name) {
                return Ok(env);
            }
        }
    }

    // Try virtualenvwrapper
    if source_filter.is_none() || source_filter == Some(MigrateSource::Virtualenvwrapper) {
        if let Some(discovery) = VenvWrapperDiscovery::default_root() {
            if let Ok(env) = discovery.find_environment(name) {
                return Ok(env);
            }
        }
    }

    // Try conda
    if source_filter.is_none() || source_filter == Some(MigrateSource::Conda) {
        if let Some(discovery) = CondaDiscovery::default_roots() {
            if let Ok(env) = discovery.find_environment(name) {
                return Ok(env);
            }
        }
    }

    // If a specific source was requested, return that error
    match source_filter {
        Some(MigrateSource::Pyenv) => Err(ScoopError::PyenvEnvNotFound {
            name: name.to_string(),
        }),
        Some(MigrateSource::Virtualenvwrapper) => Err(ScoopError::VenvWrapperEnvNotFound {
            name: name.to_string(),
        }),
        Some(MigrateSource::Conda) => Err(ScoopError::CondaEnvNotFound {
            name: name.to_string(),
        }),
        None => Err(ScoopError::PyenvEnvNotFound {
            name: name.to_string(),
        }),
    }
}
