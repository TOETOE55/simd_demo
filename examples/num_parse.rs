#![feature(stdarch_x86_avx512, slice_swap_unchecked)]

use simd_demo::num_parse;

fn main() {
    let s = "123123123";
    let num = num_parse::avx::parse_u64(s);
    assert_eq!(num, Some(123123123));

    let s = "123123123a";
    let num = num_parse::avx::parse_u64(s);
    assert_eq!(num, None);

    let s = "18446744073709551616";
    let num = num_parse::avx::parse_u64(s);
    assert_eq!(num, None);

    let s = "18446744073709551615";
    let num = num_parse::avx::parse_u64(s);
    assert_eq!(num, Some(18446744073709551615));
}
