#![allow(unused, unsafe_op_in_unsafe_fn)]
use std::arch::x86_64::*;
use aise_core::state::{Lane, State};
use aise_core::pi_b;
use aise_core::field_b;

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

    // prod_hi << 7
    let sh7_lo = _mm512_slli_epi64(prod_hi, 7);
    let sh7_hi_r = _mm512_srli_epi64(prod_hi, 64 - 7);
    let sh7_hi = _mm512_bslli_epi128(sh7_hi_r, 8);
    let sh7 = _mm512_xor_si512(sh7_lo, sh7_hi);
    let overflow7 = _mm512_srli_epi64(_mm512_bsrli_epi128(prod_hi, 8), 64 - 7);

    // prod_hi << 2
    let sh2_lo = _mm512_slli_epi64(prod_hi, 2);
    let sh2_hi_r = _mm512_srli_epi64(prod_hi, 64 - 2);
    let sh2_hi = _mm512_bslli_epi128(sh2_hi_r, 8);
    let sh2 = _mm512_xor_si512(sh2_lo, sh2_hi);
    let overflow2 = _mm512_srli_epi64(_mm512_bsrli_epi128(prod_hi, 8), 64 - 2);

    // prod_hi << 1
    let sh1_lo = _mm512_slli_epi64(prod_hi, 1);
    let sh1_hi_r = _mm512_srli_epi64(prod_hi, 64 - 1);
    let sh1_hi = _mm512_bslli_epi128(sh1_hi_r, 8);
    let sh1 = _mm512_xor_si512(sh1_lo, sh1_hi);
    let overflow1 = _mm512_srli_epi64(_mm512_bsrli_epi128(prod_hi, 8), 64 - 1);

    let red1 = _mm512_xor_si512(_mm512_xor_si512(sh7, sh2), _mm512_xor_si512(sh1, prod_hi));
    let t_lo = _mm512_xor_si512(prod_lo, red1);
    let overflow_sum = _mm512_xor_si512(_mm512_xor_si512(overflow7, overflow2), overflow1);

    // Second reduction
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

#[target_feature(enable = "avx512f,vpclmulqdq,avx512bw,avx512dq")]
pub unsafe fn v_sq(va: __m512i) -> __m512i {
    v_mul(va, va)
}

