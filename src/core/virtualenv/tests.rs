use super::*;
use crate::test_utils::{create_mock_venv, with_temp_scoop_home};
use serial_test::serial;

/// Helper to get VirtualenvService, skipping test if uv not available.
/// Returns None if uv is not installed, allowing graceful test skip.
fn get_service() -> Option<VirtualenvService> {
    crate::uv::UvClient::new().ok().map(VirtualenvService::new)
}

/// Macro to skip test if uv is not available.
/// This makes the skip explicit in test output.
macro_rules! require_uv {
    () => {
        match get_service() {
            Some(service) => service,
            None => {
                eprintln!("SKIPPED: uv not installed");
                return;
            }
        }
    };
}

#[test]
fn test_virtualenv_info_struct() {
    let info = VirtualenvInfo {
        name: "testenv".to_string(),
        path: PathBuf::from("/path/to/env"),
        python_version: Some("3.12".to_string()),
        created_at: None,
        last_used: None,
    };

    assert_eq!(info.name, "testenv");
    assert_eq!(info.path, PathBuf::from("/path/to/env"));
    assert_eq!(info.python_version, Some("3.12".to_string()));
}

#[test]
#[serial]
fn test_list_empty_when_no_venvs_dir() {
    with_temp_scoop_home(|_temp_dir| {
        let service = require_uv!();
        let result = service.list().unwrap();
        assert!(result.is_empty());
    });
}

#[test]
#[serial]
fn test_list_returns_envs_sorted() {
    with_temp_scoop_home(|temp_dir| {
        // Arrange: Create mock venvs in reverse alphabetical order
        create_mock_venv(temp_dir, "zeta", Some("3.11"));
        create_mock_venv(temp_dir, "alpha", Some("3.12"));
        create_mock_venv(temp_dir, "beta", None);

        // Act
        let service = require_uv!();
        let envs = service.list().unwrap();

        // Assert
        assert_eq!(envs.len(), 3);
        assert_eq!(envs[0].name, "alpha");
        assert_eq!(envs[1].name, "beta");
        assert_eq!(envs[2].name, "zeta");
    });
}

#[test]
#[serial]
fn test_list_reads_python_version_from_metadata() {
    with_temp_scoop_home(|temp_dir| {
        create_mock_venv(temp_dir, "withversion", Some("3.12.1"));
        create_mock_venv(temp_dir, "noversion", None);

        let service = require_uv!();
        let envs = service.list().unwrap();

        let with_ver = envs.iter().find(|e| e.name == "withversion").unwrap();
        let no_ver = envs.iter().find(|e| e.name == "noversion").unwrap();

        assert_eq!(with_ver.python_version, Some("3.12.1".to_string()));
        assert_eq!(no_ver.python_version, None);
    });
}

#[test]
#[serial]
fn test_exists_returns_false_for_nonexistent() {
    with_temp_scoop_home(|_temp_dir| {
        let service = require_uv!();
        assert!(!service.exists("nonexistent").unwrap());
    });
}

#[test]
#[serial]
fn test_exists_returns_true_for_existing() {
    with_temp_scoop_home(|temp_dir| {
        create_mock_venv(temp_dir, "exists", None);

        let service = require_uv!();
        assert!(service.exists("exists").unwrap());
    });
}

#[test]
#[serial]
fn test_get_path_returns_error_for_nonexistent() {
    with_temp_scoop_home(|_temp_dir| {
        let service = require_uv!();
        let result = service.get_path("nonexistent");

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ScoopError::VirtualenvNotFound { .. }));
    });
}

#[test]
#[serial]
fn test_get_path_returns_path_for_existing() {
    with_temp_scoop_home(|temp_dir| {
        create_mock_venv(temp_dir, "myenv", None);

        let service = require_uv!();
        let path = service.get_path("myenv").unwrap();
        assert!(path.ends_with("myenv"));
        assert!(path.exists());
    });
}

#[test]
#[serial]
fn test_delete_removes_directory() {
    with_temp_scoop_home(|temp_dir| {
        create_mock_venv(temp_dir, "todelete", Some("3.12"));
        let venv_path = temp_dir.path().join("virtualenvs").join("todelete");
        assert!(venv_path.exists());

        let service = require_uv!();
        service.delete("todelete").unwrap();
        assert!(!venv_path.exists());
    });
}

