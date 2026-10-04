//! Uninstall command

use std::io::IsTerminal;

use dialoguer::Confirm;
use rust_i18n::t;

use crate::core::VirtualenvService;
use crate::error::{Result, ScoopError};
use crate::output::{CascadeFailure, Output, UninstallData};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::uv::UvClient;
use crate::validate::PythonVersion;

/// Execute the uninstall command
pub fn execute(output: &Output, version: &str, cascade: bool, force: bool) -> Result<()> {
    // A cascade works out from the version numbers which installs uv will
    // remove; for a request that names more (`3.13t`, `3.12.0rc1`,
    // `cpython@3.12`) the numbers do not say, and a working env could be
    // deleted. Refuse before anything is touched.
    if cascade && !is_plain_version(version) {
        return Err(ScoopError::InvalidArgument {
            message: t!("uninstall.cascade_needs_plain_version", version = version).to_string(),
        });
    }

    let uv = UvClient::new()?;

    // Decide (and confirm) which envs go before anything is removed, but
    // remove them only once uv has uninstalled the Python: if that fails,
    // every env is still there.
    let planned = if cascade {
        Some(plan_cascade(output, &uv, version, force)?)
    } else {
        None
    };

    output.info(&t!("uninstall.uninstalling", version = version));

    uv.uninstall_python(version)?;

    let (removed_envs, failed_envs) = match planned {
        Some(planned) => {
            let (removed, failed) = remove_envs(output, &planned);
            (Some(removed), failed)
        }
        None => (None, Vec::new()),
    };

    let data = UninstallData {
        version: version.to_string(),
        removed_envs,
        failed_envs,
    };

    // Envs left without a Python must not pass as a success: the Python is
    // gone either way, so report what was removed and what was not, and
    // exit non-zero.
    if !data.failed_envs.is_empty() {
        let err = ScoopError::CascadeIncomplete {
            failed_count: data.failed_envs.len(),
        };
        if output.is_json() {
            println!("{}", incomplete_json(&err, &data));
        } else {
            output.error(&err.to_string());
        }
        return Err(err);
    }

    if output.is_json() {
        output.json_success("uninstall", data);
        return Ok(());
    }

    output.success(&t!("uninstall.success", version = version));

    Ok(())
}

/// The JSON for a cascade that left envs behind: the error, plus the same
/// `data` a success carries, so scripts see which envs went and which did not.
fn incomplete_json(err: &ScoopError, data: &UninstallData) -> String {
    serde_json::to_string_pretty(&serde_json::json!({
        "status": "error",
        "command": "uninstall",
        "error": { "code": err.code(), "message": err.to_string() },
        "data": data,
    }))
    .unwrap_or_default()
}

/// An env the cascade plans to remove, with the interpreter directory its
/// `pyvenv.cfg` points at (checked again after the uninstall).
struct Planned {
    name: String,
    home: PathBuf,
}

/// Work out which envs the uninstall takes the Python away from, list them
/// and ask for confirmation (unless `--force` or `--json`).
fn plan_cascade(
    output: &Output,
    uv: &UvClient,
    version: &str,
    force: bool,
) -> Result<Vec<Planned>> {
    let service = VirtualenvService::auto()?;
    let envs = service.list()?;

    let planned: Vec<Planned> = match PythonVersion::parse(version) {
        Some(filter) => {
            let install_dir = uv.python_dir()?;
            let (removed, remaining): (Vec<String>, Vec<String>) = installed_keys(&install_dir)
                .into_iter()
                .partition(|key| removed_by(key, &filter));
            let links: Vec<(String, Option<String>, Option<PathBuf>)> = envs
                .into_iter()
                .map(|e| {
                    let home = env_home(&e.path);
                    let linked = home
                        .as_deref()
                        .and_then(|h| linked_install(h, &install_dir));
                    (e.name, linked, home)
                })
                .collect();
            let doomed = affected_envs(
                links.iter().map(|(n, l, _)| (n.clone(), l.clone())),
                &removed,
                &remaining,
            );
            links
                .into_iter()
                .filter(|(name, _, _)| doomed.contains(name))
                .filter_map(|(name, _, home)| Some(Planned { name, home: home? }))
                .collect()
        }
        None => Vec::new(),
    };

    // No matching environments
    if planned.is_empty() {
        if !output.is_json() {
            output.info(&t!("uninstall.cascade_none", version = version));
        }
        return Ok(planned);
    }

    // Show matching environments and confirm (unless --force or --json)
    if !force && !output.is_json() {
        output.info(&t!(
            "uninstall.cascade_found",
            count = planned.len(),
            version = version
        ));
        for env in &planned {
            output.info(&t!("uninstall.cascade_env", name = &env.name));
        }

        // Check if stdin is a TTY; if not, abort rather than hanging
        if !std::io::stdin().is_terminal() {
            return Err(ScoopError::CascadeAborted);
        }

        let confirmed = Confirm::new()
            .with_prompt(t!("uninstall.cascade_confirm", version = version).to_string())
            .default(false)
            .interact()
            .unwrap_or(false);

        if !confirmed {
            output.info(&t!("uninstall.cascade_cancelled"));
            return Err(ScoopError::CascadeAborted);
        }
    }

    Ok(planned)
}

