//! Handler for the `scoop sync` command.
//!
//! Reads `.scuv.toml` (or legacy `.scoop.toml`, walking cwd → parents), creates the declared env if
//! it doesn't exist (with implicit lazy Python install), and installs the
//! merged `default` + selected groups via uv pip. Idempotent — re-running on
//! a clean env is a no-op past the pip resolve.

use rust_i18n::t;

use crate::core::{ScoopManifest, VirtualenvService, manifest};
use crate::error::{Result, ScoopError};
use crate::output::{Output, SyncData};
use crate::validate::PythonVersion;

/// Execute the `sync` command.
pub fn execute(output: &Output, extra_groups: &[String], dry_run: bool) -> Result<()> {
    let manifest_path =
        manifest::find_manifest_from_cwd().ok_or_else(|| ScoopError::ManifestNotFound {
            start_dir: std::env::current_dir().unwrap_or_default(),
        })?;
    let parsed = ScoopManifest::load(&manifest_path)?;

    // Validate --with groups exist before touching any env state — fail fast.
    for g in extra_groups {
        if parsed.group(g).is_none() {
            let available = available_groups(&parsed).join(", ");
            return Err(ScoopError::InvalidArgument {
                message: t!("sync.unknown_group_error", group = g, groups = available).to_string(),
            });
        }
    }

    let service = VirtualenvService::auto()?;
    let env_name = parsed.environment.name.clone();
    let wanted_python = parsed.environment.python.clone();
    let env_existed = service.exists(&env_name)?;

    let packages = parsed.packages_for(extra_groups);
    let groups_resolved = resolved_groups(extra_groups);

    if dry_run {
        emit_plan(
            output,
            &manifest_path,
            &env_name,
            &wanted_python,
            &groups_resolved,
            &packages,
            env_existed,
        );
        return Ok(());
    }

    if !env_existed {
        output.info(&t!(
            "sync.creating_env",
            name = env_name,
            python = wanted_python
        ));
        // Implicit lazy install: sync is a declarative entry point, so the
        // ergonomic default is "make it work" rather than "fail because the
        // Python version isn't preinstalled".
        if !service.is_python_installed(&wanted_python)? {
            output.info(&t!("create.installing_python", version = wanted_python));
            service.install_python(&wanted_python)?;
        }
        service.create(&env_name, &wanted_python)?;
    } else if let Some(actual) = installed_python_version(&service, &env_name)? {
        if !python_matches(&actual, &wanted_python) {
            // Warn-and-proceed: recreating an existing env on a version
            // mismatch is destructive, so leave that to an explicit
            // `scoop remove` + `scoop sync`.
            output.warn(&t!(
                "sync.python_mismatch_warn",
                name = env_name,
                actual = actual,
                wanted = wanted_python
            ));
        }
    }

    if packages.is_empty() {
        output.info(&t!("sync.up_to_date"));
    } else {
        output.info(&t!("sync.installing", count = packages.len()));
        let venv_path = service.get_path(&env_name)?;
        service.pip_install(&venv_path, &packages)?;
    }

    if output.is_json() {
        output.json_success(
            "sync",
            SyncData {
                manifest_path: manifest_path.display().to_string(),
                environment: env_name.clone(),
                python: wanted_python.clone(),
                groups: groups_resolved,
                packages,
                env_created: !env_existed,
                dry_run: false,
            },
        );
        return Ok(());
    }

    output.success(&t!("sync.success", count = packages.len(), name = env_name));
    Ok(())
}

/// Loose-match the env's installed Python against the manifest's specifier:
/// `"3.12"` (manifest) matches `"3.12.7"` (env metadata) etc. Falls back to
/// string equality when either side fails to parse so unusual specifiers
/// (`cpython@3.12`, `pypy@3.10`) still get a reasonable comparison.
fn python_matches(actual: &str, wanted: &str) -> bool {
    match (PythonVersion::parse(actual), PythonVersion::parse(wanted)) {
        (Some(a), Some(w)) => w.matches(&a),
        _ => actual == wanted,
    }
}

fn installed_python_version(service: &VirtualenvService, env_name: &str) -> Result<Option<String>> {
    let path = service.get_path(env_name)?;
    Ok(service.read_metadata(&path).map(|m| m.python_version))
}

/// All groups that will be installed: `["default", ...extra]`.
fn resolved_groups(extra: &[String]) -> Vec<String> {
    let mut g = vec!["default".to_string()];
    g.extend(extra.iter().cloned());
    g
}

/// Names of groups defined in the manifest (default + named groups).
fn available_groups(parsed: &ScoopManifest) -> Vec<String> {
    let mut names = vec!["default".to_string()];
    names.extend(parsed.packages.groups.keys().cloned());
    names
}

