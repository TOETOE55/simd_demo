use std::{
    fmt::Debug,
    ops::{Add, AddAssign, Deref, DerefMut, Mul, MulAssign, Sub, SubAssign},
    ptr::{addr_of, addr_of_mut},
};

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::{
    __m256d, _mm256_add_pd, _mm256_broadcast_sd, _mm256_cmp_pd, _mm256_fmadd_pd, _mm256_loadu_pd,
    _mm256_movemask_pd, _mm256_mul_pd, _mm256_permute4x64_pd, _mm256_storeu_pd, _mm256_sub_pd,
    _CMP_EQ_UQ,
};

#[cfg(target_arch = "x86")]
use std::arch::x86::{
    __m256d, _mm256_add_pd, _mm256_broadcast_sd, _mm256_cmp_pd, _mm256_fmadd_pd, _mm256_loadu_pd,
    _mm256_movemask_pd, _mm256_mul_pd, _mm256_permute4x64_pd, _mm256_storeu_pd, _mm256_sub_pd,
    _CMP_EQ_UQ,
};

use crate::Align32;

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Matrix2x2(__m256d);

impl Matrix2x2 {
    #[inline(always)]
    pub fn new(matrix: [[f64; 2]; 2]) -> Self {
        Self::from(matrix)
    }

    #[inline(always)]
    pub fn from_align(matrix: &Align32<[[f64; 2]; 2]>) -> &Self {
        unsafe { &*(matrix as *const Align32<[[f64; 2]; 2]> as *const Self) }
    }

    #[inline(always)]
    pub fn from_align_slice(matrix: &Align32<[[[f64; 2]; 2]]>) -> &[Self] {
        unsafe { &*(matrix as *const Align32<[[[f64; 2]; 2]]> as *const [Self]) }
    }

    #[inline(always)]
    pub fn from_align_mut(matrix: &mut Align32<[[f64; 2]; 2]>) -> &mut Self {
        unsafe { &mut *(matrix as *mut Align32<[[f64; 2]; 2]> as *mut Self) }
    }

    #[inline(always)]
    pub fn from_align_slice_mut(matrix: &mut Align32<[[[f64; 2]; 2]]>) -> &mut [Self] {
        unsafe { &mut *(matrix as *mut Align32<[[[f64; 2]; 2]]> as *mut [Self]) }
    }

    #[inline(always)]
    pub fn scale(s: f64) -> Self {
        Self::new([[s, 0.0], [0.0, s]])
    }

    #[inline(always)]
    pub fn unit() -> Self {
        Self::scale(1.0)
    }

    #[inline(always)]
    pub fn zero() -> Self {
        Self::scale(0.0)
    }
}

impl Add for Matrix2x2 {
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: Self) -> Self::Output {
        unsafe { Self(_mm256_add_pd(self.0, rhs.0)) }
    }
}

impl AddAssign for Matrix2x2 {
    #[inline(always)]
    fn add_assign(&mut self, rhs: Self) {
        unsafe {
            self.0 = _mm256_add_pd(self.0, rhs.0);
        }
    }
}

impl AddAssign<&Matrix2x2> for Matrix2x2 {
    #[inline(always)]
    fn add_assign(&mut self, rhs: &Self) {
        unsafe {
            self.0 = _mm256_add_pd(self.0, rhs.0);
        }
    }
}

impl Sub for Matrix2x2 {
    type Output = Self;

    #[inline(always)]
    fn sub(self, rhs: Self) -> Self::Output {
        unsafe { Self(_mm256_sub_pd(self.0, rhs.0)) }
    }
}

impl SubAssign for Matrix2x2 {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Self) {
        unsafe {
            self.0 = _mm256_sub_pd(self.0, rhs.0);
        }
    }
}

impl SubAssign<&Matrix2x2> for Matrix2x2 {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: &Self) {
        unsafe {
            self.0 = _mm256_sub_pd(self.0, rhs.0);
        }
    }
}

impl Mul for Matrix2x2 {
    type Output = Self;

    #[inline(always)]
    fn mul(self, rhs: Self) -> Self::Output {
        unsafe {
            // [[a, b], [c, d]] -> [[a, a], [c, c]]
            let a_row1_dup = _mm256_permute4x64_pd::<0xA0>(self.0);
            // [[a, b], [c, d]] -> [[b, b], [d, d]]
            let a_row2_dup = _mm256_permute4x64_pd::<0xF5>(self.0);

            // [[x, y], [z, t]] -> [[x, y], [x, y]]
            let b_col1_dup = _mm256_permute4x64_pd::<0x44>(rhs.0);
            // [[x, y], [z, t]] -> [[z, t], [z, t]]
            let b_col2_dup = _mm256_permute4x64_pd::<0xEE>(rhs.0);

            let mut res = _mm256_mul_pd(a_row2_dup, b_col2_dup);

            res = _mm256_fmadd_pd(a_row1_dup, b_col1_dup, res);

            Self(res)
        }
    }
}

