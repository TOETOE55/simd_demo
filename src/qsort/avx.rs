// The MIT License (MIT)
//
// Copyright (c) 2017 Bramas, Berenger (bbramas)
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::{
    __m512i, _mm512_cmp_epi32_mask, _mm512_loadu_epi32, _mm512_mask_compressstoreu_epi32,
    _mm512_mask_mov_epi32, _mm512_maskz_loadu_epi32, _mm512_maskz_set1_epi32, _mm512_max_epi32,
    _mm512_min_epi32, _mm512_or_epi32, _mm512_or_si512, _mm512_permutexvar_epi32,
    _mm512_set1_epi32, _mm512_set_epi32, _mm512_storeu_epi32, _MM_CMPINT_LE,
};

#[cfg(target_arch = "x86")]
use std::arch::x86_64::{
    __m512i, _mm512_cmp_epi32_mask, _mm512_loadu_epi32, _mm512_mask_compressstoreu_epi32,
    _mm512_mask_mov_epi32, _mm512_maskz_loadu_epi32, _mm512_maskz_set1_epi32, _mm512_max_epi32,
    _mm512_min_epi32, _mm512_or_epi32, _mm512_or_si512, _mm512_permutexvar_epi32,
    _mm512_set1_epi32, _mm512_set_epi32, _mm512_storeu_epi32, _MM_CMPINT_LE,
};
use std::mem::size_of;

const S: usize = size_of::<__m512i>() / size_of::<i32>();
pub fn qsort(arr: &mut [i32]) {
    let len = arr.len();
    match len {
        ..=1 => return,
        2..16 => unsafe {
            let ptr = arr.as_mut_ptr();
            let rest = S - len;
            let mut v1 = _mm512_or_si512(
                _mm512_maskz_loadu_epi32(0xFFFF >> rest, ptr),
                _mm512_maskz_set1_epi32(0xFFFF << len, i32::MAX),
            );
            bitonic_sort1_impl(&mut v1);
            _mm512_mask_compressstoreu_epi32(ptr.cast(), 0xFFFF >> rest, v1);
        },
        16 => unsafe {
            let ptr = arr.as_mut_ptr();
            let mut v1 = _mm512_loadu_epi32(ptr);
            bitonic_sort1_impl(&mut v1);
            _mm512_storeu_epi32(ptr, v1);
        },

        17..32 => unsafe {
            let ptr = arr.as_mut_ptr();
            let rest = 2 * S - len;
            let last_size = len - S;
            let mut v1 = _mm512_loadu_epi32(ptr);
            let mut v2 = _mm512_or_epi32(
                _mm512_maskz_loadu_epi32(0xFFFF >> rest, ptr.offset(S as isize)),
                _mm512_maskz_set1_epi32(0xFFFF << last_size, i32::MAX),
            );
            bitonic_sort2_impl(&mut v1, &mut v2);
            _mm512_storeu_epi32(ptr, v1);
            _mm512_mask_compressstoreu_epi32(ptr.offset(S as isize).cast(), 0xFFFF >> rest, v2);
        },

        32 => unsafe {
            let ptr = arr.as_mut_ptr();
            let mut v1 = _mm512_loadu_epi32(ptr);
            let mut v2 = _mm512_loadu_epi32(ptr.offset(S as isize));
            bitonic_sort2_impl(&mut v1, &mut v2);

            _mm512_storeu_epi32(ptr, v1);
            _mm512_storeu_epi32(ptr.offset(S as isize), v2);
        },
        _ => unsafe {
            let part = partition(arr);
            qsort(arr.get_unchecked_mut(..part));
            qsort(arr.get_unchecked_mut(part + 1..));
        },
    }
}

#[inline(always)]
unsafe fn get_pivot(arr: &[i32]) -> (usize, i32) {
    let mid = arr.len() / 2;
    let right = arr.len() - 1;
    let a = *arr.get_unchecked(0);
    let b = *arr.get_unchecked(mid);
    let c = *arr.get_unchecked(right);
    if (a..c).contains(&b) {
        (mid, b)
    } else if (b..c).contains(&a) {
        (0, a)
    } else {
        (right, c)
    }
}

