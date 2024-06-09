#![feature(stdarch_x86_avx512, slice_swap_unchecked)]

use std::ops::{Deref, DerefMut};

pub mod matrix;
pub mod num_parse;
pub mod qsort;

#[repr(align(32))]
pub struct Align32<T: ?Sized>(pub T);

impl<T> Align32<T> {
    pub fn slice_align(slice: &[Align32<T>]) -> &Align32<[T]> {
        unsafe { &*(slice as *const [Align32<T>] as *const _) }
    }

    pub fn slice_align_mut(slice: &mut [Align32<T>]) -> &mut Align32<[T]> {
        unsafe { &mut *(slice as *mut [Align32<T>] as *mut _) }
    }
}

impl<T: ?Sized> Deref for Align32<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: ?Sized> DerefMut for Align32<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