#[test]
#[serial]
fn test_delete_returns_error_for_nonexistent() {
    with_temp_scoop_home(|temp_dir| {
        // Arrange: Create virtualenvs dir but not the specific venv
        fs::create_dir_all(temp_dir.path().join("virtualenvs")).unwrap();

        // Act
        let service = require_uv!();
        let result = service.delete("nonexistent");

        // Assert
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ScoopError::VirtualenvNotFound { .. }));
    });
}

// ==========================================================================
// Path-traversal regression suite — every public name-taking method
// on VirtualenvService must reject hostile names BEFORE the path is
// joined. `PathBuf::join("/tmp/x")` returns `/tmp/x` (base discarded
// when right side is absolute), and `join("../foo")` doesn't strip
// the `..`; without the internal validate guard, a CLI handler that
// forgot to validate could hand attacker input straight to
// `remove_dir_all`. The guard is the trust boundary.
// ==========================================================================

fn bad_names() -> &'static [&'static str] {
    &["/tmp/evil", "../escape", "../../tmp/evil", "/", "."]
}

#[test]
#[serial]
fn test_delete_rejects_path_traversal() {
    with_temp_scoop_home(|temp_dir| {
        fs::create_dir_all(temp_dir.path().join("virtualenvs")).unwrap();
        let service = require_uv!();
        for bad in bad_names() {
            let err = service.delete(bad).unwrap_err();
            assert!(
                matches!(err, ScoopError::InvalidEnvName { .. }),
                "delete({bad:?}) should reject as InvalidEnvName, got: {err:?}"
            );
        }
    });
}

#[test]
#[serial]
fn test_exists_rejects_path_traversal() {
    with_temp_scoop_home(|temp_dir| {
        fs::create_dir_all(temp_dir.path().join("virtualenvs")).unwrap();
        let service = require_uv!();
        for bad in bad_names() {
            let err = service.exists(bad).unwrap_err();
            assert!(
                matches!(err, ScoopError::InvalidEnvName { .. }),
                "exists({bad:?}): {err:?}"
            );
        }
    });
}

#[test]
#[serial]
fn test_get_path_rejects_path_traversal() {
    with_temp_scoop_home(|temp_dir| {
        fs::create_dir_all(temp_dir.path().join("virtualenvs")).unwrap();
        let service = require_uv!();
        for bad in bad_names() {
            let err = service.get_path(bad).unwrap_err();
            assert!(
                matches!(err, ScoopError::InvalidEnvName { .. }),
                "get_path({bad:?}): {err:?}"
            );
        }
    });
}

#[test]
#[serial]
fn test_touch_metadata_at_blocks_relative_path_escape() {
    // Real teeth on the validation guard: plant a metadata file at
    // the *exact* location `"../escape"` would resolve to without
    // the guard (`<SCOOP_HOME>/virtualenvs/../escape` =
    // `<SCOOP_HOME>/escape`). Without the guard, touch would read
    // this file, mutate last_used, and persist it back — so the
    // captured serialized bytes would change. With the guard, the
    // file is never opened and the bytes stay identical.
    //
    // We use raw bytes (not the live Metadata struct) for the
    // canary so the assertion is byte-for-byte; a fresh re-serialize
    // would re-order fields or rewrite whitespace identically and
    // mask the read-modify-write that this test exists to catch.
    with_temp_scoop_home(|temp_dir| {
        fs::create_dir_all(temp_dir.path().join("virtualenvs")).unwrap();
        let escape_dir = temp_dir.path().join("escape");
        fs::create_dir_all(&escape_dir).unwrap();
        // Hand-rolled JSON that's parseable as Metadata (so the
        // hypothetical unguarded read would succeed) but written
        // with a recognizable last_used the test can distinguish
        // from the new write.
        let canary_path = escape_dir.join(".scoop-metadata.json");
        let canary_bytes = b"{\
            \"name\":\"escape\",\
            \"python_version\":\"3.12\",\
            \"created_at\":\"2024-01-01T00:00:00Z\",\
            \"created_by\":\"scoop canary\",\
            \"uv_version\":null,\
            \"last_used\":\"1999-12-31T23:59:59Z\"\
        }";
        fs::write(&canary_path, canary_bytes).unwrap();

        let service = require_uv!();
        let new_now = "2026-06-02T12:00:00Z".parse().unwrap();
        service.touch_metadata_at("../escape", new_now);

        // Byte-for-byte unchanged ⇒ the validation guard short-
        // circuited before read_metadata_result was called.
        // Without the guard, last_used would have flipped from
        // 1999-... to 2026-06-02 (and serde_json::to_string_pretty
        // would have reformatted whitespace too).
        let actual = fs::read(&canary_path).unwrap();
        assert_eq!(
            actual, canary_bytes,
            "validation guard must short-circuit before the escape \
             location's metadata is read or overwritten",
        );
    });
}