#[inline(always)]
fn partition(arr: &mut [i32]) -> usize {
    unsafe {
        let len = arr.len();
        let (idx, pivot) = get_pivot(arr);
        arr.swap_unchecked(idx, len - 1);
        let part = if len <= 2 * S {
            normal_partition(arr, pivot)
        } else {
            vector_partition(arr, pivot)
        };
        arr.swap_unchecked(part, len - 1);
        part
    }
}

#[inline(always)]
fn normal_partition(arr: &mut [i32], pivot: i32) -> usize {
    let mut i = 0;
    for j in 0..arr.len() - 1 {
        unsafe {
            if *arr.get_unchecked(j) <= pivot {
                arr.swap_unchecked(i, j);
                i += 1;
            }
        }
    }

    i
}

#[inline(always)]
unsafe fn vector_partition(arr: &mut [i32], pivot: i32) -> usize {
    let mut left = 0;
    let mut right = arr.len() - 1;
    unsafe {
        let pivotvec = _mm512_set1_epi32(pivot);

        let left_val = _mm512_loadu_epi32(arr.as_ptr().offset(left as isize));
        let mut left_w = left;
        left += S;

        let mut right_w = right;
        right -= S;
        let right_val = _mm512_loadu_epi32(arr.as_ptr().offset(right as isize));

        while left + S <= right {
            let val;
            if left - left_w <= right_w - right {
                val = _mm512_loadu_epi32(arr.as_ptr().offset(left as isize));
                left += S;
            } else {
                right -= S;
                val = _mm512_loadu_epi32(arr.as_ptr().offset(right as isize));
            }

            let mask = _mm512_cmp_epi32_mask::<_MM_CMPINT_LE>(val, pivotvec);

            let nb_low = mask.count_ones() as usize;
            let nb_high = S - nb_low;

            _mm512_mask_compressstoreu_epi32(
                arr.as_mut_ptr().offset(left_w as isize) as *mut i32 as _,
                mask,
                val,
            );
            left_w += nb_low;

            right_w -= nb_high;
            _mm512_mask_compressstoreu_epi32(
                arr.as_mut_ptr().offset(right_w as isize) as *mut i32 as _,
                !mask,
                val,
            );
        }

        {
            let remaining = right - left;
            let val = _mm512_loadu_epi32(arr.as_ptr().offset(left as isize));
            // left = right;

            let mask = _mm512_cmp_epi32_mask::<_MM_CMPINT_LE>(val, pivotvec);

            let mask_low = mask & !(!0 << remaining);
            let mask_high = !mask & !(!0 << remaining);

            let nb_low = mask_low.count_ones() as usize;
            let nb_high = mask_high.count_ones() as usize;

            _mm512_mask_compressstoreu_epi32(
                arr.as_mut_ptr().offset(left_w as isize) as *mut i32 as _,
                mask_low,
                val,
            );
            left_w += nb_low;

            right_w -= nb_high;
            _mm512_mask_compressstoreu_epi32(
                arr.as_mut_ptr().offset(right_w as isize) as *mut i32 as _,
                mask_high,
                val,
            );
        }
        {
            let mask = _mm512_cmp_epi32_mask::<_MM_CMPINT_LE>(left_val, pivotvec);

            let nb_low = mask.count_ones() as usize;
            let nb_high = S - nb_low;

            _mm512_mask_compressstoreu_epi32(
                arr.as_mut_ptr().offset(left_w as isize) as *mut i32 as _,
                mask,
                left_val,
            );
            left_w += nb_low;

            right_w -= nb_high;
            _mm512_mask_compressstoreu_epi32(
                arr.as_mut_ptr().offset(right_w as isize) as *mut i32 as _,
                !mask,
                left_val,
            );
        }
        {
            let mask = _mm512_cmp_epi32_mask::<_MM_CMPINT_LE>(right_val, pivotvec);

            let nb_low = mask.count_ones() as usize;
            let nb_high = S - nb_low;

            _mm512_mask_compressstoreu_epi32(
                arr.as_mut_ptr().offset(left_w as isize) as *mut i32 as _,
                mask,
                right_val,
            );
            left_w += nb_low;

            right_w -= nb_high;
            _mm512_mask_compressstoreu_epi32(
                arr.as_mut_ptr().offset(right_w as isize) as *mut i32 as _,
                !mask,
                right_val,
            );
        }
        left_w
    }
}

