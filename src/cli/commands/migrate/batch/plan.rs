//! Planning a batch: which scanned environments migrate, which conflict,
//! which are skipped.

use std::collections::HashMap;
use std::path::Path;

use crate::core::migrate::{EnvironmentStatus, SourceEnvironment, SourceType};

use super::super::types::{MigrateSkipped, MigrationConflictDetail};

/// The three buckets a scan splits into.
pub(super) struct PartitionedEnvs<'a> {
    pub(super) migratable: Vec<&'a SourceEnvironment>,
    pub(super) conflicts: Vec<MigrationConflictDetail>,
    pub(super) skipped: Vec<MigrateSkipped>,
}

/// Split scanned envs into migratable / conflict / skipped buckets.
///
/// `conflicts` carries the structured detail (added in 0.14). `skipped`
/// still includes name-conflicts under the historical
/// "name conflict (use --force)" reason so consumers reading
/// `data.skipped[]` continue to work — this is intentional, see the
/// `MigrateAllData` doc comment.
///
/// `venvs_dir` is where migrated envs land; it only names the target in
/// conflict details.
pub(super) fn partition_envs<'a>(
    environments: &'a [SourceEnvironment],
    force: bool,
    allow_eol: bool,
    venvs_dir: &Path,
) -> PartitionedEnvs<'a> {
    let mut migratable: Vec<&SourceEnvironment> = Vec::new();
    let mut conflicts: Vec<MigrationConflictDetail> = Vec::new();
    let mut skipped: Vec<MigrateSkipped> = Vec::new();
    // Target names already claimed by an earlier entry in this same batch.
    //
    // `EnvironmentStatus::NameConflict` only reports collisions with an
    // environment that already exists on disk (see
    // `core::migrate::common::check_name_conflict`), so two sources offering
    // the same name — pyenv `web` and conda `web`, say — both arrive here as
    // `Ready`. They then migrate in parallel (`par_iter` in `run.rs`) to the same
    // path, and `create_target_env` decides what happens by whichever thread
    // wins the race: without `--force` the loser reports `VirtualenvExists`;
    // with it, the loser calls `remove_dir_all` on the winner's environment
    // mid-install, and the winner's `RollbackGuard` can then delete the
    // replacement. Combined with `--delete-source` that destroys one of the
    // two source environments for good, while the summary reports both as
    // migrated.
    //
    // `force` deliberately does not override this. `--force` means "overwrite
    // the environment that is already there", and for a collision inside one
    // batch there is no such environment — both are new. Letting them through
    // is the bug, not the fix. The first claim migrates; the user moves the
    // skipped one with `scuv migrate @env <name> --source <tool> --rename
    // <other>`. Without `--source`, `@env` searches pyenv first and finds the
    // env that already migrated.
    //
    // Names compare case-insensitively: on a case-insensitive filesystem
    // (the macOS and Windows defaults) `Web` and `web` are the same target
    // directory, so they race exactly like two `web`s. On a case-sensitive
    // one this costs a second batch run for one of them, nothing worse.
    let mut claimed: HashMap<String, SourceType> = HashMap::new();

    for env in environments {
        let eligible = match env.status {
            EnvironmentStatus::Ready => true,
            EnvironmentStatus::PythonEol { .. } => allow_eol,
            EnvironmentStatus::NameConflict { .. } => force,
            EnvironmentStatus::Corrupted { .. } => false,
        };

        if eligible {
            let key = env.name.to_ascii_lowercase();
            if let Some(winner) = claimed.get(&key) {
                let target = venvs_dir.join(&env.name);
                conflicts.push(MigrationConflictDetail {
                    name: env.name.clone(),
                    source_type: env.source_type,
                    existing: target,
                });
                skipped.push(MigrateSkipped {
                    name: env.name.clone(),
                    reason: format!(
                        "name already claimed by {winner} in this batch (migrate it with `scuv migrate @env {} --source {} --rename <new-name>`)",
                        env.name, env.source_type
                    ),
                });
                continue;
            }
            claimed.insert(key, env.source_type);
            migratable.push(env);
            continue;
        }

        match &env.status {
            EnvironmentStatus::Corrupted { reason } => skipped.push(MigrateSkipped {
                name: env.name.clone(),
                reason: format!("corrupted: {}", reason),
            }),
            EnvironmentStatus::PythonEol { version } => skipped.push(MigrateSkipped {
                name: env.name.clone(),
                reason: format!("Python {} is EOL (use --allow-eol)", version),
            }),
            EnvironmentStatus::NameConflict { existing } => {
                conflicts.push(MigrationConflictDetail {
                    name: env.name.clone(),
                    source_type: env.source_type,
                    existing: existing.clone(),
                });
                skipped.push(MigrateSkipped {
                    name: env.name.clone(),
                    reason: "name conflict (use --force)".to_string(),
                });
            }
            EnvironmentStatus::Ready => {} // unreachable given the matched arms above
        }
    }

    PartitionedEnvs {
        migratable,
        conflicts,
        skipped,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn create_test_env(name: &str, status: EnvironmentStatus) -> SourceEnvironment {
        SourceEnvironment {
            name: name.to_string(),
            python_version: "3.12.0".to_string(),
            path: PathBuf::from(format!("/test/{}", name)),
            source_type: SourceType::Pyenv,
            size_bytes: None,
            status,
        }
    }

    // =========================================================================
    // partition_envs Tests (replaces filter_migratable + collect_skipped)
    // =========================================================================

    /// Build a Ready env attributed to a specific source tool, so a test can
    /// stage the same name arriving from two different tools.
    fn create_test_env_from(name: &str, source_type: SourceType) -> SourceEnvironment {
        SourceEnvironment {
            name: name.to_string(),
            python_version: "3.12.0".to_string(),
            path: PathBuf::from(format!("/test/{source_type}/{name}")),
            source_type,
            size_bytes: None,
            status: EnvironmentStatus::Ready,
        }
    }

    /// Two sources offering the same name must not both be migratable.
    ///
    /// `check_name_conflict` only sees environments already on disk, so both
    /// arrive as `Ready`; without the batch-level guard they would race
    /// `par_iter` to the same target path.
    #[test]
    fn partition_rejects_duplicate_name_within_batch() {
        let envs = vec![
            create_test_env_from("web", SourceType::Pyenv),
            create_test_env_from("web", SourceType::Conda),
        ];

        let p = partition_envs(&envs, false, false, Path::new("/venvs"));

        assert_eq!(p.migratable.len(), 1, "only the first claim may migrate");
        assert_eq!(p.migratable[0].source_type, SourceType::Pyenv);
        assert_eq!(p.conflicts.len(), 1);
        assert_eq!(p.conflicts[0].name, "web");
        assert_eq!(p.conflicts[0].source_type, SourceType::Conda);
        assert_eq!(p.skipped.len(), 1);
        assert!(
            p.skipped[0].reason.contains("already claimed"),
            "reason should say the name was taken in this batch, got: {}",
            p.skipped[0].reason
        );
        // `@env web` alone would pick the pyenv env that already migrated.
        assert!(
            p.skipped[0]
                .reason
                .contains("@env web --source conda --rename"),
            "reason should name the skipped env's source, got: {}",
            p.skipped[0].reason
        );
    }

    /// `--force` means "overwrite what is already installed". Inside one batch
    /// there is nothing installed yet to overwrite, and letting both through
    /// is precisely the destructive race the guard exists to prevent — so
    /// force must not bypass it.
    #[test]
    fn partition_duplicate_name_not_bypassed_by_force() {
        let envs = vec![
            create_test_env_from("web", SourceType::Pyenv),
            create_test_env_from("web", SourceType::VirtualenvWrapper),
        ];

        let p = partition_envs(&envs, true, true, Path::new("/venvs"));

        assert_eq!(p.migratable.len(), 1);
        assert_eq!(p.conflicts.len(), 1);
    }

    /// `Web` and `web` are one directory on a case-insensitive filesystem,
    /// so they must not both migrate in one batch. Fails if the claim key
    /// goes back to the exact name.
    #[test]
    fn partition_rejects_names_differing_only_in_case() {
        let envs = vec![
            create_test_env_from("Web", SourceType::Pyenv),
            create_test_env_from("web", SourceType::Conda),
        ];

        let p = partition_envs(&envs, false, false, Path::new("/venvs"));

        assert_eq!(p.migratable.len(), 1);
        assert_eq!(p.migratable[0].name, "Web");
        assert_eq!(p.conflicts.len(), 1);
        assert_eq!(p.conflicts[0].name, "web");
    }

    /// Distinct names from distinct sources stay unaffected.
    #[test]
    fn partition_same_source_distinct_names_all_migratable() {
        let envs = vec![
            create_test_env_from("web", SourceType::Pyenv),
            create_test_env_from("api", SourceType::Conda),
        ];

        let p = partition_envs(&envs, false, false, Path::new("/venvs"));

        assert_eq!(p.migratable.len(), 2);
        assert!(p.conflicts.is_empty());
        assert!(p.skipped.is_empty());
    }

    #[test]
    fn partition_ready_always_migratable() {
        let envs = vec![
            create_test_env("ready1", EnvironmentStatus::Ready),
            create_test_env("ready2", EnvironmentStatus::Ready),
        ];

        let p = partition_envs(&envs, false, false, Path::new("/venvs"));
        assert_eq!(p.migratable.len(), 2);
        assert!(p.conflicts.is_empty());
        assert!(p.skipped.is_empty());
    }

    #[test]
    fn partition_corrupted_never_migratable() {
        let envs = vec![create_test_env(
            "corrupted",
            EnvironmentStatus::Corrupted {
                reason: "No python".to_string(),
            },
        )];

        let p = partition_envs(&envs, false, false, Path::new("/venvs"));
        assert!(p.migratable.is_empty());
        assert_eq!(p.skipped.len(), 1);
        assert!(p.skipped[0].reason.contains("corrupted"));

        let p_force = partition_envs(&envs, true, true, Path::new("/venvs"));
        assert!(p_force.migratable.is_empty());
    }

    #[test]
    fn partition_eol_excluded_without_force() {
        let envs = vec![create_test_env(
            "eol",
            EnvironmentStatus::PythonEol {
                version: "2.7.18".to_string(),
            },
        )];

        let p = partition_envs(&envs, false, false, Path::new("/venvs"));
        assert!(p.migratable.is_empty());
        assert_eq!(p.skipped.len(), 1);
        assert!(p.skipped[0].reason.contains("2.7.18"));
        assert!(p.skipped[0].reason.contains("EOL"));
    }

    #[test]
    fn partition_eol_included_with_force() {
        let envs = vec![create_test_env(
            "eol",
            EnvironmentStatus::PythonEol {
                version: "2.7.18".to_string(),
            },
        )];

        let p = partition_envs(&envs, true, true, Path::new("/venvs"));
        assert_eq!(p.migratable.len(), 1);
        assert!(p.skipped.is_empty());
    }

    /// `--allow-eol` alone takes the EOL env and leaves the conflict alone.
    /// Fails if `--allow-eol` stops admitting EOL envs, or starts admitting
    /// name conflicts (which would overwrite without `--force`).
    #[test]
    fn partition_allow_eol_includes_eol_but_not_conflicts() {
        let envs = vec![
            create_test_env(
                "eol",
                EnvironmentStatus::PythonEol {
                    version: "3.10.14".to_string(),
                },
            ),
            create_test_env(
                "taken",
                EnvironmentStatus::NameConflict {
                    existing: PathBuf::from("/venvs/taken"),
                },
            ),
        ];

        let p = partition_envs(&envs, false, true, Path::new("/venvs"));
        let names: Vec<&str> = p.migratable.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, ["eol"]);
        assert_eq!(p.conflicts.len(), 1);
        assert_eq!(p.conflicts[0].name, "taken");
    }

    #[test]
    fn partition_conflict_excluded_without_force_appears_in_both_buckets() {
        // Codex MUST FIX #2 contract: name-conflict appears in BOTH
        // `conflicts` (new structured surface) AND `skipped` (legacy
        // shape for backward compatibility with existing JSON consumers).
        let envs = vec![create_test_env(
            "dup",
            EnvironmentStatus::NameConflict {
                existing: PathBuf::from("/existing"),
            },
        )];

        let p = partition_envs(&envs, false, false, Path::new("/venvs"));
        assert!(p.migratable.is_empty());
        assert_eq!(p.conflicts.len(), 1);
        assert_eq!(p.conflicts[0].name, "dup");
        assert_eq!(p.conflicts[0].existing, PathBuf::from("/existing"));
        assert_eq!(p.skipped.len(), 1);
        assert!(p.skipped[0].reason.contains("conflict"));
    }

    #[test]
    fn partition_conflict_included_with_force() {
        let envs = vec![create_test_env(
            "dup",
            EnvironmentStatus::NameConflict {
                existing: PathBuf::from("/existing"),
            },
        )];

        let p = partition_envs(&envs, true, true, Path::new("/venvs"));
        assert_eq!(p.migratable.len(), 1);
        assert!(p.conflicts.is_empty());
        assert!(p.skipped.is_empty());
    }

    #[test]
    fn partition_mixed_statuses() {
        let envs = vec![
            create_test_env("ready", EnvironmentStatus::Ready),
            create_test_env(
                "eol",
                EnvironmentStatus::PythonEol {
                    version: "2.7".to_string(),
                },
            ),
            create_test_env(
                "corrupted",
                EnvironmentStatus::Corrupted {
                    reason: "broken".to_string(),
                },
            ),
            create_test_env(
                "conflict",
                EnvironmentStatus::NameConflict {
                    existing: PathBuf::from("/x"),
                },
            ),
        ];

        let p = partition_envs(&envs, false, false, Path::new("/venvs"));
        assert_eq!(p.migratable.len(), 1);
        assert_eq!(p.migratable[0].name, "ready");
        assert_eq!(p.conflicts.len(), 1);
        // skipped has: eol + corrupted + conflict
        assert_eq!(p.skipped.len(), 3);

        let p_force = partition_envs(&envs, true, true, Path::new("/venvs"));
        // ready + eol + conflict (not corrupted)
        assert_eq!(p_force.migratable.len(), 3);
        assert!(p_force.conflicts.is_empty());
        assert_eq!(p_force.skipped.len(), 1); // only corrupted
    }
}
