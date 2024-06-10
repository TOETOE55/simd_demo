use std::iter::repeat_with;

use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use rand::{thread_rng, Rng};
use simd_demo::qsort::{avx, normal};

pub fn qsort_bench(c: &mut Criterion) {
    let mut rand = thread_rng();

    let mut group = c.benchmark_group("qsort");

    for count in [10, 100, 1000, 10_000, 100_000, 1_000_000] {
        let xs: Vec<i32> = repeat_with(|| rand.gen::<i32>()).take(count).collect();

        group.bench_with_input(BenchmarkId::new("simd", count), &xs, |b, xs| {
            b.iter_batched(
                || xs.clone(),
                |mut xs| avx::qsort(&mut xs),
                BatchSize::SmallInput,
            );
        });

    
        group.bench_with_input(BenchmarkId::new("normal", count), &xs, |b, xs| {
            b.iter_batched(
                || xs.clone(),
                |mut xs| normal::qsort(&mut xs),
                BatchSize::SmallInput,
            );
        });

        group.bench_with_input(BenchmarkId::new("std", count), &xs, |b, xs| {
            b.iter_batched(
                || xs.clone(),
                |mut xs| xs.sort_unstable(),
                BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}
criterion_group!(benches, qsort_bench);
criterion_main!(benches);
