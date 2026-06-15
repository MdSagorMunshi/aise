//! Permutation C (Pi_C)

use crate::constants::{RC_C, SIGMA_C};
use crate::sbox_c;
use crate::mds_c;
use crate::field_p;

pub fn pi_c(f: &mut [u128; 128]) {
    for r in 0..32 {
        for i in 0..128 {
            f[i] = sbox_c::apply(f[i], r);
        }

        mds_c::mix_lanes(f);

        let mut next = [0u128; 128];
        for i in 0..128 {
            next[i] = f[SIGMA_C[i]];
        }

        for i in 0..128 {
            let rc = ((RC_C[r][i].0 as u128) << 64) | (RC_C[r][i].1 as u128);
            next[i] = field_p::add(next[i], rc);
        }

        *f = next;
    }
}
