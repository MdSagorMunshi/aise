#![allow(unsafe_op_in_unsafe_fn)]
use std::arch::x86_64::*;

// Packs two __m512i (which hold 8x u128) into three __m512i (L0, L1, L2 limbs)
#[inline(always)]
unsafe fn pack_52(l0: __m512i, l1: __m512i) -> (__m512i, __m512i, __m512i) {
    // A __m512i holding u128s has structure:
    // [u0_lo, u0_hi, u1_lo, u1_hi, u2_lo, u2_hi, u3_lo, u3_hi] for l0
    // [u4_lo, u4_hi, u5_lo, u5_hi, u6_lo, u6_hi, u7_lo, u7_hi] for l1

    // We want a register containing all the `lo` halves:
    // [u0_lo, u1_lo, u2_lo, u3_lo, u4_lo, u5_lo, u6_lo, u7_lo]
    // The indices for `lo` in the 16 64-bit slots are: 0, 2, 4, 6, 8, 10, 12, 14
    let idx_lo = _mm512_set_epi64(14, 12, 10, 8, 6, 4, 2, 0);
    let lo = _mm512_permutex2var_epi64(l0, idx_lo, l1);

    // We want a register containing all the `hi` halves:
    // [u0_hi, u1_hi, u2_hi, u3_hi, u4_hi, u5_hi, u6_hi, u7_hi]
    // The indices for `hi` are: 1, 3, 5, 7, 9, 11, 13, 15
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

// Unpacks three __m512i (L0, L1, L2 limbs) back into two __m512i (holding 8x u128)
#[inline(always)]
unsafe fn unpack_52(a0: __m512i, a1: __m512i, a2: __m512i) -> (__m512i, __m512i) {
    let lo = _mm512_or_si512(a0, _mm512_slli_epi64(a1, 52));
    let hi = _mm512_or_si512(_mm512_srli_epi64(a1, 12), _mm512_slli_epi64(a2, 40));

    // lo contains [u0_lo, u1_lo, u2_lo, ...]
    // hi contains [u0_hi, u1_hi, u2_hi, ...]
    // We want to interleave them to restore [u0_lo, u0_hi, u1_lo, u1_hi, ...]
    let l0 = _mm512_unpacklo_epi64(lo, hi);
    let l1 = _mm512_unpackhi_epi64(lo, hi);
    
    // unpacklo interleaves the low 64 bits of each 128-bit lane.
    // Let's trace it:
    // lo = [0, 1, 2, 3, 4, 5, 6, 7]
    // hi = [0h, 1h, 2h, 3h, 4h, 5h, 6h, 7h]
    // unpacklo(lo, hi) produces:
    // [0, 0h, 2, 2h, 4, 4h, 6, 6h]
    // unpackhi(lo, hi) produces:
    // [1, 1h, 3, 3h, 5, 5h, 7, 7h]
    // Wait! This means unpacklo and unpackhi do NOT reconstruct l0 and l1 correctly!
    // We need l0 to be [0, 0h, 1, 1h, 2, 2h, 3, 3h]
    // and l1 to be [4, 4h, 5, 5h, 6, 6h, 7, 7h].
    // So unpacklo and unpackhi mix elements from the whole register!
    
    // Instead of unpacklo/unpackhi, we can use permutex2var again!
    // l0 = [0, 0h, 1, 1h, 2, 2h, 3, 3h]
    // 0 is from lo[0] -> index 0
    // 0h is from hi[0] -> index 8
    // 1 is from lo[1] -> index 1
    // 1h is from hi[1] -> index 9
    // 2 is from lo[2] -> index 2
    // 2h is from hi[2] -> index 10
    // 3 is from lo[3] -> index 3
    // 3h is from hi[3] -> index 11
    let idx_l0 = _mm512_set_epi64(11, 3, 10, 2, 9, 1, 8, 0);
    let l0_out = _mm512_permutex2var_epi64(lo, idx_l0, hi);

    // l1 = [4, 4h, 5, 5h, 6, 6h, 7, 7h]
    // 4 is from lo[4] -> index 4
    // 4h is from hi[4] -> index 12
    // 5 is from lo[5] -> index 5
    // 5h is from hi[5] -> index 13
    // 6 is from lo[6] -> index 6
    // 6h is from hi[6] -> index 14
    // 7 is from lo[7] -> index 7
    // 7h is from hi[7] -> index 15
    let idx_l1 = _mm512_set_epi64(15, 7, 14, 6, 13, 5, 12, 4);
    let l1_out = _mm512_permutex2var_epi64(lo, idx_l1, hi);

    (l0_out, l1_out)
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

fn main() {
    println!("Testing pack/unpack memory transpose...");

    let iters = 10_000;
    let mut rng = XorShift::new(0x1234567890ABCDEF);

    for _ in 0..iters {
        let mut original = [0u128; 8];
        for i in 0..8 {
            let lo = rng.next_u64();
            let hi = rng.next_u64() & 0x7FFFFFFF_FFFFFFFF; // Ensure it fits in 127 bits
            original[i] = (lo as u128) | ((hi as u128) << 64);
        }

        unsafe {
            let mut l0_raw = [0u128; 4];
            let mut l1_raw = [0u128; 4];
            for i in 0..4 {
                l0_raw[i] = original[i];
                l1_raw[i] = original[i + 4];
            }

            let l0_v = _mm512_loadu_si512(l0_raw.as_ptr() as *const _);
            let l1_v = _mm512_loadu_si512(l1_raw.as_ptr() as *const _);

            let (a0, a1, a2) = pack_52(l0_v, l1_v);
            let (out_l0, out_l1) = unpack_52(a0, a1, a2);

            let mut out_l0_arr = [0u128; 4];
            let mut out_l1_arr = [0u128; 4];
            _mm512_storeu_si512(out_l0_arr.as_mut_ptr() as *mut _, out_l0);
            _mm512_storeu_si512(out_l1_arr.as_mut_ptr() as *mut _, out_l1);

            for i in 0..4 {
                if out_l0_arr[i] != original[i] {
                    panic!("Mismatch at index {}! Expected {:X}, got {:X}", i, original[i], out_l0_arr[i]);
                }
                if out_l1_arr[i] != original[i + 4] {
                    panic!("Mismatch at index {}! Expected {:X}, got {:X}", i + 4, original[i + 4], out_l1_arr[i]);
                }
            }
        }
    }

    println!("pack_52 and unpack_52 flawlessly round-trip 127-bit elements!");
}
