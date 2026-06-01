//! Env-name validation — runs on every `scoop` command that takes a name.
//! Regex is compiled once via `once_cell::Lazy`; we measure the match cost
//! only.

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};

use scoop_uv::validate::is_valid_env_name;

fn bench_is_valid_env_name(c: &mut Criterion) {
    let mut group = c.benchmark_group("is_valid_env_name");

    // A handful of realistic inputs covering accept / reject / version-like
    // / reserved-word paths. Inputs picked to exercise each early-return.
    let inputs: &[(&str, &str)] = &[
        ("typical", "myproject"),
        ("hyphenated", "data-pipeline-v2"),
        ("digit_start_reject", "123env"),
        ("version_like_reject", "3.12.0"),
        ("reserved_reject", "activate"),
        ("max_length", &"a".repeat(64).leak()[..]), // boundary input
    ];

    for (label, input) in inputs {
        group.bench_with_input(BenchmarkId::from_parameter(label), input, |b, &input| {
            b.iter(|| is_valid_env_name(black_box(input)))
        });
    }

    group.finish();
}

criterion_group!(benches, bench_is_valid_env_name);
criterion_main!(benches);
