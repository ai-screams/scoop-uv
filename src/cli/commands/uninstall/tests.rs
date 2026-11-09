//! Unit tests for `cli::commands::uninstall`.

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
        unverified_envs: Vec::new(),
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
        unverified_envs: vec![UnverifiedEnv {
            name: "nocfg".to_string(),
            reason: "no pyvenv.cfg".to_string(),
        }],
    };
    let err = ScoopError::CascadeIncomplete { failed_count: 1 };
    let json: serde_json::Value = serde_json::from_str(&incomplete_json(&err, &data)).unwrap();
    assert_eq!(json["status"], "error");
    assert_eq!(json["command"], "uninstall");
    assert_eq!(json["error"]["code"], "UNINSTALL_CASCADE_INCOMPLETE");
    assert_eq!(json["data"]["removed_envs"][0], "web");
    assert_eq!(json["data"]["failed_envs"][0]["name"], "1bad");
    assert_eq!(json["data"]["unverified_envs"][0]["name"], "nocfg");
}

/// `unverified_envs` is left out when empty, so a success without one
/// keeps the earlier shape. Fails if it is always written.
#[test]
fn uninstall_data_json_omits_empty_unverified_envs() {
    let data = UninstallData {
        version: "3.12".to_string(),
        removed_envs: Some(Vec::new()),
        failed_envs: Vec::new(),
        unverified_envs: Vec::new(),
    };
    let json = serde_json::to_value(&data).unwrap();
    assert!(json.get("unverified_envs").is_none(), "{json}");
}

/// The reason JSON carries is English whatever the user's language;
/// the warning's is translated. Fails if either side switches.
#[test]
#[serial_test::serial]
fn unverified_reason_is_english_for_json_and_localized_for_the_warning() {
    let _locale = crate::test_utils::LocaleGuard::capture();
    rust_i18n::set_locale("ko");
    let (env, localized) = HomeUnknown::NoConfig.into_unverified("x".to_string());
    assert_eq!(env.reason, "no pyvenv.cfg");
    assert_eq!(localized, "pyvenv.cfg 없음");
    assert_eq!(HomeUnknown::NoConfig.reason("ko"), "pyvenv.cfg 없음");
    assert_eq!(
        HomeUnknown::DirUnreadable("e".to_string()).reason("en"),
        "cannot read the environment directory: e"
    );
}

/// The snapshot reads the env directory's identity before its `home`,
/// and an env whose directory cannot be read is not planned. Fails if
/// an unreadable directory yields a snapshot.
#[cfg(unix)]
#[test]
fn snapshot_reports_an_unreadable_env_directory() {
    let root = tempfile::tempdir().unwrap();
    let blocker = root.path().join("blocker");
    std::fs::write(&blocker, "").unwrap();
    assert!(matches!(
        snapshot(&blocker.join("web")),
        Err(HomeUnknown::DirUnreadable(_))
    ));
    let env = planned_env(root.path(), "web", &root.path().join("py"));
    assert_eq!(
        snapshot(&env.path).unwrap(),
        (env.identity, env.home.clone())
    );
}

#[test]
fn uninstall_data_json_has_correct_field_name() {
    let data = UninstallData {
        version: "3.12.0".to_string(),
        removed_envs: None,
        failed_envs: Vec::new(),
        unverified_envs: Vec::new(),
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
        unverified_envs: Vec::new(),
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
        unverified_envs: Vec::new(),
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
        unverified_envs: Vec::new(),
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
        unverified_envs: Vec::new(),
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
        unverified_envs: Vec::new(),
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
            unverified_envs: Vec::new(),
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
            unverified_envs: Vec::new(),
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
        unverified_envs: Vec::new(),
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
#[serial_test::serial]
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
        removed_by(key, &PythonVersion::parse(request).unwrap(), true),
        removed
    );
}

/// Before uv 0.9.1 a patch request takes that patch's pre-releases too.
/// Fails if the pre-release is left out of the plan regardless of uv.
#[test]
fn removed_by_counts_prereleases_for_an_older_uv() {
    let filter = PythonVersion::parse("3.14.0").unwrap();
    assert!(removed_by(
        "cpython-3.14.0rc1-macos-aarch64-none",
        &filter,
        false
    ));
}

