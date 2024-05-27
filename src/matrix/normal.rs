use std::ops::{Add, AddAssign, Deref, DerefMut, Mul, MulAssign, Sub, SubAssign};

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(transparent)]
pub struct Matrix2x2([[f64; 2]; 2]);

impl Matrix2x2 {
    #[inline(always)]
    pub fn new(matrix: [[f64; 2]; 2]) -> Self {
        Self(matrix)
    }

    #[inline(always)]
    pub fn from_ref(matrix: &[[f64; 2]; 2]) -> &Self {
        unsafe { &*(matrix as *const [[f64; 2]; 2] as *const Self) }
    }

    #[inline(always)]
    pub fn from_slice(matrix: &[[[f64; 2]; 2]]) -> &[Self] {
        unsafe { &*(matrix as *const [[[f64; 2]; 2]] as *const [Self]) }
    }

    #[inline(always)]
    pub fn from_ref_mut(matrix: &mut [[f64; 2]; 2]) -> &mut Self {
        unsafe { &mut *(matrix as *mut [[f64; 2]; 2] as *mut Self) }
    }

    #[inline(always)]
    pub fn from_slice_mut(matrix: &mut [[[f64; 2]; 2]]) -> &mut [Self] {
        unsafe { &mut *(matrix as *mut [[[f64; 2]; 2]] as *mut [Self]) }
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
        let a = &self.0;
        let b = &rhs.0;

        Self([
            [a[0][0] + b[0][0], a[0][1] + b[0][1]],
            [a[1][0] + b[1][0], a[1][1] + b[1][1]],
        ])
    }
}

impl AddAssign for Matrix2x2 {
    #[inline(always)]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}

impl AddAssign<&Matrix2x2> for Matrix2x2 {
    #[inline(always)]
    fn add_assign(&mut self, rhs: &Self) {
        *self += *rhs
    }
}

impl Sub for Matrix2x2 {
    type Output = Self;

    #[inline(always)]
    fn sub(self, rhs: Self) -> Self::Output {
        let a = &self.0;
        let b = &rhs.0;

        Self([
            [a[0][0] - b[0][0], a[0][1] - b[0][1]],
            [a[1][0] - b[1][0], a[1][1] - b[1][1]],
        ])
    }
}

impl SubAssign for Matrix2x2 {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl SubAssign<&Matrix2x2> for Matrix2x2 {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: &Self) {
        *self -= *rhs
    }
}

impl Mul for Matrix2x2 {
    type Output = Self;

    /// [Strassen algorithm](https://en.wikipedia.org/wiki/Strassen_algorithm)
    #[inline(always)]
    fn mul(self, rhs: Self) -> Self::Output {
        let a = &self.0;
        let b = &rhs.0;

        let m1 = (a[0][0] + a[1][1]) * (b[0][0] + b[1][1]);
        let m2 = (a[1][0] + a[1][1]) * b[0][0];
        let m3 = a[0][0] * (b[0][1] - b[1][1]);
        let m4 = a[1][1] * (b[1][0] - b[0][0]);
        let m5 = (a[0][0] + a[0][1]) * b[1][1];
        let m6 = (a[1][0] - a[0][0]) * (b[0][0] + b[0][1]);
        let m7 = (a[0][1] - a[1][1]) * (b[1][0] + b[1][1]);

        Self([[m1 + m4 - m5 + m7, m3 + m5], [m2 + m4, m1 - m2 + m3 + m6]])
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
        *self *= *rhs;
    }
}

impl From<[[f64; 2]; 2]> for Matrix2x2 {
    #[inline(always)]
    fn from(value: [[f64; 2]; 2]) -> Self {
        Self(value)
    }
}

impl From<Matrix2x2> for [[f64; 2]; 2] {
    #[inline(always)]
    fn from(value: Matrix2x2) -> Self {
        value.0
    }
}

impl Deref for Matrix2x2 {
    type Target = [[f64; 2]; 2];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Matrix2x2 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Clone, Copy, PartialEq)]
#[repr(transparent)]
pub struct Matrix4x4(pub [[f64; 4]; 4]);

impl Matrix4x4 {
    #[inline(always)]
    pub fn from_slice(matrix: &[[[f64; 4]; 4]]) -> &[Self] {
        unsafe { &*(matrix as *const [[[f64; 4]; 4]] as *const [Self]) }
    }
}

impl Mul for Matrix4x4 {
    type Output = Self;

    #[inline(always)]
    fn mul(self, rhs: Self) -> Self::Output {
        let l = &self.0;
        let r = &rhs.0;
        let mut d = [[0.0; 4]; 4];
        d[0][0] = l[0][0] * r[0][0] + l[0][1] * r[1][0] + l[0][2] * r[2][0] + l[0][3] * r[3][0];
        d[0][1] = l[0][0] * r[0][1] + l[0][1] * r[1][1] + l[0][2] * r[2][1] + l[0][3] * r[3][1];
        d[0][2] = l[0][0] * r[0][2] + l[0][1] * r[1][2] + l[0][2] * r[2][2] + l[0][3] * r[3][2];
        d[0][3] = l[0][0] * r[0][3] + l[0][1] * r[1][3] + l[0][2] * r[2][3] + l[0][3] * r[3][3];
        d[1][0] = l[1][0] * r[0][0] + l[1][1] * r[1][0] + l[1][2] * r[2][0] + l[1][3] * r[3][0];
        d[1][1] = l[1][0] * r[0][1] + l[1][1] * r[1][1] + l[1][2] * r[2][1] + l[1][3] * r[3][1];
        d[1][2] = l[1][0] * r[0][2] + l[1][1] * r[1][2] + l[1][2] * r[2][2] + l[1][3] * r[3][2];
        d[1][3] = l[1][0] * r[0][3] + l[1][1] * r[1][3] + l[1][2] * r[2][3] + l[1][3] * r[3][3];
        d[2][0] = l[2][0] * r[0][0] + l[2][1] * r[1][0] + l[2][2] * r[2][0] + l[2][3] * r[3][0];
        d[2][1] = l[2][0] * r[0][1] + l[2][1] * r[1][1] + l[2][2] * r[2][1] + l[2][3] * r[3][1];
        d[2][2] = l[2][0] * r[0][2] + l[2][1] * r[1][2] + l[2][2] * r[2][2] + l[2][3] * r[3][2];
        d[2][3] = l[2][0] * r[0][3] + l[2][1] * r[1][3] + l[2][2] * r[2][3] + l[2][3] * r[3][3];
        d[3][0] = l[3][0] * r[0][0] + l[3][1] * r[1][0] + l[3][2] * r[2][0] + l[3][3] * r[3][0];
        d[3][1] = l[3][0] * r[0][1] + l[3][1] * r[1][1] + l[3][2] * r[2][1] + l[3][3] * r[3][1];
        d[3][2] = l[3][0] * r[0][2] + l[3][1] * r[1][2] + l[3][2] * r[2][2] + l[3][3] * r[3][2];
        d[3][3] = l[3][0] * r[0][3] + l[3][1] * r[1][3] + l[3][2] * r[2][3] + l[3][3] * r[3][3];
        Self(d)
    }
}
