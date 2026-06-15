//! Permutation B (Pi_B)

use crate::state::Lane;
use crate::constants::{RC_B, SIGMA_B};
use crate::sbox_b;
use crate::mds_b;

pub fn pi_b(lanes: &mut [Lane; 128]) {
    for r in 0..32 {
        for i in 0..128 {
            lanes[i] = sbox_b::apply(lanes[i]);
        }

        mds_b::mix_lanes(lanes);

        let mut next = [Lane::new(0, 0); 128];
        for i in 0..128 {
            next[i] = lanes[SIGMA_B[i]];
        }

        for i in 0..128 {
            next[i].hi ^= RC_B[r][i].0;
            next[i].lo ^= RC_B[r][i].1;
        }

        *lanes = next;
    }
}