#[inline(always)]
unsafe fn bitonic_sort1_impl(input: &mut __m512i) {
    unsafe {
        let idx_no_neigh = _mm512_set_epi32(14, 15, 12, 13, 10, 11, 8, 9, 6, 7, 4, 5, 2, 3, 0, 1);
        let perm_neigh = _mm512_permutexvar_epi32(idx_no_neigh, *input);
        let pperm_neigh_min = _mm512_min_epi32(*input, perm_neigh);
        let perm_neigh_max = _mm512_max_epi32(perm_neigh, *input);
        *input = _mm512_mask_mov_epi32(pperm_neigh_min, 0xAAAA, perm_neigh_max);
    }
    unsafe {
        let idx_no_neigh = _mm512_set_epi32(12, 13, 14, 15, 8, 9, 10, 11, 4, 5, 6, 7, 0, 1, 2, 3);
        let perm_neigh = _mm512_permutexvar_epi32(idx_no_neigh, *input);
        let pperm_neigh_min = _mm512_min_epi32(*input, perm_neigh);
        let perm_neigh_max = _mm512_max_epi32(perm_neigh, *input);
        *input = _mm512_mask_mov_epi32(pperm_neigh_min, 0xCCCC, perm_neigh_max);
    }
    unsafe {
        let idx_no_neigh = _mm512_set_epi32(14, 15, 12, 13, 10, 11, 8, 9, 6, 7, 4, 5, 2, 3, 0, 1);
        let perm_neigh = _mm512_permutexvar_epi32(idx_no_neigh, *input);
        let pperm_neigh_min = _mm512_min_epi32(*input, perm_neigh);
        let perm_neigh_max = _mm512_max_epi32(perm_neigh, *input);
        *input = _mm512_mask_mov_epi32(pperm_neigh_min, 0xAAAA, perm_neigh_max);
    }
    unsafe {
        let idx_no_neigh = _mm512_set_epi32(8, 9, 10, 11, 12, 13, 14, 15, 0, 1, 2, 3, 4, 5, 6, 7);
        let perm_neigh = _mm512_permutexvar_epi32(idx_no_neigh, *input);
        let pperm_neigh_min = _mm512_min_epi32(*input, perm_neigh);
        let perm_neigh_max = _mm512_max_epi32(perm_neigh, *input);
        *input = _mm512_mask_mov_epi32(pperm_neigh_min, 0xF0F0, perm_neigh_max);
    }
    unsafe {
        let idx_no_neigh = _mm512_set_epi32(13, 12, 15, 14, 9, 8, 11, 10, 5, 4, 7, 6, 1, 0, 3, 2);
        let perm_neigh = _mm512_permutexvar_epi32(idx_no_neigh, *input);
        let pperm_neigh_min = _mm512_min_epi32(*input, perm_neigh);
        let perm_neigh_max = _mm512_max_epi32(perm_neigh, *input);
        *input = _mm512_mask_mov_epi32(pperm_neigh_min, 0xCCCC, perm_neigh_max);
    }
    unsafe {
        let idx_no_neigh = _mm512_set_epi32(14, 15, 12, 13, 10, 11, 8, 9, 6, 7, 4, 5, 2, 3, 0, 1);
        let perm_neigh = _mm512_permutexvar_epi32(idx_no_neigh, *input);
        let pperm_neigh_min = _mm512_min_epi32(*input, perm_neigh);
        let perm_neigh_max = _mm512_max_epi32(perm_neigh, *input);
        *input = _mm512_mask_mov_epi32(pperm_neigh_min, 0xAAAA, perm_neigh_max);
    }
    unsafe {
        let idx_no_neigh = _mm512_set_epi32(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);
        let perm_neigh = _mm512_permutexvar_epi32(idx_no_neigh, *input);
        let pperm_neigh_min = _mm512_min_epi32(*input, perm_neigh);
        let perm_neigh_max = _mm512_max_epi32(perm_neigh, *input);
        *input = _mm512_mask_mov_epi32(pperm_neigh_min, 0xFF00, perm_neigh_max);
    }
    unsafe {
        let idx_no_neigh = _mm512_set_epi32(11, 10, 9, 8, 15, 14, 13, 12, 3, 2, 1, 0, 7, 6, 5, 4);
        let perm_neigh = _mm512_permutexvar_epi32(idx_no_neigh, *input);
        let pperm_neigh_min = _mm512_min_epi32(*input, perm_neigh);
        let perm_neigh_max = _mm512_max_epi32(perm_neigh, *input);
        *input = _mm512_mask_mov_epi32(pperm_neigh_min, 0xF0F0, perm_neigh_max);
    }
    unsafe {
        let idx_no_neigh = _mm512_set_epi32(13, 12, 15, 14, 9, 8, 11, 10, 5, 4, 7, 6, 1, 0, 3, 2);
        let perm_neigh = _mm512_permutexvar_epi32(idx_no_neigh, *input);
        let pperm_neigh_min = _mm512_min_epi32(*input, perm_neigh);
        let perm_neigh_max = _mm512_max_epi32(perm_neigh, *input);
        *input = _mm512_mask_mov_epi32(pperm_neigh_min, 0xCCCC, perm_neigh_max);
    }
    unsafe {
        let idx_no_neigh = _mm512_set_epi32(14, 15, 12, 13, 10, 11, 8, 9, 6, 7, 4, 5, 2, 3, 0, 1);
        let perm_neigh = _mm512_permutexvar_epi32(idx_no_neigh, *input);
        let pperm_neigh_min = _mm512_min_epi32(*input, perm_neigh);
        let perm_neigh_max = _mm512_max_epi32(perm_neigh, *input);
        *input = _mm512_mask_mov_epi32(pperm_neigh_min, 0xAAAA, perm_neigh_max);
    }
}

