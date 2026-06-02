use super::*;
use crate::test_utils::with_temp_scoop_home;
use serial_test::serial;
use std::fs;

fn make_env(dir: &Path, name: &str, with_metadata: bool, with_python: bool) {
    let env_dir = dir.join(name);
    fs::create_dir_all(&env_dir).unwrap();
    if with_metadata {
        fs::write(env_dir.join(".scoop-metadata.json"), "{}").unwrap();
    }
    if with_python {
        let bin = if cfg!(windows) {
            env_dir.join("Scripts")
        } else {
            env_dir.join("bin")
        };
        fs::create_dir_all(&bin).unwrap();
        let py = if cfg!(windows) {
            bin.join("python.exe")
        } else {
            bin.join("python")
        };
        fs::write(&py, "").unwrap();
    }
}

/// Build a fully-healthy env (so it doesn't get classified as an
/// orphan) with a `last_used` set to `last_used`. None means
/// "never used"; Some(t) lets the caller pin the age relative to
/// the cutoff under test.
fn make_env_with_last_used(dir: &Path, name: &str, last_used: Option<DateTime<Utc>>) {
    make_env(dir, name, true, true);
    let meta_path = dir.join(name).join(".scoop-metadata.json");
    let last_used_field = match last_used {
        Some(t) => format!("\"last_used\":\"{}\",", t.to_rfc3339()),
        None => String::new(),
    };
    let payload = format!(
        "{{\"name\":\"{name}\",\"python_version\":\"3.12\",\
         \"created_at\":\"2024-01-01T00:00:00Z\",\
         \"created_by\":\"scoop test\",\"uv_version\":null,\
         {last_used_field}\"python_path\":null}}"
    );
    fs::write(meta_path, payload).unwrap();
}

#[test]
#[serial]
fn classifies_missing_metadata_as_orphan() {
    with_temp_scoop_home(|_| {
        let dir = paths::virtualenvs_dir().unwrap();
        fs::create_dir_all(&dir).unwrap();
        make_env(&dir, "no-meta", false, true);

        let orphans = scan_orphan_envs().unwrap();
        assert_eq!(orphans.len(), 1);
        assert_eq!(orphans[0].name, "no-meta");
        assert_eq!(orphans[0].reason, EnvGcReason::OrphanMissingMetadata);
    });
}

#[test]
#[serial]
fn classifies_broken_python_as_orphan() {
    with_temp_scoop_home(|_| {
        let dir = paths::virtualenvs_dir().unwrap();
        fs::create_dir_all(&dir).unwrap();
        make_env(&dir, "no-python", true, false);

        let orphans = scan_orphan_envs().unwrap();
        assert_eq!(orphans.len(), 1);
        assert_eq!(orphans[0].name, "no-python");
        assert_eq!(orphans[0].reason, EnvGcReason::OrphanBrokenPython);
    });
}

#[test]
#[serial]
fn healthy_env_is_not_an_orphan() {
    with_temp_scoop_home(|_| {
        let dir = paths::virtualenvs_dir().unwrap();
        fs::create_dir_all(&dir).unwrap();
        make_env(&dir, "ok", true, true);

        let orphans = scan_orphan_envs().unwrap();
        assert_eq!(orphans.len(), 0);
    });
}

#[test]
#[serial]
fn dotfile_entries_are_skipped() {
    with_temp_scoop_home(|_| {
        let dir = paths::virtualenvs_dir().unwrap();
        fs::create_dir_all(&dir).unwrap();
        fs::create_dir_all(dir.join(".cache")).unwrap();

        let orphans = scan_orphan_envs().unwrap();
        assert!(orphans.is_empty());
    });
}

#[test]
#[serial]
fn dry_run_does_not_remove() {
    with_temp_scoop_home(|_| {
        let dir = paths::virtualenvs_dir().unwrap();
        fs::create_dir_all(&dir).unwrap();
        make_env(&dir, "no-meta", false, true);

        let output = Output::new(0, true, true, false);
        execute(&output, false, false, None).unwrap();

        assert!(
            dir.join("no-meta").exists(),
            "dry-run must not delete orphans"
        );
    });
}

#[test]
#[serial]
fn yes_actually_removes_orphans() {
    with_temp_scoop_home(|_| {
        let dir = paths::virtualenvs_dir().unwrap();
        fs::create_dir_all(&dir).unwrap();
        make_env(&dir, "no-meta", false, true);

        let output = Output::new(0, true, true, false);
        execute(&output, true, false, None).unwrap();

        assert!(!dir.join("no-meta").exists(), "--yes should remove orphans");
    });
}

