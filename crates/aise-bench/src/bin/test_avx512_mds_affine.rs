#![allow(unsafe_op_in_unsafe_fn)]
use aise_core::state::Lane;
use aise_core::constants::{M_ROW_P, M_COL_P, RC_C, SIGMA_C};
use aise_core::field_p;
use std::arch::x86_64::*;

// We use a custom XorShift to avoid rand dependencies in the bench crate if it's missing.
struct XorShift {
    state: u64,
}

impl XorShift {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }
    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }
    fn next_u128(&mut self) -> u128 {
        ((self.next_u64() as u128) << 64) | (self.next_u64() as u128)
    }
}

#[inline]
#[target_feature(enable = "avx512f,avx512dq,avx512ifma")]
unsafe fn v_mac_ifma(
    mut z0: __m512i, mut z1: __m512i, mut z2: __m512i, mut z3: __m512i, mut z4: __m512i,
    a0: __m512i, a1: __m512i, a2: __m512i, 
    b0: __m512i, b1: __m512i, b2: __m512i
) -> (__m512i, __m512i, __m512i, __m512i, __m512i) {
    z0 = _mm512_madd52lo_epu64(z0, a0, b0);
    z1 = _mm512_madd52hi_epu64(z1, a0, b0);
    z1 = _mm512_madd52lo_epu64(z1, a0, b1);
    z1 = _mm512_madd52lo_epu64(z1, a1, b0);
    z2 = _mm512_madd52hi_epu64(z2, a0, b1);
    z2 = _mm512_madd52hi_epu64(z2, a1, b0);
    z2 = _mm512_madd52lo_epu64(z2, a0, b2);
    z2 = _mm512_madd52lo_epu64(z2, a1, b1);
    z2 = _mm512_madd52lo_epu64(z2, a2, b0);
    z3 = _mm512_madd52hi_epu64(z3, a0, b2);
    z3 = _mm512_madd52hi_epu64(z3, a1, b1);
    z3 = _mm512_madd52hi_epu64(z3, a2, b0);
    z3 = _mm512_madd52lo_epu64(z3, a1, b2);
    z3 = _mm512_madd52lo_epu64(z3, a2, b1);
    z4 = _mm512_madd52hi_epu64(z4, a1, b2);
    z4 = _mm512_madd52hi_epu64(z4, a2, b1);
    z4 = _mm512_madd52lo_epu64(z4, a2, b2);
    (z0, z1, z2, z3, z4)
}

#[inline]
#[target_feature(enable = "avx512f,avx512dq,avx512ifma")]
unsafe fn v_reduce_ifma(z0: __m512i, z1: __m512i, z2: __m512i, z3: __m512i, z4: __m512i) -> (__m512i, __m512i, __m512i) {
    let m52 = _mm512_set1_epi64(0xFFFFFFFFFFFFF);
    let c0 = z0;
    
    let mut c1 = z1;
    let c1_carry = _mm512_srli_epi64(c1, 52);
    c1 = _mm512_and_si512(c1, m52);

    let mut c2 = _mm512_add_epi64(z2, c1_carry);
    let c2_carry = _mm512_srli_epi64(c2, 52);
    c2 = _mm512_and_si512(c2, m52);

    let mut c3 = _mm512_add_epi64(z3, c2_carry);
    let c3_carry = _mm512_srli_epi64(c3, 52);
    c3 = _mm512_and_si512(c3, m52);

    let c4 = _mm512_add_epi64(z4, c3_carry);

    let m23 = _mm512_set1_epi64(0x7FFFFF);

    let c2_lo = _mm512_and_si512(c2, m23);
    let c2_hi = _mm512_srli_epi64(c2, 23);
    let c3_lo = _mm512_and_si512(c3, m23);
    let c3_hi = _mm512_srli_epi64(c3, 23);
    let c4_lo = _mm512_and_si512(c4, m23);
    let c4_hi = _mm512_srli_epi64(c4, 23);

    let h0 = _mm512_or_si512(c2_hi, _mm512_slli_epi64(c3_lo, 29));
    let h1 = _mm512_or_si512(c3_hi, _mm512_slli_epi64(c4_lo, 29));
    let h2 = c4_hi;

    let mut out0 = _mm512_add_epi64(c0, h0);
    let mut out1 = _mm512_add_epi64(c1, h1);
    let mut out2 = _mm512_add_epi64(c2_lo, h2);

    out1 = _mm512_add_epi64(out1, _mm512_srli_epi64(out0, 52));
    out0 = _mm512_and_si512(out0, m52);

    out2 = _mm512_add_epi64(out2, _mm512_srli_epi64(out1, 52));
    out1 = _mm512_and_si512(out1, m52);

    let carry = _mm512_srli_epi64(out2, 23);
    out2 = _mm512_and_si512(out2, m23);
    out0 = _mm512_add_epi64(out0, carry);

    let is_p0 = _mm512_cmpeq_epi64_mask(out0, m52);
    let is_p1 = _mm512_cmpeq_epi64_mask(out1, m52);
    let is_p2 = _mm512_cmpeq_epi64_mask(out2, m23);
    
    let is_p = is_p0 & is_p1 & is_p2;
    let keep_mask = !is_p;
    
    out0 = _mm512_maskz_mov_epi64(keep_mask, out0);
    out1 = _mm512_maskz_mov_epi64(keep_mask, out1);
    out2 = _mm512_maskz_mov_epi64(keep_mask, out2);

    (out0, out1, out2)
}

