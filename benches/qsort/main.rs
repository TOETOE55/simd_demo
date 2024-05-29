use std::iter::repeat_with;

use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use rand::{thread_rng, Rng};
use simd_demo::qsort::{avx, normal};

pub fn qsort_bench(c: &mut Criterion) {
    let mut rand = thread_rng();
    let xs: Vec<i32> = repeat_with(|| rand.gen::<i32>()).take(1_000_000).collect();

    c.bench_function("simd qsort", |b| {
        b.iter_batched(
            || xs.clone(),
            |mut xs| avx::qsort(&mut xs),
            BatchSize::SmallInput,
        );
    });

    c.bench_function("normal qsort", |b| {
        b.iter_batched(
            || xs.clone(),
            |mut xs| normal::qsort(&mut xs),
            BatchSize::SmallInput,
        );
    });

    c.bench_function("std qsort", |b| {
        b.iter_batched(
            || xs.clone(),
            |mut xs| xs.sort_unstable(),
            BatchSize::SmallInput,
        );
    });
}
criterion_group!(benches, qsort_bench);
criterion_main!(benches);
