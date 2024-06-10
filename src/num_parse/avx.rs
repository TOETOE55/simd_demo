use std::arch::x86_64::{
    _mm256_cvtepi16_epi8, _mm256_maddubs_epi16, _mm256_mask_cmpgt_epu8_mask,
    _mm256_maskz_loadu_epi8, _mm256_maskz_sub_epi8, _mm256_set1_epi8, _mm256_set_epi8,
    _mm_extract_epi32, _mm_madd_epi16, _mm_maddubs_epi16, _mm_set_epi16, _mm_set_epi8,
};

#[inline(always)]
pub fn parse_u64(s: &str) -> Option<u64> {
    let bytes = s.as_bytes();

    // 1. 超过20字节的字符串超过u64的范围
    if bytes.len() > 20 {
        return None;
    }

    let start = bytes.as_ptr();
    let end = unsafe { start.offset(bytes.len() as isize) };
    let mask = 0xFFFFFFFF_u32 << (32 - s.len());

    let base10_8bit = unsafe {
        let ascii_zero = _mm256_set1_epi8('0' as i8);
        let nine = _mm256_set1_epi8(9);

        // 2. 将字符串读入到向量的高位中，向量低位置为0（最少浪费12位）
        //    *注意*：
        //    - `__m256i` 可以表示32个 `u8`
        //    - 字符串的高低位与数字的高低位是相反的
        let s_bytes_v = _mm256_maskz_loadu_epi8(mask, end.offset(-32).cast());

        // 3. 将向量读到的字符串部分，每个字节 - '0'
        let base10_8bit = _mm256_maskz_sub_epi8(mask, s_bytes_v, ascii_zero);

        // 4. 如果存在字节 > 9的，说明存在非数字的字节
        //    *注意*: 这里按u8解释i8，所以如果是负数，也会 > 9
        let nondigits = _mm256_mask_cmpgt_epu8_mask(mask, base10_8bit, nine);
        if nondigits != 0 {
            return None;
        }

        base10_8bit
    };

    // 使用向量计算10进制求和
    // 最后得到8位(digits)整数向量(4x32bits)
    //
    // `u64::MAX == 1844_67440737_09551615`(20bytes)
    // 向量只看高位3个分量
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
}