#[test]
#[serial]
fn test_touch_metadata_at_blocks_absolute_path_escape() {
    // Companion to the relative-escape test: absolute paths bypass
    // the base entirely via `PathBuf::join`. Plant a canary in a
    // separate tempdir, hand `touch_metadata_at` that absolute
    // path, and confirm the canary's bytes are unchanged.
    with_temp_scoop_home(|temp_dir| {
        fs::create_dir_all(temp_dir.path().join("virtualenvs")).unwrap();
        let outside = tempfile::TempDir::new().unwrap();
        let canary_path = outside.path().join(".scoop-metadata.json");
        let canary_bytes = b"{\
            \"name\":\"outside\",\
            \"python_version\":\"3.12\",\
            \"created_at\":\"2024-01-01T00:00:00Z\",\
            \"created_by\":\"scoop canary\",\
            \"uv_version\":null,\
            \"last_used\":\"1999-12-31T23:59:59Z\"\
        }";
        fs::write(&canary_path, canary_bytes).unwrap();

        let service = require_uv!();
        let new_now = "2026-06-02T12:00:00Z".parse().unwrap();
        // Pass the absolute path to outside/ as the "env name".
        // Without the validation guard, paths::virtualenv_path
        // would join this onto virtualenvs_dir and discard the
        // base (Rust's PathBuf::join semantics), landing exactly
        // on the canary.
        let bad_name = outside.path().to_str().unwrap();
        service.touch_metadata_at(bad_name, new_now);

        let actual = fs::read(&canary_path).unwrap();
        assert_eq!(
            actual, canary_bytes,
            "validation guard must reject absolute path before the \
             canary is read or overwritten",
        );
    });
}

#[test]
#[serial]
fn test_list_ignores_files() {
    with_temp_scoop_home(|temp_dir| {
        let venvs_dir = temp_dir.path().join("virtualenvs");
        fs::create_dir_all(&venvs_dir).unwrap();

        // Create a file (not directory) - should be ignored
        fs::write(venvs_dir.join("notadir"), "test").unwrap();
        // Create a real venv directory
        create_mock_venv(temp_dir, "realenv", None);

        let service = require_uv!();
        let envs = service.list().unwrap();

        assert_eq!(envs.len(), 1);
        assert_eq!(envs[0].name, "realenv");
    });
}

// ==========================================================================
// last_used / atomic write / touch_metadata_best_effort
// ==========================================================================

fn seed_metadata_file(env_path: &Path, json: &str) {
    fs::create_dir_all(env_path).unwrap();
    fs::write(env_path.join(Metadata::FILE_NAME), json).unwrap();
}

#[test]
#[serial]
fn test_read_metadata_result_distinguishes_missing_from_corrupt() {
    with_temp_scoop_home(|temp_dir| {
        let service = require_uv!();
        let env_dir = temp_dir.path().join("virtualenvs").join("subject");
        fs::create_dir_all(&env_dir).unwrap();

        // No file → Ok(None). Distinct from "corrupt" because we
        // shouldn't warn or refuse anything for a missing file.
        assert!(service.read_metadata_result(&env_dir).unwrap().is_none());

        // Corrupt JSON → Err(Json(..)). Caller needs this signal to
        // decide whether to overwrite (no) or warn (yes).
        fs::write(env_dir.join(Metadata::FILE_NAME), "{ not json").unwrap();
        let err = service.read_metadata_result(&env_dir).unwrap_err();
        assert!(
            matches!(err, ScoopError::Json(_)),
            "corrupt JSON must yield ScoopError::Json, got: {err:?}"
        );
    });
}

#[test]
#[serial]
fn test_read_metadata_result_returns_err_on_non_notfound_io() {
    // Pins the NotFound match-guard contract: only ErrorKind::NotFound
    // collapses to Ok(None); every other IO error surfaces as Err so
    // touch_metadata_at refuses to overwrite. Without this assertion
    // a mutation `e.kind() == NotFound` → `true` would silently treat
    // every IO failure (permission denied, "is a directory", etc.) as
    // "file missing" and let touch happily synthesize a new file.
    //
    // Simulate a non-NotFound IO error by planting a directory at the
    // metadata file path — `fs::read_to_string` then fails with
    // ErrorKind::IsADirectory (or InvalidInput on some platforms),
    // neither of which match the NotFound arm.
    with_temp_scoop_home(|temp_dir| {
        let service = require_uv!();
        let env_dir = temp_dir.path().join("virtualenvs").join("dir_as_meta");
        fs::create_dir_all(env_dir.join(Metadata::FILE_NAME)).unwrap();

        let err = service.read_metadata_result(&env_dir).unwrap_err();
        assert!(
            matches!(err, ScoopError::Io(_)),
            "non-NotFound IO error must surface as Err, not collapse to Ok(None); got {err:?}"
        );
    });
}