#[allow(clippy::too_many_arguments)]
fn emit_plan(
    output: &Output,
    manifest_path: &std::path::Path,
    env_name: &str,
    python: &str,
    groups: &[String],
    packages: &[String],
    env_existed: bool,
) {
    if output.is_json() {
        output.json_success(
            "sync",
            SyncData {
                manifest_path: manifest_path.display().to_string(),
                environment: env_name.to_string(),
                python: python.to_string(),
                groups: groups.to_vec(),
                packages: packages.to_vec(),
                env_created: !env_existed,
                dry_run: true,
            },
        );
        return;
    }
    output.info(&t!("sync.dry_run_header"));
    println!("  manifest:    {}", manifest_path.display());
    println!("  environment: {env_name} (Python {python})");
    println!("  groups:      {}", groups.join(", "));
    println!(
        "  action:      {}",
        if env_existed {
            "sync packages (env exists)"
        } else {
            "create env + install packages"
        }
    );
    println!("  packages:    {} total", packages.len());
    for p in packages {
        println!("    - {p}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::with_temp_scoop_home;
    use serial_test::serial;
    use tempfile::TempDir;

    fn write_manifest(dir: &std::path::Path, content: &str) {
        std::fs::write(dir.join(".scuv.toml"), content).unwrap();
    }

    #[test]
    fn python_matches_loose_specifier_against_full_version() {
        assert!(python_matches("3.12.7", "3.12"));
        assert!(python_matches("3.12.0", "3.12"));
        assert!(!python_matches("3.11.5", "3.12"));
    }

    #[test]
    fn python_matches_exact_when_both_full() {
        assert!(python_matches("3.12.5", "3.12.5"));
        assert!(!python_matches("3.12.5", "3.12.6"));
    }

    #[test]
    fn python_matches_falls_back_to_string_for_unparseable() {
        // Custom specifiers (e.g. cpython@3.12, pypy@3.10) don't parse as
        // PythonVersion; we accept the equality fallback so they round-trip.
        assert!(python_matches("cpython@3.12", "cpython@3.12"));
        assert!(!python_matches("cpython@3.12", "pypy@3.10"));
    }

    #[test]
    fn resolved_groups_always_includes_default_first() {
        assert_eq!(resolved_groups(&[]), vec!["default"]);
        assert_eq!(
            resolved_groups(&["dev".into(), "docs".into()]),
            vec!["default", "dev", "docs"]
        );
    }

    #[test]
    fn available_groups_lists_default_plus_named() {
        let m = ScoopManifest::parse(
            r#"
            [environment]
            name = "proj"
            python = "3.12"

            [packages]
            default = []
            dev = ["pytest"]
            docs = ["mkdocs"]
            "#,
        )
        .unwrap();
        let mut names = available_groups(&m);
        names.sort();
        assert_eq!(names, vec!["default", "dev", "docs"]);
    }

    #[test]
    #[serial]
    fn execute_returns_manifest_not_found_when_absent() {
        with_temp_scoop_home(|_| {
            // Use a sandboxed working dir that has no .scuv.toml (or legacy
            // .scoop.toml). The walk may still hit one in a parent of the
            // real cwd on dev machines, so isolate cwd inside a tempdir.
            let workdir = TempDir::new().unwrap();
            let prev = std::env::current_dir().ok();
            std::env::set_current_dir(workdir.path()).unwrap();

            let output = Output::new(0, true, true, false);
            let result = execute(&output, &[], false);

            if let Some(p) = prev {
                std::env::set_current_dir(p).unwrap();
            }

            // Some test machines may have .scuv.toml (or legacy .scoop.toml)
            // in /tmp's parents; we can't reliably assert ManifestNotFound
            // there. Only assert when no manifest exists anywhere along the
            // path.
            match result {
                Err(ScoopError::ManifestNotFound { .. }) => {}
                Err(other) => panic!("expected ManifestNotFound, got {other:?}"),
                Ok(()) => {
                    // A real manifest was found upstream; the test environment
                    // can't pin the absent-manifest case. Skip silently.
                }
            }
        });
    }

    #[test]
    #[serial]
    fn execute_rejects_unknown_with_group() {
        with_temp_scoop_home(|_| {
            let workdir = TempDir::new().unwrap();
            write_manifest(
                workdir.path(),
                r#"
                [environment]
                name = "proj"
                python = "3.12"

                [packages]
                default = ["pytest"]
                "#,
            );

            let prev = std::env::current_dir().ok();
            std::env::set_current_dir(workdir.path()).unwrap();

            let output = Output::new(0, true, true, false);
            let result = execute(&output, &["ghost".to_string()], true /* dry-run */);

            if let Some(p) = prev {
                std::env::set_current_dir(p).unwrap();
            }

            let err = result.unwrap_err();
            assert!(matches!(err, ScoopError::InvalidArgument { .. }));
            assert!(err.message_in("en").contains("ghost"));
        });
    }

    #[test]
    #[serial]
    fn execute_dry_run_emits_plan_without_creating_env() {
        with_temp_scoop_home(|temp_dir| {
            let workdir = TempDir::new().unwrap();
            write_manifest(
                workdir.path(),
                r#"
                [environment]
                name = "dryrunenv"
                python = "3.12"

                [packages]
                default = ["pytest", "black"]
                "#,
            );

            let prev = std::env::current_dir().ok();
            std::env::set_current_dir(workdir.path()).unwrap();

            let output = Output::new(0, true, true, false);
            let result = execute(&output, &[], true /* dry-run */);

            if let Some(p) = prev {
                std::env::set_current_dir(p).unwrap();
            }

            assert!(result.is_ok(), "dry-run should succeed: {result:?}");
            // Most importantly: no env was created on disk.
            let env_dir = temp_dir.path().join("virtualenvs").join("dryrunenv");
            assert!(!env_dir.exists(), "dry-run must not create env directory");
        });
    }
}
