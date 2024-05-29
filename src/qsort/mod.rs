#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "avx512f",
))]
/// [A Novel Hybrid Quicksort Algorithm Vectorized using AVX-512 on Intel Skylake](http://arxiv.org/pdf/1704.08579)
pub mod avx;
pub mod normal;