#[test]
#[serial]
fn test_touch_metadata_best_effort_updates_only_last_used() {
    with_temp_scoop_home(|temp_dir| {
        let service = require_uv!();
        let env_path = temp_dir.path().join("virtualenvs").join("touched");
        let seed = r#"{
            "name": "touched",
            "python_version": "3.12.1",
            "created_at": "2024-01-15T10:30:00Z",
            "created_by": "scoop 0.5.0",
            "uv_version": "0.4.0"
        }"#;
        seed_metadata_file(&env_path, seed);

        let now = "2026-06-02T12:00:00Z".parse::<DateTime<Utc>>().unwrap();
        service.touch_metadata_at("touched", now);

        let after = service
            .read_metadata_result(&env_path)
            .unwrap()
            .expect("file present after touch");
        assert_eq!(after.last_used, Some(now));
        // Provenance fields must survive a touch — they describe how
        // the env was created and a touch is not a re-creation.
        assert_eq!(after.name, "touched");
        assert_eq!(after.python_version, "3.12.1");
        assert_eq!(after.created_by, "scoop 0.5.0");
        assert_eq!(after.uv_version, Some("0.4.0".to_string()));
        let expected_created_at = "2024-01-15T10:30:00Z".parse::<DateTime<Utc>>().unwrap();
        assert_eq!(after.created_at, expected_created_at);
    });
}

#[test]
#[serial]
fn test_touch_metadata_best_effort_preserves_corrupt_file() {
    with_temp_scoop_home(|temp_dir| {
        let service = require_uv!();
        let env_path = temp_dir.path().join("virtualenvs").join("broken");
        let garbage = "{ this is not valid json";
        seed_metadata_file(&env_path, garbage);

        // Must not panic / propagate the error. Must not overwrite.
        service.touch_metadata_at("broken", "2026-06-02T12:00:00Z".parse().unwrap());

        let on_disk = fs::read_to_string(env_path.join(Metadata::FILE_NAME)).unwrap();
        assert_eq!(
            on_disk, garbage,
            "corrupt metadata must be preserved verbatim — overwriting it \
             would destroy the user's only forensic trace of the corruption"
        );
    });
}

#[test]
#[serial]
fn test_touch_metadata_best_effort_noop_on_missing_metadata() {
    with_temp_scoop_home(|temp_dir| {
        let service = require_uv!();
        let env_path = temp_dir.path().join("virtualenvs").join("nofile");
        fs::create_dir_all(&env_path).unwrap();

        // Legacy env with no metadata file. Must NOT synthesize one
        // (that would lie about created_at) and must NOT error.
        service.touch_metadata_at("nofile", "2026-06-02T12:00:00Z".parse().unwrap());
        assert!(!env_path.join(Metadata::FILE_NAME).exists());
    });
}

#[test]
#[serial]
fn test_touch_metadata_best_effort_wall_clock_writes_recent() {
    // Smoke test for the public no-arg entry point: it must actually
    // call Utc::now() at write time, not silently no-op. We can't pin
    // the exact value, so we assert it's bracketed by a before/after
    // wall-clock window.
    with_temp_scoop_home(|temp_dir| {
        let service = require_uv!();
        let env_path = temp_dir.path().join("virtualenvs").join("wallclock");
        let seed = r#"{
            "name": "wallclock",
            "python_version": "3.12",
            "created_at": "2024-01-01T00:00:00Z",
            "created_by": "scoop test",
            "uv_version": null
        }"#;
        seed_metadata_file(&env_path, seed);

        let before = Utc::now();
        service.touch_metadata_best_effort("wallclock");
        let after = Utc::now();

        let m = service.read_metadata_result(&env_path).unwrap().unwrap();
        let touched = m.last_used.expect("last_used set");
        assert!(
            touched >= before && touched <= after,
            "{touched} not in [{before},{after}]"
        );
    });
}

