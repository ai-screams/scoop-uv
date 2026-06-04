//! `scoop diff <env-a> <env-b>` — compare two virtualenvs.
//!
//! Compares three observable surfaces: Python version, installed
//! packages (via `uv pip list`), and metadata (`.scoop-metadata.json`).
//! Output as either a human table or a structured JSON envelope.
//! `--strict` opts the command into `exit 1` when at least one
//! difference is found (mirrors `verify --strict`).
//!
//! ## Layering
//!
//! - [`types`] — pure data shapes, no I/O.
//! - [`compute`] — pure diff algorithm + PEP 503 normalisation.
//! - [`enumerator`] — boundary trait for package enumeration;
//!   production uses `uv pip list`, tests use mocks.
//! - [`render`] — human-form output.
//! - this file — orchestration + JSON envelope.

mod compute;
mod enumerator;
mod render;
mod types;

#[cfg(test)]
mod tests;

pub use types::{DiffMode, DiffOpts};

use rust_i18n::t;
use serde::Serialize;

use crate::core::{Metadata, VirtualenvService};
use crate::error::{Result, ScoopError};
use crate::output::Output;
use crate::uv::UvClient;

use compute::compute_package_diff;
use enumerator::{PackageEnumerator, UvPipEnumerator};
use types::{DiffData, DiffSummary, MetadataDiff, PackageDiff, ScalarDiff};

/// Orchestrate the `scoop diff` command.
///
/// Build a [`UvClient`] + production [`UvPipEnumerator`], delegate
/// to [`execute_with`] for the rest. The split keeps the trait
/// boundary visible at the top level.
pub fn execute(output: &Output, opts: &DiffOpts) -> Result<()> {
    let uv = UvClient::new()?;
    let enumerator = UvPipEnumerator { uv: &uv };
    let service = VirtualenvService::auto()?;
    execute_with(output, opts, &enumerator, &service)
}

/// Orchestration body — separated from [`execute`] so tests can
/// inject a mock [`PackageEnumerator`] without needing a real `uv`.
pub(crate) fn execute_with<E: PackageEnumerator>(
    output: &Output,
    opts: &DiffOpts,
    enumerator: &E,
    service: &VirtualenvService,
) -> Result<()> {
    // Validate both names + obtain paths in a single pre-flight pass
    // so neither side has done partial work when the other is missing.
    let (path_a, path_b) = resolve_env_paths(service, &opts.env_a, &opts.env_b)?;

    // Metadata: use the Result-returning variant so corrupt JSON
    // surfaces as an explicit failure rather than collapsing into
    // "no metadata" (which would lie to the user).
    let meta_a = service.read_metadata_result(&path_a)?;
    let meta_b = service.read_metadata_result(&path_b)?;

    let python = build_python_scalar(meta_a.as_ref(), meta_b.as_ref());

    let packages = if opts.mode != DiffMode::MetadataOnly {
        let a = enumerator.list(&path_a)?;
        let b = enumerator.list(&path_b)?;
        Some(compute_package_diff(&a, &b))
    } else {
        None
    };

    let metadata = if opts.mode != DiffMode::PackagesOnly {
        Some(build_metadata_diff(meta_a.as_ref(), meta_b.as_ref()))
    } else {
        None
    };

    let summary = build_summary(&python, packages.as_ref(), metadata.as_ref());
    let identical = summary.differences == 0;

    let data = DiffData {
        env_a: opts.env_a.clone(),
        env_b: opts.env_b.clone(),
        identical,
        python,
        packages,
        metadata,
        summary,
    };

    // Render BEFORE the strict-mode Err so the Quiet contract on
    // DiffMismatch holds — main.rs trusts the command emitted
    // everything it had to say.
    if output.is_json() {
        emit_json_outcome(&data, opts.strict);
    } else {
        render::render_human(output, &data, opts.mode);
        if opts.strict && !identical {
            output.info(&t!("diff.hint_strict"));
        }
    }

    if opts.strict && !identical {
        return Err(ScoopError::DiffMismatch {
            env_a: opts.env_a.clone(),
            env_b: opts.env_b.clone(),
            differences: data.summary.differences,
        });
    }

    Ok(())
}

/// Look up both env paths. Returns the first missing name as
/// [`ScoopError::VirtualenvNotFound`].
///
/// Existence check goes through [`VirtualenvService::exists`] (which
/// honours the same name-validation rules as every other command);
/// the path itself comes from [`crate::paths::virtualenv_path`] so
/// callers don't need a service handle just to know where an env
/// lives on disk.
fn resolve_env_paths(
    service: &VirtualenvService,
    env_a: &str,
    env_b: &str,
) -> Result<(std::path::PathBuf, std::path::PathBuf)> {
    if !service.exists(env_a)? {
        return Err(ScoopError::VirtualenvNotFound {
            name: env_a.to_string(),
        });
    }
    if !service.exists(env_b)? {
        return Err(ScoopError::VirtualenvNotFound {
            name: env_b.to_string(),
        });
    }
    let path_a = crate::paths::virtualenv_path(env_a)?;
    let path_b = crate::paths::virtualenv_path(env_b)?;
    Ok((path_a, path_b))
}

