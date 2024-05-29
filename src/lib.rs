#![feature(stdarch_x86_avx512, slice_swap_unchecked)]

pub mod matrix;
pub mod qsort;

#[repr(align(32))]
pub struct Align32<T: ?Sized>(pub T);
