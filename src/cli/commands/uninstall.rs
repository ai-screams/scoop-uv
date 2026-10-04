//! Uninstall command

use std::io::IsTerminal;

use dialoguer::Confirm;
use rust_i18n::t;

use crate::core::VirtualenvService;
use crate::error::{Result, ScoopError};
use crate::output::{Output, UninstallData};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::uv::{ManagedInstall, UvClient};
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
    let doomed = if cascade {
        Some(plan_cascade(output, &uv, version, force)?)
    } else {
        None
    };

    output.info(&t!("uninstall.uninstalling", version = version));

    uv.uninstall_python(version)?;

    let removed_envs = match doomed {
        Some(names) => Some(remove_envs(output, &names)?),
        None => None,
    };

    // JSON output
    if output.is_json() {
        output.json_success(
            "uninstall",
            UninstallData {
                version: version.to_string(),
                removed_envs,
            },
        );
        return Ok(());
    }

    output.success(&t!("uninstall.success", version = version));

    Ok(())
}

/// Work out which envs the uninstall takes the Python away from, list them
/// and ask for confirmation (unless `--force` or `--json`).
///
/// Returns the env names to remove once the Python is gone.
fn plan_cascade(output: &Output, uv: &UvClient, version: &str, force: bool) -> Result<Vec<String>> {
    let service = VirtualenvService::auto()?;
    let envs = service.list()?;

    let matching_envs = match PythonVersion::parse(version) {
        Some(filter) => {
            let (removed, remaining): (Vec<ManagedInstall>, Vec<ManagedInstall>) =
                uv.list_managed_installs()?.into_iter().partition(|i| {
                    PythonVersion::parse(&i.version).is_some_and(|v| filter.matches(&v))
                });
            let install_dir = uv.python_dir()?;
            affected_envs(
                envs.into_iter()
                    .map(|e| (e.name, linked_install(&e.path, &install_dir))),
                &removed,
                &remaining,
            )
        }
        None => Vec::new(),
    };

    // No matching environments
    if matching_envs.is_empty() {
        if !output.is_json() {
            output.info(&t!("uninstall.cascade_none", version = version));
        }
        return Ok(vec![]);
    }

    // Show matching environments and confirm (unless --force or --json)
    if !force && !output.is_json() {
        output.info(&t!(
            "uninstall.cascade_found",
            count = matching_envs.len(),
            version = version
        ));
        for name in &matching_envs {
            output.info(&t!("uninstall.cascade_env", name = name));
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

    Ok(matching_envs)
}

/// Remove the envs a cascade planned to remove.
fn remove_envs(output: &Output, names: &[String]) -> Result<Vec<String>> {
    let service = VirtualenvService::auto()?;
    let mut removed = Vec::new();
    for name in names {
        if !output.is_json() {
            output.info(&t!("uninstall.cascade_removing", name = name));
        }
        service.delete(name)?;
        removed.push(name.clone());
    }

    if !output.is_json() && !removed.is_empty() {
        output.info(&t!("uninstall.cascade_removed", count = removed.len()));
    }

    Ok(removed)
}

/// The uv-managed install an env's interpreter comes from: the directory
/// name under `install_dir` that the `home` line of its `pyvenv.cfg` points
/// into (`cpython-3.12.14-…` for a patch install, `cpython-3.12-…` for uv's
/// minor-version link). `None` when the env uses a Python outside uv's
/// install directory (Homebrew, a `--python-path` interpreter) or its
/// `pyvenv.cfg` cannot be read: uninstalling a uv Python cannot break it, or
/// there is no telling.
fn linked_install(env_path: &Path, install_dir: &Path) -> Option<String> {
    let cfg = std::fs::read_to_string(env_path.join("pyvenv.cfg")).ok()?;
    let home = cfg.lines().find_map(|line| {
        let (key, value) = line.split_once('=')?;
        (key.trim() == "home").then(|| PathBuf::from(value.trim()))
    })?;
    // POSIX homes end in `bin`; on Windows the interpreter sits in the
    // install directory itself.
    let entry = if home.file_name().is_some_and(|n| n == "bin") {
        home.parent()?.to_path_buf()
    } else {
        home
    };
    // Compare real paths (`/tmp` vs `/private/tmp`); the entry itself may be
    // the minor-version symlink, so resolve only its parent.
    let parent = std::fs::canonicalize(entry.parent()?).ok()?;
    if parent != std::fs::canonicalize(install_dir).ok()? {
        return None;
    }
    Some(entry.file_name()?.to_string_lossy().into_owned())
}

/// The name of uv's minor-version link for an install key: the patch dropped
/// from the version part, any variant kept (`cpython-3.12.14-macos-…` →
/// `cpython-3.12-macos-…`, `cpython-3.13.1+freethreaded-…` →
/// `cpython-3.13+freethreaded-…`).
fn minor_link_name(key: &str) -> Option<String> {
    let mut parts: Vec<&str> = key.split('-').collect();
    let version = parts.get(1)?;
    let (numbers, variant) = match version.split_once('+') {
        Some((n, v)) => (n, Some(v)),
        None => (*version, None),
    };
    let minor: Vec<&str> = numbers.split('.').take(2).collect();
    if minor.len() < 2 {
        return None;
    }
    let mut minor = minor.join(".");
    if let Some(variant) = variant {
        minor = format!("{minor}+{variant}");
    }
    parts[1] = &minor;
    Some(parts.join("-"))
}

/// The envs an uninstall takes the Python away from, given what each env's
/// interpreter is linked to (see [`linked_install`]).
///
/// An env linked to a removed install directly is affected. An env linked
/// to uv's minor-version link is affected only when no remaining install of
/// the same implementation, variant and platform takes over that link (uv
/// points it at the newest such install left). Envs linked to nothing uv
/// manages, or to an install that stays, are not.
fn affected_envs(
    envs: impl IntoIterator<Item = (String, Option<String>)>,
    removed: &[ManagedInstall],
    remaining: &[ManagedInstall],
) -> Vec<String> {
    let removed_keys: HashSet<&str> = removed.iter().map(|i| i.key.as_str()).collect();
    let removed_links: HashSet<String> = removed
        .iter()
        .filter_map(|i| minor_link_name(&i.key))
        .collect();
    let kept_links: HashSet<String> = remaining
        .iter()
        .filter_map(|i| minor_link_name(&i.key))
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

    #[test]
    fn uninstall_data_json_has_correct_field_name() {
        let data = UninstallData {
            version: "3.12.0".to_string(),
            removed_envs: None,
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

    fn install(key: &str) -> ManagedInstall {
        let version = key.split('-').nth(1).unwrap().split('+').next().unwrap();
        ManagedInstall {
            key: key.to_string(),
            version: version.to_string(),
        }
    }

    /// uv's minor-version link name for an install key. Fails if the patch
    /// is not dropped, the variant or platform is lost, or a key without a
    /// minor version yields a name.
    #[rstest::rstest]
    #[case(
        "cpython-3.12.14-macos-aarch64-none",
        Some("cpython-3.12-macos-aarch64-none")
    )]
    #[case(
        "cpython-3.13.1+freethreaded-linux-x86_64-gnu",
        Some("cpython-3.13+freethreaded-linux-x86_64-gnu")
    )]
    #[case(
        "pypy-3.10.14-macos-aarch64-none",
        Some("pypy-3.10-macos-aarch64-none")
    )]
    #[case("cpython-3-macos-aarch64-none", None)]
    #[case("garbage", None)]
    fn minor_link_name_cases(#[case] key: &str, #[case] link: Option<&str>) {
        assert_eq!(minor_link_name(key).as_deref(), link);
    }

    const LINK: &str = "cpython-3.12-macos-aarch64-none";
    const P14: &str = "cpython-3.12.14-macos-aarch64-none";
    const P13: &str = "cpython-3.12.13-macos-aarch64-none";
    const FT13: &str = "cpython-3.12.13+freethreaded-macos-aarch64-none";
    const PYPY: &str = "pypy-3.12.9-macos-aarch64-none";

    /// Which envs lose their Python, by what their interpreter links to.
    /// Fails if a pinned removed install is missed, if a minor link is kept
    /// with no compatible install left (or taken with one left), if another
    /// build (free-threaded, PyPy) is counted as taking over, or if an env
    /// outside uv's installs is touched.
    #[rstest::rstest]
    #[case::pinned_to_removed(Some(P14), &[P14], &[P13], true)]
    #[case::pinned_to_kept(Some(P13), &[P14], &[P13], false)]
    #[case::minor_link_nothing_left(Some(LINK), &[P14], &[], true)]
    #[case::minor_link_other_patch_left(Some(LINK), &[P14], &[P13], false)]
    #[case::minor_link_only_freethreaded_left(Some(LINK), &[P14], &[FT13], true)]
    #[case::minor_link_only_pypy_left(Some(LINK), &[P14], &[PYPY], true)]
    #[case::minor_link_untouched(Some(LINK), &[FT13], &[P14], false)]
    #[case::outside_uv(None, &[P14, P13], &[], false)]
    fn affected_envs_cases(
        #[case] linked: Option<&str>,
        #[case] removed: &[&str],
        #[case] remaining: &[&str],
        #[case] affected: bool,
    ) {
        let removed: Vec<_> = removed.iter().map(|k| install(k)).collect();
        let remaining: Vec<_> = remaining.iter().map(|k| install(k)).collect();
        let got = affected_envs(
            [("web".to_string(), linked.map(str::to_string))],
            &removed,
            &remaining,
        );
        assert_eq!(got == ["web"], affected, "{got:?}");
    }

    /// The install an env is linked to: the directory under uv's install dir
    /// its `pyvenv.cfg` `home` points into, through a symlinked minor link
    /// too, and on Windows where `home` is the install directory itself.
    /// Fails if a `home` outside the install dir (Homebrew, a --python-path
    /// interpreter) or a missing `pyvenv.cfg` yields a name, or a `bin`-less
    /// `home` is not recognised.
    #[cfg(unix)]
    #[test]
    fn linked_install_reads_the_home_line() {
        let root = tempfile::tempdir().unwrap();
        let installs = root.path().join("py");
        std::fs::create_dir_all(installs.join(P14).join("bin")).unwrap();
        std::os::unix::fs::symlink(installs.join(P14), installs.join(LINK)).unwrap();
        let env = |name: &str, home: Option<&Path>| {
            let dir = root.path().join(name);
            std::fs::create_dir_all(&dir).unwrap();
            if let Some(home) = home {
                std::fs::write(
                    dir.join("pyvenv.cfg"),
                    format!("home = {}\nversion_info = 3.12\n", home.display()),
                )
                .unwrap();
            }
            dir
        };
        let minor = env("minor", Some(&installs.join(LINK).join("bin")));
        let pinned = env("pinned", Some(&installs.join(P14).join("bin")));
        let brew = env("brew", Some(Path::new("/opt/homebrew/opt/python@3.12/bin")));
        let bare = env("bare", None);
        // Windows: `home` is the install directory itself, no `bin`.
        let windows = env("windows", Some(&installs.join(P14)));
        assert_eq!(linked_install(&windows, &installs).as_deref(), Some(P14));
        assert_eq!(linked_install(&minor, &installs).as_deref(), Some(LINK));
        assert_eq!(linked_install(&pinned, &installs).as_deref(), Some(P14));
        assert_eq!(linked_install(&brew, &installs), None);
        assert_eq!(linked_install(&bare, &installs), None);
    }

    /// A cascade run against a fake uv listing `installed`, with one env
    /// `web` whose `pyvenv.cfg` points at `home_in_installs` under the fake
    /// uv's install dir (or, with `None`, at a Homebrew path). Returns
    /// whether `web` still exists, what uv was asked to uninstall, and the
    /// command's result.
    #[cfg(unix)]
    fn cascade_run(
        request: &str,
        installed: &[&str],
        home_in_installs: Option<&str>,
        uninstall_fails: bool,
    ) -> (bool, Vec<String>, Result<()>) {
        use crate::test_utils::{FakeUv, env_guard};
        let home = tempfile::tempdir().unwrap();
        let uv = FakeUv::new(installed);
        let path = uv.path_var();
        let _env = env_guard(&[
            (
                crate::paths::SCUV_HOME_ENV,
                Some(home.path().to_str().unwrap()),
            ),
            ("PATH", Some(path.as_str())),
            ("FAKE_UV_UNINSTALL_FAILS", uninstall_fails.then_some("1")),
        ]);
        let target = match home_in_installs {
            Some(dir) => {
                let target = uv.python_dir().join(dir).join("bin");
                std::fs::create_dir_all(&target).unwrap();
                target
            }
            None => PathBuf::from("/opt/homebrew/opt/python@3.12/bin"),
        };
        let env = home.path().join("virtualenvs").join("web");
        std::fs::create_dir_all(env.join("bin")).unwrap();
        std::fs::write(
            env.join("pyvenv.cfg"),
            format!("home = {}\nversion_info = 3.12\n", target.display()),
        )
        .unwrap();
        std::fs::write(
            env.join(crate::core::Metadata::FILE_NAME),
            r#"{"name":"web","python_version":"3.12","created_at":"2026-01-01T00:00:00Z","created_by":"test","uv_version":null,"python_path":null}"#,
        )
        .unwrap();

        let out = Output::new(0, true, crate::output::Colors::NONE, false);
        let result = execute(&out, request, true, true);
        (env.exists(), uv.uninstalled(), result)
    }

    /// An env on uv's 3.12 minor link, with 3.12.14 the only 3.12 install:
    /// uninstalling 3.12.14 breaks it, so --cascade removes it. Fails if the
    /// cascade matches by recorded version (`3.12` vs `3.12.14`: the bug —
    /// the Python went and the broken env stayed).
    #[cfg(unix)]
    #[test]
    #[serial_test::serial]
    fn cascade_removes_a_minor_link_env_when_no_install_takes_over() {
        let (left, uninstalled, result) = cascade_run("3.12.14", &["3.12.14"], Some(LINK), false);
        result.unwrap();
        assert!(!left, "the env lost its only Python and must go");
        assert_eq!(uninstalled, ["3.12.14"]);
    }

    /// The same env with 3.12.13 also installed: uv repoints the minor link
    /// at it, the env keeps working and is kept. Fails if remaining installs
    /// are not considered (a working env would be deleted).
    #[cfg(unix)]
    #[test]
    #[serial_test::serial]
    fn cascade_keeps_a_minor_link_env_another_patch_takes_over() {
        let (left, uninstalled, result) =
            cascade_run("3.12.14", &["3.12.13", "3.12.14"], Some(LINK), false);
        result.unwrap();
        assert!(left, "3.12.13 takes over the link");
        assert_eq!(uninstalled, ["3.12.14"]);
    }

    /// An env on a Python outside uv's installs (Homebrew) that also
    /// records `3.12` is never removed by uninstalling a uv Python. Fails
    /// if the cascade goes by recorded version instead of the env's link.
    #[cfg(unix)]
    #[test]
    #[serial_test::serial]
    fn cascade_keeps_an_env_on_a_python_uv_does_not_manage() {
        let (left, _, result) = cascade_run("3.12", &["3.12.14"], None, false);
        result.unwrap();
        assert!(left, "a Homebrew env is not uv's to remove");
    }

    /// When uv fails to uninstall, no env has been removed. Fails if envs
    /// are deleted before the Python uninstall succeeds.
    #[cfg(unix)]
    #[test]
    #[serial_test::serial]
    fn cascade_removes_nothing_when_the_uninstall_fails() {
        let (left, uninstalled, result) = cascade_run("3.12.14", &["3.12.14"], Some(LINK), true);
        assert!(result.is_err());
        assert!(left, "the env must survive a failed uninstall");
        assert!(uninstalled.is_empty());
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
