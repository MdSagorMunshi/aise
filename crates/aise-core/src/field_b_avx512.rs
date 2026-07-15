#![allow(unsafe_op_in_unsafe_fn)]
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;
use crate::state::Lane;

#[inline(always)]
#[cfg(target_arch = "x86_64")]
pub unsafe fn swap_halves(v: __m512i) -> __m512i {
    _mm512_shuffle_epi32(v, 0b01_00_11_10) // Swap 64-bit halves in each 128-bit lane
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f,vpclmulqdq,avx512bw,avx512dq")]
pub unsafe fn v_mul(va: __m512i, vb: __m512i) -> __m512i {
    let t_ll = _mm512_clmulepi64_epi128(va, vb, 0x00);
    let t_lh = _mm512_clmulepi64_epi128(va, vb, 0x01);
    let t_hl = _mm512_clmulepi64_epi128(va, vb, 0x10);
    let t_hh = _mm512_clmulepi64_epi128(va, vb, 0x11);

    let mid = _mm512_xor_si512(t_lh, t_hl);
    let mid_lo = _mm512_bslli_epi128(mid, 8);
    let mid_hi = _mm512_bsrli_epi128(mid, 8);

    let prod_lo = _mm512_xor_si512(t_ll, mid_lo);
    let prod_hi = _mm512_xor_si512(t_hh, mid_hi);

    let sh7_lo = _mm512_slli_epi64(prod_hi, 7);
    let sh7_hi_r = _mm512_srli_epi64(prod_hi, 64 - 7);
    let sh7_hi = _mm512_bslli_epi128(sh7_hi_r, 8);
    let sh7 = _mm512_xor_si512(sh7_lo, sh7_hi);
    let overflow7 = _mm512_srli_epi64(_mm512_bsrli_epi128(prod_hi, 8), 64 - 7);

    let sh2_lo = _mm512_slli_epi64(prod_hi, 2);
    let sh2_hi_r = _mm512_srli_epi64(prod_hi, 64 - 2);
    let sh2_hi = _mm512_bslli_epi128(sh2_hi_r, 8);
    let sh2 = _mm512_xor_si512(sh2_lo, sh2_hi);
    let overflow2 = _mm512_srli_epi64(_mm512_bsrli_epi128(prod_hi, 8), 64 - 2);

    let sh1_lo = _mm512_slli_epi64(prod_hi, 1);
    let sh1_hi_r = _mm512_srli_epi64(prod_hi, 64 - 1);
    let sh1_hi = _mm512_bslli_epi128(sh1_hi_r, 8);
    let sh1 = _mm512_xor_si512(sh1_lo, sh1_hi);
    let overflow1 = _mm512_srli_epi64(_mm512_bsrli_epi128(prod_hi, 8), 64 - 1);

    let red1 = _mm512_xor_si512(_mm512_xor_si512(sh7, sh2), _mm512_xor_si512(sh1, prod_hi));
    let t_lo = _mm512_xor_si512(prod_lo, red1);
    let overflow_sum = _mm512_xor_si512(_mm512_xor_si512(overflow7, overflow2), overflow1);

    let o7_lo = _mm512_slli_epi64(overflow_sum, 7);
    let o7_hi_r = _mm512_srli_epi64(overflow_sum, 64 - 7);
    let o7_hi = _mm512_bslli_epi128(o7_hi_r, 8);
    let o7 = _mm512_xor_si512(o7_lo, o7_hi);

    let o2_lo = _mm512_slli_epi64(overflow_sum, 2);
    let o2_hi_r = _mm512_srli_epi64(overflow_sum, 64 - 2);
    let o2_hi = _mm512_bslli_epi128(o2_hi_r, 8);
    let o2 = _mm512_xor_si512(o2_lo, o2_hi);

    let o1_lo = _mm512_slli_epi64(overflow_sum, 1);
    let o1_hi_r = _mm512_srli_epi64(overflow_sum, 64 - 1);
    let o1_hi = _mm512_bslli_epi128(o1_hi_r, 8);
    let o1 = _mm512_xor_si512(o1_lo, o1_hi);

    let red2 = _mm512_xor_si512(_mm512_xor_si512(o7, o2), _mm512_xor_si512(o1, overflow_sum));
    
    _mm512_xor_si512(t_lo, red2)
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f,vpclmulqdq,avx512bw,avx512dq")]
pub unsafe fn v_sq(va: __m512i) -> __m512i {
    v_mul(va, va)
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f,vpclmulqdq,avx512bw,avx512dq")]
pub unsafe fn v_inv(a: __m512i) -> __m512i {
    let a2 = v_sq(a);
    let t_2 = v_mul(a2, a);
    
    let mut t_4 = t_2;
    for _ in 0..2 { t_4 = v_sq(t_4); }
    t_4 = v_mul(t_4, t_2);

    let mut t_8 = t_4;
    for _ in 0..4 { t_8 = v_sq(t_8); }
    t_8 = v_mul(t_8, t_4);

    let mut t_16 = t_8;
    for _ in 0..8 { t_16 = v_sq(t_16); }
    t_16 = v_mul(t_16, t_8);

    let mut t_32 = t_16;
    for _ in 0..16 { t_32 = v_sq(t_32); }
    t_32 = v_mul(t_32, t_16);

    let mut t_64 = t_32;
    for _ in 0..32 { t_64 = v_sq(t_64); }
    t_64 = v_mul(t_64, t_32);

    let mut inv_base = t_64;
    for _ in 0..32 { inv_base = v_sq(inv_base); }
    inv_base = v_mul(inv_base, t_32);

    for _ in 0..16 { inv_base = v_sq(inv_base); }
    inv_base = v_mul(inv_base, t_16);

    for _ in 0..8 { inv_base = v_sq(inv_base); }
    inv_base = v_mul(inv_base, t_8);

    for _ in 0..4 { inv_base = v_sq(inv_base); }
    inv_base = v_mul(inv_base, t_4);

    for _ in 0..2 { inv_base = v_sq(inv_base); }
    inv_base = v_mul(inv_base, t_2);

    inv_base = v_sq(inv_base);
    inv_base = v_mul(inv_base, a);
    
    v_sq(inv_base)
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f,vpclmulqdq,avx512bw,avx512dq")]
pub unsafe fn batch_inv_avx512(lanes: &mut [Lane; 128]) {
    let ptr = lanes.as_mut_ptr() as *mut i8;
    
    let mut masks = [_mm512_setzero_si512(); 32];
    let mut c = [_mm512_setzero_si512(); 32];
    
    let one_vec = _mm512_set_epi64(0, 1, 0, 1, 0, 1, 0, 1); 
    let zero_vec = _mm512_setzero_si512();
    let mut acc = one_vec;

    for i in 0..32 {
        let v_raw = _mm512_loadu_si512(ptr.add(i * 64) as *const _);
        let v = swap_halves(v_raw);
        
        let swapped = _mm512_shuffle_epi32(v, 0b01_00_11_10);
        let combined = _mm512_or_si512(v, swapped);
        let is_zero_mask8 = _mm512_cmpeq_epi64_mask(combined, zero_vec);
        
        let mask_vec = _mm512_maskz_set1_epi64(is_zero_mask8, -1i64);
        masks[i] = mask_vec;
        
        let not_mask = _mm512_xor_si512(mask_vec, _mm512_set1_epi64(-1));
        let a_prime = _mm512_or_si512(_mm512_and_si512(v, not_mask), _mm512_and_si512(one_vec, mask_vec));
        
        acc = v_mul(acc, a_prime);
        c[i] = acc;
    }
    
    let mut inv_prod = v_inv(acc);
    
    for i in (1..32).rev() {
        let mask_vec = masks[i];
        let v_raw = _mm512_loadu_si512(ptr.add(i * 64) as *const _);
        let v = swap_halves(v_raw);
        
        let not_mask = _mm512_xor_si512(mask_vec, _mm512_set1_epi64(-1));
        let a_prime = _mm512_or_si512(_mm512_and_si512(v, not_mask), _mm512_and_si512(one_vec, mask_vec));
        
        let a_inv = v_mul(inv_prod, c[i - 1]);
        inv_prod = v_mul(inv_prod, a_prime);
        
        let res = _mm512_and_si512(a_inv, not_mask);
        _mm512_storeu_si512(ptr.add(i * 64) as *mut _, swap_halves(res));
    }
    
    let mask_vec = masks[0];
    let not_mask = _mm512_xor_si512(mask_vec, _mm512_set1_epi64(-1));
    let res0 = _mm512_and_si512(inv_prod, not_mask);
    _mm512_storeu_si512(ptr as *mut _, swap_halves(res0));
}

#[cfg(target_arch = "x86_64")]
pub mod tables {
    include!("field_b_avx512_tables.rs");
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f,avx512bw,avx512dq")]
pub unsafe fn mix_cols_avx512(lanes: &[Lane; 128], next_lanes: &mut [Lane; 128]) {
    let mask0f = _mm512_set1_epi8(0x0F);
    
    // We process 4 columns at a time. half=0 for cols 0..3, half=1 for cols 4..7
    for half in 0..2 {
        let mut out = [_mm512_setzero_si512(); 16];
        
        // Accumulate over j (the 16 elements of the column)
        for j in 0..16 {
            let v = _mm512_loadu_si512(lanes.as_ptr().add(j * 8 + half * 4) as *const _);
            let lo = _mm512_and_si512(v, mask0f);
            let hi = _mm512_and_si512(_mm512_srli_epi16(v, 4), mask0f);
            
            for i in 0..16 {
                let t_lo = _mm512_broadcast_i32x4(std::mem::transmute(tables::M_COL_T_LO[i][j]));
                let t_hi = _mm512_broadcast_i32x4(std::mem::transmute(tables::M_COL_T_HI[i][j]));
                
                let p_lo = _mm512_shuffle_epi8(t_lo, lo);
                let p_hi = _mm512_shuffle_epi8(t_hi, hi);
                
                out[i] = _mm512_xor_si512(out[i], _mm512_xor_si512(p_lo, p_hi));
            }
        }
        
        for i in 0..16 {
            _mm512_storeu_si512(next_lanes.as_mut_ptr().add(i * 8 + half * 4) as *mut _, out[i]);
        }
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f,avx512bw,avx512dq")]
pub unsafe fn mix_rows_avx512(lanes: &[Lane; 128], next_lanes: &mut [Lane; 128]) {
    let mask0f = _mm512_set1_epi16(0x0F);
    
    let offset_01 = _mm512_set_epi16(
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
        16, 16, 16, 16, 16, 16, 16, 16,
        0, 0, 0, 0, 0, 0, 0, 0
    );
    let mask_01 = _mm512_set_epi16(
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
        -1, -1, -1, -1, -1, -1, -1, -1,
        -1, -1, -1, -1, -1, -1, -1, -1,
    );
    
    let offset_23 = _mm512_set_epi16(
        16, 16, 16, 16, 16, 16, 16, 16,
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0
    );
    let mask_23 = _mm512_set_epi16(
        -1, -1, -1, -1, -1, -1, -1, -1,
        -1, -1, -1, -1, -1, -1, -1, -1,
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
    );
    
    for row in 0..16 {
        let v0 = _mm512_loadu_si512(lanes.as_ptr().add(row * 8) as *const _);
        let v1 = _mm512_loadu_si512(lanes.as_ptr().add(row * 8 + 4) as *const _);
        
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
            
            for pair in 0..4 {
                let k0 = pair * 2;
                let k1 = pair * 2 + 1;
                
                for nib in 0..4 {
                    let mut t01 = [0u16; 32];
                    t01[0..16].copy_from_slice(&tables::M_ROW_T[i][k0][nib]);
                    t01[16..32].copy_from_slice(&tables::M_ROW_T[i][k1][nib]);
                    let table01 = _mm512_loadu_si512(t01.as_ptr() as *const _);
                    
                    if pair < 2 {
                        let n = match nib { 0 => n0_0, 1 => n0_1, 2 => n0_2, _ => n0_3 };
                        let idx = if pair == 0 {
                            _mm512_add_epi16(n, offset_01)
                        } else {
                            _mm512_add_epi16(n, offset_23)
                        };
                        let p = _mm512_permutexvar_epi16(idx, table01);
                        let p_masked = _mm512_and_si512(p, if pair == 0 { mask_01 } else { mask_23 });
                        sum0 = _mm512_xor_si512(sum0, p_masked);
                    } else {
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
            
            let sum_all = _mm512_xor_si512(sum0, sum1);
            let s_23_01 = _mm512_shuffle_i32x4(sum_all, sum_all, 0b01_00_11_10);
            let s_folded = _mm512_xor_si512(sum_all, s_23_01);
            let s_10_32 = _mm512_shuffle_i32x4(s_folded, s_folded, 0b10_11_00_01);
            let final_lane = _mm512_xor_si512(s_folded, s_10_32);
            
            let mut out_lane = [Lane::new(0,0)];
            _mm_storeu_si128(out_lane.as_mut_ptr() as *mut _, _mm512_castsi512_si128(final_lane));
            
            next_lanes[row * 8 + i] = out_lane[0];
        }
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f,vpclmulqdq,avx512bw,avx512dq")]
pub unsafe fn pi_b_round_avx512(f: &mut [Lane; 128], r: usize) {
    batch_inv_avx512(f);
    
    let mut temp = [Lane::new(0, 0); 128];
    mix_cols_avx512(f, &mut temp);
    
    let mut temp2 = [Lane::new(0, 0); 128];
    mix_rows_avx512(&temp, &mut temp2);
    
    // Scalar Affine + SIGMA_B finish
    for i in 0..128 {
        let mut val = temp2[crate::constants::SIGMA_B[i]];
        val.hi ^= crate::constants::RC_B[r][i].0;
        val.lo ^= crate::constants::RC_B[r][i].1;
        f[i] = val;
    }
}