// ==========================================================================
// S1 regression — symlinks must never be classified as orphans, even
// when their target lacks .scoop-metadata.json. Otherwise gc --yes
// would follow the symlink via remove_dir_all and delete the target.
// ==========================================================================
#[cfg(unix)]
#[test]
#[serial]
fn scan_skips_symlink_entries() {
    with_temp_scoop_home(|_| {
        let dir = paths::virtualenvs_dir().unwrap();
        fs::create_dir_all(&dir).unwrap();

        // Set up a real directory OUTSIDE the venvs dir — the would-be
        // deletion target. It deliberately lacks .scoop-metadata.json
        // so if the symlink were followed it would classify as
        // MissingMetadata.
        let outside = tempfile::TempDir::new().unwrap();
        let canary = outside.path().join("important.txt");
        fs::write(&canary, b"do not delete").unwrap();

        // Symlink "evil" inside virtualenvs/ → outside dir.
        std::os::unix::fs::symlink(outside.path(), dir.join("evil")).unwrap();

        let orphans = scan_orphan_envs().unwrap();
        assert!(
            orphans.iter().all(|o| o.name != "evil"),
            "symlink must not be classified as an orphan: {:?}",
            orphans
        );

        // Defense-in-depth: even the full --yes path must leave the
        // canary intact.
        let output = Output::new(0, true, true, false);
        execute(&output, true, false, None).unwrap();
        assert!(
            canary.exists(),
            "gc --yes followed the symlink and deleted the target"
        );
    });
}

// ==========================================================================
// Q2 regression — unreadable metadata on a surviving env must NOT
// cause gc --aggressive to claim that env's Python is unused. The
// scan must bail conservatively and return zero unused pythons.
// ==========================================================================
#[test]
#[serial]
fn aggressive_bails_when_metadata_unreadable() {
    with_temp_scoop_home(|_| {
        let dir = paths::virtualenvs_dir().unwrap();
        fs::create_dir_all(&dir).unwrap();

        // Healthy-shaped env (metadata file + python binary present)
        // so classify() doesn't flag it as an orphan. But the
        // metadata content is garbage so read_metadata returns None.
        let env_path = dir.join("corrupt");
        fs::create_dir_all(env_path.join("bin")).unwrap();
        fs::write(env_path.join("bin/python"), "").unwrap();
        fs::write(env_path.join(".scoop-metadata.json"), "{ not json").unwrap();

        // Sanity: this env is healthy by classify(), so it's not in
        // orphans — exactly the dangerous case where the old code
        // silently dropped it from the "used" set.
        let orphans = scan_orphan_envs().unwrap();
        assert!(orphans.iter().all(|o| o.name != "corrupt"));

        // The scan must bail with `unreadable_envs > 0` and return
        // an empty pythons list — refusing to mark any Python as
        // unused, no matter what `uv python list` reports.
        let (pythons, unreadable_envs) = scan_unused_pythons(&orphans).unwrap();
        assert_eq!(unreadable_envs, 1, "should count one unreadable env");
        assert!(
            pythons.is_empty(),
            "must not claim any Python is unused when metadata is unreadable; got {:?}",
            pythons
        );
    });
}

// ==========================================================================
// Q3 regression — TOCTOU between scan and remove. We simulate by
// building a fake orphan record that points at a path which is
// currently healthy. remove_orphans must re-classify and skip.
// ==========================================================================
#[test]
#[serial]
fn remove_skips_env_that_became_healthy() {
    with_temp_scoop_home(|_| {
        let dir = paths::virtualenvs_dir().unwrap();
        fs::create_dir_all(&dir).unwrap();
        // Make a fully healthy env at the path the fake orphan record
        // will reference. This simulates the racing `scoop create`
        // that ran between scan and remove.
        let env_path = dir.join("racy");
        fs::create_dir_all(env_path.join("bin")).unwrap();
        fs::write(env_path.join("bin/python"), "").unwrap();
        fs::write(env_path.join(".scoop-metadata.json"), "{}").unwrap();

        // Hand-construct an orphan record as if the original scan had
        // flagged it (before the user re-populated the dir).
        let stale_orphan = OrphanEnv {
            name: "racy".to_string(),
            path: env_path.display().to_string(),
            reason: EnvGcReason::OrphanMissingMetadata,
            age_days: None,
        };
        let mut env_records = vec![EnvRecord {
            name: stale_orphan.name.clone(),
            path: stale_orphan.path.clone(),
            reason: stale_orphan.reason,
            age_days: stale_orphan.age_days,
            outcome: EnvOutcome::Pending,
            error: None,
        }];

        let output = Output::new(0, true, true, false);
        remove_orphans(
            &output,
            &[stale_orphan],
            &[],
            &mut env_records,
            &mut Vec::<PythonRecord>::new(),
            None,
        );

        // The env was healthy at remove-time so the destructive path
        // must not have run — both the disk state AND the JSON-bound
        // outcome record need to agree on that.
        assert!(
            env_path.exists(),
            "remove_orphans deleted an env that re-classified as healthy"
        );
        assert_eq!(
            env_records[0].outcome,
            EnvOutcome::SkippedHealthy,
            "outcome must be SkippedHealthy so JSON consumers see the skip"
        );
    });
}

