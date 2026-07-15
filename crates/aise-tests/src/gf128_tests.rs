//! Comprehensive tests for GF(2^128) multiplication (Fix 1 validation).
//!
//! Tests cover:
//! - Edge cases: zero inputs, all-ones
//! - Boundary: top-bit, bottom-bit, reduction polynomial stress
//! - Random: 10,000 pairs verified against mul_portable
//! - Cross-check: independent Python/SageMath reference values

use aise_core::state::Lane;
use aise_core::field_b;
use rand::Rng;

// ---------------------------------------------------------------------------
// Cross-check: independent reference values computed by Python
// (a, b, expected_product) — verified against a standalone GF(2^128) implementation
// ---------------------------------------------------------------------------

/// Independent GF(2^128) reference values from Python.
/// Polynomial: x^128 + x^7 + x^2 + x + 1
const GF128_REFERENCE: [((u64, u64), (u64, u64), (u64, u64)); 10] = [
    // zero * zero
    ((0x0000000000000000, 0x0000000000000000), (0x0000000000000000, 0x0000000000000000), (0x0000000000000000, 0x0000000000000000)),
    // zero * nonzero
    ((0x0000000000000000, 0x0000000000000000), (0xdeadbeefcafebabe, 0x0123456789abcdef), (0x0000000000000000, 0x0000000000000000)),
    // all-ones * all-ones
    ((0xffffffffffffffff, 0xffffffffffffffff), (0xffffffffffffffff, 0xffffffffffffffff), (0x5555555555555555, 0x555555555555402f)),
    // top-bit * top-bit (stresses reduction: x^127 * x^127 = x^254 mod P)
    ((0x8000000000000000, 0x0000000000000000), (0x8000000000000000, 0x0000000000000000), (0xc000000000000000, 0x0000000000001067)),
    // bottom-bit * bottom-bit (1 * 1 = 1)
    ((0x0000000000000000, 0x0000000000000001), (0x0000000000000000, 0x0000000000000001), (0x0000000000000000, 0x0000000000000001)),
    // top-bit * bottom-bit (x^127 * 1 = x^127)
    ((0x8000000000000000, 0x0000000000000000), (0x0000000000000000, 0x0000000000000001), (0x8000000000000000, 0x0000000000000000)),
    // bottom-bit * top-bit (1 * x^127 = x^127, commutativity check)
    ((0x0000000000000000, 0x0000000000000001), (0x8000000000000000, 0x0000000000000000), (0x8000000000000000, 0x0000000000000000)),
    // generic random
    ((0xdeadbeefcafebabe, 0x0123456789abcdef), (0xcafebabedeadbeef, 0xfedcba9876543210), (0xb372cea992f32822, 0xe6c46ee37f795f48)),
    // reduction constant * itself (0x87 * 0x87)
    ((0x0000000000000000, 0x0000000000000087), (0x0000000000000000, 0x0000000000000087), (0x0000000000000000, 0x0000000000004015)),
    // top-bit * all-ones
    ((0x8000000000000000, 0x0000000000000000), (0xffffffffffffffff, 0xffffffffffffffff), (0x0000000000000000, 0x0000000000001fc7)),
];

#[test]
fn test_gf128_mul_cross_check_python() {
    for (i, &(a, b, expected)) in GF128_REFERENCE.iter().enumerate() {
        let la = Lane::new(a.0, a.1);
        let lb = Lane::new(b.0, b.1);
        let result = field_b::mul(la, lb);
        assert_eq!(
            (result.hi, result.lo), expected,
            "GF(2^128) mul cross-check failed at case {} (a={:?}, b={:?}): got ({:#018x}, {:#018x}), expected ({:#018x}, {:#018x})",
            i, a, b, result.hi, result.lo, expected.0, expected.1
        );
    }
}

#[test]
fn test_gf128_mul_edge_zero() {
    let zero = Lane::new(0, 0);
    let nonzero = Lane::new(0xDEADBEEFCAFEBABE, 0x0123456789ABCDEF);

    // 0 * 0 = 0
    let r = field_b::mul(zero, zero);
    assert_eq!((r.hi, r.lo), (0, 0), "0 * 0 should be 0");

    // 0 * x = 0
    let r = field_b::mul(zero, nonzero);
    assert_eq!((r.hi, r.lo), (0, 0), "0 * x should be 0");

    // x * 0 = 0
    let r = field_b::mul(nonzero, zero);
    assert_eq!((r.hi, r.lo), (0, 0), "x * 0 should be 0");
}

#[test]
fn test_gf128_mul_edge_identity() {
    let one = Lane::new(0, 1); // x^0 = 1 (identity element)
    let val = Lane::new(0xDEADBEEFCAFEBABE, 0x0123456789ABCDEF);

    // 1 * x = x
    let r = field_b::mul(one, val);
    assert_eq!((r.hi, r.lo), (val.hi, val.lo), "1 * x should be x");

    // x * 1 = x
    let r = field_b::mul(val, one);
    assert_eq!((r.hi, r.lo), (val.hi, val.lo), "x * 1 should be x");
}

#[test]
fn test_gf128_mul_edge_all_ones() {
    let all_ones = Lane::new(u64::MAX, u64::MAX);
    let r = field_b::mul(all_ones, all_ones);
    // Cross-checked with Python: (0x5555555555555555, 0x555555555555402f)
    assert_eq!(
        (r.hi, r.lo),
        (0x5555555555555555, 0x555555555555402f),
        "all-ones * all-ones mismatch"
    );
}

