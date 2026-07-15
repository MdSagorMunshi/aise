#![allow(unsafe_op_in_unsafe_fn)]
use std::arch::x86_64::*;
use aise_core::state::{Lane, State};
use aise_core::constants::{M_COL, M_ROW};
use aise_core::mds_b;

// Include the generated tables
include!("../../../aise-core/src/field_b_avx512_tables.rs");

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f,avx512bw,avx512dq")]
pub unsafe fn mix_cols_avx512(lanes: &mut [Lane; 128]) {
    let mask0f = _mm512_set1_epi8(0x0F);
    let mut next_lanes = [Lane::new(0, 0); 128];
    
    // We process 4 columns at a time. half=0 for cols 0..3, half=1 for cols 4..7
    for half in 0..2 {
        let mut out = [_mm512_setzero_si512(); 16];
        
        // Accumulate over j (the 16 elements of the column)
        for j in 0..16 {
            let v = _mm512_loadu_si512(lanes.as_ptr().add(j * 8 + half * 4) as *const _);
            let lo = _mm512_and_si512(v, mask0f);
            let hi = _mm512_and_si512(_mm512_srli_epi16(v, 4), mask0f);
            
            for i in 0..16 {
                // Broadcast the 16-byte lookup tables to all 4 sublanes
                let t_lo = _mm512_broadcast_i32x4(std::mem::transmute(M_COL_T_LO[i][j]));
                let t_hi = _mm512_broadcast_i32x4(std::mem::transmute(M_COL_T_HI[i][j]));
                
                let p_lo = _mm512_shuffle_epi8(t_lo, lo);
                let p_hi = _mm512_shuffle_epi8(t_hi, hi);
                
                out[i] = _mm512_xor_si512(out[i], _mm512_xor_si512(p_lo, p_hi));
            }
        }
        
        for i in 0..16 {
            _mm512_storeu_si512(next_lanes.as_mut_ptr().add(i * 8 + half * 4) as *mut _, out[i]);
        }
    }
    *lanes = next_lanes;
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f,avx512bw,avx512dq")]
pub unsafe fn mix_rows_avx512(lanes: &mut [Lane; 128]) {
    let mask0f = _mm512_set1_epi16(0x0F);
    
    // Offsets to select table 0 or table 1 within a 32-word vpermw table
    // Sublanes: 0, 1, 2, 3 (each 8 words)
    // For pass 0 (cols 0, 1): sublane 0 gets offset 0, sublane 1 gets offset 16. Sublanes 2,3 zeroed.
    let offset_01 = _mm512_set_epi16(
        0, 0, 0, 0, 0, 0, 0, 0,    // sublane 3 (unused)
        0, 0, 0, 0, 0, 0, 0, 0,    // sublane 2 (unused)
        16, 16, 16, 16, 16, 16, 16, 16, // sublane 1 (offset 16 for table 1)
        0, 0, 0, 0, 0, 0, 0, 0     // sublane 0 (offset 0 for table 0)
    );
    let mask_01 = _mm512_set_epi16(
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
        -1, -1, -1, -1, -1, -1, -1, -1,
        -1, -1, -1, -1, -1, -1, -1, -1,
    );
    
    // For pass 1 (cols 2, 3): sublane 2 gets offset 0, sublane 3 gets offset 16. Sublanes 0,1 zeroed.
    let offset_23 = _mm512_set_epi16(
        16, 16, 16, 16, 16, 16, 16, 16, // sublane 3 (offset 16)
        0, 0, 0, 0, 0, 0, 0, 0,    // sublane 2 (offset 0)
        0, 0, 0, 0, 0, 0, 0, 0,    // sublane 1 (unused)
        0, 0, 0, 0, 0, 0, 0, 0     // sublane 0 (unused)
    );
    let mask_23 = _mm512_set_epi16(
        -1, -1, -1, -1, -1, -1, -1, -1,
        -1, -1, -1, -1, -1, -1, -1, -1,
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
    );
    
    let mut next_lanes = [Lane::new(0, 0); 128];
    
    // We process 1 row at a time.
    for row in 0..16 {
        // v0 contains cols 0..3, v1 contains cols 4..7
        let v0_raw = _mm512_loadu_si512(lanes.as_ptr().add(row * 8) as *const _);
        let v1_raw = _mm512_loadu_si512(lanes.as_ptr().add(row * 8 + 4) as *const _);
        
        // BE conversion: M_ROW assumes 16-bit chunks are Big-Endian.
        // We can just byte-swap the 16-bit chunks before processing!
        // wait, vpermw can operate on Little-Endian, but the constants in M_ROW_T might be LE?
        // Let's swap bytes in each 16-bit word so they are LE, compute, then swap back.
        let v0 = v0_raw;
        let v1 = v1_raw;

        let n0_0 = _mm512_and_si512(v0, mask0f);
        let n0_1 = _mm512_and_si512(_mm512_srli_epi16(v0, 4), mask0f);
        let n0_2 = _mm512_and_si512(_mm512_srli_epi16(v0, 8), mask0f);
        let n0_3 = _mm512_srli_epi16(v0, 12);
        
        let n1_0 = _mm512_and_si512(v1, mask0f);
        let n1_1 = _mm512_and_si512(_mm512_srli_epi16(v1, 4), mask0f);
        let n1_2 = _mm512_and_si512(_mm512_srli_epi16(v1, 8), mask0f);
        let n1_3 = _mm512_srli_epi16(v1, 12);
        
        for i in 0..8 {
            let mut sum0 = _mm512_setzero_si512();
            let mut sum1 = _mm512_setzero_si512();
            
            // Iterate over the 4 pairs of columns
            for pair in 0..4 {
                let k0 = pair * 2;
                let k1 = pair * 2 + 1;
                
                for nib in 0..4 {
                    // table01 contains T[k0] at offset 0, T[k1] at offset 16
                    let mut t01 = [0u16; 32];
                    t01[0..16].copy_from_slice(&M_ROW_T[i][k0][nib]);
                    t01[16..32].copy_from_slice(&M_ROW_T[i][k1][nib]);
                    let table01 = _mm512_loadu_si512(t01.as_ptr() as *const _);
                    
                    if pair < 2 { // Cols 0..3 (in v0)
                        let n = match nib { 0 => n0_0, 1 => n0_1, 2 => n0_2, _ => n0_3 };
                        let idx = if pair == 0 {
                            _mm512_add_epi16(n, offset_01)
                        } else {
                            _mm512_add_epi16(n, offset_23)
                        };
                        let p = _mm512_permutexvar_epi16(idx, table01);
                        let p_masked = _mm512_and_si512(p, if pair == 0 { mask_01 } else { mask_23 });
                        sum0 = _mm512_xor_si512(sum0, p_masked);
                    } else { // Cols 4..7 (in v1)
                        let n = match nib { 0 => n1_0, 1 => n1_1, 2 => n1_2, _ => n1_3 };
                        let idx = if pair == 2 {
                            _mm512_add_epi16(n, offset_01)
                        } else {
                            _mm512_add_epi16(n, offset_23)
                        };
                        let p = _mm512_permutexvar_epi16(idx, table01);
                        let p_masked = _mm512_and_si512(p, if pair == 2 { mask_01 } else { mask_23 });
                        sum1 = _mm512_xor_si512(sum1, p_masked);
                    }
                }
            }
            
            // sum0 contains the output for cols 0..3 (for row `i`? No, wait)
            // Wait! The scalar code computes out_col_i. 
            // So sum0 (cols 0..3) + sum1 (cols 4..7) is the total sum for output column `i`!
            // But sum0 and sum1 are vectors of 4 sublanes!
            // We need to horizontally add the 4 sublanes of `sum0 ^ sum1` to get the final `Lane` for `out[row][i]`.
            // Ah! The output of the multiplication is spread across the sublanes!
            // Because col k was multiplied by M_ROW[i][k], and left in sublane k.
            // To sum them, we need to XOR the 4 sublanes together.
            let sum_all = _mm512_xor_si512(sum0, sum1); // still 4 sublanes
            
            // Horizontally XOR the 4 128-bit sublanes to produce one 128-bit Lane
            let s_23_01 = _mm512_shuffle_i32x4(sum_all, sum_all, 0b01_00_11_10); // swap 256-bit halves
            let s_folded = _mm512_xor_si512(sum_all, s_23_01); // sublane 0 has sub0^sub2, sublane 1 has sub1^sub3
            let s_10_32 = _mm512_shuffle_i32x4(s_folded, s_folded, 0b10_11_00_01); // swap 128-bit halves
            let final_lane_be = _mm512_xor_si512(s_folded, s_10_32);
            
            let final_lane = final_lane_be;
            
            let mut out_lane = [Lane::new(0,0)];
            // Store just the low 128 bits
            _mm_storeu_si128(out_lane.as_mut_ptr() as *mut _, _mm512_castsi512_si128(final_lane));
            
            next_lanes[row * 8 + i] = out_lane[0];
        }
    }
    
    *lanes = next_lanes;
}

fn test_col_mix() {
    println!("Testing Column Mix AVX-512...");
    let mut lanes_scalar = [Lane::new(0, 0); 128];
    let mut lanes_avx = [Lane::new(0, 0); 128];
    for i in 0..128 {
        let l = Lane::new(i as u64 * 13, i as u64 * 17 + 1);
        lanes_scalar[i] = l;
        lanes_avx[i] = l;
    }
    
    // Perform column mix scalar (extracting from mix_lanes)
    for col in 0..8 {
        let mut col_lanes = [Lane::new(0,0); 16];
        for row in 0..16 { col_lanes[row] = lanes_scalar[row * 8 + col]; }
        let mut out_lanes = [Lane::new(0,0); 16];
        for b in 0..16 {
            let mut vec = [0u8; 16];
            for row in 0..16 {
                let lane = col_lanes[row];
                vec[row] = if b < 8 { (lane.hi >> ((7 - b) * 8)) as u8 } else { (lane.lo >> ((15 - b) * 8)) as u8 };
            }
            let mut out_vec = [0u8; 16];
            for i in 0..16 {
                for j in 0..16 { out_vec[i] ^= aise_core::field8::mul(M_COL[i][j], vec[j]); }
            }
            for row in 0..16 {
                let out_byte = out_vec[row] as u64;
                if b < 8 { out_lanes[row].hi |= out_byte << ((7 - b) * 8); } else { out_lanes[row].lo |= out_byte << ((15 - b) * 8); }
            }
        }
        for row in 0..16 { lanes_scalar[row * 8 + col] = out_lanes[row]; }
    }
    
    unsafe { mix_cols_avx512(&mut lanes_avx); }
    
    for i in 0..128 {
        assert_eq!(lanes_scalar[i], lanes_avx[i], "Col mix mismatch at index {}", i);
    }
    println!("Column Mix: PASSED");
}

fn test_row_mix() {
    println!("Testing Row Mix AVX-512...");
    let mut lanes_scalar = [Lane::new(0, 0); 128];
    let mut lanes_avx = [Lane::new(0, 0); 128];
    for i in 0..128 {
        let l = Lane::new(i as u64 * 31, i as u64 * 7 + 3);
        lanes_scalar[i] = l;
        lanes_avx[i] = l;
    }
    
    // Perform row mix scalar
    for row in 0..16 {
        let mut row_lanes = [Lane::new(0,0); 8];
        for col in 0..8 { row_lanes[col] = lanes_scalar[row * 8 + col]; }
        let mut out_lanes = [Lane::new(0,0); 8];
        for c in 0..8 {
            let mut vec = [0u16; 8];
            for col in 0..8 {
                let lane = row_lanes[col];
                vec[col] = if c < 4 { (lane.hi >> ((3 - c) * 16)) as u16 } else { (lane.lo >> ((7 - c) * 16)) as u16 };
            }
            let mut out_vec = [0u16; 8];
            for i in 0..8 {
                for j in 0..8 { out_vec[i] ^= aise_core::field16::mul(M_ROW[i][j], vec[j]); }
            }
            for col in 0..8 {
                let out_chunk = out_vec[col] as u64;
                if c < 4 { out_lanes[col].hi |= out_chunk << ((3 - c) * 16); } else { out_lanes[col].lo |= out_chunk << ((7 - c) * 16); }
            }
        }
        for col in 0..8 { lanes_scalar[row * 8 + col] = out_lanes[col]; }
    }
    
    unsafe { mix_rows_avx512(&mut lanes_avx); }
    
    for i in 0..128 {
        assert_eq!(lanes_scalar[i], lanes_avx[i], "Row mix mismatch at index {}", i);
    }
    println!("Row Mix: PASSED");
}

fn main() {
    if std::is_x86_feature_detected!("avx512bw") {
        test_col_mix();
        test_row_mix();
    }
}