/// Remove the planned envs whose interpreter is really gone now.
///
/// The plan was made before uv ran: if uv kept a link alive (another patch
/// installed meanwhile took it over) the env still works and is kept. Every
/// env is attempted; failures are reported, not fatal, so one stuck env
/// does not leave the rest broken and unreported.
fn remove_envs(output: &Output, planned: &[Planned]) -> (Vec<String>, Vec<CascadeFailure>) {
    let mut removed = Vec::new();
    let mut failed = Vec::new();
    let service = match VirtualenvService::auto() {
        Ok(service) => service,
        Err(e) => {
            for env in planned {
                failed.push(CascadeFailure {
                    name: env.name.clone(),
                    error: e.to_string(),
                });
            }
            return (removed, failed);
        }
    };
    for env in planned {
        if env.home.exists() {
            continue;
        }
        if !output.is_json() {
            output.info(&t!("uninstall.cascade_removing", name = &env.name));
        }
        match service.delete(&env.name) {
            Ok(()) => removed.push(env.name.clone()),
            Err(e) => {
                output.warn(&t!(
                    "uninstall.cascade_remove_failed",
                    name = &env.name,
                    error = e.to_string()
                ));
                failed.push(CascadeFailure {
                    name: env.name.clone(),
                    error: e.to_string(),
                });
            }
        }
    }

    if !output.is_json() && !removed.is_empty() {
        output.info(&t!("uninstall.cascade_removed", count = removed.len()));
    }

    (removed, failed)
}

/// The `home` line of an env's `pyvenv.cfg`: the directory its interpreter
/// lives in. `None` when the file or the line is missing.
fn env_home(env_path: &Path) -> Option<PathBuf> {
    let cfg = std::fs::read_to_string(env_path.join("pyvenv.cfg")).ok()?;
    cfg.lines().find_map(|line| {
        let (key, value) = line.split_once('=')?;
        (key.trim() == "home").then(|| PathBuf::from(value.trim()))
    })
}

/// The uv-managed install an env's interpreter `home` lies in: the
/// directory name under `install_dir` (`cpython-3.12.14-…` for a patch
/// install, `cpython-3.12-…` for uv's minor-version link). `None` for a
/// Python outside uv's install directory (Homebrew, a `--python-path`
/// interpreter): uninstalling a uv Python cannot break it.
fn linked_install(home: &Path, install_dir: &Path) -> Option<String> {
    // `<key>/bin` on POSIX; on Windows the interpreter sits in `<key>`
    // itself; older uv layouts add an `install` level (`<key>/install[/bin]`).
    let mut entry = home.to_path_buf();
    for level in ["bin", "install"] {
        if entry.file_name().is_some_and(|n| n == level) {
            entry = entry.parent()?.to_path_buf();
        }
    }
    // Compare real paths (`/tmp` vs `/private/tmp`); the entry itself may be
    // the minor-version symlink, so resolve only its parent.
    let parent = std::fs::canonicalize(entry.parent()?).ok()?;
    if parent != std::fs::canonicalize(install_dir).ok()? {
        return None;
    }
    Some(entry.file_name()?.to_string_lossy().into_owned())
}

/// The installs uv keeps under `install_dir`, by directory name (`key`):
/// entries whose version part names a patch release. The minor-version
/// links (`cpython-3.12-…`, symlinks or junctions) and uv's own files
/// (`.lock`, `.temp`) have none and are left out. Read from the directory, not
/// `uv python list`, which shows only installs for the current platform
/// while `uv python uninstall` removes matching ones for any platform.
fn installed_keys(install_dir: &Path) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(install_dir) else {
        return Vec::new();
    };
    entries
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|key| version_part(key).is_some_and(|(numbers, _)| numbers.split('.').count() >= 3))
        .collect()
}

/// The version part of an install key, split into its numbers and its
/// variant (`cpython-3.13.1+freethreaded-…` → `("3.13.1", Some("freethreaded"))`).
fn version_part(key: &str) -> Option<(&str, Option<&str>)> {
    let version = key.split('-').nth(1)?;
    Some(match version.split_once('+') {
        Some((numbers, variant)) => (numbers, Some(variant)),
        None => (version, None),
    })
}

