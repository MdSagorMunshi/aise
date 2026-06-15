//! MixLanes_C

use crate::field_p;
use crate::constants::{M_COL_P, M_ROW_P};

pub fn mix_lanes(f: &mut [u128; 128]) {
    // 1. Column mixing (M_COL_P)
    for col in 0..8 {
        let mut vec = [0u128; 16];
        for row in 0..16 {
            vec[row] = f[row * 8 + col];
        }

        let mut out_vec = [0u128; 16];
        for i in 0..16 {
            let mut sum = 0u128;
            for j in 0..16 {
                let mat_val = ((M_COL_P[i][j].0 as u128) << 64) | (M_COL_P[i][j].1 as u128);
                let prod = field_p::mul(mat_val, vec[j]);
                sum = field_p::add(sum, prod);
            }
            out_vec[i] = sum;
        }

        for row in 0..16 {
            f[row * 8 + col] = out_vec[row];
        }
    }

    // 2. Row mixing (M_ROW_P)
    for row in 0..16 {
        let mut vec = [0u128; 8];
        for col in 0..8 {
            vec[col] = f[row * 8 + col];
        }

        let mut out_vec = [0u128; 8];
        for i in 0..8 {
            let mut sum = 0u128;
            for j in 0..8 {
                let mat_val = ((M_ROW_P[i][j].0 as u128) << 64) | (M_ROW_P[i][j].1 as u128);
                let prod = field_p::mul(mat_val, vec[j]);
                sum = field_p::add(sum, prod);
            }
            out_vec[i] = sum;
        }

        for col in 0..8 {
            f[row * 8 + col] = out_vec[col];
        }
    }
}
