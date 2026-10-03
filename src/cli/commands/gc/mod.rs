//! Handler for the `scuv gc` command.
//!
//! Detects orphan virtualenvs (directories under `~/.scuv/virtualenvs/`
//! that no longer look like usable environments) and, when run with
//! `--aggressive`, also flags uv-managed Python versions that are not
//! referenced by any surviving env's metadata.
//!
//! Default behaviour is **dry-run** — destructive removal happens only when
//! the caller passes `--yes`. This mirrors how most package managers' `gc`
//! commands behave (cargo, nix, dnf): preview by default, opt-in to delete.

mod remove;
mod render;
mod scan;
mod types;

use chrono::Utc;
use rust_i18n::t;

use crate::error::Result;
use crate::output::Output;

use super::duration::parse_duration;
use remove::remove_orphans;
use render::render_human;
use scan::{scan_orphan_envs, scan_stale_envs, scan_unused_pythons};
use types::{EnvOutcome, EnvRecord, GcData, PythonOutcome, PythonRecord};

/// Execute the `gc` command.
///
/// * `yes` — actually remove the candidates (otherwise dry-run only).
/// * `aggressive` — also consider Python versions that no env uses.
/// * `older_than` — when `Some`, also flag envs whose `last_used` is
///   older than the parsed duration. `None` preserves the pre-flag
///   behaviour (orphans only).
pub fn execute(
    output: &Output,
    yes: bool,
    aggressive: bool,
    older_than: Option<&str>,
) -> Result<()> {
    // Parse the duration up front so a malformed `--older-than` fails
    // before we touch the filesystem. Cutoff is sampled once and
    // shared between scan + recheck — using a fresh `Utc::now()` at
    // recheck time would make borderline envs jitter in/out of "stale".
    let stale_cutoff = match older_than {
        Some(s) => {
            let d = parse_duration(s)?;
            Some(Utc::now().checked_sub_signed(d).ok_or_else(|| {
                crate::error::ScoopError::InvalidArgument {
                    message: format!("cutoff arithmetic overflowed for --older-than {s}"),
                }
            })?)
        }
        None => None,
    };

    let mut envs = scan_orphan_envs()?;
    if let Some(cutoff) = stale_cutoff {
        envs.extend(scan_stale_envs(cutoff)?);
        envs.sort_by(|a, b| a.name.cmp(&b.name));
    }

    let (pythons, unreadable_envs) = if aggressive {
        scan_unused_pythons(&envs)?
    } else {
        (Vec::new(), 0)
    };

    // Surface the conservative bail-out before any destructive work so the
    // user understands why `--aggressive` turned up nothing.
    if aggressive && unreadable_envs > 0 {
        output.warn(&t!(
            "gc.unreadable_metadata_warn",
            count = unreadable_envs.to_string()
        ));
    }

    // Build records up-front. Dry-run leaves everything Pending; `--yes`
    // mutates outcomes in remove_orphans so the JSON envelope reflects
    // what actually happened, not just the original scan snapshot. (The
    // old JSON shape always claimed success — partial failures were only
    // visible in human warn output, which scripts can't see.)
    let mut env_records: Vec<EnvRecord> = envs
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
    let mut python_records: Vec<PythonRecord> = pythons
        .iter()
        .map(|p| PythonRecord {
            version: p.version.clone(),
            path: p.path.clone(),
            outcome: PythonOutcome::Pending,
            error: None,
        })
        .collect();

    if yes {
        remove_orphans(
            output,
            &envs,
            &pythons,
            &mut env_records,
            &mut python_records,
            stale_cutoff,
        );
    }

    let data = GcData {
        dry_run: !yes,
        envs: env_records,
        pythons: python_records,
    };

    if output.is_json() {
        output.json_success("gc", data);
        return Ok(());
    }

    render_human(output, &data, aggressive);
    Ok(())
}

#[cfg(test)]
mod tests;
