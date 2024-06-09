
pub fn parse_u64(s: &str) -> Option<u64> {
    let mut res = 0_u64;
    let iter = s
        .chars()
        .map(|d| (d as u64).checked_sub('0' as u64));
    for digit in iter {
        let digit = digit.filter(|d| *d <= 9)?;
        res = res.checked_mul(10_u64)?.checked_add(digit)?;
    }

    Some(res)
}