impl MulAssign for Matrix2x2 {
    #[inline(always)]
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl MulAssign<&Matrix2x2> for Matrix2x2 {
    #[inline(always)]
    fn mul_assign(&mut self, rhs: &Self) {
        *self = *self * *rhs;
    }
}

impl PartialEq for Matrix2x2 {
    fn eq(&self, other: &Self) -> bool {
        unsafe {
            let cmp = _mm256_cmp_pd::<_CMP_EQ_UQ>(self.0, other.0);
            _mm256_movemask_pd(cmp) == 0
        }
    }
}

impl From<[[f64; 2]; 2]> for Matrix2x2 {
    #[inline(always)]
    fn from(value: [[f64; 2]; 2]) -> Self {
        unsafe { Self(_mm256_loadu_pd(addr_of!(value).cast())) }
    }
}

impl From<Matrix2x2> for [[f64; 2]; 2] {
    #[inline(always)]
    fn from(value: Matrix2x2) -> Self {
        let mut res = [[0f64; 2]; 2];
        unsafe {
            _mm256_storeu_pd(addr_of_mut!(res).cast(), value.0);
        }
        res
    }
}

impl Debug for Matrix2x2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Matrix2x2").field(&**self).finish()
    }
}

impl Deref for Matrix2x2 {
    type Target = [[f64; 2]; 2];

    fn deref(&self) -> &Self::Target {
        unsafe { &*(self as *const Self as *const _) }
    }
}

impl DerefMut for Matrix2x2 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *(self as *mut Self as *mut _) }
    }
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Matrix4x4([Vector4; 4]);

impl Matrix4x4 {
    #[inline(always)]
    pub fn new(matrix: Align32<[[f64; 4]; 4]>) -> Self {
        *Self::from_align(&matrix)
    }

    #[inline(always)]
    pub fn from_align(matrix: &Align32<[[f64; 4]; 4]>) -> &Self {
        unsafe { &*(matrix as *const Align32<[[f64; 4]; 4]> as *const Self) }
    }

    #[inline(always)]
    pub fn from_align_slice(matrix: &Align32<[[[f64; 4]; 4]]>) -> &[Self] {
        unsafe { &*(matrix as *const Align32<[[[f64; 4]; 4]]> as *const [Self]) }
    }

    #[inline(always)]
    pub fn from_align_mut(matrix: &mut Align32<[[f64; 4]; 4]>) -> &mut Self {
        unsafe { &mut *(matrix as *mut Align32<[[f64; 4]; 4]> as *mut Self) }
    }

    #[inline(always)]
    pub fn as_array(&self) -> &[[f64; 4]; 4] {
        unsafe { &*(self as *const Self as *const _) }
    }
}

impl Mul for Matrix4x4 {
    type Output = Self;

    #[inline(always)]
    fn mul(self, rhs: Self) -> Self::Output {
        let a = &self;
        let b = &rhs;

        unsafe {
            let mut c0 = _mm256_mul_pd(_mm256_broadcast_sd(&a[0][0]), b[0].0);
            let mut c1 = _mm256_mul_pd(_mm256_broadcast_sd(&a[1][0]), b[0].0);
            let mut c2 = _mm256_mul_pd(_mm256_broadcast_sd(&a[2][0]), b[0].0);
            let mut c3 = _mm256_mul_pd(_mm256_broadcast_sd(&a[3][0]), b[0].0);

            c0 = _mm256_fmadd_pd(_mm256_broadcast_sd(&a[0][1]), b[1].0, c0);
            c1 = _mm256_fmadd_pd(_mm256_broadcast_sd(&a[1][1]), b[1].0, c1);
            c2 = _mm256_fmadd_pd(_mm256_broadcast_sd(&a[2][1]), b[1].0, c2);
            c3 = _mm256_fmadd_pd(_mm256_broadcast_sd(&a[3][1]), b[1].0, c3);

            c0 = _mm256_fmadd_pd(_mm256_broadcast_sd(&a[0][2]), b[2].0, c0);
            c1 = _mm256_fmadd_pd(_mm256_broadcast_sd(&a[1][2]), b[2].0, c1);
            c2 = _mm256_fmadd_pd(_mm256_broadcast_sd(&a[2][2]), b[2].0, c2);
            c3 = _mm256_fmadd_pd(_mm256_broadcast_sd(&a[3][2]), b[2].0, c3);

            c0 = _mm256_fmadd_pd(_mm256_broadcast_sd(&a[0][3]), b[3].0, c0);
            c1 = _mm256_fmadd_pd(_mm256_broadcast_sd(&a[1][3]), b[3].0, c1);
            c2 = _mm256_fmadd_pd(_mm256_broadcast_sd(&a[2][3]), b[3].0, c2);
            c3 = _mm256_fmadd_pd(_mm256_broadcast_sd(&a[3][3]), b[3].0, c3);

            Matrix4x4([Vector4(c0), Vector4(c1), Vector4(c2), Vector4(c3)])
        }
    }
}

impl Deref for Matrix4x4 {
    type Target = [Vector4; 4];

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Matrix4x4 {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Vector4(__m256d);

impl Deref for Vector4 {
    type Target = [f64; 4];

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*(self as *const Self as *const _) }
    }
}

impl DerefMut for Vector4 {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *(self as *mut Self as *mut _) }
    }
}
