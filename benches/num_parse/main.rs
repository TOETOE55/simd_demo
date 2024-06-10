use std::iter::repeat_with;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::{thread_rng, Rng};
use simd_demo::num_parse::{avx, normal};

pub fn num_parse_bench(c: &mut Criterion) {
    let mut rand = thread_rng();

    let mut group = c.benchmark_group("num parse");

    let xs: Vec<String> = repeat_with(|| rand.gen::<u64>())
        .map(|n| n.to_string())
        .take(1_000_000)
        .collect();

    group.bench_function("simd", |b| {
        b.iter(|| {
            for s in &xs {
                let num = avx::parse_u64(&s);
                black_box(num);
            }
        });
    });

    group.bench_function("normal", |b| {
        b.iter(|| {
            for s in &xs {
                let num = normal::parse_u64(&s);
                black_box(num);
            }
        });
    });

    group.bench_function("std", |b| {
        b.iter(|| {
            for s in &xs {
                let num = s.parse::<u64>();
                _ = black_box(num);
            }
        });
    });

    group.finish();

    let size: usize = xs.iter().map(String::len).sum();
    println!("{size}");
}
criterion_group!(benches, num_parse_bench);
criterion_main!(benches);
