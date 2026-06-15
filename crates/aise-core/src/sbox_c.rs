//! SubField_C for GF(p)

use crate::field_p;

#[inline(always)]
pub fn apply(x: u128, round: usize) -> u128 {
    if round % 2 == 1 {
        // Odd-indexed rounds: x^5
        field_p::pow5(x)
    } else {
        // Even-indexed rounds: x^d
        field_p::powd(x)
    }
}