// ==========================================================================
// Step 5 — `gc --older-than` / EnvGcReason::Stale contract
// ==========================================================================

#[test]
#[serial]
fn stale_scan_matches_env_past_cutoff() {
    with_temp_scoop_home(|_| {
        let dir = paths::virtualenvs_dir().unwrap();
        fs::create_dir_all(&dir).unwrap();

        let now = Utc::now();
        // 60 days idle vs a 30-day cutoff → must be flagged.
        make_env_with_last_used(&dir, "old", Some(now - chrono::Duration::days(60)));
        // 5 days idle vs a 30-day cutoff → must NOT be flagged.
        make_env_with_last_used(&dir, "fresh", Some(now - chrono::Duration::days(5)));

        let cutoff = now - chrono::Duration::days(30);
        let stale = scan_stale_envs(cutoff).unwrap();

        let names: Vec<_> = stale.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, vec!["old"], "only past-cutoff env is stale");
        assert!(matches!(stale[0].reason, EnvGcReason::Stale));
        assert!(
            stale[0].age_days.unwrap_or(0) >= 59,
            "age_days should be ~60: {:?}",
            stale[0].age_days
        );
    });
}

#[test]
#[serial]
fn stale_scan_never_matches_last_used_none() {
    // Conservative rule from the v2 plan: `last_used = None` could
    // mean "never used" (fresh env) or "predates the field" (legacy
    // metadata). Either way no positive evidence the env is unused —
    // refuse to flag it.
    with_temp_scoop_home(|_| {
        let dir = paths::virtualenvs_dir().unwrap();
        fs::create_dir_all(&dir).unwrap();

        make_env_with_last_used(&dir, "untouched", None);

        // Any cutoff: a None last_used must never be flagged stale.
        let cutoff = Utc::now() - chrono::Duration::days(1);
        let stale = scan_stale_envs(cutoff).unwrap();
        assert!(
            stale.is_empty(),
            "last_used=None must never match stale: {stale:?}"
        );
    });
}

#[test]
#[serial]
fn stale_scan_ignores_orphans() {
    // Orphan-classified envs are removed via the orphan path; the
    // stale scan must skip them so we don't double-count.
    with_temp_scoop_home(|_| {
        let dir = paths::virtualenvs_dir().unwrap();
        fs::create_dir_all(&dir).unwrap();
        make_env(&dir, "no-meta", false, true);

        let cutoff = Utc::now() - chrono::Duration::days(1);
        let stale = scan_stale_envs(cutoff).unwrap();
        assert!(
            stale.is_empty(),
            "orphans must not double-appear in stale scan"
        );
    });
}

#[test]
#[serial]
fn stale_scan_skips_corrupt_metadata() {
    // Same conservative rule as None: corrupt metadata means we
    // can't tell when the env was last used.
    with_temp_scoop_home(|_| {
        let dir = paths::virtualenvs_dir().unwrap();
        fs::create_dir_all(&dir).unwrap();

        let env_path = dir.join("corrupt");
        fs::create_dir_all(env_path.join("bin")).unwrap();
        fs::write(env_path.join("bin/python"), "").unwrap();
        fs::write(env_path.join(".scoop-metadata.json"), "{ not json").unwrap();

        let cutoff = Utc::now() - chrono::Duration::days(1);
        let stale = scan_stale_envs(cutoff).unwrap();
        assert!(
            stale.is_empty(),
            "corrupt metadata must not match stale: {stale:?}"
        );
    });
}

