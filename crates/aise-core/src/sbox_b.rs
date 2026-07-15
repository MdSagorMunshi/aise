//! S-Box for GF(2^128)

use crate::state::Lane;
use crate::field_b;

#[inline(always)]
pub fn apply(x: Lane) -> Lane {
    field_b::inv(x)
}

#[inline(always)]
pub fn batch_apply(lanes: &mut [Lane; 128]) {
    field_b::batch_inv(lanes);
}