fn build_python_scalar(a: Option<&Metadata>, b: Option<&Metadata>) -> ScalarDiff<String> {
    ScalarDiff::from_sides(
        a.map(|m| m.python_version.clone()),
        b.map(|m| m.python_version.clone()),
    )
}

fn build_metadata_diff(a: Option<&Metadata>, b: Option<&Metadata>) -> MetadataDiff {
    MetadataDiff {
        python_version: ScalarDiff::from_sides(
            a.map(|m| m.python_version.clone()),
            b.map(|m| m.python_version.clone()),
        ),
        created_at: ScalarDiff::from_sides(
            a.map(|m| m.created_at.to_rfc3339()),
            b.map(|m| m.created_at.to_rfc3339()),
        ),
        last_used: ScalarDiff::from_sides(
            a.and_then(|m| m.last_used.map(|t| t.to_rfc3339())),
            b.and_then(|m| m.last_used.map(|t| t.to_rfc3339())),
        ),
        uv_version: ScalarDiff::from_sides(
            a.and_then(|m| m.uv_version.clone()),
            b.and_then(|m| m.uv_version.clone()),
        ),
    }
}

fn build_summary(
    python: &ScalarDiff<String>,
    packages: Option<&PackageDiff>,
    metadata: Option<&MetadataDiff>,
) -> DiffSummary {
    let packages_added = packages.map(|p| p.added.len()).unwrap_or(0);
    let packages_removed = packages.map(|p| p.removed.len()).unwrap_or(0);
    let packages_changed = packages.map(|p| p.changed.len()).unwrap_or(0);
    let metadata_fields_changed = metadata
        .map(|m| {
            [
                m.python_version.changed,
                m.created_at.changed,
                m.last_used.changed,
                m.uv_version.changed,
            ]
            .iter()
            .filter(|c| **c)
            .count()
        })
        .unwrap_or(0);

    // python.changed counts only when the python section is in-scope.
    // packages mode excludes metadata (which carries python_version
    // duplicated); to avoid double counting in MetadataOnly mode, the
    // python ScalarDiff is the source of truth for python_changed,
    // and metadata's python_version contributes only via the metadata
    // section's own field counter.
    let python_changed = python.changed;

    let differences = (python_changed as usize)
        + packages_added
        + packages_removed
        + packages_changed
        + metadata_fields_changed;

    DiffSummary {
        differences,
        python_changed,
        packages_added,
        packages_removed,
        packages_changed,
        metadata_fields_changed,
    }
}

/// Emit the JSON envelope for the diff outcome.
///
/// Success shape: `{status: "success", command: "diff", data}`.
/// Strict failure shape: `{status: "error", command: "diff", error:
/// {code, message, env_a, env_b, differences}, data}`. Both shapes
/// carry the full `DiffData` so consumers don't lose detail on the
/// failure side.
fn emit_json_outcome(data: &DiffData, strict: bool) {
    #[derive(Serialize)]
    struct SuccessEnvelope<'a> {
        status: &'a str,
        command: &'a str,
        data: &'a DiffData,
    }
    #[derive(Serialize)]
    struct FailureEnvelope<'a> {
        status: &'a str,
        command: &'a str,
        error: ErrorBody<'a>,
        data: &'a DiffData,
    }
    #[derive(Serialize)]
    struct ErrorBody<'a> {
        code: &'static str,
        message: String,
        env_a: &'a str,
        env_b: &'a str,
        differences: usize,
    }

    if strict && !data.identical {
        let envelope = FailureEnvelope {
            status: "error",
            command: "diff",
            error: ErrorBody {
                code: "DIFF_MISMATCH",
                message: t!(
                    "error.diff_mismatch",
                    a = data.env_a,
                    b = data.env_b,
                    n = data.summary.differences.to_string()
                )
                .to_string(),
                env_a: &data.env_a,
                env_b: &data.env_b,
                differences: data.summary.differences,
            },
            data,
        };
        emit_envelope(&envelope);
    } else {
        let envelope = SuccessEnvelope {
            status: "success",
            command: "diff",
            data,
        };
        emit_envelope(&envelope);
    }
}

/// Serialise an envelope to stdout, falling back to a minimal
/// hand-rolled error envelope on serde failure.
///
/// Same panic-safety pattern as
/// `cli::commands::migrate::batch::emit_envelope_or_fallback`: the
/// payload may embed `PathBuf` via metadata fields and serde's default
/// PathBuf adapter can fail on non-UTF-8 paths. An unwrap in the
/// error-reporting path would be a real panic risk.
fn emit_envelope<T: Serialize>(envelope: &T) {
    match serde_json::to_string(envelope) {
        Ok(json) => println!("{json}"),
        Err(err) => {
            println!(
                "{{\"status\":\"error\",\"command\":\"diff\",\"error\":{{\"code\":\"INTERNAL_JSON_ERROR\",\"message\":\"failed to serialise diff envelope: {}\"}}}}",
                err.to_string().replace('"', "\\\"")
            );
        }
    }
}
