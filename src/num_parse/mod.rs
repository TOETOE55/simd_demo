#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "avx512f",
))]
/// [Parsing integers quickly with AVX-512](https://lemire.me/blog/2023/09/22/parsing-integers-quickly-with-AVX-512/)
pub mod avx;
pub mod normal;