#[inline]
unsafe fn pack_52(l0: __m512i, l1: __m512i) -> (__m512i, __m512i, __m512i) {
    let idx_lo = _mm512_set_epi64(14, 12, 10, 8, 6, 4, 2, 0);
    let lo = _mm512_permutex2var_epi64(l0, idx_lo, l1);

    let idx_hi = _mm512_set_epi64(15, 13, 11, 9, 7, 5, 3, 1);
    let hi = _mm512_permutex2var_epi64(l0, idx_hi, l1);

    let m52 = _mm512_set1_epi64(0xFFFFFFFFFFFFF);
    let m40 = _mm512_set1_epi64(0xFFFFFFFFFF);

    let a0 = _mm512_and_si512(lo, m52);
    let a1_part1 = _mm512_srli_epi64(lo, 52);
    let a1_part2 = _mm512_slli_epi64(_mm512_and_si512(hi, m40), 12);
    let a1 = _mm512_or_si512(a1_part1, a1_part2);
    let a2 = _mm512_srli_epi64(hi, 40);

    (a0, a1, a2)
}

#[inline]
unsafe fn unpack_52(a0: __m512i, a1: __m512i, a2: __m512i) -> (__m512i, __m512i) {
    let lo = _mm512_or_si512(a0, _mm512_slli_epi64(a1, 52));
    let hi = _mm512_or_si512(_mm512_srli_epi64(a1, 12), _mm512_slli_epi64(a2, 40));

    let idx_l0 = _mm512_set_epi64(11, 3, 10, 2, 9, 1, 8, 0);
    let l0_out = _mm512_permutex2var_epi64(lo, idx_l0, hi);

    let idx_l1 = _mm512_set_epi64(15, 7, 14, 6, 13, 5, 12, 4);
    let l1_out = _mm512_permutex2var_epi64(lo, idx_l1, hi);

    (l0_out, l1_out)
}

#[inline]
unsafe fn scalar_to_limbs(val: u128) -> (u64, u64, u64) {
    let a0 = (val & 0xFFFFFFFFFFFFF) as u64;
    let a1 = ((val >> 52) & 0xFFFFFFFFFFFFF) as u64;
    let a2 = ((val >> 104) & 0x7FFFFF) as u64;
    (a0, a1, a2)
}