/// Fails if the cut-over moves off 0.9.1, or an unknown version is
/// taken to leave pre-releases.
#[rstest::rstest]
#[case(Some("uv 0.9.1"), true)]
#[case(Some("uv 0.12.22 (Homebrew 2026-05-21 aarch64-apple-darwin)"), true)]
#[case(Some("uv 0.9.0"), false)]
#[case(Some("uv 0.5.19"), false)]
#[case(Some("garbage"), false)]
#[case(None, false)]
fn patch_request_skips_prereleases_cases(#[case] raw: Option<&str>, #[case] skips: bool) {
    assert_eq!(patch_request_skips_prereleases(raw), skips);
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
    let mut keys = installed_keys(dir.path()).unwrap();
    keys.sort();
    assert_eq!(keys, [FT13, P14]);
    // Not created yet: nothing installed. Unreadable: an error, not an
    // empty list that would leave envs out of the plan.
    assert_eq!(
        installed_keys(&dir.path().join("none")).unwrap(),
        Vec::<String>::new()
    );
    assert!(installed_keys(&dir.path().join(".lock")).is_err());
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
    let root = std::fs::canonicalize(&installs).unwrap();
    let at = |p: &Path| linked_install(p, &root);
    assert_eq!(at(&installs.join(LINK).join("bin")).as_deref(), Some(LINK));
    assert_eq!(at(&installs.join(P14).join("bin")).as_deref(), Some(P14));
    assert_eq!(at(&installs.join(P14)).as_deref(), Some(P14));
    assert_eq!(
        at(&installs.join(P13).join("install").join("bin")).as_deref(),
        Some(P13)
    );
    assert_eq!(at(Path::new("/opt/homebrew/opt/python@3.12/bin")), None);
}

/// `pyvenv.cfg`'s `home` line, or why it cannot be read. Fails if a
/// missing file, an unreadable one (here a directory) and a file without
/// the line stop being told apart, or the line stops being found.
#[test]
fn env_home_reads_the_home_line() {
    let dir = tempfile::tempdir().unwrap();
    assert_eq!(env_home(dir.path()), Err(HomeUnknown::NoConfig));
    let cfg = dir.path().join("pyvenv.cfg");
    std::fs::create_dir(&cfg).unwrap();
    assert!(matches!(
        env_home(dir.path()),
        Err(HomeUnknown::Unreadable(_))
    ));
    std::fs::remove_dir(&cfg).unwrap();
    std::fs::write(&cfg, "version_info = 3.12\n").unwrap();
    assert_eq!(env_home(dir.path()), Err(HomeUnknown::NoHomeLine));
    std::fs::write(&cfg, "home =\n").unwrap();
    assert_eq!(
        env_home(dir.path()),
        Err(HomeUnknown::NoHomeLine),
        "empty home"
    );
    std::fs::write(&cfg, "version_info = 3.12\nhome = /x/bin\n").unwrap();
    assert_eq!(env_home(dir.path()), Ok(PathBuf::from("/x/bin")));
}

/// A planned env `envs/<name>`, linked to a `home` under `py` that
/// exists until the test removes it.
#[cfg(unix)]
fn planned_env(envs: &Path, name: &str, py: &Path) -> Planned {
    let path = envs.join(name);
    let home = py.join(P14).join("bin");
    std::fs::create_dir_all(&path).unwrap();
    std::fs::create_dir_all(&home).unwrap();
    std::fs::write(
        path.join("pyvenv.cfg"),
        format!("home = {}\n", home.display()),
    )
    .unwrap();
    Planned {
        name: name.to_string(),
        identity: dir_identity(&path).unwrap(),
        path,
        home,
        home_existed: true,
    }
}