#[test]
#[serial]
fn recheck_stale_skips_recently_touched_env() {
    // TOCTOU: between scan and remove the env was touched
    // (last_used moved past the original cutoff). recheck_stale
    // must report SkippedRecentlyUsed.
    with_temp_scoop_home(|_| {
        let dir = paths::virtualenvs_dir().unwrap();
        fs::create_dir_all(&dir).unwrap();

        let now = Utc::now();
        let cutoff = now - chrono::Duration::days(30);
        make_env_with_last_used(&dir, "racy", Some(now - chrono::Duration::days(5)));

        let outcome = recheck_stale("racy", cutoff);
        assert_eq!(outcome, Some(EnvOutcome::SkippedRecentlyUsed));
    });
}

#[test]
#[serial]
fn recheck_stale_reports_no_data_when_metadata_lost() {
    // Metadata disappearing between scan and remove must NOT let
    // us blindly delete the env — caller marks the record
    // SkippedNoData.
    with_temp_scoop_home(|_| {
        let dir = paths::virtualenvs_dir().unwrap();
        fs::create_dir_all(&dir).unwrap();

        let env_path = dir.join("gone");
        fs::create_dir_all(env_path.join("bin")).unwrap();
        fs::write(env_path.join("bin/python"), "").unwrap();

        let cutoff = Utc::now() - chrono::Duration::days(30);
        let outcome = recheck_stale("gone", cutoff);
        assert_eq!(outcome, Some(EnvOutcome::SkippedNoData));
    });
}

#[test]
#[serial]
fn execute_rejects_invalid_older_than() {
    // Surface the parse error so a user with a typo gets a clear
    // message instead of "no envs to remove". Walk the major
    // rejection arms so the Codex MEDIUM-2 ("`garbage` only" gap)
    // is closed at the execute boundary too — including the
    // u64::MAX value that Codex STOP-2 caught silently truncating
    // in the previous parse_duration impl.
    with_temp_scoop_home(|_| {
        let dir = paths::virtualenvs_dir().unwrap();
        fs::create_dir_all(&dir).unwrap();

        let output = Output::new(0, true, true, false);
        for bad in ["garbage", "0d", "6m", "18446744073709551615d", "200y1d"] {
            let err = execute(&output, false, false, Some(bad)).unwrap_err();
            assert!(
                matches!(err, crate::error::ScoopError::InvalidArgument { .. }),
                "expected InvalidArgument for {bad}, got {err:?}"
            );
        }
    });
}

#[test]
#[serial]
fn stale_scan_equality_boundary_is_inclusive_of_cutoff() {
    // The contract is `last_used < cutoff → stale`. An env whose
    // `last_used` is *exactly* the cutoff must NOT be flagged.
    // Without this assertion a `<` → `<=` mutation could flip the
    // boundary undetected and start removing barely-fresh envs.
    with_temp_scoop_home(|_| {
        let dir = paths::virtualenvs_dir().unwrap();
        fs::create_dir_all(&dir).unwrap();

        let cutoff = Utc::now() - chrono::Duration::days(30);
        make_env_with_last_used(&dir, "borderline", Some(cutoff));

        let stale = scan_stale_envs(cutoff).unwrap();
        assert!(
            stale.is_empty(),
            "equality-at-cutoff must NOT be stale: {stale:?}"
        );
    });
}

#[test]
#[serial]
fn recheck_stale_rejects_path_traversal() {
    // Defense-in-depth: even though current callers only feed
    // disk-walked basenames to recheck_stale, a path-traversal
    // name must surface as SkippedNoData (refuse to operate),
    // never as Removed.
    with_temp_scoop_home(|_| {
        let dir = paths::virtualenvs_dir().unwrap();
        fs::create_dir_all(&dir).unwrap();
        let cutoff = Utc::now() - chrono::Duration::days(30);
        for bad in ["/tmp/evil", "../escape", "../../etc/passwd"] {
            assert_eq!(
                recheck_stale(bad, cutoff),
                Some(EnvOutcome::SkippedNoData),
                "recheck_stale({bad:?}) must report SkippedNoData",
            );
        }
    });
}

#[test]
#[serial]
fn recheck_stale_returns_none_when_still_past_cutoff() {
    // The "still-stale, proceed with delete" path: recheck must
    // return None so the caller falls through to remove_dir_all.
    // No previous test pinned this branch — a mutation flipping
    // `last_used >= cutoff` to `last_used > cutoff` would have
    // slipped through.
    with_temp_scoop_home(|_| {
        let dir = paths::virtualenvs_dir().unwrap();
        fs::create_dir_all(&dir).unwrap();

        let now = Utc::now();
        // last_used 60d ago, cutoff 30d ago → still stale.
        make_env_with_last_used(&dir, "still_stale", Some(now - chrono::Duration::days(60)));
        let cutoff = now - chrono::Duration::days(30);

        let outcome = recheck_stale("still_stale", cutoff);
        assert_eq!(outcome, None, "recheck must say 'proceed with delete'");
    });
}