#[inline]
#[target_feature(enable = "avx512f,avx512dq,avx512ifma")]
unsafe fn mds_affine_fused(f: &mut [u128; 128], r: usize) {
    // 1. Column mixing: tiled over output batches
    let mut out_col = [(_mm512_setzero_si512(), _mm512_setzero_si512(), _mm512_setzero_si512()); 16];
    
    // We can pre-pack all 16 input batches since they fit in memory or arrays easily.
    let mut in_batches = [(_mm512_setzero_si512(), _mm512_setzero_si512(), _mm512_setzero_si512()); 16];
    let ptr = f.as_ptr() as *const i8;
    for i in 0..16 {
        let l0 = _mm512_loadu_si512(ptr.add(i * 128) as *const _);
        let l1 = _mm512_loadu_si512(ptr.add(i * 128 + 64) as *const _);
        in_batches[i] = pack_52(l0, l1);
    }

    for tile in 0..4 {
        let mut z_0 = (_mm512_setzero_si512(), _mm512_setzero_si512(), _mm512_setzero_si512(), _mm512_setzero_si512(), _mm512_setzero_si512());
        let mut z_1 = (_mm512_setzero_si512(), _mm512_setzero_si512(), _mm512_setzero_si512(), _mm512_setzero_si512(), _mm512_setzero_si512());
        let mut z_2 = (_mm512_setzero_si512(), _mm512_setzero_si512(), _mm512_setzero_si512(), _mm512_setzero_si512(), _mm512_setzero_si512());
        let mut z_3 = (_mm512_setzero_si512(), _mm512_setzero_si512(), _mm512_setzero_si512(), _mm512_setzero_si512(), _mm512_setzero_si512());
        
        for j in 0..16 {
            let (b0, b1, b2) = in_batches[j];
            
            // Output batch 0 for this tile (index = tile * 4 + 0)
            let m0 = ((M_COL_P[tile * 4 + 0][j].0 as u128) << 64) | (M_COL_P[tile * 4 + 0][j].1 as u128);
            let (m0_0, m0_1, m0_2) = scalar_to_limbs(m0);
            z_0 = v_mac_ifma(z_0.0, z_0.1, z_0.2, z_0.3, z_0.4, b0, b1, b2, _mm512_set1_epi64(m0_0 as i64), _mm512_set1_epi64(m0_1 as i64), _mm512_set1_epi64(m0_2 as i64));
            
            let m1 = ((M_COL_P[tile * 4 + 1][j].0 as u128) << 64) | (M_COL_P[tile * 4 + 1][j].1 as u128);
            let (m1_0, m1_1, m1_2) = scalar_to_limbs(m1);
            z_1 = v_mac_ifma(z_1.0, z_1.1, z_1.2, z_1.3, z_1.4, b0, b1, b2, _mm512_set1_epi64(m1_0 as i64), _mm512_set1_epi64(m1_1 as i64), _mm512_set1_epi64(m1_2 as i64));
            
            let m2 = ((M_COL_P[tile * 4 + 2][j].0 as u128) << 64) | (M_COL_P[tile * 4 + 2][j].1 as u128);
            let (m2_0, m2_1, m2_2) = scalar_to_limbs(m2);
            z_2 = v_mac_ifma(z_2.0, z_2.1, z_2.2, z_2.3, z_2.4, b0, b1, b2, _mm512_set1_epi64(m2_0 as i64), _mm512_set1_epi64(m2_1 as i64), _mm512_set1_epi64(m2_2 as i64));
            
            let m3 = ((M_COL_P[tile * 4 + 3][j].0 as u128) << 64) | (M_COL_P[tile * 4 + 3][j].1 as u128);
            let (m3_0, m3_1, m3_2) = scalar_to_limbs(m3);
            z_3 = v_mac_ifma(z_3.0, z_3.1, z_3.2, z_3.3, z_3.4, b0, b1, b2, _mm512_set1_epi64(m3_0 as i64), _mm512_set1_epi64(m3_1 as i64), _mm512_set1_epi64(m3_2 as i64));
        }
        
        out_col[tile * 4 + 0] = v_reduce_ifma(z_0.0, z_0.1, z_0.2, z_0.3, z_0.4);
        out_col[tile * 4 + 1] = v_reduce_ifma(z_1.0, z_1.1, z_1.2, z_1.3, z_1.4);
        out_col[tile * 4 + 2] = v_reduce_ifma(z_2.0, z_2.1, z_2.2, z_2.3, z_2.4);
        out_col[tile * 4 + 3] = v_reduce_ifma(z_3.0, z_3.1, z_3.2, z_3.3, z_3.4);
    }

    // 2. Row mixing + Fused Affine
    let mut out_row = [(_mm512_setzero_si512(), _mm512_setzero_si512(), _mm512_setzero_si512()); 16];
    
    // Precompute SIGMA_C_INV to map RC_C to pre-permutation indices.
    let mut sigma_inv = [0; 128];
    for i in 0..128 { sigma_inv[SIGMA_C[i]] = i; }

    for i in 0..16 {
        let (l0, l1, l2) = out_col[i];
        
        let mut z = (_mm512_setzero_si512(), _mm512_setzero_si512(), _mm512_setzero_si512(), _mm512_setzero_si512(), _mm512_setzero_si512());
        for j in 0..8 {
            let idx = _mm512_set1_epi64(j as i64);
            let e0 = _mm512_permutexvar_epi64(idx, l0);
            let e1 = _mm512_permutexvar_epi64(idx, l1);
            let e2 = _mm512_permutexvar_epi64(idx, l2);
            
            let mut arr_c0 = [0u64; 8];
            let mut arr_c1 = [0u64; 8];
            let mut arr_c2 = [0u64; 8];
            for out_idx in 0..8 {
                let m = ((M_ROW_P[out_idx][j].0 as u128) << 64) | (M_ROW_P[out_idx][j].1 as u128);
                let (c0, c1, c2) = scalar_to_limbs(m);
                arr_c0[out_idx] = c0;
                arr_c1[out_idx] = c1;
                arr_c2[out_idx] = c2;
            }
            let c0 = _mm512_loadu_si512(arr_c0.as_ptr() as *const _);
            let c1 = _mm512_loadu_si512(arr_c1.as_ptr() as *const _);
            let c2 = _mm512_loadu_si512(arr_c2.as_ptr() as *const _);
            
            z = v_mac_ifma(z.0, z.1, z.2, z.3, z.4, e0, e1, e2, c0, c1, c2);
        }

        // FUSE AFFINE ADDITION
        // We add RC_C mapped through sigma_inv to the unreduced accumulators
        let mut rc_c0 = [0u64; 8];
        let mut rc_c1 = [0u64; 8];
        let mut rc_c2 = [0u64; 8];
        for lane in 0..8 {
            let global_idx = i * 8 + lane;
            let orig_idx = sigma_inv[global_idx];
            let rc = ((RC_C[r][orig_idx].0 as u128) << 64) | (RC_C[r][orig_idx].1 as u128);
            let (c0, c1, c2) = scalar_to_limbs(rc);
            rc_c0[lane] = c0;
            rc_c1[lane] = c1;
            rc_c2[lane] = c2;
        }
        let rc0 = _mm512_loadu_si512(rc_c0.as_ptr() as *const _);
        let rc1 = _mm512_loadu_si512(rc_c1.as_ptr() as *const _);
        let rc2 = _mm512_loadu_si512(rc_c2.as_ptr() as *const _);
        
        z.0 = _mm512_add_epi64(z.0, rc0);
        z.1 = _mm512_add_epi64(z.1, rc1);
        z.2 = _mm512_add_epi64(z.2, rc2);

        out_row[i] = v_reduce_ifma(z.0, z.1, z.2, z.3, z.4);
    }
    
    // 3. Unpack and Apply SIGMA_C
    let mut tmp = [0u128; 128];
    for i in 0..16 {
        let (out_l0, out_l1) = unpack_52(out_row[i].0, out_row[i].1, out_row[i].2);
        let ptr = tmp.as_mut_ptr() as *mut i8;
        _mm512_storeu_si512(ptr.add(i * 128) as *mut _, out_l0);
        _mm512_storeu_si512(ptr.add(i * 128 + 64) as *mut _, out_l1);
    }
    for i in 0..128 {
        f[i] = tmp[SIGMA_C[i]];
    }
}

