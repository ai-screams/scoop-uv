//! Human-form rendering for `scuv diff`.
//!
//! Three sections, suppressible per [`DiffMode`]: Python / Packages /
//! Metadata. Output goes through [`Output`] so quiet / no-color flags
//! flow through; this module renders structure, not styling.

use rust_i18n::t;

use crate::output::Output;

use super::types::{DiffData, DiffMode, PackageDiff, ScalarDiff};

/// Render the diff in human-readable form to the user's terminal.
///
/// Identical envs short-circuit to a single summary line. Otherwise
/// three sections are printed (sub-selected by `mode`). The exit-code
/// / Quiet-render contract is enforced at the caller (`execute`); this
/// function only prints.
pub fn render_human(output: &Output, data: &DiffData, mode: DiffMode) {
    if data.identical {
        output.info(&t!("diff.identical_summary"));
        return;
    }

    // Header runs for every mode; the previous `matches!` enumerated
    // all three variants and was always-true. Plain unconditional emit.
    output.info(&format!("{} vs {}", data.env_a, data.env_b));

    if mode != DiffMode::MetadataOnly {
        render_python_section(output, data);
        if let Some(pkgs) = &data.packages {
            render_packages_section(output, pkgs);
        }
    }

    if mode != DiffMode::PackagesOnly {
        if let Some(meta) = &data.metadata {
            render_metadata_section(output, meta);
        }
    }

    if !data.summary.python_changed
        && data.summary.packages_added == 0
        && data.summary.packages_removed == 0
        && data.summary.packages_changed == 0
        && data.summary.metadata_fields_changed == 0
    {
        // Mode-suppressed sections produced no visible diffs; remind
        // the user with the same line the identical branch shows.
        output.info(&t!("diff.identical_summary"));
    }
}

fn render_python_section(output: &Output, data: &DiffData) {
    output.info("");
    output.info(&t!("diff.section_python"));
    render_scalar_row(output, "python", &data.python);
}

fn render_packages_section(output: &Output, pkgs: &PackageDiff) {
    let total = pkgs.added.len() + pkgs.removed.len() + pkgs.changed.len();
    output.info("");
    output.info(&t!("diff.section_packages", n = total));
    for removed in &pkgs.removed {
        output.info(&format!(
            "  - {}=={}",
            removed.display_name, removed.version
        ));
    }
    for added in &pkgs.added {
        output.info(&format!("  + {}=={}", added.display_name, added.version));
    }
    for changed in &pkgs.changed {
        output.info(&format!(
            "  ~ {}: {} → {}",
            changed.name, changed.version_a, changed.version_b
        ));
    }
}

fn render_metadata_section(output: &Output, meta: &super::types::MetadataDiff) {
    output.info("");
    output.info(&t!("diff.section_metadata"));
    render_scalar_row(output, "python_version", &meta.python_version);
    render_scalar_row(output, "created_at", &meta.created_at);
    render_scalar_row(output, "last_used", &meta.last_used);
    render_scalar_row(output, "uv_version", &meta.uv_version);
}

fn render_scalar_row(output: &Output, label: &str, scalar: &ScalarDiff<String>) {
    let marker = if scalar.changed { "~" } else { " " };
    let a = scalar.a.as_deref().unwrap_or("-");
    let b = scalar.b.as_deref().unwrap_or("-");
    output.info(&format!("  {marker} {label:<16} a: {a:<24}  b: {b}"));
}
