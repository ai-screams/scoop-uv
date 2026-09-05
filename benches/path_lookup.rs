//! `paths::find_executable_in` — shared by `scoop which` and `scoop run`.
//! Touches the filesystem (single stat per candidate), so variance is
//! higher than pure-CPU benches: 38 runs of the same code on GitHub
//! runners span 574-1931 ns, a 3.4x spread. `.github/workflows/bench.yml`
//! therefore tracks this group on gh-pages without gating on it — see the
//! "Run filesystem benchmarks" step.

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use tempfile::TempDir;

use scoop_uv::paths::find_executable_in;

fn bench_find_executable_hit(c: &mut Criterion) {
    // Layout: bin/ with python + pip + pytest + black; we ask for python.
    // The dir + files are created once per benchmark setup, not per iteration.
    let dir = TempDir::new().expect("tempdir");
    for name in ["python", "pip", "pytest", "black"] {
        std::fs::write(dir.path().join(name), b"").expect("write");
    }

    c.bench_function("find_executable_in_hit", |b| {
        b.iter(|| find_executable_in(black_box(dir.path()), black_box("python")))
    });
}

fn bench_find_executable_miss(c: &mut Criterion) {
    let dir = TempDir::new().expect("tempdir");
    for name in ["python", "pip"] {
        std::fs::write(dir.path().join(name), b"").expect("write");
    }

    c.bench_function("find_executable_in_miss", |b| {
        b.iter(|| find_executable_in(black_box(dir.path()), black_box("ghost")))
    });
}

criterion_group!(
    benches,
    bench_find_executable_hit,
    bench_find_executable_miss
);
criterion_main!(benches);
