//! Tests for GF(2^16) lookup table implementation.

use aise_core::field16;
use rand::Rng;

fn mul_portable(mut a: u16, mut b: u16) -> u16 {
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

#[test]
fn test_field16_mul_random() {
    let mut rng = rand::thread_rng();
    for _ in 0..1_000_000 {
        let a: u16 = rng.r#gen();
        let b: u16 = rng.r#gen();
        let expected = mul_portable(a, b);
        let actual = field16::mul(a, b);
        assert_eq!(
            expected, actual,
            "GF(2^16) mul mismatch: a={:#06x}, b={:#06x}, expected={:#06x}, actual={:#06x}",
            a, b, expected, actual
        );
    }
}
