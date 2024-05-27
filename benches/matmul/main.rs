use criterion::{black_box, criterion_group, criterion_main, Criterion};
use simd_demo::{
    matrix::{avx, normal},
    Align32,
};

pub fn matrixes_2x2<const N: usize>() -> [[[f64; 2]; 2]; N] {
    std::array::from_fn(|i| {
        let f = i as f64;
        [[f + 0.0, f + 1.0], [f + 2.0, f + 3.0]]
    })
}

pub fn matrixes_4x4<const N: usize>() -> [[[f64; 4]; 4]; N] {
    std::array::from_fn(|i| {
        let f = i as f64;
        [
            [f + 0.0, f + 1.0, f + 2.0, f + 3.0],
            [f + 4.0, f + 5.0, f + 6.0, f + 7.0],
            [f + 8.0, f + 9.0, f + 10.0, f + 10.0],
            [f + 11.0, f + 12.0, f + 13.0, f + 14.0],
        ]
    })
}

pub fn avx_matmul(c: &mut Criterion) {
    {
        let mats = Align32(matrixes_2x2::<10000>());

        c.bench_function("avx matmul 2x2", |b| {
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

    {
        let mats = Box::new(Align32(matrixes_4x4::<5000>()));

        c.bench_function("avx matmul 4x4", |b| {
            b.iter(|| {
                let xs = avx::Matrix4x4::from_align_slice(&*mats);
                for tuple in xs.chunks(2) {
                    if let [a, b] = tuple {
                        _ = black_box(*a) * black_box(*b);
                    }
                }
            })
        });
    }
}

pub fn normal_matmul(c: &mut Criterion) {
    {
        let mats = matrixes_2x2::<10000>();

        c.bench_function("normal matmul 2x2", |b| {
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

    {
        let mats = Box::new(matrixes_4x4::<5000>());

        c.bench_function("normal matmul 4x4", |b| {
            b.iter(|| {
                let xs = normal::Matrix4x4::from_slice(&*mats);
                for tuple in xs.chunks(2) {
                    if let [a, b] = tuple {
                        _ = black_box(*a) * black_box(*b);
                    }
                }
            })
        });
    }
}

criterion_group!(benches, avx_matmul, normal_matmul);
criterion_main!(benches);
