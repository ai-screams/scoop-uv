//! Human-form rendering for `scuv diff`.
//!
//! Three sections, suppressible per [`DiffMode`]: Python / Packages /
//! Metadata. Output goes through [`Output`] so quiet / no-color flags
//! flow through; this module renders structure, not styling.

use rust_i18n::t;

use crate::core::packages::{Origin, origins_differ};
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

    if mode != DiffMode::PackagesOnly
        && let Some(meta) = &data.metadata
    {
        render_metadata_section(output, meta);
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
    for line in package_lines(pkgs) {
        output.info(&line);
    }
}

/// One line per package difference. A package with a recorded origin shows
/// it after the version; a changed package whose origin differs shows both
/// origins, so a same-version change is not an unexplained `1.0 → 1.0`.
fn package_lines(pkgs: &PackageDiff) -> Vec<String> {
    let mut lines = Vec::new();
    for removed in &pkgs.removed {
        lines.push(format!(
            "  - {}=={}{}",
            removed.display_name,
            removed.version,
            with_origin(removed.source.as_ref())
        ));
    }
    for added in &pkgs.added {
        lines.push(format!(
            "  + {}=={}{}",
            added.display_name,
            added.version,
            with_origin(added.source.as_ref())
        ));
    }
    for changed in &pkgs.changed {
        let (a, b) = if origins_differ(changed.source_a.as_ref(), changed.source_b.as_ref()) {
            (
                origin_or_none(changed.source_a.as_ref()),
                origin_or_none(changed.source_b.as_ref()),
            )
        } else {
            (String::new(), String::new())
        };
        lines.push(format!(
            "  ~ {}: {}{a} → {}{b}",
            changed.name, changed.version_a, changed.version_b
        ));
    }
    lines
}

/// ` (editable /p)` for a recorded origin, nothing otherwise.
fn with_origin(origin: Option<&Origin>) -> String {
    origin
        .map(|o| format!(" ({})", o.describe()))
        .unwrap_or_default()
}

/// ` (editable /p)`, or ` (no recorded origin)`.
fn origin_or_none(origin: Option<&Origin>) -> String {
    match origin {
        Some(o) => format!(" ({})", o.describe()),
        None => format!(" ({})", t!("diff.no_origin")),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::commands::diff::types::{PackageChanged, PackageEntry};

    fn editable(path: &str) -> Origin {
        Origin::Dir {
            url: format!("file://{path}"),
            editable: true,
            subdirectory: None,
        }
    }

    /// Origins are shown when recorded, and on a changed line only when
    /// they differ. What fails this: dropping either branch.
    #[test]
    #[serial_test::serial]
    fn package_lines_show_origins_where_they_matter() {
        let _locale = crate::test_utils::LocaleGuard::capture();
        rust_i18n::set_locale("en");
        let entry = |name: &str, source| PackageEntry {
            name: name.into(),
            version: "1".into(),
            display_name: name.into(),
            source,
        };
        let changed = |name: &str, a: &str, b: &str, sa, sb| PackageChanged {
            name: name.into(),
            version_a: a.into(),
            version_b: b.into(),
            source_a: sa,
            source_b: sb,
        };
        let pkgs = PackageDiff {
            removed: vec![entry("gone", None)],
            added: vec![entry("new", Some(editable("/w/new")))],
            changed: vec![
                changed("idna", "3.4", "3.4", Some(editable("/w/idna")), None),
                changed("six", "1.15", "1.16", None, None),
            ],
        };
        assert_eq!(
            package_lines(&pkgs),
            [
                "  - gone==1",
                "  + new==1 (editable /w/new)",
                "  ~ idna: 3.4 (editable /w/idna) → 3.4 (no recorded origin)",
                "  ~ six: 1.15 → 1.16",
            ]
        );
    }
}
