//! List command

use std::cmp::Ordering;
use std::collections::BTreeSet;

use chrono::{DateTime, Utc};
use owo_colors::OwoColorize;
use rust_i18n::t;

use crate::cli::ListSortMode;
use crate::core::{VirtualenvInfo as CoreVirtualenvInfo, VirtualenvService, get_active_env};
use crate::error::Result;
use crate::output::{ListEnvsData, ListPythonsData, Output, PythonInfo, VirtualenvInfo};
use crate::paths::abbreviate_home;
use crate::uv::UvClient;
use crate::validate::PythonVersion;

/// Execute the list command
pub fn execute(
    output: &Output,
    pythons: bool,
    bare: bool,
    python_version: Option<&str>,
    sort: ListSortMode,
) -> Result<()> {
    if pythons {
        list_pythons(output, bare)
    } else {
        list_virtualenvs(output, bare, python_version, sort)
    }
}

/// Sort a list of envs in place according to the chosen mode.
///
/// Pulled out as a free function so the ordering can be unit-tested
/// without standing up a `VirtualenvService` or touching the filesystem.
/// Two contracts pinned by tests in this module:
///
/// 1. **None-last for timestamp modes.** Envs missing `created_at` or
///    `last_used` always sort *after* envs that have the field, so the
///    interesting ones surface at the top instead of being buried under
///    legacy-metadata neighbours.
/// 2. **Name tie-break.** Equal timestamps (and the entire "None"
///    bucket) fall back to alphabetical-by-name so output is
///    deterministic across invocations.
pub(crate) fn sort_envs(envs: &mut [CoreVirtualenvInfo], mode: ListSortMode) {
    match mode {
        ListSortMode::Name => envs.sort_by(|a, b| a.name.cmp(&b.name)),
        ListSortMode::Created => envs
            .sort_by(|a, b| compare_desc_none_last(a.created_at, b.created_at, &a.name, &b.name)),
        ListSortMode::LastUsed => {
            envs.sort_by(|a, b| compare_desc_none_last(a.last_used, b.last_used, &a.name, &b.name))
        }
    }
}

/// Newest-first ordering with `None` pushed to the end, then a name
/// tie-break. Lifted into its own helper so the same rules apply to
/// `--sort=created` and `--sort=last-used` without copy-paste.
fn compare_desc_none_last(
    a: Option<DateTime<Utc>>,
    b: Option<DateTime<Utc>>,
    a_name: &str,
    b_name: &str,
) -> Ordering {
    match (a, b) {
        (Some(av), Some(bv)) => bv.cmp(&av).then_with(|| a_name.cmp(b_name)),
        (Some(_), None) => Ordering::Less, // Some sorts before None
        (None, Some(_)) => Ordering::Greater,
        (None, None) => a_name.cmp(b_name),
    }
}

