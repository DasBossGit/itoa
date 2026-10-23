#[cfg(feature = "no-panic")]
use no_panic::no_panic;

/// Multiply unsigned 128 bit integers, return upper 128 bits of the result
#[inline]
#[cfg_attr(feature = "no-panic", no_panic)]
pub(crate) const fn mulhi(x: u128, y: u128) -> u128 {
    let x_lo = x as u64 as u128;
    let x_hi = (x >> 64) as u64 as u128;
    let y_lo = y as u64 as u128;
    let y_hi = (y >> 64) as u64 as u128;

    // SAFETY: every operand is < 2^64, so each product is <= (2^64-1)^2,
    // and each sum is bounded by 2^128 - 2^64 (see proof above).
    unsafe {
        let carry = x_lo.unchecked_mul(y_lo) >> 64;
        let m = x_lo.unchecked_mul(y_hi).unchecked_add(carry);
        let high1 = m >> 64;

        let m_lo = m as u64 as u128;
        let high2 = x_hi.unchecked_mul(y_lo).unchecked_add(m_lo) >> 64;

        x_hi.unchecked_mul(y_hi)
            .unchecked_add(high1)
            .unchecked_add(high2)
    }
}
