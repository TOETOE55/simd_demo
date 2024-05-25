use simd_demo::matrix::{avx, normal};

fn main() {
    let x1 = avx::Matrix2x2::new([[1.0, 2.0], [3.0, 4.0]]);
    let y1 = avx::Matrix2x2::new([[5.0, 6.0], [7.0, 8.0]]);
    let z1 = x1 * y1;

    let x2 = normal::Matrix2x2::new([[1.0, 2.0], [3.0, 4.0]]);
    let y2 = normal::Matrix2x2::new([[5.0, 6.0], [7.0, 8.0]]);
    let z2 = x2 * y2;

    assert_eq!(*z1, *z2);

    println!("{z1:?}");
}
