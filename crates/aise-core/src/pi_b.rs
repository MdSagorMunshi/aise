//! Permutation B (Pi_B)

use crate::state::Lane;
use crate::constants::{RC_B, SIGMA_B};
use crate::sbox_b;
use crate::mds_b;

pub fn pi_b(lanes: &mut [Lane; 128]) {
    for r in 0..32 {
        #[cfg(target_arch = "x86_64")]
        {
            if std::is_x86_feature_detected!("avx512f") 
                && std::is_x86_feature_detected!("vpclmulqdq") 
                && std::is_x86_feature_detected!("avx512bw") 
                && std::is_x86_feature_detected!("avx512dq") 
            {
                unsafe { crate::field_b_avx512::pi_b_round_avx512(lanes, r); }
                continue;
            }
        }
        
        sbox_b::batch_apply(lanes);

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