#[test]
#[serial]
fn json_envelope_keeps_flat_reason_for_orphans() {
    // Codex STOP-1 on this commit's predecessor: switching from
    // `#[serde(rename)]` to `#[serde(tag = "kind")]` had silently
    // changed the wire shape from `reason: "missing_metadata"` to
    // `reason: {"kind": "missing_metadata"}`. Pin both contracts
    // (orphan = flat string, stale = flat string + sibling
    // age_days) so any future enum re-tagging immediately fails
    // here instead of breaking JSON consumers in the field.
    let orphan = EnvRecord {
        name: "ghost".into(),
        path: "/tmp/ghost".into(),
        reason: EnvGcReason::OrphanMissingMetadata,
        age_days: None,
        outcome: EnvOutcome::Pending,
        error: None,
    };
    let json = serde_json::to_value(&orphan).unwrap();
    assert_eq!(
        json["reason"],
        serde_json::Value::String("missing_metadata".into())
    );
    assert!(
        json.get("age_days").is_none(),
        "no age_days for orphans: {json}"
    );

    let stale = EnvRecord {
        name: "old".into(),
        path: "/tmp/old".into(),
        reason: EnvGcReason::Stale,
        age_days: Some(62),
        outcome: EnvOutcome::Pending,
        error: None,
    };
    let json = serde_json::to_value(&stale).unwrap();
    assert_eq!(json["reason"], serde_json::Value::String("stale".into()));
    assert_eq!(json["age_days"], serde_json::Value::from(62u64));
}

#[test]
#[serial]
fn remove_treats_not_found_as_already_removed() {
    // Codex MEDIUM-1: parallel `--yes` would otherwise classify
    // the second runner's `remove_dir_all` as Failed via a noisy
    // NotFound. Pin the "absent at remove time → success"
    // contract so the JSON envelope doesn't lie about a phantom
    // failure.
    with_temp_scoop_home(|_| {
        let dir = paths::virtualenvs_dir().unwrap();
        fs::create_dir_all(&dir).unwrap();

        let phantom = OrphanEnv {
            name: "phantom".into(),
            path: dir.join("phantom-already-gone").display().to_string(),
            reason: EnvGcReason::OrphanMissingMetadata,
            age_days: None,
        };
        let mut env_records = vec![EnvRecord {
            name: phantom.name.clone(),
            path: phantom.path.clone(),
            reason: phantom.reason,
            age_days: None,
            outcome: EnvOutcome::Pending,
            error: None,
        }];

        let output = Output::new(0, true, true, false);
        remove_orphans(
            &output,
            &[phantom],
            &[],
            &mut env_records,
            &mut Vec::<PythonRecord>::new(),
            None,
        );

        assert_eq!(
            env_records[0].outcome,
            EnvOutcome::Removed,
            "NotFound at remove time must report Removed, not Failed"
        );
        assert!(env_records[0].error.is_none());
    });
}

// ==========================================================================
// C3 regression — JSON envelope must reflect actual outcomes, not the
// pre-action scan snapshot. We don't run the full `execute` here (uv
// would need to be present for --aggressive paths); we verify the
// record-mutation contract directly.
// ==========================================================================
#[test]
#[serial]
fn remove_records_actual_outcomes_for_each_env() {
    with_temp_scoop_home(|_| {
        let dir = paths::virtualenvs_dir().unwrap();
        fs::create_dir_all(&dir).unwrap();
        // Two orphans we expect to be successfully removed.
        make_env(&dir, "ghost-a", false, true);
        make_env(&dir, "ghost-b", false, true);

        let orphans = scan_orphan_envs().unwrap();
        assert_eq!(orphans.len(), 2);
        let mut env_records: Vec<EnvRecord> = orphans
            .iter()
            .map(|o| EnvRecord {
                name: o.name.clone(),
                path: o.path.clone(),
                reason: o.reason,
                age_days: o.age_days,
                outcome: EnvOutcome::Pending,
                error: None,
            })
            .collect();

        let output = Output::new(0, true, true, false);
        remove_orphans(
            &output,
            &orphans,
            &[],
            &mut env_records,
            &mut Vec::<PythonRecord>::new(),
            None,
        );

        for record in &env_records {
            assert_eq!(
                record.outcome,
                EnvOutcome::Removed,
                "expected Removed outcome for {}, got {:?}",
                record.name,
                record.outcome
            );
            assert!(record.error.is_none());
        }
    });
}