/// List virtual environments
fn list_virtualenvs(
    output: &Output,
    bare: bool,
    python_version: Option<&str>,
    sort: ListSortMode,
) -> Result<()> {
    use crate::core::VersionService;
    use crate::validate::validate_python_version;

    // Validate and parse version filter
    let version_filter = if let Some(ver_str) = python_version {
        validate_python_version(ver_str)?;
        PythonVersion::parse(ver_str)
    } else {
        None
    };

    let service = VirtualenvService::auto()?;
    let mut envs = service.list()?;
    let active_env = get_active_env();

    // Apply python version filter
    if let Some(ref filter) = version_filter {
        envs.retain(|env| {
            env.python_version
                .as_ref()
                .and_then(|v| PythonVersion::parse(v))
                .is_some_and(|v| filter.matches(&v))
        });
    }

    // Sort *after* filtering so the user sees the requested ordering
    // applied to the same set their filter produced.
    sort_envs(&mut envs, sort);

    // Check if "system" is the resolved version
    let resolved = VersionService::resolve_current();
    let system_active = resolved.as_deref() == Some("system");

    // Get system Python info, filtered if needed
    let system_python = get_system_python_info().filter(|(version, _)| match version_filter {
        Some(ref filter) => PythonVersion::parse(version).is_some_and(|v| filter.matches(&v)),
        None => true,
    });

    // JSON output
    if output.is_json() {
        let mut virtualenvs: Vec<VirtualenvInfo> = envs
            .iter()
            .map(|env| VirtualenvInfo {
                name: env.name.clone(),
                python: env.python_version.clone(),
                path: env.path.display().to_string(),
                active: active_env.as_ref() == Some(&env.name),
                created_at: env.created_at.map(|t| t.to_rfc3339()),
                last_used: env.last_used.map(|t| t.to_rfc3339()),
            })
            .collect();

        // Add system Python to JSON output
        if let Some((version, path)) = &system_python {
            virtualenvs.push(VirtualenvInfo {
                name: "system".to_string(),
                python: Some(version.clone()),
                path: path.clone(),
                active: system_active,
                // System Python isn't a scoop-managed env, so there's
                // no on-disk metadata to source these from.
                created_at: None,
                last_used: None,
            });
        }

        let total = virtualenvs.len();
        output.json_success("list", ListEnvsData { virtualenvs, total });
        return Ok(());
    }

    if envs.is_empty() && system_python.is_none() {
        if !bare {
            if let Some(ver_str) = python_version {
                output.info(&t!("list.filtered_no_envs", version = ver_str));
                output.info(&t!("list.filtered_hint"));
            } else {
                output.info(&t!("list.no_envs"));
                output.info(&t!("list.no_envs_hint"));
            }
        }
        return Ok(());
    }

    if bare {
        // Output names only, one per line (for completion)
        for env in &envs {
            println!("{}", env.name);
        }
        // Add system to bare output
        if system_python.is_some() {
            println!("system");
        }
    } else {
        // Calculate column widths for alignment (include "system" in calculation)
        let mut max_name_len = envs.iter().map(|e| e.name.len()).max().unwrap_or(0);
        if system_python.is_some() {
            max_name_len = max_name_len.max(6); // "system".len() == 6
        }

        let mut max_ver_len = envs
            .iter()
            .filter_map(|e| e.python_version.as_ref())
            .map(|v| v.len())
            .max()
            .unwrap_or(1);
        if let Some((version, _)) = &system_python {
            max_ver_len = max_ver_len.max(version.len());
        }

        // Output with marker, name, version, and path
        for env in &envs {
            let is_active = active_env.as_ref() == Some(&env.name);
            let marker = if is_active { "*" } else { " " };
            let version = env.python_version.as_deref().unwrap_or("-");
            let path = abbreviate_home(&env.path);

            if output.use_color() && is_active {
                println!(
                    "{} {:<name_w$}  {:<ver_w$}  {}",
                    marker.green(),
                    env.name.green(),
                    version,
                    path,
                    name_w = max_name_len,
                    ver_w = max_ver_len
                );
            } else {
                println!(
                    "{} {:<name_w$}  {:<ver_w$}  {}",
                    marker,
                    env.name,
                    version,
                    path,
                    name_w = max_name_len,
                    ver_w = max_ver_len
                );
            }
        }

        // Add system Python at the end
        if let Some((version, path)) = system_python {
            let marker = if system_active { "*" } else { " " };
            let display_path = format!("{} (system)", path);

            if output.use_color() && system_active {
                println!(
                    "{} {:<name_w$}  {:<ver_w$}  {}",
                    marker.green(),
                    "system".green(),
                    version,
                    display_path.dimmed(),
                    name_w = max_name_len,
                    ver_w = max_ver_len
                );
            } else {
                println!(
                    "{} {:<name_w$}  {:<ver_w$}  {}",
                    marker,
                    "system",
                    version,
                    display_path,
                    name_w = max_name_len,
                    ver_w = max_ver_len
                );
            }
        }
    }

    Ok(())
}

