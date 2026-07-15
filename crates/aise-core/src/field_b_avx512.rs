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
