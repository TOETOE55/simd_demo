# parse number

第二个SIMD的发力点是decode/encode相关，比如base64/utf8，这方面有很多库。不过我觉得这里面最有意思的是simdjson，于是这里介绍一下，simdjson中的number parsing的部分，整数的parsing（simdjson的作者又在他的[博客](https://lemire.me/blog/2022/05/25/parsing-json-faster-with-intel-AVX-512/)里用AVX512优化了这部分工作）：

算法分成几部分：

1. 将字符串按字节读入到向量中，大于20bytes直接返回失败（`u64::MAX == 1844_67440737_09551615` 用字符串表示为20个字节）
2. 将向量中对应数字的有效位的分量减去`'0'`
3. 将向量中有效位的分量与9比较，如果存在>9的返回失败（说明字符串中存在非数字）
4. 通过向量计算尽可能累加结果，比如说`[1, 2, 3, 4, 5, 6, 7, 8, 9]`
   1. 每两个分量计算`a + 10*b`，得到`[1, 23, 45, 67, 89]`
   2. 每两个分量计算`a + 100*b`，得到`[1, 2345, 6789]`
   3. 每两个分量计算`a + 10000*b`，得到`[1, 23456789]`
5. 通过标量计算累加剩余结果，比如上面的`1*1_0000_0000 + 23456789`

前三步可以使用AVX512指令来优化：

```rust
let bytes = s.as_bytes();

// 超过20字节的字符串超过u64的范围
if bytes.len() > 20 {
    return None;
}

let start = bytes.as_ptr();
let end = unsafe { start.offset(bytes.len() as isize) };
let mask = 0xFFFFFFFF_u32 << (32 - s.len());

let base10_8bit = unsafe {
    let ascii_zero = _mm256_set1_epi8('0' as i8);
    let nine = _mm256_set1_epi8(9);

    // 1. 将字符串读入到向量的高位中，向量低位置为0
    //    *注意*：
    //    - `__m256i` 可以表示32个 `u8`
    //    - 字符串的高低位与数字的高低位是相反的
    let s_bytes_v = _mm256_maskz_loadu_epi8(mask, end.offset(-32).cast());

    // 2. 将向量读到的字符串部分，每个字节 - '0'
    let base10_8bit = _mm256_maskz_sub_epi8(mask, s_bytes_v, ascii_zero);

    // 3. 如果存在字节 > 9的，说明存在非数字的字节
    //    *注意*: 这里按u8解释i8，所以如果是负数，也会 > 9
    let nondigits = _mm256_mask_cmpgt_epu8_mask(mask, base10_8bit, nine);
    if nondigits == 0 {
        return None;
    }

    base10_8bit
};
```



然后使用向量运算进行累加，这里要用到`madd`类操作——纵向乘横向加（不过支持的位数不多）：

```rust
// 使用向量计算10进制求和
// 最后得到8位(digits)整数向量(4x32bits)
//
let base10e8_32bit = unsafe {
    let digit_value_base10_8bit = _mm256_set_epi8(
        1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1,
        10, 1, 10, 1, 10, 1, 10,
    );
    let digit_value_base10e2_8bit = _mm_set_epi8(
        1, 100, 1, 100, 1, 100, 1, 100, 1, 100, 1, 100, 1, 100, 1, 100,
    );
    let digit_value_base10e4_16bit = _mm_set_epi16(1, 10000, 1, 10000, 1, 10000, 1, 10000);

    // example:
    // s = "1234"
    // base10_8bit             = [0.., 1, 2, 3, 4]
    // digit_value_base10_8bit = [.., 10, 1, 10, 1]
    // maddubs_epi16           = [0.., 1*10 + 2*1, 3*10 + 4*1]
    //                         = [0.., 12, 34]
    let base10e2_16bit = _mm256_maddubs_epi16(base10_8bit, digit_value_base10_8bit);
    let base10e2_8bit = _mm256_cvtepi16_epi8(base10e2_16bit);

    // = [0.., 1234]
    let base10e4_16bit = _mm_maddubs_epi16(base10e2_8bit, digit_value_base10e2_8bit);

    let base10e8_32bit = _mm_madd_epi16(base10e4_16bit, digit_value_base10e4_16bit);
    base10e8_32bit
};
```



最后剩余的累加通过标量加法完成：

```rust
// 使用标量计算剩余数的十进制求和
unsafe {
    let res_1digit = _mm_extract_epi32(base10e8_32bit, 3) as u64;
    if mask & 0xFFFFFFFF == 0 {
        return Some(res_1digit);
    }

    let middle_part = _mm_extract_epi32(base10e8_32bit, 2) as u64;
    let res_2digit = res_1digit + 1_0000_0000 * middle_part;
    if mask & 0xFFFF == 0 {
        return Some(res_2digit);
    }

    let high_part = _mm_extract_epi32(base10e8_32bit, 1) as u64;
    if high_part > 1844 || res_2digit > 67440737_09551615 {
        return None;
    } else {
        return Some(res_2digit + 1_0000_0000_0000_0000 * high_part);
    }
}
```



## bench

处理器是11th Gen Intel(R) Core(TM) i7-1185G7 @ 3.00GHz。处理1000000个随机u64的parse，速度比标准库提升了两倍多。

|        | 平均耗时  | 处理速度   |
| ------ | --------- | ---------- |
| simd   | 10.900 ms | 1.657 GB/s |
| normal | 30.463 ms | 0.593 GB/s |
| std    | 23.778 ms | 0.760 GB/s |



simdjson其实我还没仔细看，这里就不展开了，大家可以直接去看simdjson的[仓库](https://github.com/simdjson/simdjson)，算法相关的资料也都非常齐全。