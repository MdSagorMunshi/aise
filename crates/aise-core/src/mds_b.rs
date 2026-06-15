//! MixLanes_B

use crate::state::Lane;
use crate::field8;
use crate::field16;
use crate::constants::{M_COL, M_ROW};

pub fn mix_lanes(lanes: &mut [Lane; 128]) {
    // 1. Column mixing (M_COL)
    for col in 0..8 {
        let mut col_lanes = [Lane::new(0,0); 16];
        for row in 0..16 {
            col_lanes[row] = lanes[row * 8 + col];
        }

        let mut out_lanes = [Lane::new(0,0); 16];

        for b in 0..16 {
            let mut vec = [0u8; 16];
            for row in 0..16 {
                let lane = col_lanes[row];
                let byte = if b < 8 {
                    (lane.hi >> ((7 - b) * 8)) as u8
                } else {
                    (lane.lo >> ((15 - b) * 8)) as u8
                };
                vec[row] = byte;
            }

            let mut out_vec = [0u8; 16];
            for i in 0..16 {
                let mut sum = 0u8;
                for j in 0..16 {
                    sum ^= field8::mul(M_COL[i][j], vec[j]);
                }
                out_vec[i] = sum;
            }

            for row in 0..16 {
                let out_byte = out_vec[row] as u64;
                if b < 8 {
                    out_lanes[row].hi |= out_byte << ((7 - b) * 8);
                } else {
                    out_lanes[row].lo |= out_byte << ((15 - b) * 8);
                }
            }
        }

        for row in 0..16 {
            lanes[row * 8 + col] = out_lanes[row];
        }
    }

    // 2. Row mixing (M_ROW)
    for row in 0..16 {
        let mut row_lanes = [Lane::new(0,0); 8];
        for col in 0..8 {
            row_lanes[col] = lanes[row * 8 + col];
        }

        let mut out_lanes = [Lane::new(0,0); 8];

        for c in 0..8 {
            let mut vec = [0u16; 8];
            for col in 0..8 {
                let lane = row_lanes[col];
                let chunk = if c < 4 {
                    (lane.hi >> ((3 - c) * 16)) as u16
                } else {
                    (lane.lo >> ((7 - c) * 16)) as u16
                };
                vec[col] = chunk;
            }

            let mut out_vec = [0u16; 8];
            for i in 0..8 {
                let mut sum = 0u16;
                for j in 0..8 {
                    sum ^= field16::mul(M_ROW[i][j], vec[j]);
                }
                out_vec[i] = sum;
            }

            for col in 0..8 {
                let out_chunk = out_vec[col] as u64;
                if c < 4 {
                    out_lanes[col].hi |= out_chunk << ((3 - c) * 16);
                } else {
                    out_lanes[col].lo |= out_chunk << ((7 - c) * 16);
                }
            }
        }

        for col in 0..8 {
            lanes[row * 8 + col] = out_lanes[col];
        }
    }
}
