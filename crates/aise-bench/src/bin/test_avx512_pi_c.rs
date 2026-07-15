#![allow(unsafe_op_in_unsafe_fn)]
use aise_core::state::{Lane, State};
use aise_core::field_p;
use std::arch::x86_64::*;

#[inline(always)]
unsafe fn v_mul_ifma(a0: __m512i, a1: __m512i, a2: __m512i, b0: __m512i, b1: __m512i, b2: __m512i) -> (__m512i, __m512i, __m512i) {
    let mut z0 = _mm512_madd52lo_epu64(_mm512_setzero_si512(), a0, b0);
    let mut z1 = _mm512_madd52hi_epu64(_mm512_setzero_si512(), a0, b0);

    z1 = _mm512_madd52lo_epu64(z1, a0, b1);
    z1 = _mm512_madd52lo_epu64(z1, a1, b0);
    let mut z2 = _mm512_madd52hi_epu64(_mm512_setzero_si512(), a0, b1);
    z2 = _mm512_madd52hi_epu64(z2, a1, b0);

    z2 = _mm512_madd52lo_epu64(z2, a0, b2);
    z2 = _mm512_madd52lo_epu64(z2, a1, b1);
    z2 = _mm512_madd52lo_epu64(z2, a2, b0);
    let mut z3 = _mm512_madd52hi_epu64(_mm512_setzero_si512(), a0, b2);
    z3 = _mm512_madd52hi_epu64(z3, a1, b1);
    z3 = _mm512_madd52hi_epu64(z3, a2, b0);

    z3 = _mm512_madd52lo_epu64(z3, a1, b2);
    z3 = _mm512_madd52lo_epu64(z3, a2, b1);
    let mut z4 = _mm512_madd52hi_epu64(_mm512_setzero_si512(), a1, b2);
    z4 = _mm512_madd52hi_epu64(z4, a2, b1);

    z4 = _mm512_madd52lo_epu64(z4, a2, b2);

    let m52 = _mm512_set1_epi64(0xFFFFFFFFFFFFF);

    let mut c0 = z0;
    
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

    // Final canonical reduction (if result == P, set to 0)
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

struct XorShift { state: u64 }
impl XorShift {
    fn new(seed: u64) -> Self { Self { state: seed } }
    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }
}

fn to_u128(l0: u64, l1: u64, l2: u64) -> u128 {
    (l0 as u128) | ((l1 as u128) << 52) | ((l2 as u128) << 104)
}

fn main() {
    println!("Testing v_mul_ifma vs scalar field_p::mul...");

    let iters = 10_000;
    let mut rng = XorShift::new(0xDEADBEEFCAFEBABE);

    for _ in 0..iters {
        let mut a0_arr = [0u64; 8];
        let mut a1_arr = [0u64; 8];
        let mut a2_arr = [0u64; 8];

        let mut b0_arr = [0u64; 8];
        let mut b1_arr = [0u64; 8];
        let mut b2_arr = [0u64; 8];

        let mut a_u128 = [0u128; 8];
        let mut b_u128 = [0u128; 8];

        for i in 0..8 {
            a0_arr[i] = rng.next_u64() & 0xFFFFFFFFFFFFF;
            a1_arr[i] = rng.next_u64() & 0xFFFFFFFFFFFFF;
            a2_arr[i] = rng.next_u64() & 0x7FFFFF;

            b0_arr[i] = rng.next_u64() & 0xFFFFFFFFFFFFF;
            b1_arr[i] = rng.next_u64() & 0xFFFFFFFFFFFFF;
            b2_arr[i] = rng.next_u64() & 0x7FFFFF;
            
            a_u128[i] = to_u128(a0_arr[i], a1_arr[i], a2_arr[i]);
            b_u128[i] = to_u128(b0_arr[i], b1_arr[i], b2_arr[i]);
            
            if rng.next_u64() % 100 == 0 {
                a0_arr[i] = 0xFFFFFFFFFFFFF;
                a1_arr[i] = 0xFFFFFFFFFFFFF;
                a2_arr[i] = 0x7FFFFF;
                a_u128[i] = field_p::P;
            }
        }

        unsafe {
            let a0_v = _mm512_loadu_si512(a0_arr.as_ptr() as *const _);
            let a1_v = _mm512_loadu_si512(a1_arr.as_ptr() as *const _);
            let a2_v = _mm512_loadu_si512(a2_arr.as_ptr() as *const _);

            let b0_v = _mm512_loadu_si512(b0_arr.as_ptr() as *const _);
            let b1_v = _mm512_loadu_si512(b1_arr.as_ptr() as *const _);
            let b2_v = _mm512_loadu_si512(b2_arr.as_ptr() as *const _);

            let (r0, r1, r2) = v_mul_ifma(a0_v, a1_v, a2_v, b0_v, b1_v, b2_v);

            let mut out0_arr = [0u64; 8];
            let mut out1_arr = [0u64; 8];
            let mut out2_arr = [0u64; 8];

            _mm512_storeu_si512(out0_arr.as_mut_ptr() as *mut _, r0);
            _mm512_storeu_si512(out1_arr.as_mut_ptr() as *mut _, r1);
            _mm512_storeu_si512(out2_arr.as_mut_ptr() as *mut _, r2);

            for i in 0..8 {
                let actual = to_u128(out0_arr[i], out1_arr[i], out2_arr[i]);
                let expected = field_p::mul(a_u128[i], b_u128[i]);

                if actual != expected {
                    panic!("Mismatch at iter {}! a={}, b={}, expected={}, actual={}", i, a_u128[i], b_u128[i], expected, actual);
                }
            }
        }
    }

    println!("v_mul_ifma perfectly matches scalar across {} random batches ({} total tests)!", iters, iters * 8);
}
