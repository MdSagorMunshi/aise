pub mod level2;
pub mod level4;

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
}
