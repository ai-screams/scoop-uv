//! Parsing hot-paths: clap arg-parse, TOML manifest, uv `python list` JSON.
//!
//! Why these: each runs once on every relevant invocation and is on the
//! shortest critical path users notice. Stable inputs (fixed `const` blobs)
//! keep variance low so CI can gate a 130% regression threshold meaningfully.

use std::hint::black_box;

use clap::Parser;
use criterion::{Criterion, criterion_group, criterion_main};

use scoop_uv::cli::Cli;
use scoop_uv::core::ScoopManifest;

/// Representative `.scoop.toml` covering the schema fully: name + python +
/// default group + two named groups. Realistic for a small project.
const TOML_FIXTURE: &str = r#"
[environment]
name = "myproject"
python = "3.12"

[packages]
default = ["pytest", "black", "mypy"]
dev = ["ipython", "debugpy"]
docs = ["mkdocs", "mkdocs-material"]
"#;

/// 5-entry `uv python list --output-format=json` payload. Mirrors the
/// real shape so serde_json's parser sees the same field set.
const UV_PYTHON_LIST_JSON: &str = r#"[
    {"version": "3.12.7", "implementation": "cpython", "path": "/Users/me/.local/share/uv/python/cpython-3.12.7"},
    {"version": "3.12.0", "implementation": "cpython", "path": "/Users/me/.local/share/uv/python/cpython-3.12.0"},
    {"version": "3.11.10", "implementation": "cpython", "path": null},
    {"version": "3.10.15", "implementation": "cpython", "path": null},
    {"version": "3.9.20", "implementation": "cpython", "path": null}
]"#;

fn bench_clap_parse_create(c: &mut Criterion) {
    c.bench_function("clap_parse_create", |b| {
        b.iter(|| Cli::try_parse_from(black_box(["scoop", "create", "myenv", "3.12"])).ok())
    });
}

fn bench_clap_parse_migrate_all(c: &mut Criterion) {
    // Subcommand-with-flags is a realistic worst-case for clap derive.
    c.bench_function("clap_parse_migrate_all", |b| {
        b.iter(|| {
            Cli::try_parse_from(black_box([
                "scoop",
                "migrate",
                "all",
                "--dry-run",
                "--json",
                "--source",
                "pyenv",
            ]))
            .ok()
        })
    });
}

fn bench_toml_parse_manifest(c: &mut Criterion) {
    c.bench_function("toml_parse_scoop_manifest", |b| {
        b.iter(|| ScoopManifest::parse(black_box(TOML_FIXTURE)).expect("valid"))
    });
}

fn bench_json_parse_uv_python_list(c: &mut Criterion) {
    c.bench_function("json_parse_uv_python_list", |b| {
        b.iter(|| {
            serde_json::from_str::<Vec<serde_json::Value>>(black_box(UV_PYTHON_LIST_JSON))
                .expect("valid")
        })
    });
}

criterion_group!(
    benches,
    bench_clap_parse_create,
    bench_clap_parse_migrate_all,
    bench_toml_parse_manifest,
    bench_json_parse_uv_python_list,
);
criterion_main!(benches);