/// Whether `uv python uninstall <plain version>` removes the install `key`:
/// its numbers match and it is the default build. A plain request means
/// the default variant to uv; free-threaded and debug builds need their own
/// request (`3.13t`). A request that names the patch (`3.14.0`) means that
/// release, not its pre-releases (`3.14.0rc1`), while `3.14` takes both.
fn removed_by(key: &str, filter: &PythonVersion) -> bool {
    version_part(key).is_some_and(|(numbers, variant)| {
        let prerelease = !numbers
            .split('.')
            .all(|part| part.bytes().all(|b| b.is_ascii_digit()));
        variant.is_none()
            && !(prerelease && filter.patch.is_some())
            && PythonVersion::parse(numbers).is_some_and(|v| filter.matches(&v))
    })
}

/// The name of uv's minor-version link for an install key: the patch dropped
/// from the version part, any variant kept (`cpython-3.12.14-macos-…` →
/// `cpython-3.12-macos-…`, `cpython-3.13.1+freethreaded-…` →
/// `cpython-3.13+freethreaded-…`).
fn minor_link_name(key: &str) -> Option<String> {
    let (numbers, variant) = version_part(key)?;
    let minor: Vec<&str> = numbers.split('.').take(2).collect();
    if minor.len() < 2 {
        return None;
    }
    let mut minor = minor.join(".");
    if let Some(variant) = variant {
        minor = format!("{minor}+{variant}");
    }
    let mut parts: Vec<&str> = key.split('-').collect();
    parts[1] = &minor;
    Some(parts.join("-"))
}

/// The envs an uninstall takes the Python away from, given what each env's
/// interpreter is linked to (see [`linked_install`]) and which install keys
/// go and stay.
///
/// An env linked to a removed install directly is affected. An env linked
/// to uv's minor-version link is affected only when no remaining install of
/// the same implementation, variant and platform takes over that link (uv
/// points it at the newest such install left). Envs linked to nothing uv
/// manages, or to an install that stays, are not.
fn affected_envs(
    envs: impl IntoIterator<Item = (String, Option<String>)>,
    removed: &[String],
    remaining: &[String],
) -> Vec<String> {
    let removed_keys: HashSet<&str> = removed.iter().map(String::as_str).collect();
    let removed_links: HashSet<String> =
        removed.iter().filter_map(|k| minor_link_name(k)).collect();
    let kept_links: HashSet<String> = remaining
        .iter()
        .filter_map(|k| minor_link_name(k))
        .collect();
    envs.into_iter()
        .filter(|(_, linked)| {
            linked.as_deref().is_some_and(|name| {
                removed_keys.contains(name)
                    || (removed_links.contains(name) && !kept_links.contains(name))
            })
        })
        .map(|(name, _)| name)
        .collect()
}

