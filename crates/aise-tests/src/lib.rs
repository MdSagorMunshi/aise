pub mod level2;
pub mod level4;
pub mod frozen_vectors;
pub mod frozen_vectors_c;
pub mod gf128_tests;
pub mod field8_tests;
pub mod field16_tests;

#[cfg(test)]
mod tests {
    use aise_core::state::{Lane, State};
    use aise_core::field_p;
    use aise_core::field_b;
    use num_bigint::{BigUint, ToBigUint};

    #[test]
    fn test_field_p_arithmetic() {
        let a = field_p::add(10, 20);
        assert_eq!(a, 30);
    }

    #[test]
    #[ignore] // Takes a bit of time for 100k, run explicitly or with --ignored
    fn test_powd_matches_binary() {
        let mut rng = 1u128;
        
        // Edge cases
        assert_eq!(field_p::powd(0), field_p::powd_binary_fallback(0), "Mismatch on zero");
        assert_eq!(field_p::powd(1), field_p::powd_binary_fallback(1), "Mismatch on 1");
        assert_eq!(field_p::powd(field_p::P - 1), field_p::powd_binary_fallback(field_p::P - 1), "Mismatch on P-1");
        assert_eq!(field_p::powd(1 << 126), field_p::powd_binary_fallback(1 << 126), "Mismatch on top bit");
        assert_eq!(field_p::powd(2), field_p::powd_binary_fallback(2), "Mismatch on bottom bit");

        // 100,000 random cases
        for _ in 0..100_000 {
            rng = rng.wrapping_mul(0xDEADBEEFCAFEBABE0123456789ABCDEF).wrapping_add(1);
            let a = rng & field_p::P;
            
            let expected = field_p::powd_binary_fallback(a);
            let actual = field_p::powd(a);
            
            assert_eq!(actual, expected, "Mismatch at a={}", a);
        }
    }
}
