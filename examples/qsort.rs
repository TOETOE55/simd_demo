#![feature(stdarch_x86_avx512, slice_swap_unchecked)]

use std::array;

use simd_demo::qsort;

fn main() {
    let mut arr: [_; 1000] =
        array::from_fn(|i| (i as f64 * f64::sin((i as f64) / 5.0)).ceil() as i32);
    qsort::avx::qsort(&mut arr);
    works(&arr);
}

fn works(arr: &[i32]) {
    for (a, b) in arr.iter().zip(arr.iter().skip(1)) {
        assert!(a <= b);
    }
}