/// List installed Python versions
fn list_pythons(output: &Output, bare: bool) -> Result<()> {
    let uv = UvClient::new()?;
    let pythons = uv.list_installed_pythons()?;

    // JSON output
    if output.is_json() {
        let python_infos: Vec<PythonInfo> = pythons
            .iter()
            .map(|p| PythonInfo {
                version: p.version.clone(),
                implementation: Some(p.implementation.clone()),
                path: p.path.as_ref().map(|path| path.display().to_string()),
            })
            .collect();
        let total = python_infos.len();
        output.json_success(
            "list",
            ListPythonsData {
                pythons: python_infos,
                total,
            },
        );
        return Ok(());
    }

    if pythons.is_empty() {
        if !bare {
            output.info(&t!("list.no_pythons"));
            output.info(&t!("list.no_pythons_hint"));
        }
        return Ok(());
    }

    if bare {
        // Output unique, sorted versions for shell completion
        // This eliminates the need for `| sort -u` in shell scripts
        let versions: BTreeSet<PythonVersion> = pythons
            .iter()
            .filter_map(|p| PythonVersion::parse(&p.version))
            .collect();

        for version in versions {
            println!("{version}");
        }
    } else {
        // Build display labels: "implementation-version" (e.g., "cpython-3.12.0")
        let labels: Vec<String> = pythons
            .iter()
            .map(|p| format!("{}-{}", p.implementation, p.version))
            .collect();

        // Calculate max label length for alignment
        let max_label_len = labels.iter().map(|l| l.len()).max().unwrap_or(0);

        // Normal output with path info
        for (python, label) in pythons.iter().zip(&labels) {
            let path_str = python
                .path
                .as_ref()
                .map(|p| format!("({})", p.display()))
                .unwrap_or_default();

            println!("{:<width$}  {}", label, path_str, width = max_label_len);
        }
    }

    Ok(())
}