/// Each answer of the recheck. Fails if a gone interpreter is not
/// removed, a working one is, or a replaced or unreadable env is
/// removed instead of kept.
#[cfg(unix)]
#[test]
fn recheck_removes_only_the_same_env_whose_interpreter_is_gone() {
    let root = tempfile::tempdir().unwrap();
    let env = planned_env(root.path(), "web", &root.path().join("py"));
    assert_eq!(recheck(&env), Recheck::Works);
    std::fs::remove_dir_all(root.path().join("py")).unwrap();
    assert_eq!(recheck(&env), Recheck::Remove);

    // Another env under the name, pointing at the same home.
    let cfg = std::fs::read_to_string(env.path.join("pyvenv.cfg")).unwrap();
    std::fs::rename(&env.path, root.path().join("old")).unwrap();
    std::fs::create_dir_all(&env.path).unwrap();
    std::fs::write(env.path.join("pyvenv.cfg"), &cfg).unwrap();
    assert_eq!(recheck(&env), Recheck::Replaced);

    // Same directory, another home.
    let env = Planned {
        identity: dir_identity(&env.path).unwrap(),
        ..env
    };
    std::fs::write(env.path.join("pyvenv.cfg"), "home = /elsewhere/bin\n").unwrap();
    assert_eq!(recheck(&env), Recheck::Replaced);

    // pyvenv.cfg unreadable (a directory): cannot tell.
    std::fs::remove_file(env.path.join("pyvenv.cfg")).unwrap();
    std::fs::create_dir(env.path.join("pyvenv.cfg")).unwrap();
    assert!(matches!(recheck(&env), Recheck::Unknown(_)));

    std::fs::remove_dir_all(&env.path).unwrap();
    assert_eq!(recheck(&env), Recheck::Gone);
}

/// What `remove_envs` does with each recheck answer: an env replaced
/// meanwhile and one it cannot judge both stay; only the latter is a
/// failure. Fails if either is removed, or the undecided one is not
/// reported.
#[cfg(unix)]
#[test]
#[serial_test::serial]
fn remove_envs_keeps_replaced_and_undecided_envs() {
    use crate::test_utils::env_guard;
    let home = tempfile::tempdir().unwrap();
    let _env = env_guard(&[(
        crate::paths::SCUV_HOME_ENV,
        Some(home.path().to_str().unwrap()),
    )]);
    let envs = home.path().join("virtualenvs");
    let py = home.path().join("py");
    let swap = planned_env(&envs, "swap", &py);
    let odd = planned_env(&envs, "odd", &py);
    std::fs::remove_dir_all(&py).unwrap();
    // "swap" is replaced: another directory under the name, same home.
    std::fs::rename(&swap.path, home.path().join("old")).unwrap();
    std::fs::create_dir_all(&swap.path).unwrap();
    std::fs::copy(
        home.path().join("old").join("pyvenv.cfg"),
        swap.path.join("pyvenv.cfg"),
    )
    .unwrap();
    // "odd" cannot be read: its pyvenv.cfg is a directory.
    std::fs::remove_file(odd.path.join("pyvenv.cfg")).unwrap();
    std::fs::create_dir(odd.path.join("pyvenv.cfg")).unwrap();

    let out = Output::new(0, true, crate::output::Colors::NONE, false);
    let (removed, failed) = remove_envs(&out, &[swap, odd]);
    assert!(removed.is_empty(), "{removed:?}");
    assert_eq!(failed.len(), 1);
    assert_eq!(failed[0].name, "odd");
    assert!(envs.join("swap").exists() && envs.join("odd").exists());
}

/// A pyvenv.cfg that turns unreadable during the recheck is reported in
/// the user's language, like the other removal failures. Fails if the
/// recheck falls back to the English reason.
#[cfg(unix)]
#[test]
#[serial_test::serial]
fn recheck_reports_an_unreadable_cfg_in_the_users_language() {
    let _locale = crate::test_utils::LocaleGuard::capture();
    rust_i18n::set_locale("ko");
    let root = tempfile::tempdir().unwrap();
    let env = planned_env(root.path(), "web", &root.path().join("py"));
    std::fs::remove_file(env.path.join("pyvenv.cfg")).unwrap();
    assert_eq!(
        recheck(&env),
        Recheck::Unknown("pyvenv.cfg 없음".to_string())
    );
}

/// A `home` (or env directory) that cannot be checked is not taken as
/// gone: `Path::exists` says false for it, `try_exists` reports the
/// error. Fails if the recheck reads an error as absence and removes the
/// env.
#[cfg(unix)]
#[test]
fn recheck_keeps_an_env_whose_home_cannot_be_checked() {
    let root = tempfile::tempdir().unwrap();
    let mut env = planned_env(root.path(), "web", &root.path().join("py"));
    // A regular file where a directory of the path should be: stat
    // fails with ENOTDIR, not "not found".
    let blocker = root.path().join("blocker");
    std::fs::write(&blocker, "").unwrap();
    env.home = blocker.join("bin");
    std::fs::write(
        env.path.join("pyvenv.cfg"),
        format!("home = {}\n", env.home.display()),
    )
    .unwrap();
    assert!(!env.home.exists(), "exists() would read it as gone");
    assert!(matches!(recheck(&env), Recheck::Unknown(_)));
    // The same for the env directory itself: an error is not "gone".
    env.path = blocker.join("web");
    assert!(matches!(recheck(&env), Recheck::Unknown(_)));
}

