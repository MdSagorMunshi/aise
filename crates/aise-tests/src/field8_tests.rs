//! Exhaustive tests for GF(2^8) lookup table implementation.

use aise_core::field8;

fn mul_portable(mut a: u8, mut b: u8) -> u8 {
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

#[test]
fn test_field8_mul_exhaustive() {
    for a in 0..=255 {
        for b in 0..=255 {
            let expected = mul_portable(a, b);
            let actual = field8::mul(a, b);
            assert_eq!(
                expected, actual,
                "GF(2^8) mul mismatch: a={:#04x}, b={:#04x}, expected={:#04x}, actual={:#04x}",
                a, b, expected, actual
            );
        }
    }
}
