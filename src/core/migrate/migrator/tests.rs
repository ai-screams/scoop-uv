//! Unit tests for `core::migrate::migrator`.

use super::*;
use crate::core::migrate::source::SourceType;

fn mock_source(name: &str, status: EnvironmentStatus) -> SourceEnvironment {
    SourceEnvironment {
        name: name.to_string(),
        python_version: "3.12.0".to_string(),
        path: PathBuf::from("/mock/path"),
        source_type: SourceType::Pyenv,
        size_bytes: Some(1024),
        status,
    }
}

/// `RollbackGuard` deletes a half-built environment when a migration
/// fails, and `disarm` is what stops it after success. Both halves were
/// untested, which matters more than usual here: this guard is one of the
/// pieces in the `--force` batch race fixed in #168 — an armed guard
/// dropping late can delete an environment another migration just built.
#[test]
fn rollback_guard_removes_the_directory_when_not_disarmed() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("half-built");
    fs::create_dir_all(path.join("bin")).unwrap();

    {
        let _guard = RollbackGuard::new(path.clone());
    } // dropped here, still armed

    assert!(!path.exists(), "an armed guard must clean up on drop");
}

#[test]
fn rollback_guard_leaves_the_directory_once_disarmed() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("finished");
    fs::create_dir_all(path.join("bin")).unwrap();

    {
        let mut guard = RollbackGuard::new(path.clone());
        guard.disarm();
    }

    assert!(
        path.exists(),
        "a disarmed guard must leave the finished environment alone"
    );
}

/// `write_metadata` is what makes a migrated directory recognisable to
/// the rest of scuv; replaced with a bare `Ok(())` the environment ends up
/// without `.scoop-metadata.json` and `scuv list` cannot see it. The uv
/// version lookup inside is `.ok()`, so an unusable UvClient is fine here.
#[test]
fn write_metadata_creates_a_readable_metadata_file() {
    let temp = tempfile::tempdir().unwrap();
    let target = temp.path().join("web");
    fs::create_dir_all(&target).unwrap();

    offline_migrator()
        .write_metadata(&target, "web", "3.12.0")
        .unwrap();

    let raw = fs::read_to_string(target.join(".scoop-metadata.json")).unwrap();
    assert!(raw.contains("\"web\""), "name should be recorded: {raw}");
    assert!(
        raw.contains("3.12.0"),
        "python version should be recorded: {raw}"
    );
}

/// A migrator with a UvClient that is never invoked — enough to reach
/// the filesystem-only paths below without needing uv on PATH.
fn offline_migrator() -> Migrator {
    Migrator::with_uv(crate::uv::UvClient::with_path(PathBuf::from(
        "/nonexistent/uv",
    )))
}

/// `delete_source` guards with `!source.path.exists()` and returns early
/// when the directory is already gone. Dropping the `!` inverts it: a
/// live source is left in place and a missing one is handed to
/// `remove_dir_all`. Both halves are asserted here.
#[test]
fn delete_source_removes_existing_directory() {
    let temp = tempfile::tempdir().unwrap();
    let env_path = temp.path().join("web");
    fs::create_dir_all(env_path.join("bin")).unwrap();

    let source = SourceEnvironment {
        name: "web".to_string(),
        python_version: "3.12.0".to_string(),
        path: env_path.clone(),
        source_type: SourceType::Pyenv,
        size_bytes: None,
        status: EnvironmentStatus::Ready,
    };

    offline_migrator().delete_source(&source).unwrap();
    assert!(!env_path.exists(), "source directory should be gone");
}

#[test]
fn delete_source_is_ok_when_already_gone() {
    let temp = tempfile::tempdir().unwrap();
    let source = SourceEnvironment {
        name: "web".to_string(),
        python_version: "3.12.0".to_string(),
        path: temp.path().join("never-created"),
        source_type: SourceType::Pyenv,
        size_bytes: None,
        status: EnvironmentStatus::Ready,
    };

    assert!(offline_migrator().delete_source(&source).is_ok());
}

#[test]
fn target_path_rejects_path_traversal_name() {
    let migrator = Migrator {
        uv: UvClient::with_path(PathBuf::from("/mock/uv")),
    };
    // Names that escape the virtualenvs dir must be rejected before any
    // filesystem access (validation runs before uv is invoked).
    for evil in ["../../etc/evil", "..", "foo/bar", "/abs", ".hidden"] {
        let result = migrator.target_path(evil, false);
        assert!(
            matches!(result, Err(ScoopError::InvalidEnvName { .. })),
            "name {evil:?} should be rejected as invalid"
        );
    }
}

#[test]
fn test_validate_source_ready() {
    let migrator = Migrator {
        uv: UvClient::with_path(PathBuf::from("/mock/uv")),
    };
    let source = mock_source("test", EnvironmentStatus::Ready);
    let options = MigrateOptions::default();

    assert!(migrator.validate_source(&source, &options).is_ok());
}

/// A name conflict is about the source's name, so migrating under a new
/// name (--rename, --auto-rename) goes ahead; "renaming" to the same
/// name does not, and neither does a renamed env on an EOL Python
/// without --force. Fails if the conflict check ignores `rename_to` (the
/// rename then always fails), accepts a rename to the same name, or lets
/// a rename skip the EOL guard.
#[test]
fn validate_source_lets_a_renamed_conflict_through() {
    let migrator = Migrator {
        uv: UvClient::with_path(PathBuf::from("/mock/uv")),
    };
    let source = mock_source(
        "test",
        EnvironmentStatus::NameConflict {
            existing: PathBuf::from("/home/u/.scuv/virtualenvs/test"),
        },
    );
    let renamed = |to: &str| MigrateOptions {
        rename_to: Some(to.to_string()),
        ..MigrateOptions::default()
    };
    assert!(
        migrator
            .validate_source(&source, &MigrateOptions::default())
            .is_err()
    );
    assert!(
        migrator
            .validate_source(&source, &renamed("test-pyenv"))
            .is_ok()
    );
    assert!(migrator.validate_source(&source, &renamed("test")).is_err());

    // The conflict hid an EOL Python: renaming must not skip that guard.
    // 3.10 is the newest EOL minor, 3.11 the oldest supported one.
    let mut eol = source.clone();
    eol.python_version = "3.10.14".to_string();
    assert!(
        migrator
            .validate_source(&eol, &renamed("test-pyenv"))
            .is_err()
    );
    // Overwrite permission is not EOL permission (it used to be, through
    // the single `force` flag); `allow_eol` is.
    let overwrite = MigrateOptions {
        overwrite: true,
        ..renamed("test-pyenv")
    };
    assert!(migrator.validate_source(&eol, &overwrite).is_err());
    let allowed = MigrateOptions {
        allow_eol: true,
        ..renamed("test-pyenv")
    };
    assert!(migrator.validate_source(&eol, &allowed).is_ok());
    let mut supported = source.clone();
    supported.python_version = "3.11.0".to_string();
    assert!(
        migrator
            .validate_source(&supported, &renamed("test-pyenv"))
            .is_ok()
    );
}