#[inline(always)]
unsafe fn bitonic_sort2_impl(input: &mut __m512i, input2: &mut __m512i) {
    unsafe {
        let idx_no_neigh = _mm512_set_epi32(14, 15, 12, 13, 10, 11, 8, 9, 6, 7, 4, 5, 2, 3, 0, 1);
        let perm_neigh = _mm512_permutexvar_epi32(idx_no_neigh, *input);
        let perm_neigh2 = _mm512_permutexvar_epi32(idx_no_neigh, *input2);
        let perm_neigh_min = _mm512_min_epi32(*input, perm_neigh);
        let perm_neigh_min2 = _mm512_min_epi32(*input2, perm_neigh2);
        let perm_neigh_max = _mm512_max_epi32(perm_neigh, *input);
        let perm_neigh_max2 = _mm512_max_epi32(perm_neigh2, *input2);
        *input = _mm512_mask_mov_epi32(perm_neigh_min, 0xAAAA, perm_neigh_max);
        *input2 = _mm512_mask_mov_epi32(perm_neigh_min2, 0xAAAA, perm_neigh_max2);
    }
    unsafe {
        let idx_no_neigh = _mm512_set_epi32(12, 13, 14, 15, 8, 9, 10, 11, 4, 5, 6, 7, 0, 1, 2, 3);
        let perm_neigh = _mm512_permutexvar_epi32(idx_no_neigh, *input);
        let perm_neigh2 = _mm512_permutexvar_epi32(idx_no_neigh, *input2);
        let perm_neigh_min = _mm512_min_epi32(*input, perm_neigh);
        let perm_neigh_min2 = _mm512_min_epi32(*input2, perm_neigh2);
        let perm_neigh_max = _mm512_max_epi32(perm_neigh, *input);
        let perm_neigh_max2 = _mm512_max_epi32(perm_neigh2, *input2);
        *input = _mm512_mask_mov_epi32(perm_neigh_min, 0xCCCC, perm_neigh_max);
        *input2 = _mm512_mask_mov_epi32(perm_neigh_min2, 0xCCCC, perm_neigh_max2);
    }
    unsafe {
        let idx_no_neigh = _mm512_set_epi32(14, 15, 12, 13, 10, 11, 8, 9, 6, 7, 4, 5, 2, 3, 0, 1);
        let perm_neigh = _mm512_permutexvar_epi32(idx_no_neigh, *input);
        let perm_neigh2 = _mm512_permutexvar_epi32(idx_no_neigh, *input2);
        let perm_neigh_min = _mm512_min_epi32(*input, perm_neigh);
        let perm_neigh_min2 = _mm512_min_epi32(*input2, perm_neigh2);
        let perm_neigh_max = _mm512_max_epi32(perm_neigh, *input);
        let perm_neigh_max2 = _mm512_max_epi32(perm_neigh2, *input2);
        *input = _mm512_mask_mov_epi32(perm_neigh_min, 0xAAAA, perm_neigh_max);
        *input2 = _mm512_mask_mov_epi32(perm_neigh_min2, 0xAAAA, perm_neigh_max2);
    }
    unsafe {
        let idx_no_neigh = _mm512_set_epi32(8, 9, 10, 11, 12, 13, 14, 15, 0, 1, 2, 3, 4, 5, 6, 7);
        let perm_neigh = _mm512_permutexvar_epi32(idx_no_neigh, *input);
        let perm_neigh2 = _mm512_permutexvar_epi32(idx_no_neigh, *input2);
        let perm_neigh_min = _mm512_min_epi32(*input, perm_neigh);
        let perm_neigh_min2 = _mm512_min_epi32(*input2, perm_neigh2);
        let perm_neigh_max = _mm512_max_epi32(perm_neigh, *input);
        let perm_neigh_max2 = _mm512_max_epi32(perm_neigh2, *input2);
        *input = _mm512_mask_mov_epi32(perm_neigh_min, 0xF0F0, perm_neigh_max);
        *input2 = _mm512_mask_mov_epi32(perm_neigh_min2, 0xF0F0, perm_neigh_max2);
    }
    unsafe {
        let idx_no_neigh = _mm512_set_epi32(13, 12, 15, 14, 9, 8, 11, 10, 5, 4, 7, 6, 1, 0, 3, 2);
        let perm_neigh = _mm512_permutexvar_epi32(idx_no_neigh, *input);
        let perm_neigh2 = _mm512_permutexvar_epi32(idx_no_neigh, *input2);
        let perm_neigh_min = _mm512_min_epi32(*input, perm_neigh);
        let perm_neigh_min2 = _mm512_min_epi32(*input2, perm_neigh2);
        let perm_neigh_max = _mm512_max_epi32(perm_neigh, *input);
        let perm_neigh_max2 = _mm512_max_epi32(perm_neigh2, *input2);
        *input = _mm512_mask_mov_epi32(perm_neigh_min, 0xCCCC, perm_neigh_max);
        *input2 = _mm512_mask_mov_epi32(perm_neigh_min2, 0xCCCC, perm_neigh_max2);
    }
    unsafe {
        let idx_no_neigh = _mm512_set_epi32(14, 15, 12, 13, 10, 11, 8, 9, 6, 7, 4, 5, 2, 3, 0, 1);
        let perm_neigh = _mm512_permutexvar_epi32(idx_no_neigh, *input);
        let perm_neigh2 = _mm512_permutexvar_epi32(idx_no_neigh, *input2);
        let perm_neigh_min = _mm512_min_epi32(*input, perm_neigh);
        let perm_neigh_min2 = _mm512_min_epi32(*input2, perm_neigh2);
        let perm_neigh_max = _mm512_max_epi32(perm_neigh, *input);
        let perm_neigh_max2 = _mm512_max_epi32(perm_neigh2, *input2);
        *input = _mm512_mask_mov_epi32(perm_neigh_min, 0xAAAA, perm_neigh_max);
        *input2 = _mm512_mask_mov_epi32(perm_neigh_min2, 0xAAAA, perm_neigh_max2);
    }
    unsafe {
        let idx_no_neigh = _mm512_set_epi32(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);
        let perm_neigh = _mm512_permutexvar_epi32(idx_no_neigh, *input);
        let perm_neigh2 = _mm512_permutexvar_epi32(idx_no_neigh, *input2);
        let perm_neigh_min = _mm512_min_epi32(*input, perm_neigh);
        let perm_neigh_min2 = _mm512_min_epi32(*input2, perm_neigh2);
        let perm_neigh_max = _mm512_max_epi32(perm_neigh, *input);
        let perm_neigh_max2 = _mm512_max_epi32(perm_neigh2, *input2);
        *input = _mm512_mask_mov_epi32(perm_neigh_min, 0xFF00, perm_neigh_max);
        *input2 = _mm512_mask_mov_epi32(perm_neigh_min2, 0xFF00, perm_neigh_max2);
    }
    unsafe {
        let idx_no_neigh = _mm512_set_epi32(11, 10, 9, 8, 15, 14, 13, 12, 3, 2, 1, 0, 7, 6, 5, 4);
        let perm_neigh = _mm512_permutexvar_epi32(idx_no_neigh, *input);
        let perm_neigh2 = _mm512_permutexvar_epi32(idx_no_neigh, *input2);
        let perm_neigh_min = _mm512_min_epi32(*input, perm_neigh);
        let perm_neigh_min2 = _mm512_min_epi32(*input2, perm_neigh2);
        let perm_neigh_max = _mm512_max_epi32(perm_neigh, *input);
        let perm_neigh_max2 = _mm512_max_epi32(perm_neigh2, *input2);
        *input = _mm512_mask_mov_epi32(perm_neigh_min, 0xF0F0, perm_neigh_max);
        *input2 = _mm512_mask_mov_epi32(perm_neigh_min2, 0xF0F0, perm_neigh_max2);
    }
    unsafe {
        let idx_no_neigh = _mm512_set_epi32(13, 12, 15, 14, 9, 8, 11, 10, 5, 4, 7, 6, 1, 0, 3, 2);
        let perm_neigh = _mm512_permutexvar_epi32(idx_no_neigh, *input);
        let perm_neigh2 = _mm512_permutexvar_epi32(idx_no_neigh, *input2);
        let perm_neigh_min = _mm512_min_epi32(*input, perm_neigh);
        let perm_neigh_min2 = _mm512_min_epi32(*input2, perm_neigh2);
        let perm_neigh_max = _mm512_max_epi32(perm_neigh, *input);
        let perm_neigh_max2 = _mm512_max_epi32(perm_neigh2, *input2);
        *input = _mm512_mask_mov_epi32(perm_neigh_min, 0xCCCC, perm_neigh_max);
        *input2 = _mm512_mask_mov_epi32(perm_neigh_min2, 0xCCCC, perm_neigh_max2);
    }
    unsafe {
        let idx_no_neigh = _mm512_set_epi32(14, 15, 12, 13, 10, 11, 8, 9, 6, 7, 4, 5, 2, 3, 0, 1);
        let perm_neigh = _mm512_permutexvar_epi32(idx_no_neigh, *input);
        let perm_neigh2 = _mm512_permutexvar_epi32(idx_no_neigh, *input2);
        let perm_neigh_min = _mm512_min_epi32(*input, perm_neigh);
        let perm_neigh_min2 = _mm512_min_epi32(*input2, perm_neigh2);
        let perm_neigh_max = _mm512_max_epi32(perm_neigh, *input);
        let perm_neigh_max2 = _mm512_max_epi32(perm_neigh2, *input2);
        *input = _mm512_mask_mov_epi32(perm_neigh_min, 0xAAAA, perm_neigh_max);
        *input2 = _mm512_mask_mov_epi32(perm_neigh_min2, 0xAAAA, perm_neigh_max2);
    }
    bitonic_sort2_merge_impl(input, input2);
}

