use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use simd_demo::{
    matrix::{avx, normal},
    Align32,
};

pub fn matrixes_2x2(n: usize) -> Vec<Align32<[[f64; 2]; 2]>> {
    (0..n)
        .map(|i| {
            let f = i as f64;
            [[f + 0.0, f + 1.0], [f + 2.0, f + 3.0]]
        })
        .map(Align32)
        .collect()
}

pub fn matrixes_4x4(n: usize) -> Vec<Align32<[[f64; 4]; 4]>> {
    (0..n)
        .map(|i| {
            let f = i as f64;
            [
                [f + 0.0, f + 1.0, f + 2.0, f + 3.0],
                [f + 4.0, f + 5.0, f + 6.0, f + 7.0],
                [f + 8.0, f + 9.0, f + 10.0, f + 10.0],
                [f + 11.0, f + 12.0, f + 13.0, f + 14.0],
            ]
        })
        .map(Align32)
        .collect()
}

pub fn matmul2x2(c: &mut Criterion) {
    let matss = (1000..=10000).step_by(200).map(matrixes_2x2);
    let mut group = c.benchmark_group("matmul 2x2");

    for mats in matss {
        let mats = Align32::slice_align(&*mats);
        group.bench_with_input(BenchmarkId::new("avx", mats.len()), mats, |b, mats| {
            b.iter(|| {
                let xs = avx::Matrix2x2::from_align_slice(&mats);
                for tuple in xs.chunks(2) {
                    if let [a, b] = tuple {
                        let _c = *a * *b;
                        black_box(_c);
                    }
                }
            })
        });
        group.bench_with_input(BenchmarkId::new("normal", mats.len()), mats, |b, mats| {
            b.iter(|| {
                let xs = normal::Matrix2x2::from_slice(&mats);
                for tuple in xs.chunks(2) {
                    if let [a, b] = tuple {
                        let _c = *a * *b;
                        black_box(_c);
                    }
                }
            })
        });
    }

    group.finish();
}

fn matmul4x4(c: &mut Criterion) {
    let matss = (1000..=10000).step_by(200).map(matrixes_4x4);
    let mut group = c.benchmark_group("matmul 4x4");

    for mats in matss {
        let mats = Align32::slice_align(&*mats);
        group.bench_with_input(BenchmarkId::new("avx", mats.len()), mats, |b, mats| {
            b.iter(|| {
                let xs = avx::Matrix4x4::from_align_slice(&mats);
                for tuple in xs.chunks(2) {
                    if let [a, b] = tuple {
                        let _c = *a * *b;
                        black_box(_c);
                    }
                }
            })
        });
        group.bench_with_input(BenchmarkId::new("normal", mats.len()), mats, |b, mats| {
            b.iter(|| {
                let xs = normal::Matrix4x4::from_slice(&mats);
                for tuple in xs.chunks(2) {
                    if let [a, b] = tuple {
                        let _c = *a * *b;
                        black_box(_c);
                    }
                }
            })
        });
    }

    group.finish();
}

criterion_group!(benches, matmul2x2, matmul4x4);
criterion_main!(benches);