/// Get system Python version and path
///
/// Returns `(version, path)` tuple if system Python is found.
fn get_system_python_info() -> Option<(String, String)> {
    use std::process::Command;

    // Try python3 first, then python - reuse the output to avoid double process calls
    let (python_cmd, version_output) = {
        let output = Command::new("python3").arg("--version").output().ok();
        match output {
            Some(ref out) if out.status.success() => ("python3", output),
            _ => (
                "python",
                Command::new("python").arg("--version").output().ok(),
            ),
        }
    };

    let version_output = version_output?;

    if !version_output.status.success() {
        return None;
    }

    let version_str = String::from_utf8_lossy(&version_output.stdout);
    // "Python 3.12.1" -> "3.12.1"
    let version = version_str
        .trim()
        .strip_prefix("Python ")
        .unwrap_or(version_str.trim())
        .to_string();

    // Get path using 'which' on Unix
    let path_output = Command::new("which").arg(python_cmd).output().ok()?;

    if !path_output.status.success() {
        return None;
    }

    let path = String::from_utf8_lossy(&path_output.stdout)
        .trim()
        .to_string();

    Some((version, path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use std::path::PathBuf;

    fn ts(year: i32, month: u32, day: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(year, month, day, 0, 0, 0).unwrap()
    }

    fn env(
        name: &str,
        created_at: Option<DateTime<Utc>>,
        last_used: Option<DateTime<Utc>>,
    ) -> CoreVirtualenvInfo {
        CoreVirtualenvInfo {
            name: name.to_string(),
            path: PathBuf::from(format!("/tmp/{name}")),
            python_version: None,
            created_at,
            last_used,
        }
    }

    #[test]
    fn sort_by_name_is_alphabetical() {
        let mut envs = vec![env("zeta", None, None), env("alpha", None, None)];
        sort_envs(&mut envs, ListSortMode::Name);
        let names: Vec<_> = envs.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, vec!["alpha", "zeta"]);
    }

    #[test]
    fn sort_by_created_is_newest_first_none_last() {
        let mut envs = vec![
            env("old", Some(ts(2024, 1, 1)), None),
            env("none", None, None),
            env("new", Some(ts(2026, 6, 1)), None),
        ];
        sort_envs(&mut envs, ListSortMode::Created);
        let names: Vec<_> = envs.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(
            names,
            vec!["new", "old", "none"],
            "newest first, None at end"
        );
    }

    #[test]
    fn sort_by_last_used_is_recent_first_none_last() {
        let mut envs = vec![
            env("stale", None, Some(ts(2024, 1, 1))),
            env("fresh-no-touch", Some(ts(2026, 6, 1)), None),
            env("recent", None, Some(ts(2026, 5, 30))),
        ];
        sort_envs(&mut envs, ListSortMode::LastUsed);
        let names: Vec<_> = envs.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(
            names,
            vec!["recent", "stale", "fresh-no-touch"],
            "last_used recency wins; created_at is irrelevant for this sort"
        );
    }

    #[test]
    fn sort_tie_break_by_name() {
        let same = ts(2026, 6, 1);
        let mut envs = vec![
            env("zebra", Some(same), None),
            env("alpha", Some(same), None),
            env("mike", Some(same), None),
        ];
        sort_envs(&mut envs, ListSortMode::Created);
        let names: Vec<_> = envs.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(
            names,
            vec!["alpha", "mike", "zebra"],
            "equal timestamps must fall back to alphabetical-by-name"
        );
    }

    #[test]
    fn sort_none_bucket_tie_breaks_by_name() {
        // The "Some sorts before None" contract still has to leave the
        // None-bucket internally deterministic, otherwise `--sort` could
        // shuffle no-metadata envs randomly between invocations.
        let mut envs = vec![
            env("zulu", None, None),
            env("alpha", None, None),
            env("mike", None, None),
        ];
        sort_envs(&mut envs, ListSortMode::LastUsed);
        let names: Vec<_> = envs.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, vec!["alpha", "mike", "zulu"]);
    }

    /// Helper: simulate the filtering logic used in list_virtualenvs
    fn filter_envs_by_version<'a>(
        envs: &'a [(String, Option<String>)],
        version_str: &str,
    ) -> Vec<&'a str> {
        let filter = PythonVersion::parse(version_str).expect("valid version filter");
        envs.iter()
            .filter(|(_, ver)| {
                ver.as_ref()
                    .and_then(|v| PythonVersion::parse(v))
                    .is_some_and(|v| filter.matches(&v))
            })
            .map(|(name, _)| name.as_str())
            .collect()
    }

    #[test]
    fn test_filter_by_major_minor_prefix() {
        let envs = vec![
            ("web".into(), Some("3.12.0".into())),
            ("api".into(), Some("3.12.1".into())),
            ("old".into(), Some("3.11.5".into())),
            ("ml".into(), Some("3.13.0".into())),
        ];

        let result = filter_envs_by_version(&envs, "3.12");
        assert_eq!(result, vec!["web", "api"]);
    }

    #[test]
    fn test_filter_by_exact_patch_version() {
        let envs = vec![
            ("web".into(), Some("3.12.0".into())),
            ("api".into(), Some("3.12.1".into())),
        ];

        let result = filter_envs_by_version(&envs, "3.12.0");
        assert_eq!(result, vec!["web"]);
    }

    #[test]
    fn test_filter_by_major_only() {
        let envs = vec![
            ("py3".into(), Some("3.12.0".into())),
            ("py2".into(), Some("2.7.18".into())),
        ];

        let result = filter_envs_by_version(&envs, "3");
        assert_eq!(result, vec!["py3"]);
    }

    #[test]
    fn test_filter_no_matches() {
        let envs = vec![
            ("web".into(), Some("3.12.0".into())),
            ("api".into(), Some("3.11.5".into())),
        ];

        let result = filter_envs_by_version(&envs, "3.10");
        assert!(result.is_empty());
    }

    #[test]
    fn test_filter_skips_envs_without_version() {
        let envs = vec![
            ("web".into(), Some("3.12.0".into())),
            ("broken".into(), None),
            ("api".into(), Some("3.12.1".into())),
        ];

        let result = filter_envs_by_version(&envs, "3.12");
        assert_eq!(result, vec!["web", "api"]);
    }

    #[test]
    fn test_filter_all_envs_match() {
        let envs = vec![
            ("a".into(), Some("3.12.0".into())),
            ("b".into(), Some("3.12.1".into())),
            ("c".into(), Some("3.12.3".into())),
        ];

        let result = filter_envs_by_version(&envs, "3.12");
        assert_eq!(result, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_filter_empty_envs() {
        let envs: Vec<(String, Option<String>)> = vec![];
        let result = filter_envs_by_version(&envs, "3.12");
        assert!(result.is_empty());
    }

    #[test]
    fn test_system_python_filtered_by_version() {
        // Simulate the system python filter logic from list_virtualenvs
        let system_python = Some(("3.12.1".to_string(), "/usr/bin/python3".to_string()));
        let filter = PythonVersion::parse("3.12").unwrap();

        let filtered = system_python.filter(|(version, _)| {
            PythonVersion::parse(version).is_some_and(|v| filter.matches(&v))
        });
        assert!(filtered.is_some());

        // Non-matching filter
        let filter_311 = PythonVersion::parse("3.11").unwrap();
        let system_python2 = Some(("3.12.1".to_string(), "/usr/bin/python3".to_string()));
        let filtered2 = system_python2.filter(|(version, _)| {
            PythonVersion::parse(version).is_some_and(|v| filter_311.matches(&v))
        });
        assert!(filtered2.is_none());
    }
}