#[test]
fn test_gf128_mul_boundary_top_bit() {
    // x^127 (top bit of hi only)
    let top = Lane::new(0x8000000000000000, 0);

    // x^127 * x^127 = x^254 mod P — heavy reduction test
    let r = field_b::mul(top, top);
    assert_eq!(
        (r.hi, r.lo),
        (0xc000000000000000, 0x0000000000001067),
        "top-bit * top-bit (x^254 mod P) mismatch"
    );

    // x^127 * 1 = x^127 (no reduction)
    let one = Lane::new(0, 1);
    let r = field_b::mul(top, one);
    assert_eq!(
        (r.hi, r.lo),
        (0x8000000000000000, 0),
        "top-bit * 1 should be top-bit"
    );
}

#[test]
fn test_gf128_mul_boundary_bottom_bit() {
    // x^0 = 1 (bottom bit of lo only)
    let bottom = Lane::new(0, 1);

    // 1 * 1 = 1
    let r = field_b::mul(bottom, bottom);
    assert_eq!((r.hi, r.lo), (0, 1), "1 * 1 should be 1");
}

#[test]
fn test_gf128_mul_boundary_single_bits() {
    // Test x^k * x^k for several k values that stress different parts of the reduction
    let one = Lane::new(0, 1);

    // x^64 (bit 0 of hi)
    let x64 = Lane::new(1, 0);
    let r = field_b::mul(x64, x64);
    // x^128 mod P = x^7 + x^2 + x + 1 = 0x87
    assert_eq!(
        (r.hi, r.lo),
        (0, 0x87),
        "x^64 * x^64 = x^128 mod P should be 0x87"
    );

    // x^1 * x^127 = x^128 mod P = 0x87
    let x1 = Lane::new(0, 2);
    let x127 = Lane::new(0x8000000000000000, 0);
    let r = field_b::mul(x1, x127);
    assert_eq!(
        (r.hi, r.lo),
        (0, 0x87),
        "x^1 * x^127 = x^128 mod P should be 0x87"
    );

    // Verify commutativity: x^127 * x^1
    let r2 = field_b::mul(x127, x1);
    assert_eq!(
        (r.hi, r.lo), (r2.hi, r2.lo),
        "Commutativity violation: x^1 * x^127 != x^127 * x^1"
    );
}

#[test]
fn test_gf128_mul_commutativity_random() {
    let mut rng = rand::thread_rng();
    for _ in 0..10000 {
        let a = Lane::new(rng.r#gen(), rng.r#gen());
        let b = Lane::new(rng.r#gen(), rng.r#gen());
        let ab = field_b::mul(a, b);
        let ba = field_b::mul(b, a);
        assert_eq!(
            (ab.hi, ab.lo), (ba.hi, ba.lo),
            "Commutativity failure: a={:?}, b={:?}", a, b
        );
    }
}

#[test]
fn test_gf128_mul_matches_portable() {
    // Verify mul() produces identical results to mul_portable for 10,000 random pairs.
    // This catches any CLMUL reduction bugs since both code paths are independent.
    let mut rng = rand::thread_rng();
    for trial in 0..10000 {
        let a = Lane::new(rng.r#gen(), rng.r#gen());
        let b = Lane::new(rng.r#gen(), rng.r#gen());
        let fast = field_b::mul(a, b);
        let slow = field_b::mul_portable(a, b);
        assert_eq!(
            (fast.hi, fast.lo), (slow.hi, slow.lo),
            "mul vs mul_portable mismatch at trial {}: a=({:#018x},{:#018x}), b=({:#018x},{:#018x})",
            trial, a.hi, a.lo, b.hi, b.lo
        );
    }
}

#[test]
fn test_gf128_mul_portable_matches_reference() {
    // Verify mul_portable also matches the Python reference (guards against both sharing a bug)
    for (i, &(a, b, expected)) in GF128_REFERENCE.iter().enumerate() {
        let la = Lane::new(a.0, a.1);
        let lb = Lane::new(b.0, b.1);
        let result = field_b::mul_portable(la, lb);
        assert_eq!(
            (result.hi, result.lo), expected,
            "mul_portable cross-check failed at case {}", i
        );
    }
}

#[test]
fn test_sq_matches_mul() {
    let edge_cases = [
        Lane::new(0, 0),
        Lane::new(u64::MAX, u64::MAX),
        Lane::new(0x8000000000000000, 0), // top bit
        Lane::new(0, 1), // bottom bit
    ];
    
    for (i, &a) in edge_cases.iter().enumerate() {
        let fast = field_b::sq(a);
        let slow = field_b::mul(a, a);
        assert_eq!(
            (fast.hi, fast.lo), (slow.hi, slow.lo),
            "sq vs mul mismatch at edge case {}: a=({:#018x},{:#018x})",
            i, a.hi, a.lo
        );
    }

    let mut rng = rand::thread_rng();
    for trial in 0..10000 {
        let a = Lane::new(rng.r#gen(), rng.r#gen());
        let fast = field_b::sq(a);
        let slow = field_b::mul(a, a);
        assert_eq!(
            (fast.hi, fast.lo), (slow.hi, slow.lo),
            "sq vs mul mismatch at random trial {}: a=({:#018x},{:#018x})",
            trial, a.hi, a.lo
        );
    }
}