#[inline(always)]
unsafe fn bitonic_sort2_merge_impl(input: &mut __m512i, input2: &mut __m512i) {
    unsafe {
        let idx_no_neigh = _mm512_set_epi32(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);
        let perm_neigh = _mm512_permutexvar_epi32(idx_no_neigh, *input);
        *input = _mm512_min_epi32(perm_neigh, *input2);
        *input2 = _mm512_max_epi32(*input2, perm_neigh);
    }
    unsafe {
        let idx_no_neigh = _mm512_set_epi32(7, 6, 5, 4, 3, 2, 1, 0, 15, 14, 13, 12, 11, 10, 9, 8);
        let perm_neigh = _mm512_permutexvar_epi32(idx_no_neigh, *input);
        let perm_neigh2 = _mm512_permutexvar_epi32(idx_no_neigh, *input2);
        let perm_neigh_min = _mm512_min_epi32(*input, perm_neigh);
        let perm_neigh_min2 = _mm512_min_epi32(*input2, perm_neigh2);
        let perm_neigh_max = _mm512_max_epi32(perm_neigh, *input);
        let perm_neigh_max2 = _mm512_max_epi32(perm_neigh2, *input2);
        *input = _mm512_mask_mov_epi32(perm_neigh_min, 0xFF00, perm_neigh_max);
        *input2 = _mm512_mask_mov_epi32(perm_neigh_min2, 0xFF00, perm_neigh_max2);
    }
    unsafe {
        let idx_no_neigh = _mm512_set_epi32(11, 10, 9, 8, 15, 14, 13, 12, 3, 2, 1, 0, 7, 6, 5, 4);
        let perm_neigh = _mm512_permutexvar_epi32(idx_no_neigh, *input);
        let perm_neigh2 = _mm512_permutexvar_epi32(idx_no_neigh, *input2);
        let perm_neigh_min = _mm512_min_epi32(*input, perm_neigh);
        let perm_neigh_min2 = _mm512_min_epi32(*input2, perm_neigh2);
        let perm_neigh_max = _mm512_max_epi32(perm_neigh, *input);
        let perm_neigh_max2 = _mm512_max_epi32(perm_neigh2, *input2);
        *input = _mm512_mask_mov_epi32(perm_neigh_min, 0xF0F0, perm_neigh_max);
        *input2 = _mm512_mask_mov_epi32(perm_neigh_min2, 0xF0F0, perm_neigh_max2);
    }
    unsafe {
        let idx_no_neigh = _mm512_set_epi32(13, 12, 15, 14, 9, 8, 11, 10, 5, 4, 7, 6, 1, 0, 3, 2);
        let perm_neigh = _mm512_permutexvar_epi32(idx_no_neigh, *input);
        let perm_neigh2 = _mm512_permutexvar_epi32(idx_no_neigh, *input2);
        let perm_neigh_min = _mm512_min_epi32(*input, perm_neigh);
        let perm_neigh_min2 = _mm512_min_epi32(*input2, perm_neigh2);
        let perm_neigh_max = _mm512_max_epi32(perm_neigh, *input);
        let perm_neigh_max2 = _mm512_max_epi32(perm_neigh2, *input2);
        *input = _mm512_mask_mov_epi32(perm_neigh_min, 0xCCCC, perm_neigh_max);
        *input2 = _mm512_mask_mov_epi32(perm_neigh_min2, 0xCCCC, perm_neigh_max2);
    }
    unsafe {
        let idx_no_neigh = _mm512_set_epi32(14, 15, 12, 13, 10, 11, 8, 9, 6, 7, 4, 5, 2, 3, 0, 1);
        let perm_neigh = _mm512_permutexvar_epi32(idx_no_neigh, *input);
        let perm_neigh2 = _mm512_permutexvar_epi32(idx_no_neigh, *input2);
        let perm_neigh_min = _mm512_min_epi32(*input, perm_neigh);
        let perm_neigh_min2 = _mm512_min_epi32(*input2, perm_neigh2);
        let perm_neigh_max = _mm512_max_epi32(perm_neigh, *input);
        let perm_neigh_max2 = _mm512_max_epi32(perm_neigh2, *input2);
        *input = _mm512_mask_mov_epi32(perm_neigh_min, 0xAAAA, perm_neigh_max);
        *input2 = _mm512_mask_mov_epi32(perm_neigh_min2, 0xAAAA, perm_neigh_max2);
    }
}
