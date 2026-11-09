//! Environment scanning and discovery
//!
//! Discovers environments from pyenv, virtualenvwrapper, and conda sources.

use crate::cli::MigrateSource;
use crate::core::migrate::{
    CondaDiscovery, EnvironmentSource, PyenvDiscovery, SourceEnvironment, SourceType,
    VenvWrapperDiscovery,
};
use crate::error::{Result, ScoopError};
use rust_i18n::t;

/// Scan environments from all available sources or a specific source.
///
/// Results are sorted by source type (pyenv, virtualenvwrapper, conda),
/// then alphabetically by name.
///
/// # Examples
///
/// ```text
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

    // A source that fails to scan is skipped; the others still count.
    for discovery in discoveries(source_filter) {
        if let Ok(envs) = discovery.scan_environments() {
            all_envs.extend(envs);
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

/// Returns `true` if at least one source tool matching `filter` is
/// installed on this system (i.e. its `default_root()` /
/// `default_roots()` returned `Some`).
///
/// Used by `batch.rs` to distinguish two empty-list cases:
///
/// - **No tools installed** — `scan_all_environments` returned an empty
///   vec because no source tool exists. Surfaces as
///   [`ScoopError::MigrationSourcesNotFound`] (exit 3) so CI scripts
///   can detect a misconfigured environment.
/// - **Tools present but no envs** — current `Ok(())` behaviour stays.
///
/// `filter == None` checks all three sources; a specific `MigrateSource`
/// only checks that one.
pub fn any_source_tool_available(filter: Option<MigrateSource>) -> bool {
    // Stops at the first installed source, as the roots are probed lazily.
    discoveries(filter).next().is_some()
}

/// The discovery for one source tool, or `None` when the tool's root is not
/// on this system. Exhaustive, so a new [`MigrateSource`] has to say how it
/// is found.
fn discovery(source: MigrateSource) -> Option<Box<dyn EnvironmentSource>> {
    match source {
        MigrateSource::Pyenv => {
            PyenvDiscovery::default_root().map(|d| Box::new(d) as Box<dyn EnvironmentSource>)
        }
        MigrateSource::Virtualenvwrapper => {
            VenvWrapperDiscovery::default_root().map(|d| Box::new(d) as Box<dyn EnvironmentSource>)
        }
        MigrateSource::Conda => {
            CondaDiscovery::default_roots().map(|d| Box::new(d) as Box<dyn EnvironmentSource>)
        }
    }
}

/// The installed sources `filter` selects, in search order: the declaration
/// order of [`MigrateSource`] (pyenv, virtualenvwrapper, conda). Lazy, so a
/// caller that stops early does not probe the remaining tools' roots.
fn discoveries(filter: Option<MigrateSource>) -> impl Iterator<Item = Box<dyn EnvironmentSource>> {
    use clap::ValueEnum;
    MigrateSource::value_variants()
        .iter()
        .copied()
        .filter(move |source| filter.is_none_or(|f| f == *source))
        .filter_map(discovery)
}

/// Find an environment by name, searching across sources.
///
/// Searches in order: pyenv, virtualenvwrapper, conda.
/// Returns the first match found.
///
/// # Examples
///
/// ```text
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
/// Returns [`ScoopError::SourceEnvNotFound`] if no source has the
/// environment, or the source-specific error when a filter was given:
/// - [`ScoopError::PyenvEnvNotFound`]
/// - [`ScoopError::VenvWrapperEnvNotFound`]
/// - [`ScoopError::CondaEnvNotFound`]
pub fn find_environment_by_name(
    name: &str,
    source_filter: Option<MigrateSource>,
) -> Result<SourceEnvironment> {
    // The first source that has it wins; one that fails to look is skipped.
    if let Some(env) =
        discoveries(source_filter).find_map(|discovery| discovery.find_environment(name).ok())
    {
        return Ok(env);
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
        None => Err(ScoopError::SourceEnvNotFound {
            name: name.to_string(),
        }),
    }
}

/// The "Scanning ..." line, in `locale`. Without a filter it names all
/// sources in that language rather than splicing English into the template.
pub(super) fn scanning_message(source_filter: Option<MigrateSource>, locale: &str) -> String {
    match source_filter {
        Some(source) => t!("migrate.scanning", locale = locale, source = source).to_string(),
        None => t!("migrate.scanning_all", locale = locale).to_string(),
    }
}

/// The "No ... environments found." line, in `locale`. Without a filter the
/// template's `%{source}` slot used to be filled with "", which left two
/// spaces in "No  environments found.".
pub(super) fn no_envs_message(source_filter: Option<MigrateSource>, locale: &str) -> String {
    match source_filter {
        Some(source) => t!("migrate.no_envs", locale = locale, source = source).to_string(),
        None => t!("migrate.no_envs_any", locale = locale).to_string(),
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::with_isolated_migrate_env;
    use serial_test::serial;

    #[test]
    fn scanning_and_no_envs_messages_name_the_source_or_all_sources() {
        let pyenv = Some(MigrateSource::Pyenv);
        assert_eq!(
            scanning_message(pyenv, "en"),
            "Scanning pyenv for environments..."
        );
        assert_eq!(
            scanning_message(None, "en"),
            "Scanning all sources for environments..."
        );
        assert_eq!(no_envs_message(pyenv, "en"), "No pyenv environments found.");
        assert_eq!(no_envs_message(None, "en"), "No environments found.");
        // No English leaks into another locale when nothing is filtered.
        assert!(!scanning_message(None, "ko").contains("all sources"));
        for locale in ["en", "ko", "ja", "pt-BR", "es"] {
            assert!(!no_envs_message(None, locale).contains("  "), "{locale}");
        }
    }

    /// Without a filter every source was searched, so the miss names none
    /// of them in particular.
    #[test]
    fn find_environment_by_name_no_filter_returns_a_source_neutral_error() {
        with_isolated_migrate_env(|| {
            let err = find_environment_by_name("nonexistent_env_12345", None).unwrap_err();
            assert!(
                matches!(err, ScoopError::SourceEnvNotFound { ref name } if name == "nonexistent_env_12345"),
                "{err:?}"
            );
        });
    }

    /// Tests that source_filter=Pyenv returns PyenvEnvNotFound.
    #[test]
    fn find_environment_by_name_pyenv_filter_returns_pyenv_error() {
        with_isolated_migrate_env(|| {
            let result =
                find_environment_by_name("nonexistent_env_12345", Some(MigrateSource::Pyenv));

            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(
                matches!(err, ScoopError::PyenvEnvNotFound { ref name } if name == "nonexistent_env_12345")
            );
        });
    }

    /// Tests that source_filter=Virtualenvwrapper returns VenvWrapperEnvNotFound.
    #[test]
    fn find_environment_by_name_venvwrapper_filter_returns_venvwrapper_error() {
        with_isolated_migrate_env(|| {
            let result = find_environment_by_name(
                "nonexistent_env_12345",
                Some(MigrateSource::Virtualenvwrapper),
            );

            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(
                matches!(err, ScoopError::VenvWrapperEnvNotFound { ref name } if name == "nonexistent_env_12345")
            );
        });
    }

    /// Tests that source_filter=Conda returns CondaEnvNotFound.
    #[test]
    fn find_environment_by_name_conda_filter_returns_conda_error() {
        with_isolated_migrate_env(|| {
            let result =
                find_environment_by_name("nonexistent_env_12345", Some(MigrateSource::Conda));

            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(
                matches!(err, ScoopError::CondaEnvNotFound { ref name } if name == "nonexistent_env_12345")
            );
        });
    }

    /// Tests scan_all_environments with no environments installed returns empty.
    #[test]
    fn scan_all_environments_empty_when_no_sources() {
        with_isolated_migrate_env(|| {
            let envs = scan_all_environments(None);
            assert!(envs.is_empty());
        });
    }

    /// Tests scan_all_environments with specific source filter.
    #[test]
    fn scan_all_environments_with_pyenv_filter_returns_empty() {
        with_isolated_migrate_env(|| {
            let envs = scan_all_environments(Some(MigrateSource::Pyenv));
            assert!(envs.is_empty());
        });
    }

    #[test]
    fn scan_all_environments_with_venvwrapper_filter_returns_empty() {
        with_isolated_migrate_env(|| {
            let envs = scan_all_environments(Some(MigrateSource::Virtualenvwrapper));
            assert!(envs.is_empty());
        });
    }

    #[test]
    fn scan_all_environments_with_conda_filter_returns_empty() {
        with_isolated_migrate_env(|| {
            let envs = scan_all_environments(Some(MigrateSource::Conda));
            assert!(envs.is_empty());
        });
    }

    // =========================================================================
    // any_source_tool_available Tests (Inc 4)
    // =========================================================================

    /// With no source tools installed (isolated tempdir as $HOME), the
    /// availability probe returns false. This is the signal `batch.rs`
    /// uses to surface MigrationSourcesNotFound (exit 3) instead of
    /// silently succeeding when there are simply no envs.
    #[test]
    #[serial]
    fn any_source_tool_available_returns_false_when_isolated() {
        with_isolated_migrate_env(|| {
            assert!(!any_source_tool_available(None));
            assert!(!any_source_tool_available(Some(MigrateSource::Pyenv)));
            assert!(!any_source_tool_available(Some(
                MigrateSource::Virtualenvwrapper
            )));
            assert!(!any_source_tool_available(Some(MigrateSource::Conda)));
        });
    }

    /// When `with_full_migrate_env` materialises a pyenv tree, the Pyenv
    /// branch of the OR-chain matches and the probe returns true. The
    /// other filters still come back false because only pyenv is set up.
    #[test]
    #[serial]
    fn any_source_tool_available_returns_true_when_pyenv_present() {
        use crate::test_utils::with_full_migrate_env;
        with_full_migrate_env(|_scoop, _pyenv| {
            assert!(any_source_tool_available(None));
            assert!(any_source_tool_available(Some(MigrateSource::Pyenv)));
        });
    }

    // =========================================================================
    // Source order and precedence, with every source installed under the
    // isolated HOME (no PYENV_ROOT, WORKON_HOME or CONDA_PREFIX: the
    // defaults under HOME are used).
    // =========================================================================

    fn home() -> std::path::PathBuf {
        dirs::home_dir().unwrap()
    }

    fn pyenv_env(name: &str) {
        crate::test_utils::create_mock_pyenv_env(&home().join(".pyenv"), name, "3.12.0");
    }

    fn venvwrapper_env(name: &str) {
        let env = home().join(".virtualenvs").join(name);
        std::fs::create_dir_all(env.join("bin")).unwrap();
        std::fs::write(env.join("bin").join("python"), "").unwrap();
    }

    fn conda_env(name: &str) {
        let env = home().join(".conda").join("envs").join(name);
        std::fs::create_dir_all(env.join("conda-meta")).unwrap();
        std::fs::create_dir_all(env.join("bin")).unwrap();
        std::fs::write(env.join("bin").join("python"), "").unwrap();
    }

    /// Unfiltered: pyenv, then virtualenvwrapper, then conda, names sorted
    /// within each; a name more than one source has is found in the first.
    /// What fails this: a different source order, or lookup not stopping
    /// at the first source that has the name.
    #[test]
    #[serial]
    fn sources_are_scanned_and_searched_in_order() {
        with_isolated_migrate_env(|| {
            pyenv_env("zeta");
            pyenv_env("web");
            venvwrapper_env("web");
            venvwrapper_env("api");
            conda_env("web");

            let found: Vec<_> = scan_all_environments(None)
                .into_iter()
                .map(|e| (e.source_type, e.name))
                .collect();
            assert_eq!(
                found,
                [
                    (SourceType::Pyenv, "web".to_string()),
                    (SourceType::Pyenv, "zeta".to_string()),
                    (SourceType::VirtualenvWrapper, "api".to_string()),
                    (SourceType::VirtualenvWrapper, "web".to_string()),
                    (SourceType::Conda, "web".to_string()),
                ]
            );

            let web = find_environment_by_name("web", None).unwrap();
            assert_eq!(web.source_type, SourceType::Pyenv);
            let api = find_environment_by_name("api", None).unwrap();
            assert_eq!(api.source_type, SourceType::VirtualenvWrapper);
            let filtered = find_environment_by_name("web", Some(MigrateSource::Conda)).unwrap();
            assert_eq!(filtered.source_type, SourceType::Conda);
        });
    }

    /// A filter picks exactly that source.
    #[test]
    #[serial]
    fn a_filter_selects_exactly_one_source() {
        with_isolated_migrate_env(|| {
            pyenv_env("web");
            venvwrapper_env("web");
            conda_env("web");
            for (filter, want) in [
                (MigrateSource::Pyenv, SourceType::Pyenv),
                (
                    MigrateSource::Virtualenvwrapper,
                    SourceType::VirtualenvWrapper,
                ),
                (MigrateSource::Conda, SourceType::Conda),
            ] {
                let types: Vec<_> = discoveries(Some(filter)).map(|d| d.source_type()).collect();
                assert_eq!(types, [want]);
                let envs = scan_all_environments(Some(filter));
                assert_eq!(envs.len(), 1);
                assert_eq!(envs[0].source_type, want);
            }
            let all: Vec<_> = discoveries(None).map(|d| d.source_type()).collect();
            assert_eq!(
                all,
                [
                    SourceType::Pyenv,
                    SourceType::VirtualenvWrapper,
                    SourceType::Conda
                ]
            );
        });
    }

    /// A source that fails to scan does not hide the others. Here pyenv's
    /// `versions` is a file, so reading it fails.
    #[test]
    #[serial]
    fn a_failing_source_does_not_hide_the_others() {
        with_isolated_migrate_env(|| {
            std::fs::create_dir_all(home().join(".pyenv")).unwrap();
            std::fs::write(home().join(".pyenv").join("versions"), "").unwrap();
            conda_env("web");

            let names: Vec<_> = scan_all_environments(None)
                .into_iter()
                .map(|e| (e.source_type, e.name))
                .collect();
            assert_eq!(names, [(SourceType::Conda, "web".to_string())]);
            assert!(any_source_tool_available(Some(MigrateSource::Conda)));
            assert_eq!(
                find_environment_by_name("web", None).unwrap().source_type,
                SourceType::Conda
            );
        });
    }
}
