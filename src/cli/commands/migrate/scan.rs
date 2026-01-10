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