/// Whether `version` is one to three dot-separated numbers (`3`, `3.12`,
/// `3.12.14`): a request whose meaning to uv is its numbers.
fn is_plain_version(version: &str) -> bool {
    let parts: Vec<&str> = version.split('.').collect();
    parts.len() <= 3
        && parts
            .iter()
            .all(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit()))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ScoopError;
    use crate::uv::UvClient;

    // =========================================================================
    // UninstallData JSON Structure Tests
    // =========================================================================

    /// Failed removals appear as `failed_envs` with name and error; the key
    /// is absent when nothing failed. Fails if the field is dropped or
    /// always written.
    #[test]
    fn uninstall_data_json_reports_failed_envs() {
        let mut data = UninstallData {
            version: "3.12".to_string(),
            removed_envs: Some(vec!["web".to_string()]),
            failed_envs: Vec::new(),
        };
        assert!(
            serde_json::to_value(&data)
                .unwrap()
                .get("failed_envs")
                .is_none()
        );
        data.failed_envs.push(CascadeFailure {
            name: "1bad".to_string(),
            error: "Invalid".to_string(),
        });
        let json = serde_json::to_value(&data).unwrap();
        assert_eq!(json["failed_envs"][0]["name"], "1bad");
        assert_eq!(json["failed_envs"][0]["error"], "Invalid");
    }

    /// A cascade that left envs behind reports an error envelope that still
    /// carries the data. Fails if the status, code or data is dropped.
    #[test]
    fn incomplete_json_is_an_error_with_the_data() {
        let data = UninstallData {
            version: "3.12".to_string(),
            removed_envs: Some(vec!["web".to_string()]),
            failed_envs: vec![CascadeFailure {
                name: "1bad".to_string(),
                error: "Invalid".to_string(),
            }],
        };
        let err = ScoopError::CascadeIncomplete { failed_count: 1 };
        let json: serde_json::Value = serde_json::from_str(&incomplete_json(&err, &data)).unwrap();
        assert_eq!(json["status"], "error");
        assert_eq!(json["command"], "uninstall");
        assert_eq!(json["error"]["code"], "UNINSTALL_CASCADE_INCOMPLETE");
        assert_eq!(json["data"]["removed_envs"][0], "web");
        assert_eq!(json["data"]["failed_envs"][0]["name"], "1bad");
    }

    #[test]
    fn uninstall_data_json_has_correct_field_name() {
        let data = UninstallData {
            version: "3.12.0".to_string(),
            removed_envs: None,
            failed_envs: Vec::new(),
        };

        let json = serde_json::to_string(&data).unwrap();

        // Verify exact JSON structure
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed.is_object());
        assert!(parsed.get("version").is_some());
        assert_eq!(parsed["version"].as_str().unwrap(), "3.12.0");
        // removed_envs should be absent when None (skip_serializing_if)
        assert!(parsed.get("removed_envs").is_none());
    }

    /// True roundtrip test: serialize -> deserialize -> compare
    #[test]
    fn uninstall_data_json_roundtrip() {
        let original = UninstallData {
            version: "3.11.5".to_string(),
            removed_envs: None,
            failed_envs: Vec::new(),
        };

        let json = serde_json::to_string(&original).unwrap();
        let restored: UninstallData = serde_json::from_str(&json).unwrap();

        assert_eq!(original, restored);
    }

    #[test]
    fn uninstall_data_json_with_removed_envs() {
        let data = UninstallData {
            version: "3.12".to_string(),
            removed_envs: Some(vec!["env1".to_string(), "env2".to_string()]),
            failed_envs: Vec::new(),
        };

        let json = serde_json::to_string(&data).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["version"], "3.12");
        let envs = parsed["removed_envs"].as_array().unwrap();
        assert_eq!(envs.len(), 2);
        assert_eq!(envs[0], "env1");
        assert_eq!(envs[1], "env2");
    }

    #[test]
    fn uninstall_data_json_roundtrip_with_removed_envs() {
        let original = UninstallData {
            version: "3.12".to_string(),
            removed_envs: Some(vec!["web".to_string(), "api".to_string()]),
            failed_envs: Vec::new(),
        };

        let json = serde_json::to_string(&original).unwrap();
        let restored: UninstallData = serde_json::from_str(&json).unwrap();

        assert_eq!(original, restored);
    }

    #[test]
    fn uninstall_data_json_empty_removed_envs() {
        let data = UninstallData {
            version: "3.12".to_string(),
            removed_envs: Some(vec![]),
            failed_envs: Vec::new(),
        };

        let json = serde_json::to_string(&data).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        let envs = parsed["removed_envs"].as_array().unwrap();
        assert!(envs.is_empty());
    }

    // =========================================================================
    // Security & Edge Case Tests
    // =========================================================================

    #[test]
    fn uninstall_data_roundtrip_empty_version() {
        let original = UninstallData {
            version: String::new(),
            removed_envs: None,
            failed_envs: Vec::new(),
        };

        let json = serde_json::to_string(&original).unwrap();
        let restored: UninstallData = serde_json::from_str(&json).unwrap();

        assert_eq!(original, restored);
        assert!(restored.version.is_empty());
    }

    #[test]
    fn uninstall_data_roundtrip_json_special_chars() {
        let test_cases = [
            ("quote", r#"3.12.0"injected"#),
            ("backslash", r"3.12.0\path"),
            ("newline", "3.12.0\nmalicious"),
            ("tab", "3.12.0\tvalue"),
            ("null_char", "3.12.0\0null"),
            ("control_chars", "3.12.0\r\n\x08"),
        ];

        for (name, version) in test_cases {
            let original = UninstallData {
                version: version.to_string(),
                removed_envs: None,
                failed_envs: Vec::new(),
            };

            let json = serde_json::to_string(&original).unwrap();
            let restored: UninstallData = serde_json::from_str(&json).unwrap();

            assert_eq!(
                original, restored,
                "Roundtrip failed for '{}': {:?}",
                name, version
            );
        }
    }

    #[test]
    fn uninstall_data_roundtrip_unicode() {
        let test_cases = [
            ("korean", "3.12.0-\u{d55c}\u{ae00}\u{bc84}\u{c804}"),
            ("emoji", "3.12.0-\u{1f40d}python"),
            ("chinese", "3.12.0-\u{4e2d}\u{6587}\u{7248}"),
            (
                "mixed",
                "v3.12.0-\u{03b1}\u{03b2}\u{03b3}-\u{65e5}\u{672c}\u{8a9e}",
            ),
            ("rtl", "3.12.0-\u{05e2}\u{05d1}\u{05e8}\u{05d9}\u{05ea}"),
        ];

        for (name, version) in test_cases {
            let original = UninstallData {
                version: version.to_string(),
                removed_envs: None,
                failed_envs: Vec::new(),
            };

            let json = serde_json::to_string(&original).unwrap();
            let restored: UninstallData = serde_json::from_str(&json).unwrap();

            assert_eq!(
                original, restored,
                "Unicode roundtrip failed for '{}'",
                name
            );
        }
    }

    #[test]
    fn uninstall_data_roundtrip_long_string() {
        let long_version = "3.12.0-".to_string() + &"x".repeat(10_000);
        let original = UninstallData {
            version: long_version,
            removed_envs: None,
            failed_envs: Vec::new(),
        };

        let json = serde_json::to_string(&original).unwrap();
        let restored: UninstallData = serde_json::from_str(&json).unwrap();

        assert_eq!(original, restored);
        assert_eq!(restored.version.len(), 10_007);
    }

    // =========================================================================
    // execute Error Handling Tests
    // =========================================================================

    #[test]
    fn execute_returns_uv_not_found_when_uv_missing() {
        if UvClient::new().is_ok() {
            return;
        }

        let output = Output::new(0, true, crate::output::Colors::NONE, false);
        let result = execute(&output, "3.12.0", false, false);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            matches!(err, ScoopError::UvNotFound),
            "Expected UvNotFound, got {:?}",
            err
        );
    }

    // =========================================================================
    // CascadeAborted Error Tests
    // =========================================================================

    /// uv's minor-version link name for an install key. Fails if the patch
    /// is not dropped, the variant or platform is lost, or a key without a
    /// patch yields a name.
    #[rstest::rstest]
    #[case(
        "cpython-3.12.14-macos-aarch64-none",
        Some("cpython-3.12-macos-aarch64-none")
    )]
    #[case(
        "cpython-3.13.1+freethreaded-linux-x86_64-gnu",
        Some("cpython-3.13+freethreaded-linux-x86_64-gnu")
    )]
    #[case("cpython-3-macos-aarch64-none", None)]
    #[case("garbage", None)]
    fn minor_link_name_cases(#[case] key: &str, #[case] link: Option<&str>) {
        assert_eq!(minor_link_name(key).as_deref(), link);
    }

    /// A plain request removes default builds whose numbers match; a
    /// variant (free-threaded) needs its own request, and a patch request
    /// leaves that patch's pre-releases. Fails if a variant or a pre-release
    /// of an exact patch is counted as removed, or the numbers stop deciding.
    #[rstest::rstest]
    #[case("3.12.14", "cpython-3.12.14-macos-aarch64-none", true)]
    #[case("3.12", "cpython-3.12.14-macos-aarch64-none", true)]
    #[case("3.12.14", "cpython-3.12.13-macos-aarch64-none", false)]
    #[case("3.12.14", "cpython-3.12.14+freethreaded-macos-aarch64-none", false)]
    #[case("3.12", "pypy-3.12.9-macos-aarch64-none", true)]
    #[case("3.14.0", "cpython-3.14.0rc1-macos-aarch64-none", false)]
    #[case("3.14", "cpython-3.14.0rc1-macos-aarch64-none", true)]
    #[case("3.14.0", "cpython-3.14.0-macos-aarch64-none", true)]
    fn removed_by_cases(#[case] request: &str, #[case] key: &str, #[case] removed: bool) {
        assert_eq!(
            removed_by(key, &PythonVersion::parse(request).unwrap()),
            removed
        );
    }

    /// Only real install directories with a patch version count: not the
    /// minor-version links, not uv's own files. Fails if a link or `.lock`
    /// is taken for an install.
    #[cfg(unix)]
    #[test]
    fn installed_keys_lists_patch_installs_only() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(P14)).unwrap();
        std::fs::create_dir_all(dir.path().join(FT13)).unwrap();
        std::fs::create_dir_all(dir.path().join(".temp")).unwrap();
        std::fs::write(dir.path().join(".lock"), "").unwrap();
        std::os::unix::fs::symlink(dir.path().join(P14), dir.path().join(LINK)).unwrap();
        // A real directory named like a link (a Windows junction is not a
        // symlink) is still not an install.
        std::fs::create_dir_all(dir.path().join("cpython-3.13-macos-aarch64-none")).unwrap();
        let mut keys = installed_keys(dir.path());
        keys.sort();
        assert_eq!(keys, [FT13, P14]);
    }

    const LINK: &str = "cpython-3.12-macos-aarch64-none";
    const P14: &str = "cpython-3.12.14-macos-aarch64-none";
    const P13: &str = "cpython-3.12.13-macos-aarch64-none";
    const FT13: &str = "cpython-3.12.13+freethreaded-macos-aarch64-none";
    const FT_LINK: &str = "cpython-3.12+freethreaded-macos-aarch64-none";

    /// Which envs lose their Python, by what their interpreter links to.
    /// Fails if a pinned removed install is missed, if a minor link is kept
    /// with no compatible install left (or taken with one left), if another
    /// build (free-threaded) is counted as taking over, or if an env outside
    /// uv's installs, or on a variant's own link, is touched.
    #[rstest::rstest]
    #[case::pinned_to_removed(Some(P14), &[P14], &[P13], true)]
    #[case::pinned_to_kept(Some(P13), &[P14], &[P13], false)]
    #[case::minor_link_nothing_left(Some(LINK), &[P14], &[], true)]
    #[case::minor_link_other_patch_left(Some(LINK), &[P14], &[P13], false)]
    #[case::minor_link_only_freethreaded_left(Some(LINK), &[P14], &[FT13], true)]
    #[case::freethreaded_link_untouched(Some(FT_LINK), &[P14], &[FT13], false)]
    #[case::outside_uv(None, &[P14, P13], &[], false)]
    fn affected_envs_cases(
        #[case] linked: Option<&str>,
        #[case] removed: &[&str],
        #[case] remaining: &[&str],
        #[case] affected: bool,
    ) {
        let removed: Vec<String> = removed.iter().map(|k| k.to_string()).collect();
        let remaining: Vec<String> = remaining.iter().map(|k| k.to_string()).collect();
        let got = affected_envs(
            [("web".to_string(), linked.map(str::to_string))],
            &removed,
            &remaining,
        );
        assert_eq!(got == ["web"], affected, "{got:?}");
    }

    /// The install an env's `home` lies in: through a symlinked minor link,
    /// a Windows `home` without `bin`, and an older `<key>/install/bin`
    /// layout. Fails if a `home` outside the install dir (Homebrew, a
    /// --python-path interpreter) yields a name, or one of the layouts is
    /// not recognised.
    #[cfg(unix)]
    #[test]
    fn linked_install_resolves_each_layout() {
        let root = tempfile::tempdir().unwrap();
        let installs = root.path().join("py");
        std::fs::create_dir_all(installs.join(P14).join("bin")).unwrap();
        std::fs::create_dir_all(installs.join(P13).join("install").join("bin")).unwrap();
        std::os::unix::fs::symlink(installs.join(P14), installs.join(LINK)).unwrap();
        let at = |p: &Path| linked_install(p, &installs);
        assert_eq!(at(&installs.join(LINK).join("bin")).as_deref(), Some(LINK));
        assert_eq!(at(&installs.join(P14).join("bin")).as_deref(), Some(P14));
        assert_eq!(at(&installs.join(P14)).as_deref(), Some(P14));
        assert_eq!(
            at(&installs.join(P13).join("install").join("bin")).as_deref(),
            Some(P13)
        );
        assert_eq!(at(Path::new("/opt/homebrew/opt/python@3.12/bin")), None);
    }

    /// `pyvenv.cfg`'s `home` line, or `None` without the file.
    #[test]
    fn env_home_reads_the_home_line() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(env_home(dir.path()), None);
        std::fs::write(
            dir.path().join("pyvenv.cfg"),
            "version_info = 3.12\nhome = /x/bin\n",
        )
        .unwrap();
        assert_eq!(env_home(dir.path()), Some(PathBuf::from("/x/bin")));
    }

    /// How an env's `home` is laid out under the fake uv's install dir.
    #[cfg(unix)]
    enum Home {
        /// uv's minor-version link, a symlink to this install.
        LinkTo(&'static str),
        /// This install's own directory, not the link.
        Install(&'static str),
        /// A minor-version link that is a real directory: it survives the
        /// fake uninstall, as when uv relinks it to another patch.
        LinkThatSurvives,
        /// Outside uv's installs (Homebrew).
        Outside,
    }

    /// A cascade run against a fake uv with `installs` as install
    /// directories and envs `(name, home)`. The fake uninstall removes the
    /// requested default build's directory. Returns which envs still exist,
    /// what uv was asked to uninstall, and the command's result.
    #[cfg(unix)]
    fn cascade_run(
        request: &str,
        installs: &[&str],
        envs: &[(&str, Home)],
        uninstall_fails: bool,
    ) -> (Vec<String>, Vec<String>, Result<()>) {
        use crate::test_utils::{FakeUv, env_guard};
        let home = tempfile::tempdir().unwrap();
        let uv = FakeUv::new(&[]);
        let path = uv.path_var();
        let _env = env_guard(&[
            (
                crate::paths::SCUV_HOME_ENV,
                Some(home.path().to_str().unwrap()),
            ),
            ("PATH", Some(path.as_str())),
            ("FAKE_UV_UNINSTALL_FAILS", uninstall_fails.then_some("1")),
        ]);
        let py = uv.python_dir();
        for key in installs {
            std::fs::create_dir_all(py.join(key).join("bin")).unwrap();
        }
        for (name, layout) in envs {
            let target = match layout {
                Home::LinkTo(key) => {
                    let link = py.join(LINK);
                    if !link.exists() {
                        std::os::unix::fs::symlink(py.join(key), &link).unwrap();
                    }
                    link.join("bin")
                }
                Home::Install(key) => py.join(key).join("bin"),
                Home::LinkThatSurvives => {
                    std::fs::create_dir_all(py.join(LINK).join("bin")).unwrap();
                    py.join(LINK).join("bin")
                }
                Home::Outside => PathBuf::from("/opt/homebrew/opt/python@3.12/bin"),
            };
            let env = home.path().join("virtualenvs").join(name);
            std::fs::create_dir_all(env.join("bin")).unwrap();
            std::fs::write(
                env.join("pyvenv.cfg"),
                format!("home = {}\nversion_info = 3.12\n", target.display()),
            )
            .unwrap();
        }

        let out = Output::new(0, true, crate::output::Colors::NONE, false);
        let result = execute(&out, request, true, true);
        let left = envs
            .iter()
            .map(|(n, _)| n.to_string())
            .filter(|n| home.path().join("virtualenvs").join(n).exists())
            .collect();
        (left, uv.uninstalled(), result)
    }

    /// A minor request takes every patch of that minor: the env on the minor
    /// link and the env on a patch install both lose their Python, although
    /// two patches were installed. Fails if a remaining patch is counted as
    /// keeping the link for a minor request, or the minor request path stops
    /// removing envs (or the fake stops removing every patch it names).
    #[cfg(unix)]
    #[test]
    #[serial_test::serial]
    fn cascade_on_a_minor_request_removes_the_env_every_patch_served() {
        let (left, uninstalled, result) = cascade_run(
            "3.12",
            &[P14, P13],
            &[("web", Home::LinkTo(P14)), ("pinned", Home::Install(P13))],
            false,
        );
        result.unwrap();
        assert!(left.is_empty(), "no 3.12 is left for either env: {left:?}");
        assert_eq!(uninstalled, ["3.12"]);
    }

    /// An env on uv's 3.12 minor link, with 3.12.14 the only 3.12 install:
    /// uninstalling 3.12.14 breaks it, so --cascade removes it. Fails if the
    /// cascade matches by recorded version (`3.12` vs `3.12.14`: the bug —
    /// the Python went and the broken env stayed).
    #[cfg(unix)]
    #[test]
    #[serial_test::serial]
    fn cascade_removes_a_minor_link_env_when_no_install_takes_over() {
        let (left, uninstalled, result) =
            cascade_run("3.12.14", &[P14], &[("web", Home::LinkTo(P14))], false);
        result.unwrap();
        assert!(left.is_empty(), "the env lost its only Python and must go");
        assert_eq!(uninstalled, ["3.12.14"]);
    }

    /// The same env with 3.12.13 also installed: it is not even planned for
    /// removal. Fails if remaining installs are not considered.
    #[cfg(unix)]
    #[test]
    #[serial_test::serial]
    fn cascade_keeps_a_minor_link_env_another_patch_takes_over() {
        let (left, _, result) =
            cascade_run("3.12.14", &[P13, P14], &[("web", Home::LinkTo(P14))], false);
        result.unwrap();
        assert_eq!(left, ["web"]);
    }

    /// Planned for removal (3.12.14 was the only install when the plan was
    /// made), but its interpreter is still there after the uninstall, as when
    /// a patch installed meanwhile took the link over: the env is kept. Fails
    /// if the plan is not checked again after uv runs.
    #[cfg(unix)]
    #[test]
    #[serial_test::serial]
    fn cascade_keeps_a_planned_env_whose_interpreter_survived() {
        let (left, uninstalled, result) =
            cascade_run("3.12.14", &[P14], &[("web", Home::LinkThatSurvives)], false);
        result.unwrap();
        assert_eq!(uninstalled, ["3.12.14"]);
        assert_eq!(left, ["web"]);
    }

    /// An env on a Python uv does not manage (Homebrew) is never removed by
    /// uninstalling a uv Python. Fails if the cascade goes by recorded
    /// version instead of the env's link.
    #[cfg(unix)]
    #[test]
    #[serial_test::serial]
    fn cascade_keeps_an_env_on_a_python_uv_does_not_manage() {
        let (left, _, result) = cascade_run("3.12", &[P14], &[("brew", Home::Outside)], false);
        result.unwrap();
        assert_eq!(left, ["brew"]);
    }

    /// When uv fails to uninstall, no env has been removed. Fails if envs
    /// are deleted before the Python uninstall succeeds.
    #[cfg(unix)]
    #[test]
    #[serial_test::serial]
    fn cascade_removes_nothing_when_the_uninstall_fails() {
        let (left, uninstalled, result) =
            cascade_run("3.12.14", &[P14], &[("web", Home::LinkTo(P14))], true);
        assert!(result.is_err());
        assert_eq!(left, ["web"]);
        assert!(uninstalled.is_empty());
    }

    /// One env that cannot be removed (a name `delete` refuses) does not stop
    /// the others, and the command still completes. Fails if the first
    /// failure aborts the rest.
    #[cfg(unix)]
    #[test]
    #[serial_test::serial]
    fn cascade_goes_on_past_an_env_it_cannot_remove() {
        let (left, _, result) = cascade_run(
            "3.12.14",
            &[P14],
            &[("1bad", Home::LinkTo(P14)), ("web", Home::LinkTo(P14))],
            false,
        );
        assert!(
            matches!(
                result,
                Err(ScoopError::CascadeIncomplete { failed_count: 1 })
            ),
            "a failed removal must not pass as a success: {result:?}"
        );
        assert_eq!(left, ["1bad"], "web goes; 1bad fails and is reported");
    }

    /// One to three numbers are a plain request. Fails if a suffix (`rc1`,
    /// `t`), an implementation prefix, an empty part or a fourth part passes.
    #[rstest::rstest]
    #[case("3.12.14", true)]
    #[case("3.12", true)]
    #[case("3.12.0rc1", false)]
    #[case("3.13t", false)]
    #[case("cpython@3.12", false)]
    #[case("3..12", false)]
    #[case("3.12.14.1", false)]
    #[case("3", true)]
    fn is_plain_version_cases(#[case] version: &str, #[case] plain: bool) {
        assert_eq!(is_plain_version(version), plain);
    }

    /// A request that names more than numbers is refused before anything
    /// is removed: `3.13t` would otherwise match a GIL `3.13` env, and
    /// `3.12.0rc1` a stable `3.12.0` one. Fails if --cascade proceeds for a
    /// non-plain request, or refuses only after looking for uv or removing
    /// an env.
    #[cfg(unix)]
    #[rstest::rstest]
    #[case("3.13t")]
    #[case("3.12.0rc1")]
    #[case("cpython@3.12")]
    #[serial_test::serial]
    fn cascade_refuses_a_request_that_is_not_a_plain_version(#[case] request: &str) {
        use crate::test_utils::env_guard;
        // No uv on PATH and an env the numbers would match: the refusal must
        // come before the uv lookup and before any env is removed.
        let home = tempfile::tempdir().unwrap();
        let no_uv = tempfile::tempdir().unwrap();
        let _env = env_guard(&[
            (
                crate::paths::SCUV_HOME_ENV,
                Some(home.path().to_str().unwrap()),
            ),
            ("PATH", Some(no_uv.path().to_str().unwrap())),
        ]);
        let env = home.path().join("virtualenvs").join("gil");
        std::fs::create_dir_all(env.join("bin")).unwrap();
        std::fs::write(
            env.join(crate::core::Metadata::FILE_NAME),
            r#"{"name":"gil","python_version":"3.13","created_at":"2026-01-01T00:00:00Z","created_by":"test","uv_version":null,"python_path":null}"#,
        )
        .unwrap();
        let out = Output::new(0, true, crate::output::Colors::NONE, false);
        let err = execute(&out, request, true, true).unwrap_err();
        assert!(matches!(err, ScoopError::InvalidArgument { .. }), "{err:?}");
        assert!(env.exists(), "nothing may be removed");
    }

    /// The refusal is about --cascade only: a plain uninstall of the same
    /// request still goes to uv. Fails if the check runs without --cascade.
    #[cfg(unix)]
    #[test]
    #[serial_test::serial]
    fn uninstall_without_cascade_passes_any_request_to_uv() {
        use crate::test_utils::{FakeUv, env_guard};
        let home = tempfile::tempdir().unwrap();
        let uv = FakeUv::new(&["3.12.0"]);
        let path = uv.path_var();
        let _env = env_guard(&[
            (
                crate::paths::SCUV_HOME_ENV,
                Some(home.path().to_str().unwrap()),
            ),
            ("PATH", Some(path.as_str())),
        ]);
        let out = Output::new(0, true, crate::output::Colors::NONE, false);
        execute(&out, "cpython@3.12", false, false).unwrap();
        assert_eq!(uv.uninstalled(), ["cpython@3.12"]);
    }

    #[test]
    fn cascade_aborted_error_code() {
        let err = ScoopError::CascadeAborted;
        assert_eq!(err.code(), "UNINSTALL_CASCADE_ABORTED");
    }

    #[test]
    fn cascade_aborted_has_no_suggestion() {
        let err = ScoopError::CascadeAborted;
        assert!(err.suggestion().is_none());
    }
}