/// `allow_eol` admits an EOL source, including one a conflict hid behind
/// a rename, but never an unrenamed conflict. Fails if `allow_eol` is
/// ignored in either EOL branch, or starts overriding a name conflict.
#[test]
fn validate_source_allow_eol_admits_eol_but_not_conflicts() {
    let migrator = Migrator {
        uv: UvClient::with_path(PathBuf::from("/mock/uv")),
    };
    let allow = MigrateOptions {
        allow_eol: true,
        ..MigrateOptions::default()
    };
    let eol = mock_source(
        "test",
        EnvironmentStatus::PythonEol {
            version: "3.10.14".to_string(),
        },
    );
    assert!(migrator.validate_source(&eol, &allow).is_ok());

    let mut conflict = mock_source(
        "test",
        EnvironmentStatus::NameConflict {
            existing: PathBuf::from("/home/u/.scuv/virtualenvs/test"),
        },
    );
    conflict.python_version = "3.10.14".to_string();
    assert!(matches!(
        migrator.validate_source(&conflict, &allow),
        Err(ScoopError::MigrationNameConflict { .. })
    ));
    let renamed = MigrateOptions {
        rename_to: Some("test-pyenv".to_string()),
        ..allow
    };
    assert!(migrator.validate_source(&conflict, &renamed).is_ok());
}

#[test]
fn test_validate_source_corrupted() {
    let migrator = Migrator {
        uv: UvClient::with_path(PathBuf::from("/mock/uv")),
    };
    let source = mock_source(
        "test",
        EnvironmentStatus::Corrupted {
            reason: "broken".to_string(),
        },
    );
    let options = MigrateOptions::default();

    assert!(migrator.validate_source(&source, &options).is_err());
}

#[test]
fn test_validate_source_name_conflict_with_force() {
    let migrator = Migrator {
        uv: UvClient::with_path(PathBuf::from("/mock/uv")),
    };
    let source = mock_source(
        "test",
        EnvironmentStatus::NameConflict {
            existing: PathBuf::from("/existing"),
        },
    );
    let options = MigrateOptions {
        overwrite: true,
        ..Default::default()
    };

    assert!(migrator.validate_source(&source, &options).is_ok());
}

/// Q5: overwriting a conflicting env does not let an end-of-life Python
/// through, unrenamed either; `allow_eol` does. What fails this: the
/// overwrite branch returning `Ok` before the EOL guard.
#[test]
#[serial_test::serial]
fn validate_source_overwrite_still_refuses_an_eol_python() {
    let _locale = crate::test_utils::LocaleGuard::capture();
    rust_i18n::set_locale("en");
    let migrator = Migrator {
        uv: UvClient::with_path(PathBuf::from("/mock/uv")),
    };
    let mut source = mock_source(
        "test",
        EnvironmentStatus::NameConflict {
            existing: PathBuf::from("/existing"),
        },
    );
    source.python_version = "3.10.14".to_string();
    let overwrite = MigrateOptions {
        overwrite: true,
        ..Default::default()
    };
    assert!(matches!(
        migrator.validate_source(&source, &overwrite),
        Err(ScoopError::MigrationFailed { ref reason }) if reason.contains("end-of-life")
    ));
    let both = MigrateOptions {
        overwrite: true,
        allow_eol: true,
        ..Default::default()
    };
    assert!(migrator.validate_source(&source, &both).is_ok());

    // An EOL source without a conflict needs `allow_eol` too, not overwrite.
    let eol = mock_source(
        "test",
        EnvironmentStatus::PythonEol {
            version: "3.10.14".to_string(),
        },
    );
    assert!(migrator.validate_source(&eol, &overwrite).is_err());
}

#[test]
fn test_validate_source_eol_without_force() {
    let migrator = Migrator {
        uv: UvClient::with_path(PathBuf::from("/mock/uv")),
    };
    let source = mock_source(
        "test",
        EnvironmentStatus::PythonEol {
            version: "3.7.0".to_string(),
        },
    );
    let options = MigrateOptions::default();

    assert!(migrator.validate_source(&source, &options).is_err());
}

/// Origins can be read from Python 3.8 on; an unparsable version is left
/// to the reader. What fails this: `<=` at 3.8, or refusing what does not
/// parse.
#[test]
fn below_origin_floor_is_below_3_8() {
    for (version, below) in [
        ("3.7.17", true),
        ("2.7", true),
        ("3.8", false),
        ("3.8.0", false),
        ("3.12.1", false),
        ("unknown", false),
        ("3", false),
    ] {
        assert_eq!(below_origin_floor(version), below, "{version}");
    }
}

/// `migrate()` against the fake uv: `SCUV_HOME` is a fresh directory, the
/// source is a directory with its own `uv pip list` answer and a Python that
/// answers the origin query, and the migrator calls the fake by path.
#[cfg(unix)]
mod with_fake_uv {
    use super::*;
    use crate::test_utils::{FakeUv, env_guard, write_fake_python};
    use serial_test::serial;

    const SIX: &str = r#"[{"name":"six","version":"1.16.0"}]"#;
    const SIX_ROWS: &str = r#"[["six","1.16.0",null]]"#;

