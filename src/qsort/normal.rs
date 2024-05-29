pub fn qsort(v: &mut [i32]) {
    fn sort(v: &mut [i32], mut low: usize, mut high: usize) {
        while low < high {
            let (lt, gt) = partition3(v, low, high);
            if lt - low < high - gt {
                sort(v, low, lt);
                low = lt + 1;
            } else {
                sort(v, gt, high);
                high = gt - 1;
            }
        }
    }

    let len = v.len();

    if len <= 1 {
        return;
    }

    sort(v, 0, len - 1);
}

fn partition3(v: &mut [i32], low: usize, high: usize) -> (usize, usize) {
    let pivot = unsafe { *v.get_unchecked(high) };
    let mut i = low; // lt
    let mut j = low; // eq
    let mut k = high; // gt

    while j <= k {
        let e = unsafe { *v.get_unchecked(j) };
        if e < pivot {
            unsafe {
                v.swap_unchecked(i, j);
            }
            i += 1;
            j += 1;
        } else if e > pivot {
            unsafe {
                v.swap_unchecked(k, j);
            }
            k -= 1;
        } else {
            j += 1;
        }
    }

    (i, k)
}
