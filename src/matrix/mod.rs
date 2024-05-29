#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "avx2",
    target_feature = "fma",
))]
pub mod avx;

pub mod normal;