    struct Fixture {
        uv: FakeUv,
        home: tempfile::TempDir,
        source: SourceEnvironment,
    }

    impl Fixture {
        fn migrator(&self) -> Migrator {
            Migrator::with_uv(UvClient::with_path(self.uv.exe()))
        }
        fn target(&self, name: &str) -> PathBuf {
            self.home.path().join("virtualenvs").join(name)
        }
        fn migrate(&self, options: &MigrateOptions) -> Result<MigrationResult> {
            self.migrator().migrate(&self.source, options)
        }
    }

    /// A source env `web` whose `uv pip list` is `list` and whose Python
    /// answers the origin query with `rows`; the env a fake `uv venv`
    /// creates lists `target` and answers with `target_rows`.
    struct Setup<'a> {
        list: &'a str,
        rows: &'a str,
        target: &'a str,
        target_rows: &'a str,
        status: EnvironmentStatus,
        source_type: SourceType,
        python: &'a str,
    }

    impl Default for Setup<'_> {
        fn default() -> Self {
            Self {
                list: SIX,
                rows: SIX_ROWS,
                target: SIX,
                target_rows: SIX_ROWS,
                status: EnvironmentStatus::Ready,
                source_type: SourceType::Pyenv,
                python: "3.12.0",
            }
        }
    }

    /// The fixture, without touching the environment yet.
    fn make(setup: &Setup<'_>) -> Fixture {
        let uv = FakeUv::new(&[]);
        let home = tempfile::TempDir::new().unwrap();
        let src = fs::canonicalize(home.path()).unwrap().join("pyenv-web");
        write_fake_python(&src, setup.rows);
        fs::write(src.join("uv-pip-list.json"), setup.list).unwrap();
        let source = SourceEnvironment {
            name: "web".to_string(),
            python_version: setup.python.to_string(),
            path: src,
            source_type: setup.source_type,
            size_bytes: None,
            status: setup.status.clone(),
        };
        Fixture { uv, home, source }
    }

    /// Runs `f` with `SCUV_HOME` at the fixture's home, the target env's
    /// answers from `setup`, and `extra`, under one `env_guard` (a nested
    /// one deadlocks).
    fn with_env(
        fx: &Fixture,
        setup: &Setup<'_>,
        extra: &[(&'static str, Option<&str>)],
        f: impl FnOnce(&Fixture),
    ) {
        let home_str = fs::canonicalize(fx.home.path())
            .unwrap()
            .display()
            .to_string();
        let mut vars: Vec<(&'static str, Option<&str>)> = vec![
            ("SCUV_HOME", Some(home_str.as_str())),
            ("FAKE_UV_TARGET_PIP_LIST_JSON", Some(setup.target)),
            ("FAKE_UV_TARGET_ORIGINS", Some(setup.target_rows)),
        ];
        // Later entries win: `extra` may replace the defaults above.
        for (k, v) in extra {
            vars.retain(|(key, _)| key != k);
            vars.push((k, *v));
        }
        let _env = env_guard(&vars);
        f(fx);
    }

    fn run(setup: Setup<'_>, extra: &[(&'static str, Option<&str>)], f: impl FnOnce(&Fixture)) {
        let fx = make(&setup);
        with_env(&fx, &setup, extra, f);
    }

    fn dry_run() -> MigrateOptions {
        MigrateOptions {
            dry_run: true,
            ..Default::default()
        }
    }

    fn delete_source() -> MigrateOptions {
        MigrateOptions {
            delete_source: true,
            ..Default::default()
        }
    }

    /// A project directory with a setup.py, for editable origins.
    fn project(dir: &Path) -> String {
        fs::create_dir_all(dir).unwrap();
        fs::write(dir.join("setup.py"), "").unwrap();
        format!(
            r#"{{\"url\":\"file://{}\",\"dir_info\":{{\"editable\":true}}}}"#,
            dir.display()
        )
    }

    /// The dry run refuses what the real run refuses.
    #[test]
    #[serial]
    fn dry_run_refuses_what_the_real_run_refuses() {
        let corrupted = Setup {
            status: EnvironmentStatus::Corrupted {
                reason: "broken".to_string(),
            },
            ..Default::default()
        };
        run(corrupted, &[], |fx| {
            let err = fx.migrate(&dry_run()).unwrap_err();
            assert!(
                matches!(err, ScoopError::CorruptedEnvironment { .. }),
                "{err}"
            );
        });
        let eol = Setup {
            status: EnvironmentStatus::PythonEol {
                version: "3.8.0".to_string(),
            },
            ..Default::default()
        };
        run(eol, &[], |fx| {
            let err = fx.migrate(&dry_run()).unwrap_err();
            assert!(matches!(err, ScoopError::MigrationFailed { .. }), "{err}");
        });
        run(Setup::default(), &[], |fx| {
            fs::create_dir_all(fx.target("web")).unwrap();
            let err = fx.migrate(&dry_run()).unwrap_err();
            assert!(matches!(err, ScoopError::VirtualenvExists { .. }), "{err}");
        });
        run(Setup::default(), &[], |fx| {
            let options = MigrateOptions {
                rename_to: Some("../x".to_string()),
                ..dry_run()
            };
            let err = fx.migrator().migrate(&fx.source, &options).unwrap_err();
            assert!(matches!(err, ScoopError::InvalidEnvName { .. }), "{err}");
        });
    }

    /// The dry run counts what the plan would install (pip left out) and
    /// changes nothing. What fails this: counting the listing, or creating
    /// the target.
    #[test]
    #[serial]
    fn dry_run_reports_the_plan() {
        let setup = Setup {
            list: r#"[{"name":"pip","version":"24.0"},{"name":"six","version":"1.16.0"}]"#,
            rows: r#"[["pip","24.0",null],["six","1.16.0",null]]"#,
            ..Default::default()
        };
        run(setup, &[], |fx| {
            let result = fx.migrate(&dry_run()).unwrap();
            assert_eq!(result.packages_migrated, 1);
            assert_eq!(result.packages_excluded[0].name, "pip");
            assert!(!fx.target("web").exists());
        });
    }

    /// A real run installs the plan, verifies it, and reports pip as left
    /// out, not failed. What fails this: installing pip, or counting it.
    #[test]
    #[serial]
    fn a_real_run_installs_and_verifies_the_plan() {
        let setup = Setup {
            list: r#"[{"name":"pip","version":"24.0"},{"name":"six","version":"1.16.0"}]"#,
            rows: r#"[["pip","24.0",null],["six","1.16.0",null]]"#,
            ..Default::default()
        };
        run(setup, &[], |fx| {
            let result = fx.migrate(&MigrateOptions::default()).unwrap();
            assert_eq!(fx.uv.install_args().last().unwrap()[4..], ["six==1.16.0"]);
            assert_eq!(result.packages_migrated, 1);
            assert!(
                result.packages_failed.is_empty(),
                "{:?}",
                result.packages_failed
            );
            assert_eq!(
                result.packages_excluded,
                [ExcludedPackage {
                    name: "pip".into(),
                    version: "24.0".into(),
                    reason: "installer".into()
                }]
            );
        });
    }

    /// An editable source is installed with `--editable <path>`, and the
    /// new env must hold it editable from that path. What fails this:
    /// reinstalling it as `name==version`, or verifying names only.
    #[test]
    #[serial]
    fn an_editable_package_moves_and_is_verified_by_origin() {
        let work = tempfile::TempDir::new().unwrap();
        let proj = fs::canonicalize(work.path()).unwrap().join("proj");
        let origin = project(&proj);
        let list = r#"[{"name":"proj","version":"0.1"}]"#;
        let rows = format!(r#"[["proj","0.1","{origin}"]]"#);
        let ok = Setup {
            list,
            rows: &rows,
            target: list,
            target_rows: &rows,
            ..Default::default()
        };
        run(ok, &[], |fx| {
            let result = fx.migrate(&MigrateOptions::default()).unwrap();
            let args = fx.uv.install_args().last().unwrap().clone();
            assert_eq!(
                args[4..],
                ["--editable".to_string(), proj.display().to_string()]
            );
            assert_eq!(result.packages_migrated, 1);
        });
        // The new env got a registry `proj 0.1` (say, as another package's
        // dependency) instead of the editable: a failure.
        let registry = r#"[["proj","0.1",null]]"#;
        let swapped = Setup {
            list,
            rows: &rows,
            target: list,
            target_rows: registry,
            ..Default::default()
        };
        run(swapped, &[], |fx| {
            let result = fx.migrate(&MigrateOptions::default()).unwrap();
            assert_eq!(result.packages_migrated, 0);
            assert!(
                result.packages_failed[0].contains("registry"),
                "{:?}",
                result.packages_failed
            );
        });
    }

    /// Verification reports a missing package or another version as
    /// failed, an extra one as added; under `--strict` a mismatch fails the
    /// migration and removes the new env. What fails this: trusting the
    /// installer's exit status.
    #[test]
    #[serial]
    fn verification_reports_what_the_new_env_holds() {
        let two = r#"[{"name":"six","version":"1.16.0"},{"name":"idna","version":"3.4"}]"#;
        let two_rows = r#"[["six","1.16.0",null],["idna","3.4",null]]"#;
        let off = r#"[{"name":"six","version":"1.15.0"},{"name":"extra","version":"1"}]"#;
        let off_rows = r#"[["six","1.15.0",null],["extra","1",null]]"#;
        let setup = || Setup {
            list: two,
            rows: two_rows,
            target: off,
            target_rows: off_rows,
            ..Default::default()
        };
        run(setup(), &[], |fx| {
            let result = fx.migrate(&MigrateOptions::default()).unwrap();
            assert_eq!(result.packages_migrated, 0);
            assert_eq!(
                result.packages_failed.len(),
                2,
                "{:?}",
                result.packages_failed
            );
            assert!(result.packages_failed.iter().any(|f| f.contains("1.15.0")));
            assert!(result.packages_failed.iter().any(|f| f.contains("idna")));
            assert_eq!(result.packages_added, ["extra==1"]);
        });
        run(setup(), &[], |fx| {
            let strict = MigrateOptions {
                strict: true,
                ..Default::default()
            };
            let err = fx.migrate(&strict).unwrap_err();
            assert!(matches!(err, ScoopError::MigrationFailed { .. }), "{err}");
            assert!(!fx.target("web").exists(), "strict left the env");
        });
    }

    /// A package that does not install is found by trying each alone
    /// without dependencies in a recreated env, and the env is built once
    /// more with the rest in one `--no-deps` call. What fails this:
    /// retrying in the env the failed call left, or keeping the probe env.
    #[test]
    #[serial]
    fn a_failed_install_is_retried_package_by_package() {
        let two = r#"[{"name":"six","version":"1.16.0"},{"name":"bad","version":"1.0"}]"#;
        let two_rows = r#"[["six","1.16.0",null],["bad","1.0",null]]"#;
        let setup = Setup {
            list: two,
            rows: two_rows,
            ..Default::default()
        };
        let fails = [("FAKE_UV_PIP_INSTALL_FAILS", Some("bad==1.0"))];
        run(setup, &fails, |fx| {
            let result = fx.migrate(&delete_source()).unwrap();
            let calls: Vec<Vec<String>> = fx
                .uv
                .install_args()
                .into_iter()
                .map(|c| c[4..].to_vec())
                .collect();
            assert_eq!(
                calls,
                [
                    vec!["six==1.16.0", "bad==1.0"],
                    vec!["--no-deps", "six==1.16.0"],
                    vec!["--no-deps", "bad==1.0"],
                    vec!["--no-deps", "six==1.16.0"],
                ]
            );
            let venvs = fs::read_to_string(fx.uv.exe().with_file_name("venv.log")).unwrap();
            assert_eq!(venvs.lines().count(), 3, "create + recreate twice");
            // The failed attempts left `installed-bad==1.0` (the fake leaves
            // a file per package, failing or not); the rebuilt env has none.
            let target = fx.target("web");
            assert!(target.join("installed-six==1.16.0").exists());
            assert!(
                !target.join("installed-bad==1.0").exists(),
                "residue of a failed attempt"
            );
            assert_eq!(result.packages_migrated, 1);
            assert_eq!(result.packages_failed.len(), 1);
            assert!(result.packages_failed[0].starts_with("bad==1.0"));
            assert!(!result.source_deleted);
            assert!(fx.source.path.exists());
        });
    }

    /// `--strict` does not retry: the install error is the result, and the
    /// new env is removed. What fails this: retrying under strict.
    #[test]
    #[serial]
    fn strict_does_not_retry() {
        let fails = [("FAKE_UV_PIP_INSTALL_FAILS", Some("six==1.16.0"))];
        run(Setup::default(), &fails, |fx| {
            let strict = MigrateOptions {
                strict: true,
                ..Default::default()
            };
            let err = fx.migrate(&strict).unwrap_err();
            assert!(matches!(err, ScoopError::UvCommandFailed { .. }), "{err}");
            assert_eq!(fx.uv.install_args().len(), 1);
            assert!(!fx.target("web").exists());
        });
    }

    /// conda's builds come back from the registry and are reported; a
    /// conda source is never deleted. What fails this: keeping conda's
    /// build-directory origin (missing on disk), or deleting the env.
    #[test]
    #[serial]
    fn conda_packages_come_from_the_registry_and_conda_sources_stay() {
        let list = r#"[{"name":"numpy","version":"2.5.4"}]"#;
        let rows = r#"[["numpy","2.5.4","{\"url\":\"file:///home/conda/feedstock_root/build_artifacts/numpy/work\",\"dir_info\":{}}","conda"]]"#;
        let target_rows = r#"[["numpy","2.5.4",null]]"#;
        let setup = Setup {
            list,
            rows,
            target: list,
            target_rows,
            source_type: SourceType::Conda,
            ..Default::default()
        };
        run(setup, &[], |fx| {
            let result = fx.migrate(&delete_source()).unwrap();
            assert_eq!(fx.uv.install_args().last().unwrap()[4..], ["numpy==2.5.4"]);
            assert_eq!(result.packages_from_conda, ["numpy"]);
            assert_eq!(result.packages_migrated, 1);
            assert!(!result.source_deleted);
            assert!(
                result
                    .source_kept_reason
                    .as_deref()
                    .unwrap()
                    .contains("conda")
            );
            assert!(fx.source.path.exists());
        });
    }

    /// A source older than Python 3.8 is refused before uv runs (uv may not
    /// read a very old Python at all). What fails this: asking uv first.
    #[test]
    #[serial]
    fn a_python_older_than_3_8_is_refused_before_uv() {
        let setup = Setup {
            python: "3.7.17",
            ..Default::default()
        };
        run(setup, &[], |fx| {
            let options = MigrateOptions {
                allow_eol: true,
                ..dry_run()
            };
            let err = fx.migrate(&options).unwrap_err();
            assert!(
                matches!(err, ScoopError::PackageOriginsUnreadable { .. }),
                "{err}"
            );
            assert!(fx.uv.pip_calls().is_empty(), "{:?}", fx.uv.pip_calls());
        });
    }

    /// A target that would be created inside the source is refused (with
    /// `SCUV_HOME` inside the source, `--delete-source` would remove the new
    /// env). A sibling is fine. What fails this: comparing the target's
    /// parent both ways.
    #[test]
    #[serial]
    fn a_target_inside_the_source_is_refused() {
        let setup = Setup::default();
        let fx = make(&setup);
        let inner = fx.source.path.join("scuv-home");
        fs::create_dir_all(&inner).unwrap();
        let inner_str = inner.display().to_string();
        with_env(
            &fx,
            &setup,
            &[("SCUV_HOME", Some(inner_str.as_str()))],
            |fx| {
                let err = fx.migrate(&dry_run()).unwrap_err();
                assert!(matches!(err, ScoopError::MigrationFailed { .. }), "{err}");
            },
        );
        run(Setup::default(), &[], |fx| {
            fx.migrate(&dry_run()).unwrap();
        });
    }

    /// `--delete-source` removes a fully migrated source, and keeps one
    /// that a package still depends on (an editable inside it) or that the
    /// new env's Python lives in. What fails this: deleting on success
    /// alone.
    #[test]
    #[serial]
    fn delete_source_keeps_what_something_depends_on() {
        run(Setup::default(), &[], |fx| {
            let result = fx.migrate(&delete_source()).unwrap();
            assert!(result.source_deleted);
            assert!(!fx.source.path.exists());
        });

        let list = r#"[{"name":"proj","version":"0.1"}]"#;
        let setup = Setup::default();
        let fx = make(&setup);
        let inside = fx.source.path.join("proj");
        let origin = project(&inside);
        let rows = format!(r#"[["proj","0.1","{origin}"]]"#);
        write_fake_python(&fx.source.path, &rows);
        fs::write(fx.source.path.join("uv-pip-list.json"), list).unwrap();
        let extra = [
            ("FAKE_UV_TARGET_PIP_LIST_JSON", Some(list)),
            ("FAKE_UV_TARGET_ORIGINS", Some(rows.as_str())),
        ];
        with_env(&fx, &setup, &extra, |fx| {
            let result = fx.migrate(&delete_source()).unwrap();
            assert_eq!(result.packages_migrated, 1, "{:?}", result.packages_failed);
            assert!(!result.source_deleted);
            assert!(
                result
                    .source_kept_reason
                    .as_deref()
                    .unwrap()
                    .contains("proj")
            );
            assert!(inside.join("setup.py").exists());
        });

        let fx = make(&setup);
        let python_home = fx.source.path.join("pythons/bin");
        fs::create_dir_all(&python_home).unwrap();
        let home_str = python_home.display().to_string();
        with_env(
            &fx,
            &setup,
            &[("FAKE_UV_VENV_HOME", Some(home_str.as_str()))],
            |fx| {
                let result = fx.migrate(&delete_source()).unwrap();
                assert!(!result.source_deleted);
                assert!(python_home.exists());
            },
        );
    }

    /// A dry run already says the source would be kept, and why, when
    /// something depends on it (an editable inside it). What fails this:
    /// deciding deletions only during the real run.
    #[test]
    #[serial]
    fn a_dry_run_says_the_source_would_be_kept() {
        let list = r#"[{"name":"proj","version":"0.1"}]"#;
        let setup = Setup::default();
        let fx = make(&setup);
        let origin = project(&fx.source.path.join("proj"));
        write_fake_python(&fx.source.path, &format!(r#"[["proj","0.1","{origin}"]]"#));
        fs::write(fx.source.path.join("uv-pip-list.json"), list).unwrap();
        with_env(&fx, &setup, &[], |fx| {
            let options = MigrateOptions {
                delete_source: true,
                ..dry_run()
            };
            let result = fx.migrate(&options).unwrap();
            assert!(
                result
                    .source_kept_reason
                    .as_deref()
                    .unwrap_or_default()
                    .contains("proj"),
                "{:?}",
                result.source_kept_reason
            );
        });
    }

    /// A local source inside a target `--force` would replace fails the
    /// env before anything is removed. What fails this: leaving the
    /// target out of the preflight.
    #[test]
    #[serial]
    fn a_source_inside_the_replaced_target_is_refused() {
        run(Setup::default(), &[], |fx| {
            let inside = fx.target("web").join("proj");
            let origin = project(&inside);
            fs::write(fx.target("web").join("keep"), "").unwrap();
            let list = r#"[{"name":"proj","version":"0.1"}]"#;
            write_fake_python(&fx.source.path, &format!(r#"[["proj","0.1","{origin}"]]"#));
            fs::write(fx.source.path.join("uv-pip-list.json"), list).unwrap();
            let force = MigrateOptions {
                overwrite: true,
                ..Default::default()
            };
            let err = fx.migrate(&force).unwrap_err();
            assert!(
                matches!(err, ScoopError::PackageSourceInsideTarget { .. }),
                "{err}"
            );
            assert!(fx.target("web").join("keep").exists());
        });
    }

    /// A target that appears after the checks (here: while the source's
    /// Python answers) is still not deleted without `--force`.
    #[test]
    #[serial]
    fn a_target_created_during_reading_survives_without_force() {
        run(Setup::default(), &[], |fx| {
            use std::os::unix::fs::PermissionsExt;
            let target = fx.target("web");
            let python = fx.source.path.join("bin").join("python");
            let script = format!(
                "#!/bin/sh\n/bin/mkdir -p '{t}'\n: > '{t}/keep'\nprintf '%s\\n' 'SCUV_ORIGINS:{SIX_ROWS}'\n",
                t = target.display()
            );
            fs::write(&python, script).unwrap();
            fs::set_permissions(&python, fs::Permissions::from_mode(0o755)).unwrap();
            let err = fx.migrate(&MigrateOptions::default()).unwrap_err();
            assert!(matches!(err, ScoopError::VirtualenvExists { .. }), "{err}");
            assert!(
                target.join("keep").exists(),
                "migrate deleted an env it had no --force for"
            );
        });
    }

    /// Another source env `name` in the fixture's home, listing six.
    fn sibling(fx: &Fixture, name: &str) -> SourceEnvironment {
        let path = fs::canonicalize(fx.home.path())
            .unwrap()
            .join(format!("pyenv-{name}"));
        write_fake_python(&path, SIX_ROWS);
        fs::write(path.join("uv-pip-list.json"), SIX).unwrap();
        SourceEnvironment {
            name: name.to_string(),
            path,
            ..fx.source.clone()
        }
    }

    /// A batch with an env that cannot be read deletes nothing: sources are
    /// kept, and an env that would replace a target fails before running
    /// (the unread env's sources might live there). What fails this:
    /// failing only the unread env.
    #[test]
    #[serial]
    fn a_batch_with_an_unreadable_env_deletes_nothing() {
        run(Setup::default(), &[], |fx| {
            let broken = sibling(fx, "broken");
            fs::remove_file(broken.path.join("bin/python")).unwrap();
            let replacing = sibling(fx, "replacing");
            fs::create_dir_all(fx.target("replacing")).unwrap();
            fs::write(fx.target("replacing").join("keep"), "").unwrap();
            let options = MigrateOptions {
                overwrite: true,
                delete_source: true,
                ..Default::default()
            };
            let migrator = fx.migrator();
            let plan = migrator
                .plan_batch(&[&fx.source, &broken, &replacing], &[], &options)
                .unwrap();
            let failed: Vec<&str> = plan.failed.iter().map(|(s, _)| s.name.as_str()).collect();
            assert_eq!(failed, ["broken", "replacing"]);
            assert_eq!(plan.ready.len(), 1);
            assert!(plan.ready[0].kept_reason.is_some());

            let (mut done, run_failed) = migrator
                .execute_batch(&plan, &options, |_| {}, |_, _| {})
                .unwrap();
            assert!(run_failed.is_empty());
            let all: Vec<&Prepared> = plan.ready.iter().collect();
            let mut refs: Vec<(usize, &mut MigrationResult)> =
                done.iter_mut().map(|(i, r)| (*i, r)).collect();
            migrator
                .finish_deletion(&all, &plan.extra, &mut refs)
                .unwrap();
            assert!(fx.source.path.exists());
            assert!(fx.target("replacing").join("keep").exists());
        });
    }

    /// A source another env's package lives in is kept, with the reason;
    /// the dependent env still migrates. What fails this: checking each
    /// env against its own deletions only.
    #[test]
    #[serial]
    fn a_batch_keeps_a_source_another_env_depends_on() {
        run(Setup::default(), &[], |fx| {
            let other = sibling(fx, "other");
            let origin = project(&fx.source.path.join("shared"));
            let rows = format!(r#"[["shared","1","{origin}"]]"#);
            write_fake_python(&other.path, &rows);
            fs::write(
                other.path.join("uv-pip-list.json"),
                r#"[{"name":"shared","version":"1"}]"#,
            )
            .unwrap();
            let options = MigrateOptions {
                delete_source: true,
                ..Default::default()
            };
            let plan = fx
                .migrator()
                .plan_batch(&[&fx.source, &other], &[], &options)
                .unwrap();
            assert!(plan.failed.is_empty(), "{:?}", plan.failed);
            let web = plan.ready.iter().find(|p| p.source.name == "web").unwrap();
            assert!(
                web.kept_reason.as_deref().unwrap().contains("shared"),
                "{:?}",
                web.kept_reason
            );
            let other = plan
                .ready
                .iter()
                .find(|p| p.source.name == "other")
                .unwrap();
            assert!(other.kept_reason.is_none());
        });
    }

    /// Envs run one at a time, and a target replacement is checked again
    /// right before it happens: an earlier env whose new Python sits
    /// inside the target a later env replaces makes that later env fail,
    /// and the Python survives. What fails this: checking replacements only
    /// before the batch starts.
    #[test]
    #[serial]
    fn a_replacement_spares_the_python_an_earlier_env_got() {
        let setup = Setup::default();
        let fx = make(&setup);
        let later = sibling(&fx, "later");
        let pythons = fx.target("later").join("pythons/bin");
        fs::create_dir_all(&pythons).unwrap();
        let home_str = pythons.display().to_string();
        with_env(
            &fx,
            &setup,
            &[("FAKE_UV_VENV_HOME", Some(home_str.as_str()))],
            |fx| {
                let options = MigrateOptions {
                    overwrite: true,
                    ..Default::default()
                };
                let migrator = fx.migrator();
                let plan = migrator
                    .plan_batch(&[&fx.source, &later], &[], &options)
                    .unwrap();
                assert!(plan.failed.is_empty(), "{:?}", plan.failed);
                let (done, failed) = migrator
                    .execute_batch(&plan, &options, |_| {}, |_, _| {})
                    .unwrap();
                assert_eq!(done.len(), 1);
                let failed: Vec<&str> = failed.iter().map(|(s, _)| s.name.as_str()).collect();
                assert_eq!(failed, ["later"]);
                assert!(pythons.exists(), "the earlier env's Python was removed");
            },
        );
    }

    /// A target that did not exist when the batch was planned can exist
    /// when its env runs (here an earlier env's Python was put there): the
    /// replacement is checked against the filesystem then, not the plan.
    /// What fails this: trusting the plan's "nothing to replace".
    #[test]
    #[serial]
    fn a_target_that_appeared_since_planning_is_checked() {
        let setup = Setup::default();
        let fx = make(&setup);
        let later = sibling(&fx, "later");
        let pythons = fx.target("later").join("pythons/bin");
        let home_str = pythons.display().to_string();
        with_env(
            &fx,
            &setup,
            &[("FAKE_UV_VENV_HOME", Some(home_str.as_str()))],
            |fx| {
                let options = MigrateOptions {
                    overwrite: true,
                    ..Default::default()
                };
                let migrator = fx.migrator();
                let plan = migrator
                    .plan_batch(&[&fx.source, &later], &[], &options)
                    .unwrap();
                assert!(plan.ready.iter().all(|p| !p.replace), "{:?}", plan.ready);
                let (done, failed) = migrator
                    .execute_batch(
                        &plan,
                        &options,
                        |env| {
                            if env.source.name == "later" {
                                fs::create_dir_all(&pythons).unwrap();
                            }
                        },
                        |_, _| {},
                    )
                    .unwrap();
                assert_eq!(done.len(), 1);
                let failed: Vec<&str> = failed.iter().map(|(s, _)| s.name.as_str()).collect();
                assert_eq!(failed, ["later"]);
                assert!(pythons.exists(), "the earlier env's Python was removed");
            },
        );
    }

    /// An env migrate found but does not migrate (skipped, conflicting,
    /// an invalid name) still has packages: a source they live in is kept,
    /// and one that cannot be read keeps every source. Without anything to
    /// delete, those envs are not read at all. What fails this: planning
    /// only the selected envs.
    #[test]
    #[serial]
    fn envs_not_migrated_still_protect_their_packages() {
        run(Setup::default(), &[], |fx| {
            let skipped = sibling(fx, "skipped");
            let origin = project(&fx.source.path.join("shared"));
            write_fake_python(&skipped.path, &format!(r#"[["shared","1","{origin}"]]"#));
            fs::write(
                skipped.path.join("uv-pip-list.json"),
                r#"[{"name":"shared","version":"1"}]"#,
            )
            .unwrap();
            let delete = MigrateOptions {
                delete_source: true,
                ..Default::default()
            };
            let migrator = fx.migrator();
            let plan = migrator
                .plan_batch(&[&fx.source], &[&skipped], &delete)
                .unwrap();
            assert!(
                plan.ready[0]
                    .kept_reason
                    .as_deref()
                    .unwrap_or_default()
                    .contains("shared"),
                "{:?}",
                plan.ready[0].kept_reason
            );

            let unreadable = sibling(fx, "unreadable");
            fs::remove_file(unreadable.path.join("bin/python")).unwrap();
            let plan = migrator
                .plan_batch(&[&fx.source], &[&unreadable], &delete)
                .unwrap();
            assert!(plan.ready[0].kept_reason.is_some());
            assert!(!plan.may_replace);

            let calls = fx.uv.pip_calls().len();
            let plain = migrator
                .plan_batch(&[&fx.source], &[&unreadable], &MigrateOptions::default())
                .unwrap();
            assert!(plain.may_replace);
            assert_eq!(
                fx.uv.pip_calls().len(),
                calls + 1,
                "read an env it had no reason to"
            );
        });
    }

    /// Once an env could not be read, no target is replaced while the batch
    /// runs, including one that appeared after planning. What fails this:
    /// deciding "nothing may be replaced" only from the plan.
    #[test]
    #[serial]
    fn an_unreadable_env_stops_later_replacements_too() {
        run(Setup::default(), &[], |fx| {
            let unreadable = sibling(fx, "unreadable");
            fs::remove_file(unreadable.path.join("bin/python")).unwrap();
            let options = MigrateOptions {
                overwrite: true,
                ..Default::default()
            };
            let migrator = fx.migrator();
            let plan = migrator
                .plan_batch(&[&fx.source], &[&unreadable], &options)
                .unwrap();
            assert_eq!(plan.ready.len(), 1);
            let target = fx.target("web");
            let (done, failed) = migrator
                .execute_batch(
                    &plan,
                    &options,
                    |_| {
                        fs::create_dir_all(&target).unwrap();
                        fs::write(target.join("keep"), "").unwrap();
                    },
                    |_, _| {},
                )
                .unwrap();
            assert!(done.is_empty());
            assert_eq!(failed.len(), 1);
            assert!(target.join("keep").exists());
        });
    }

    /// Without --delete-source nothing is deleted, so nothing is "kept":
    /// not when another env depends on a source, not when another env
    /// cannot be read. What fails this: deciding deletions for envs that
    /// asked for none.
    #[test]
    #[serial]
    fn a_batch_without_delete_source_keeps_nothing() {
        run(Setup::default(), &[], |fx| {
            let other = sibling(fx, "other");
            let origin = project(&fx.source.path.join("shared"));
            write_fake_python(&other.path, &format!(r#"[["shared","1","{origin}"]]"#));
            fs::write(
                other.path.join("uv-pip-list.json"),
                r#"[{"name":"shared","version":"1"}]"#,
            )
            .unwrap();
            let broken = sibling(fx, "broken");
            fs::remove_file(broken.path.join("bin/python")).unwrap();
            let migrator = fx.migrator();
            let keep = MigrateOptions::default();
            for sources in [vec![&fx.source, &other], vec![&fx.source, &broken]] {
                let plan = migrator.plan_batch(&sources, &[], &keep).unwrap();
                assert!(
                    plan.ready.iter().all(|p| p.kept_reason.is_none()),
                    "{:?}",
                    plan.ready
                );
            }
        });
    }

    /// An env whose target replacement is blocked does not run, and its
    /// source still counts: another env's source it lives in is kept. What
    /// fails this: forgetting the blocked env's source.
    #[test]
    #[serial]
    fn a_blocked_env_still_protects_its_source() {
        run(Setup::default(), &[], |fx| {
            // `nested` lives inside web's source; its target exists and
            // holds web's editable project, so replacing it is blocked.
            let mut nested = sibling(fx, "nested");
            let inner = fx.source.path.join("nested-env");
            fs::rename(&nested.path, &inner).unwrap();
            nested.path = inner;
            let shared = fx.target("nested").join("shared");
            let origin = project(&shared);
            write_fake_python(&fx.source.path, &format!(r#"[["shared","1","{origin}"]]"#));
            fs::write(
                fx.source.path.join("uv-pip-list.json"),
                r#"[{"name":"shared","version":"1"}]"#,
            )
            .unwrap();
            let options = MigrateOptions {
                overwrite: true,
                delete_source: true,
                ..Default::default()
            };
            let plan = fx
                .migrator()
                .plan_batch(&[&fx.source, &nested], &[], &options)
                .unwrap();
            let failed: Vec<&str> = plan.failed.iter().map(|(s, _)| s.name.as_str()).collect();
            assert_eq!(failed, ["nested"], "{:?}", plan.failed);
            let web = &plan.ready[0];
            assert!(
                web.kept_reason
                    .as_deref()
                    .unwrap_or_default()
                    .contains("nested"),
                "{:?}",
                web.kept_reason
            );
        });
    }

    /// A single migrate whose target replacement would break the source's
    /// own Python (its pyvenv.cfg home inside the target) fails before
    /// anything is removed. What fails this: skipping the deletion
    /// decision for a single env.
    #[test]
    #[serial]
    fn a_single_replacement_that_breaks_something_fails() {
        run(Setup::default(), &[], |fx| {
            let pythons = fx.target("web").join("pythons/bin");
            fs::create_dir_all(&pythons).unwrap();
            fs::write(
                fx.source.path.join("pyvenv.cfg"),
                format!("home = {}\n", pythons.display()),
            )
            .unwrap();
            let force = MigrateOptions {
                overwrite: true,
                ..Default::default()
            };
            let err = fx.migrate(&force).unwrap_err();
            assert!(matches!(err, ScoopError::MigrationFailed { .. }), "{err}");
            assert!(pythons.exists());
        });
    }

    /// `uv venv` failing after it made the directory leaves nothing behind
    /// for the next run to trip over. What fails this: arming the rollback
    /// only after creation succeeded.
    #[test]
    #[serial]
    fn a_failed_creation_leaves_no_half_made_env() {
        run(
            Setup::default(),
            &[("FAKE_UV_VENV_FAILS", Some("1"))],
            |fx| {
                let err = fx.migrate(&MigrateOptions::default()).unwrap_err();
                assert!(matches!(err, ScoopError::UvCommandFailed { .. }), "{err}");
                assert!(!fx.target("web").exists());
            },
        );
    }

    /// `actual_python_version` is what the created env's pyvenv.cfg
    /// records, and the source's version when the file has none.
    #[test]
    #[serial]
    fn actual_python_version_comes_from_the_target_pyvenv_cfg() {
        let minor = [("FAKE_UV_VENV_VERSION_INFO", Some("3.12"))];
        run(Setup::default(), &minor, |fx| {
            let result = fx.migrate(&MigrateOptions::default()).unwrap();
            assert_eq!(result.python_version, "3.12.0");
            assert_eq!(result.actual_python_version, "3.12");
        });
        run(Setup::default(), &[], |fx| {
            let result = fx.migrate(&MigrateOptions::default()).unwrap();
            assert_eq!(result.actual_python_version, "3.12.0");
        });
    }
}

/// The EOL refusal is word for word the English it was and follows the
/// language. Fails if the reason is hard-coded again.
#[test]
#[serial_test::serial]
fn eol_error_follows_the_language() {
    let _locale = crate::test_utils::LocaleGuard::capture();
    let reason = |e: ScoopError| match e {
        ScoopError::MigrationFailed { reason } => reason,
        other => panic!("{other:?}"),
    };
    rust_i18n::set_locale("en");
    assert_eq!(
        reason(eol_error("2.7")),
        "Python 2.7 is end-of-life. Use --allow-eol to migrate anyway."
    );
    rust_i18n::set_locale("ko");
    assert_eq!(
        reason(eol_error("2.7")),
        "Python 2.7 지원 종료됨. 그래도 마이그레이션하려면 --allow-eol 사용."
    );
}
