pub mod matrix;

#[repr(align(32))]
pub struct Align32<T: ?Sized>(pub T);