#[target_feature(enable = "avx512f,vpclmulqdq,avx512bw,avx512dq")]
pub unsafe fn v_inv(a: __m512i) -> __m512i {
    // 0 is not checked here because batch_inv ensures non-zero
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

#[target_feature(enable = "avx512f,vpclmulqdq,avx512bw,avx512dq")]
pub unsafe fn v_batch_inv(lanes: &mut [__m512i; 32]) {
    let mut masks = [_mm512_setzero_si512(); 32];
    let mut c = [_mm512_setzero_si512(); 32];
    
    let mut acc = _mm512_set_epi64(0, 1, 0, 1, 0, 1, 0, 1);
    
    let zero_vec = _mm512_setzero_si512();
    let one_vec = acc;

    for i in 0..32 {
        let v = lanes[i];
        
        // Find which 64-bit words are 0
        let m8 = _mm512_cmpeq_epi64_mask(v, zero_vec);
        // We want a mask where the entire 128-bit lane is 1s if BOTH 64-bit words were 0.
        // A 128-bit lane is words (2j+1, 2j).
        // Let's do this: swap hi and lo of each 128-bit lane.
        let swapped = _mm512_shuffle_epi32(v, 0b01_00_11_10); // Swap 64-bit halves
        let combined = _mm512_or_si512(v, swapped);
        // Now if a lane was 0, both 64-bit halves of `combined` are 0.
        let is_zero_mask8 = _mm512_cmpeq_epi64_mask(combined, zero_vec);
        
        let mask_vec = _mm512_maskz_set1_epi64(is_zero_mask8, -1i64);
        masks[i] = mask_vec;
        
        // If zero, substitute with 1.
        // a' = (v & ~mask) | (1 & mask)
        let a_prime = _mm512_ternarylogic_epi64(v, mask_vec, one_vec, 0xCA); // v ^ ((v ^ one) & mask) -> actually ternarylogic is better: (v & ~mask) | (one & mask) -> CA = (A & ~B) | (C & B)
        // Wait, ternary logic is (A & ~B) | (C & B). A=v, B=mask, C=one.
        // Let's use standard ops for simplicity first:
        let not_mask = _mm512_xor_si512(mask_vec, _mm512_set1_epi64(-1));
        let a_prime = _mm512_or_si512(_mm512_and_si512(v, not_mask), _mm512_and_si512(one_vec, mask_vec));
        
        acc = v_mul(acc, a_prime);
        c[i] = acc;
    }
    
    let mut inv_prod = v_inv(acc);
    
    for i in (1..32).rev() {
        let mask_vec = masks[i];
        let v = lanes[i];
        let not_mask = _mm512_xor_si512(mask_vec, _mm512_set1_epi64(-1));
        let a_prime = _mm512_or_si512(_mm512_and_si512(v, not_mask), _mm512_and_si512(one_vec, mask_vec));
        
        let a_inv = v_mul(inv_prod, c[i - 1]);
        inv_prod = v_mul(inv_prod, a_prime);
        
        lanes[i] = _mm512_and_si512(a_inv, not_mask);
    }
    
    let mask_vec = masks[0];
    let not_mask = _mm512_xor_si512(mask_vec, _mm512_set1_epi64(-1));
    lanes[0] = _mm512_and_si512(inv_prod, not_mask);
}

fn main() {
    println!("Testing equivalence...");
    let mut rand_state = 123456789u64;
    let mut next_rand = || {
        rand_state ^= rand_state << 13;
        rand_state ^= rand_state >> 17;
        rand_state ^= rand_state << 5;
        rand_state
    };

    println!("Testing v_mul...");
    let mut state1 = State::new();
    let mut state2 = State::new();
    for i in 0..128 {
        state1.lanes[i] = Lane::new(next_rand(), next_rand());
        state2.lanes[i] = Lane::new(next_rand(), next_rand());
    }
    
    let mut expected = State::new();
    for i in 0..128 {
        expected.lanes[i] = field_b::mul(state1.lanes[i], state2.lanes[i]);
    }
    
#[inline(always)]
unsafe fn swap_halves(v: __m512i) -> __m512i {
    _mm512_shuffle_epi32(v, 0b01_00_11_10) // 1, 0, 3, 2 for 32-bit blocks means swapping the two 64-bit halves in each 128-bit lane
}

    let mut actual = State::new();
    unsafe {
        let z1_ptr = state1.lanes.as_ptr() as *const __m512i;
        let z2_ptr = state2.lanes.as_ptr() as *const __m512i;
        let out_ptr = actual.lanes.as_mut_ptr() as *mut __m512i;
        
        let z1 = core::slice::from_raw_parts(z1_ptr, 32);
        let z2 = core::slice::from_raw_parts(z2_ptr, 32);
        let out = core::slice::from_raw_parts_mut(out_ptr, 32);
        
        for i in 0..32 {
            let va = swap_halves(z1[i]);
            let vb = swap_halves(z2[i]);
            let res = v_mul(va, vb);
            out[i] = swap_halves(res);
        }
    }
    
    // Test v_batch_inv
    let mut actual_batch = State::new();
    for i in 0..128 { actual_batch.lanes[i] = state1.lanes[i]; }
    
    field_b::batch_inv(&mut state1.lanes); // scalar baseline
    
    unsafe {
        let out_ptr = actual_batch.lanes.as_mut_ptr() as *mut __m512i;
        let out = core::slice::from_raw_parts_mut(out_ptr, 32);
        
        for i in 0..32 { out[i] = swap_halves(out[i]); }
        v_batch_inv(out.try_into().unwrap());
        for i in 0..32 { out[i] = swap_halves(out[i]); }
    }
    
    for i in 0..128 {
        assert_eq!(actual_batch.lanes[i], state1.lanes[i], "Mismatch at v_batch_inv index {}", i);
    }
    println!("AVX-512 Batch Inv perfectly matches scalar!");

    test_edge_cases();
    test_fuzz_pi_b();
    test_frozen_vector();
    test_avalanche();
}

fn rand_u64(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

fn test_fuzz_pi_b() {
    println!("Fuzzing Full Pi_B AVX-512 Fused Round vs Scalar (10,000 iterations)...");
    let iters = 10_000;
    let mut rand_state = 0x123456789ABCDEF0u64;
    
    for iter in 0..iters {
        let mut f_avx = [Lane::new(0, 0); 128];
        let mut f_scl = [Lane::new(0, 0); 128];
        for i in 0..128 {
            let l = Lane::new(rand_u64(&mut rand_state), rand_u64(&mut rand_state));
            f_avx[i] = l;
            f_scl[i] = l;
        }
        
        // Full 32-round AVX-512 Pi_B
        aise_core::pi_b::pi_b(&mut f_avx);
        
        // Full 32-round Scalar Pi_B
        for r in 0..32 {
            for i in 0..128 { f_scl[i] = aise_core::sbox_b::apply(f_scl[i]); }
            aise_core::mds_b::mix_lanes(&mut f_scl);
            
            let mut next = [Lane::new(0,0); 128];
            for i in 0..128 { next[i] = f_scl[aise_core::constants::SIGMA_B[i]]; }
            for i in 0..128 {
                next[i].hi ^= aise_core::constants::RC_B[r][i].0;
                next[i].lo ^= aise_core::constants::RC_B[r][i].1;
            }
            f_scl = next;
        }
        
        for i in 0..128 {
            assert_eq!(f_scl[i], f_avx[i], "Mismatch at iteration {}, index {}", iter, i);
        }
    }
    println!("Pi_B AVX-512 Fuzz: PASSED");
}

fn test_edge_cases() {
    println!("Testing Pi_B Edge Cases...");
    let cases = vec![
        Lane::new(0, 0),
        Lane::new(u64::MAX, u64::MAX),
        Lane::new(0x00000000000000FF, 0),
        Lane::new(0, 0xFF00000000000000),
    ];
    
    for &case in &cases {
        let mut f_avx = [case; 128];
        let mut f_scl = [case; 128];
        
        aise_core::pi_b::pi_b(&mut f_avx);
        
        for r in 0..32 {
            for i in 0..128 { f_scl[i] = aise_core::sbox_b::apply(f_scl[i]); }
            aise_core::mds_b::mix_lanes(&mut f_scl);
            let mut next = [Lane::new(0,0); 128];
            for i in 0..128 { next[i] = f_scl[aise_core::constants::SIGMA_B[i]]; }
            for i in 0..128 {
                next[i].hi ^= aise_core::constants::RC_B[r][i].0;
                next[i].lo ^= aise_core::constants::RC_B[r][i].1;
            }
            f_scl = next;
        }
        
        for i in 0..128 {
            assert_eq!(f_scl[i], f_avx[i], "Edge case mismatch at index {} for case {:?}", i, case);
        }
    }
    println!("Pi_B Edge Cases: PASSED");
}

fn test_frozen_vector() {
    println!("Testing Pi_B Frozen Vector...");
    let mut f_avx = [Lane::new(0, 0); 128];
    for i in 0..128 {
        f_avx[i] = Lane::new(i as u64, (127 - i) as u64);
    }
    aise_core::pi_b::pi_b(&mut f_avx);
    
    let mut sum_hi = 0u64;
    let mut sum_lo = 0u64;
    for i in 0..128 {
        sum_hi ^= f_avx[i].hi;
        sum_lo ^= f_avx[i].lo;
    }
    println!("Frozen Vector XOR Sum: {:016x} {:016x}", sum_hi, sum_lo);
}

fn test_avalanche() {
    println!("Testing Pi_B Avalanche...");
    let mut f_base = [Lane::new(0, 0); 128];
    let mut f_flip = [Lane::new(0, 0); 128];
    f_flip[0].hi = 1; // Flip 1 bit
    
    aise_core::pi_b::pi_b(&mut f_base);
    aise_core::pi_b::pi_b(&mut f_flip);
    
    let mut diff_bits = 0;
    for i in 0..128 {
        diff_bits += (f_base[i].hi ^ f_flip[i].hi).count_ones();
        diff_bits += (f_base[i].lo ^ f_flip[i].lo).count_ones();
    }
    println!("Avalanche bit flips: {} / 16384 ({:.2}%)", diff_bits, 100.0 * diff_bits as f64 / 16384.0);
}