#[test]
#[serial]
fn test_read_metadata_legacy_api_swallows_corrupt() {
    // The old `read_metadata` API returns `Option<Metadata>` and
    // collapses both "missing" and "corrupt" into `None`. Pin that
    // behavior explicitly so a mutant that returns `Some(default)`
    // (won't compile — Metadata has no Default), or that flips the
    // collapse, gets caught.
    with_temp_scoop_home(|temp_dir| {
        let service = require_uv!();
        let env_dir = temp_dir.path().join("virtualenvs").join("legacyapi");
        fs::create_dir_all(&env_dir).unwrap();

        // No file → None
        assert!(service.read_metadata(&env_dir).is_none());

        // Healthy file → Some
        let seed = r#"{
            "name": "legacyapi",
            "python_version": "3.12",
            "created_at": "2024-01-01T00:00:00Z",
            "created_by": "scoop test",
            "uv_version": null
        }"#;
        fs::write(env_dir.join(Metadata::FILE_NAME), seed).unwrap();
        assert!(service.read_metadata(&env_dir).is_some());

        // Corrupt file → None (collapses with missing, by design)
        fs::write(env_dir.join(Metadata::FILE_NAME), "{ nope").unwrap();
        assert!(service.read_metadata(&env_dir).is_none());
    });
}

#[test]
#[serial]
fn test_write_metadata_atomic_round_trips() {
    with_temp_scoop_home(|temp_dir| {
        let service = require_uv!();
        let env_path = temp_dir.path().join("virtualenvs").join("atomic");
        fs::create_dir_all(&env_path).unwrap();

        let mut meta = Metadata::new("atomic".to_string(), "3.12".to_string(), None);
        let now = "2026-06-02T12:00:00Z".parse::<DateTime<Utc>>().unwrap();
        meta.touch(now);

        service.write_metadata_atomic(&env_path, &meta).unwrap();

        let restored = service
            .read_metadata_result(&env_path)
            .unwrap()
            .expect("file exists after atomic write");
        assert_eq!(restored.last_used, Some(now));

        // No tempfile left behind. tempfile::NamedTempFile::persist
        // promises this but we assert it explicitly so any future
        // change to the impl that breaks cleanup gets caught.
        let leftover: Vec<_> = fs::read_dir(&env_path)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| {
                let n = e.file_name();
                let s = n.to_string_lossy();
                s.starts_with(".tmp") || s.contains("scoop-metadata.json.tmp")
            })
            .collect();
        assert!(leftover.is_empty(), "tempfile residue: {leftover:?}");
    });
}

#[test]
#[serial]
fn test_write_metadata_atomic_overwrites_existing() {
    with_temp_scoop_home(|temp_dir| {
        let service = require_uv!();
        let env_path = temp_dir.path().join("virtualenvs").join("overwrite");
        fs::create_dir_all(&env_path).unwrap();

        // First write
        let m1 = Metadata::new("overwrite".to_string(), "3.11".to_string(), None);
        service.write_metadata_atomic(&env_path, &m1).unwrap();

        // Second write replaces it atomically.
        let mut m2 = Metadata::new("overwrite".to_string(), "3.12".to_string(), None);
        m2.touch("2026-06-02T12:00:00Z".parse().unwrap());
        service.write_metadata_atomic(&env_path, &m2).unwrap();

        let on_disk = service.read_metadata_result(&env_path).unwrap().unwrap();
        assert_eq!(on_disk.python_version, "3.12");
        assert!(on_disk.last_used.is_some());
    });
}

// C2 regression — symlinks under virtualenvs/ must NOT be enumerated.
// Otherwise downstream commands (gc, verify, ...) would treat the
// symlink target as a real env and end up scanning / exec'ing files
// outside the venvs dir.
#[cfg(unix)]
#[test]
#[serial]
fn test_list_skips_symlink_entries() {
    with_temp_scoop_home(|temp_dir| {
        let venvs_dir = temp_dir.path().join("virtualenvs");
        fs::create_dir_all(&venvs_dir).unwrap();

        // Real env so the list isn't empty (controls for "filter is
        // entirely broken" vs "filter caught the symlink").
        create_mock_venv(temp_dir, "real", None);

        // Plant a symlink → some other (existing) directory. Without
        // the symlink filter this would be enumerated as an env.
        let other = tempfile::TempDir::new().unwrap();
        std::os::unix::fs::symlink(other.path(), venvs_dir.join("symlinked")).unwrap();

        let service = require_uv!();
        let envs = service.list().unwrap();
        assert_eq!(envs.len(), 1, "symlink entries must be skipped: {envs:?}");
        assert_eq!(envs[0].name, "real");
    });
}
