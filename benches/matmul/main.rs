use criterion::{black_box, criterion_group, criterion_main, Criterion};
use simd_demo::{
    matrix::{avx, normal},
    Align32,
};

#[inline(never)]
pub fn matrixes<const N: usize>() -> [[[f64; 2]; 2]; N] {
    std::array::from_fn(|i| {
        let f = i as f64;
        [[f + 0.0, f + 1.0], [f + 2.0, f + 3.0]]
    })
}

pub fn avx_matmul(c: &mut Criterion) {
    let mats = Align32(matrixes::<10000>());

    c.bench_function("avx matmul", |b| {
        b.iter(|| {
            let xs = avx::Matrix2x2::from_align_slice(&mats);
            for tuple in xs.chunks(2) {
                if let [a, b] = tuple {
                    _ = black_box(*a) * black_box(*b);
                }
            }
        })
    });
}

pub fn normal_matmul(c: &mut Criterion) {
    let mats = matrixes::<10000>();

    c.bench_function("normal matmul", |b| {
        b.iter(|| {
            let xs = normal::Matrix2x2::from_slice(&mats);
            for tuple in xs.chunks(2) {
                if let [a, b] = tuple {
                    _ = black_box(*a) * black_box(*b);
                }
            }
        })
    });
}

criterion_group!(benches, avx_matmul, normal_matmul);
criterion_main!(benches);