fn test_mds() {
    let mut rng = XorShift::new(12345);
    for _ in 0..10_000 {
        let mut f = [0u128; 128];
        let mut f_ref = [0u128; 128];
        for i in 0..128 {
            let val = rng.next_u128() % 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF_u128; // p is 2^127-1, but we use random bits and mask it below
            let val = val & 0x7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF; 
            f[i] = val;
            f_ref[i] = val;
        }

        // Ref
        aise_core::mds_c::mix_lanes(&mut f_ref);
        let mut next = [0u128; 128];
        for i in 0..128 { next[i] = f_ref[SIGMA_C[i]]; }
        for i in 0..128 {
            let rc = ((RC_C[0][i].0 as u128) << 64) | (RC_C[0][i].1 as u128);
            next[i] = aise_core::field_p::add(next[i], rc);
        }
        f_ref = next;
        
        // Simd
        unsafe {
            mds_affine_fused(&mut f, 0);
        }

        for i in 0..128 {
            assert_eq!(f[i], f_ref[i], "Mismatch at index {}", i);
        }
    }
    println!("MDS isolation test passed 10,000 randomized iterations successfully!");
}

fn test_edge_cases() {
    let edge_values = vec![
        0u128, // All zeros
        1u128, // One
        (1u128 << 127) - 2, // Max - 1
        (1u128 << 127) - 1, // Max boundary
        (1u128 << 52) - 1,  // Limb 0 max
        (1u128 << 52),      // Limb 1 min
        (1u128 << 104) - 1, // Limb 1 max
        (1u128 << 104),     // Limb 2 min
    ];

    for &val in &edge_values {
        let mut f = [val; 128];
        let mut f_ref = [val; 128];

        // Ref
        aise_core::mds_c::mix_lanes(&mut f_ref);
        let mut next = [0u128; 128];
        for i in 0..128 { next[i] = f_ref[SIGMA_C[i]]; }
        for i in 0..128 {
            let rc = ((RC_C[0][i].0 as u128) << 64) | (RC_C[0][i].1 as u128);
            next[i] = aise_core::field_p::add(next[i], rc);
        }
        f_ref = next;

        // Simd
        unsafe {
            mds_affine_fused(&mut f, 0);
        }

        for i in 0..128 {
            assert_eq!(f[i], f_ref[i], "Mismatch at edge case 0x{:032x} at index {}", val, i);
        }
    }
    println!("MDS isolation test passed all deterministic edge cases!");
}

fn main() {
    test_mds();
    test_edge_cases();
}
