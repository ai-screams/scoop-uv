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
        extractor: PackageExtractor::new(),
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
        extractor: PackageExtractor::new(),
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
        extractor: PackageExtractor::new(),
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
        extractor: PackageExtractor::new(),
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
        extractor: PackageExtractor::new(),
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
        extractor: PackageExtractor::new(),
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
        extractor: PackageExtractor::new(),
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
        extractor: PackageExtractor::new(),
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

/// `migrate()` against the fake uv: `SCUV_HOME` is a fresh directory,
/// the source is a directory whose `bin/pip` prints `freeze`, and the
/// migrator calls the fake by path.
#[cfg(unix)]
mod with_fake_uv {
    use super::*;
    use crate::test_utils::{FakeUv, env_guard};
    use serial_test::serial;

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
    }

    fn run(
        freeze: &str,
        status: EnvironmentStatus,
        extra: &[(&'static str, Option<&str>)],
        f: impl FnOnce(&Fixture),
    ) {
        use std::os::unix::fs::PermissionsExt;
        let uv = FakeUv::new(&[]);
        let home = tempfile::TempDir::new().unwrap();
        let src = home.path().join("pyenv-web");
        fs::create_dir_all(src.join("bin")).unwrap();
        let pip = src.join("bin").join("pip");
        fs::write(&pip, format!("#!/bin/sh\nprintf '{freeze}'\n")).unwrap();
        fs::set_permissions(&pip, fs::Permissions::from_mode(0o755)).unwrap();
        let home_str = home.path().display().to_string();
        let mut vars: Vec<(&'static str, Option<&str>)> =
            vec![("SCUV_HOME", Some(home_str.as_str()))];
        vars.extend_from_slice(extra);
        let _env = env_guard(&vars);
        let source = SourceEnvironment {
            name: "web".to_string(),
            python_version: "3.12.0".to_string(),
            path: src,
            source_type: SourceType::Pyenv,
            size_bytes: None,
            status,
        };
        f(&Fixture { uv, home, source });
    }

    fn dry_run() -> MigrateOptions {
        MigrateOptions {
            dry_run: true,
            ..Default::default()
        }
    }

    /// The dry run refuses what the real run refuses.
    #[test]
    #[serial]
    fn dry_run_refuses_a_corrupted_source() {
        let status = EnvironmentStatus::Corrupted {
            reason: "broken".to_string(),
        };
        run("six==1.16.0\n", status, &[], |fx| {
            let err = fx.migrator().migrate(&fx.source, &dry_run()).unwrap_err();
            assert!(
                matches!(err, ScoopError::CorruptedEnvironment { .. }),
                "{err}"
            );
        });
    }

    #[test]
    #[serial]
    fn dry_run_refuses_an_eol_python_without_allow_eol() {
        let status = EnvironmentStatus::PythonEol {
            version: "3.8.0".to_string(),
        };
        run("six==1.16.0\n", status, &[], |fx| {
            let err = fx.migrator().migrate(&fx.source, &dry_run()).unwrap_err();
            assert!(matches!(err, ScoopError::MigrationFailed { .. }), "{err}");
        });
    }

    #[test]
    #[serial]
    fn dry_run_refuses_an_existing_target() {
        run("six==1.16.0\n", EnvironmentStatus::Ready, &[], |fx| {
            fs::create_dir_all(fx.target("web")).unwrap();
            let err = fx.migrator().migrate(&fx.source, &dry_run()).unwrap_err();
            assert!(matches!(err, ScoopError::VirtualenvExists { .. }), "{err}");
        });
    }

    #[test]
    #[serial]
    fn dry_run_refuses_an_invalid_new_name() {
        run("six==1.16.0\n", EnvironmentStatus::Ready, &[], |fx| {
            let options = MigrateOptions {
                rename_to: Some("../x".to_string()),
                ..dry_run()
            };
            let err = fx.migrator().migrate(&fx.source, &options).unwrap_err();
            assert!(matches!(err, ScoopError::InvalidEnvName { .. }), "{err}");
        });
    }

    /// A freeze line that does not parse is reported by the real run as
    /// it is by the dry run, without counting against migrated packages.
    #[test]
    #[serial]
    fn real_run_reports_unparsed_freeze_lines() {
        run(
            "six==1.16.0\\nnot-a-pin\\n",
            EnvironmentStatus::Ready,
            &[],
            |fx| {
                let preview = fx.migrator().migrate(&fx.source, &dry_run()).unwrap();
                let result = fx
                    .migrator()
                    .migrate(&fx.source, &MigrateOptions::default())
                    .unwrap();
                assert_eq!(result.packages_failed, ["not-a-pin"]);
                assert_eq!(result.packages_failed, preview.packages_failed);
                assert_eq!(result.packages_migrated, 1);
            },
        );
    }

    /// A target that appears after the preflight (here: while `pip
    /// freeze` runs) is still not deleted without `--force`.
    #[test]
    #[serial]
    fn a_target_created_during_extraction_survives_without_force() {
        run("six==1.16.0\\n", EnvironmentStatus::Ready, &[], |fx| {
            use std::os::unix::fs::PermissionsExt;
            let target = fx.target("web");
            let pip = fx.source.path.join("bin").join("pip");
            let script = format!(
                "#!/bin/sh\n/bin/mkdir -p '{t}'\n: > '{t}/keep'\nprintf 'six==1.16.0\\n'\n",
                t = target.display()
            );
            fs::write(&pip, script).unwrap();
            fs::set_permissions(&pip, fs::Permissions::from_mode(0o755)).unwrap();
            let err = fx
                .migrator()
                .migrate(&fx.source, &MigrateOptions::default())
                .unwrap_err();
            assert!(matches!(err, ScoopError::VirtualenvExists { .. }), "{err}");
            assert!(
                target.join("keep").exists(),
                "migrate deleted an env it had no --force for"
            );
        });
    }

    fn delete_source() -> MigrateOptions {
        MigrateOptions {
            delete_source: true,
            ..Default::default()
        }
    }

    #[test]
    #[serial]
    fn delete_source_removes_a_fully_migrated_source() {
        run("six==1.16.0\\n", EnvironmentStatus::Ready, &[], |fx| {
            let result = fx.migrator().migrate(&fx.source, &delete_source()).unwrap();
            assert!(result.source_deleted);
            assert!(!fx.source.path.exists());
        });
    }

    /// `--delete-source` keeps a source whose packages did not all move:
    /// an unparsed freeze line, or a package that failed to install.
    #[test]
    #[serial]
    fn delete_source_keeps_a_source_with_an_unparsed_line() {
        run(
            "six==1.16.0\\nnot-a-pin\\n",
            EnvironmentStatus::Ready,
            &[],
            |fx| {
                let result = fx.migrator().migrate(&fx.source, &delete_source()).unwrap();
                assert!(!result.source_deleted);
                assert!(fx.source.path.exists());
            },
        );
    }

    /// Under `--strict` the first package that does not install stops the
    /// migration, with a reason that is word for word the English it was and
    /// follows the language. Fails if the reason is hard-coded again.
    #[test]
    #[serial]
    fn strict_install_failure_reason_follows_the_language() {
        let _locale = crate::test_utils::LocaleGuard::capture();
        let fails = [("FAKE_UV_PIP_INSTALL_FAILS", Some("bad==1.0"))];
        let strict = MigrateOptions {
            strict: true,
            ..Default::default()
        };
        for (lang, want) in [
            ("en", "Failed to install package: bad==1.0"),
            ("ko", "패키지 설치 실패: bad==1.0"),
        ] {
            rust_i18n::set_locale(lang);
            run(
                "six==1.16.0\\nbad==1.0\\n",
                EnvironmentStatus::Ready,
                &fails,
                |fx| {
                    let err = fx.migrator().migrate(&fx.source, &strict).unwrap_err();
                    assert!(
                        matches!(&err, ScoopError::MigrationFailed { reason } if reason == want),
                        "{err:?}"
                    );
                },
            );
        }
    }

    #[test]
    #[serial]
    fn delete_source_keeps_a_source_with_a_failed_install() {
        let fails = [("FAKE_UV_PIP_INSTALL_FAILS", Some("bad==1.0"))];
        run(
            "six==1.16.0\\nbad==1.0\\n",
            EnvironmentStatus::Ready,
            &fails,
            |fx| {
                let result = fx.migrator().migrate(&fx.source, &delete_source()).unwrap();
                assert_eq!(result.packages_failed, ["bad==1.0"]);
                // Two found, one failed: `-` here, not `+`.
                assert_eq!(result.packages_migrated, 1);
                assert!(!result.source_deleted);
                assert!(fx.source.path.exists());
            },
        );
    }

    /// `actual_python_version` is what the created env's pyvenv.cfg
    /// records (current uv writes the minor for its minor links), and
    /// the source's version when the file has none.
    #[test]
    #[serial]
    fn actual_python_version_comes_from_the_target_pyvenv_cfg() {
        let minor = [("FAKE_UV_VENV_VERSION_INFO", Some("3.12"))];
        run("six==1.16.0\\n", EnvironmentStatus::Ready, &minor, |fx| {
            let result = fx
                .migrator()
                .migrate(&fx.source, &MigrateOptions::default())
                .unwrap();
            assert_eq!(result.python_version, "3.12.0");
            assert_eq!(result.actual_python_version, "3.12");
        });
    }

    #[test]
    #[serial]
    fn actual_python_version_falls_back_to_the_source() {
        run("six==1.16.0\\n", EnvironmentStatus::Ready, &[], |fx| {
            let result = fx
                .migrator()
                .migrate(&fx.source, &MigrateOptions::default())
                .unwrap();
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