/// Only a request that names the patch needs the uv version. Fails if a
/// minor request asks uv for it, or a patch request stops asking.
#[cfg(unix)]
#[rstest::rstest]
#[case("3.12", 0)]
#[case("3.12.14", 1)]
#[serial_test::serial]
fn cascade_asks_the_uv_version_only_for_a_patch_request(
    #[case] request: &str,
    #[case] calls: usize,
) {
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
    ]);
    let out = Output::new(0, true, crate::output::Colors::NONE, false);
    execute(&out, request, true, true).unwrap();
    assert_eq!(uv.version_calls(), calls);
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
    uninstall_fails: Option<&str>,
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
        ("FAKE_UV_UNINSTALL_FAILS", uninstall_fails),
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
        None,
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
        cascade_run("3.12.14", &[P14], &[("web", Home::LinkTo(P14))], None);
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
        cascade_run("3.12.14", &[P13, P14], &[("web", Home::LinkTo(P14))], None);
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
        cascade_run("3.12.14", &[P14], &[("web", Home::LinkThatSurvives)], None);
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
    let (left, _, result) = cascade_run("3.12", &[P14], &[("brew", Home::Outside)], None);
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
        cascade_run("3.12.14", &[P14], &[("web", Home::LinkTo(P14))], Some("1"));
    assert!(result.is_err());
    assert_eq!(left, ["web"]);
    assert!(uninstalled.is_empty());
}

/// uv removed the install and then failed (on another one, say): the env
/// it served is broken, so it still goes, and the uv error is returned.
/// Fails if a uv failure skips the cleanup of envs already broken.
#[cfg(unix)]
#[test]
#[serial_test::serial]
fn cascade_cleans_up_after_a_uv_that_removed_then_failed() {
    let (left, uninstalled, result) = cascade_run(
        "3.12.14",
        &[P14],
        &[("web", Home::LinkTo(P14)), ("brew", Home::Outside)],
        Some("after"),
    );
    assert!(result.is_err());
    assert_eq!(left, ["brew"]);
    assert_eq!(uninstalled, ["3.12.14"]);
}

/// An env already broken before uv ran (its link points at an install
/// that is not there) is not removed when uv then fails without removing
/// anything: uv did not break it. Fails if the cleanup after a uv failure
/// takes every env whose home is missing.
#[cfg(unix)]
#[test]
#[serial_test::serial]
fn cascade_keeps_an_already_broken_env_when_uv_fails() {
    let (left, _, result) =
        cascade_run("3.12.14", &[P14], &[("old", Home::LinkTo(P13))], Some("1"));
    assert!(result.is_err());
    assert_eq!(left, ["old"]);
}

/// A home that could not be checked when planned is not taken as
/// already broken: after uv removed it and failed, the env still goes.
/// Fails if a planning-time check error drops the env from the cleanup.
#[cfg(unix)]
#[test]
#[serial_test::serial]
fn cascade_cleans_up_an_env_whose_home_could_not_be_checked() {
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
        ("FAKE_UV_UNINSTALL_FAILS", Some("after")),
    ]);
    // The install is a regular file: `<install>/bin` fails to stat with
    // ENOTDIR when planned, then is gone once uv has run.
    std::fs::write(uv.python_dir().join(P14), "").unwrap();
    let env = home.path().join("virtualenvs").join("web");
    std::fs::create_dir_all(&env).unwrap();
    std::fs::write(
        env.join("pyvenv.cfg"),
        format!(
            "home = {}\n",
            uv.python_dir().join(P14).join("bin").display()
        ),
    )
    .unwrap();
    let out = Output::new(0, true, crate::output::Colors::NONE, false);
    assert!(execute(&out, "3.12.14", true, true).is_err());
    assert!(!env.exists(), "uv broke it, so it goes");
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
        None,
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
