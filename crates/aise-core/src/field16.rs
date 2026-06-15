//! GF(2^16) arithmetic.

#[inline(always)]
pub fn mul(mut a: u16, mut b: u16) -> u16 {
    let mut p = 0;
    for _ in 0..16 {
        if b & 1 != 0 { p ^= a; }
        let hi = a & 0x8000;
        a <<= 1;
        if hi != 0 { a ^= 0x002B; }
        b >>= 1;
    }
    p
}
