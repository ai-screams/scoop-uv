//! The human-readable `gc` report.

use rust_i18n::t;

use std::path::Path;

use crate::output::Output;
use crate::paths::abbreviate_home;

use super::types::{EnvGcReason, GcData};

pub(super) fn render_human(output: &Output, data: &GcData, aggressive: bool) {
    if data.envs.is_empty() && data.pythons.is_empty() {
        output.success(&t!("gc.nothing_to_remove"));
        return;
    }

    if !data.envs.is_empty() {
        output.info(&t!("gc.envs_header", count = data.envs.len().to_string()));
        for env in &data.envs {
            let reason = match env.reason {
                EnvGcReason::OrphanMissingMetadata => t!("gc.reason_missing_metadata").to_string(),
                EnvGcReason::OrphanBrokenPython => t!("gc.reason_broken_python").to_string(),
                EnvGcReason::Stale => {
                    // Pull age_days out of the record (None means
                    // some future caller produced a Stale record
                    // without one — render "?" rather than panic).
                    let days = env.age_days.map(|n| n.to_string()).unwrap_or("?".into());
                    t!("gc.reason_stale", days = days).to_string()
                }
            };
            println!(
                "  - {} ({})  {}",
                env.name,
                reason,
                abbreviate_home(Path::new(&env.path))
            );
        }
    }

    if aggressive && !data.pythons.is_empty() {
        output.info(&t!(
            "gc.pythons_header",
            count = data.pythons.len().to_string()
        ));
        for py in &data.pythons {
            println!("  - Python {}", py.version);
        }
    }

    if data.dry_run {
        output.info(&t!("gc.dry_run_hint"));
    } else {
        output.success(&t!("gc.done"));
    }
}
