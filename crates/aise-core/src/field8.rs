//! GF(2^8) arithmetic.

#[inline(always)]
pub fn mul(mut a: u8, mut b: u8) -> u8 {
    let mut p = 0;
    for _ in 0..8 {
        if b & 1 != 0 { p ^= a; }
        let hi = a & 0x80;
        a <<= 1;
        if hi != 0 { a ^= 0x1B; }
        b >>= 1;
    }
    p
}
