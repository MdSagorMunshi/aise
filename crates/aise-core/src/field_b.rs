//! GF(2^128) arithmetic.
//!
//! Irreducible polynomial: P(x) = x^128 + x^7 + x^2 + x + 1
//!
//! Representation (Lane): hi = coefficients of x^127..x^64, lo = x^63..x^0
//! (MSB of hi = x^127, LSB of lo = x^0)

use crate::state::Lane;

// ---------------------------------------------------------------------------
// Portable (bit-serial) implementation — O(128) loop, used as fallback
// ---------------------------------------------------------------------------

#[inline(always)]
pub fn mul_portable(a: Lane, b: Lane) -> Lane {
    let mut p_hi = 0u64;
    let mut p_lo = 0u64;
    let mut a_hi = a.hi;
    let mut a_lo = a.lo;
    let mut b_hi = b.hi;
    let mut b_lo = b.lo;

    for _ in 0..128 {
        if b_lo & 1 != 0 {
            p_hi ^= a_hi;
            p_lo ^= a_lo;
        }
        let hi_bit = a_hi & 0x8000_0000_0000_0000;
        a_hi = (a_hi << 1) | (a_lo >> 63);
        a_lo <<= 1;
        if hi_bit != 0 {
            a_lo ^= 0x0000_0000_0000_0087;
        }
        b_lo = (b_lo >> 1) | ((b_hi & 1) << 63);
        b_hi >>= 1;
    }
    Lane::new(p_hi, p_lo)
}

// ---------------------------------------------------------------------------
// PCLMULQDQ-accelerated implementation (x86_64 only)
// ---------------------------------------------------------------------------

#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

/// Hardware-accelerated GF(2^128) multiplication using PCLMULQDQ.
///
/// Uses schoolbook 128×128→256-bit carry-less multiplication followed by
/// shift-based reduction modulo x^128 + x^7 + x^2 + x + 1.
///
/// # Safety
/// Caller must ensure PCLMULQDQ and SSE2 are available on the current CPU.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "pclmulqdq,sse2,sse4.1")]
#[inline]
#[target_feature(enable = "pclmulqdq,sse2,sse4.1")]
#[inline]
unsafe fn mul_clmul(a: Lane, b: Lane) -> Lane {
    // Load lanes into XMM registers (high qword = hi, low qword = lo)
    let va = _mm_set_epi64x(a.hi as i64, a.lo as i64);
    let vb = _mm_set_epi64x(b.hi as i64, b.lo as i64);

    // Schoolbook 128×128 → 256-bit carry-less multiplication
    // Selector: bit 0 selects qword of va, bit 4 selects qword of vb
    let t_ll = _mm_clmulepi64_si128(va, vb, 0x00); // a.lo * b.lo
    let t_lh = _mm_clmulepi64_si128(va, vb, 0x01); // a.lo * b.hi
    let t_hl = _mm_clmulepi64_si128(va, vb, 0x10); // a.hi * b.lo
    let t_hh = _mm_clmulepi64_si128(va, vb, 0x11); // a.hi * b.hi

    // Combine cross terms: t_lh + t_hl straddles bits 64..191
    let mid = _mm_xor_si128(t_lh, t_hl);
    let mid_lo = _mm_slli_si128(mid, 8); // low qword → high qword position
    let mid_hi = _mm_srli_si128(mid, 8); // high qword → low qword position

    // 256-bit product = prod_hi : prod_lo
    let prod_lo = _mm_xor_si128(t_ll, mid_lo); // bits 0..127
    let prod_hi = _mm_xor_si128(t_hh, mid_hi); // bits 128..255

    // ------------------------------------------------------------------
    // Reduction modulo P(x) = x^128 + x^7 + x^2 + x + 1
    //
    // For each coefficient h_k at position 128+k in the product:
    //   x^(128+k) ≡ x^(k+7) + x^(k+2) + x^(k+1) + x^k  (mod P)
    //
    // So the contribution is:
    //   R = prod_hi·x^7 ⊕ prod_hi·x^2 ⊕ prod_hi·x ⊕ prod_hi
    // where · denotes polynomial (carryless) multiplication.
    //
    // The shifts may produce up to 7 overflow bits (positions 128..134)
    // which require a second (tiny) reduction pass.
    // ------------------------------------------------------------------

    // Helper: 128-bit polynomial left shift by s bits (s < 64)
    // shifted = (hi << s | lo >> (64-s), lo << s)
    // overflow = hi >> (64-s)  (in the low qword of the result)

    // prod_hi << 7
    let sh7 = _mm_xor_si128(
        _mm_slli_epi64(prod_hi, 7),
        _mm_slli_si128(_mm_srli_epi64(prod_hi, 57), 8),
    );
    let ov7 = _mm_srli_si128(_mm_srli_epi64(prod_hi, 57), 8);

    // prod_hi << 2
    let sh2 = _mm_xor_si128(
        _mm_slli_epi64(prod_hi, 2),
        _mm_slli_si128(_mm_srli_epi64(prod_hi, 62), 8),
    );
    let ov2 = _mm_srli_si128(_mm_srli_epi64(prod_hi, 62), 8);

    // prod_hi << 1
    let sh1 = _mm_xor_si128(
        _mm_slli_epi64(prod_hi, 1),
        _mm_slli_si128(_mm_srli_epi64(prod_hi, 63), 8),
    );
    let ov1 = _mm_srli_si128(_mm_srli_epi64(prod_hi, 63), 8);

    // First reduction fold (128-bit portion)
    let fold = _mm_xor_si128(
        _mm_xor_si128(sh7, sh2),
        _mm_xor_si128(sh1, prod_hi),
    );

    // Overflow bits from first fold (at most 7 bits, in low qword)
    let overflow = _mm_xor_si128(_mm_xor_si128(ov7, ov2), ov1);

    // Second reduction: fold overflow (≤7 bits) the same way.
    // These map to positions ≤ x^13, no further overflow possible.
    let ov_val = _mm_extract_epi64(overflow, 0) as u64;
    let ov_reduced = (ov_val << 7) ^ (ov_val << 2) ^ (ov_val << 1) ^ ov_val;
    let ov_xmm = _mm_set_epi64x(0, ov_reduced as i64);

    // Final result: prod_lo ⊕ fold ⊕ ov_reduced
    let result = _mm_xor_si128(_mm_xor_si128(prod_lo, fold), ov_xmm);

    Lane::new(
        _mm_extract_epi64(result, 1) as u64,
        _mm_extract_epi64(result, 0) as u64,
    )
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "pclmulqdq,sse2,sse4.1")]
#[inline]
unsafe fn sq_clmul(a: Lane) -> Lane {
    // Load lane into XMM register
    let va = _mm_set_epi64x(a.hi as i64, a.lo as i64);

    // Self-multiply: only need to square the low and high halves
    // Cross terms (va.lo * va.hi) would be multiplied by 2 (XORed with themselves), so they cancel out to 0.
    // Thus we only need t_ll and t_hh.
    let t_ll = _mm_clmulepi64_si128(va, va, 0x00); // a.lo * a.lo (bits 0..127)
    let t_hh = _mm_clmulepi64_si128(va, va, 0x11); // a.hi * a.hi (bits 128..255)

    let prod_lo = t_ll;
    let prod_hi = t_hh;

    // Same Barrett reduction as mul_clmul
    let sh7 = _mm_xor_si128(
        _mm_slli_epi64(prod_hi, 7),
        _mm_slli_si128(_mm_srli_epi64(prod_hi, 57), 8),
    );
    let ov7 = _mm_srli_si128(_mm_srli_epi64(prod_hi, 57), 8);

    let sh2 = _mm_xor_si128(
        _mm_slli_epi64(prod_hi, 2),
        _mm_slli_si128(_mm_srli_epi64(prod_hi, 62), 8),
    );
    let ov2 = _mm_srli_si128(_mm_srli_epi64(prod_hi, 62), 8);

    let sh1 = _mm_xor_si128(
        _mm_slli_epi64(prod_hi, 1),
        _mm_slli_si128(_mm_srli_epi64(prod_hi, 63), 8),
    );
    let ov1 = _mm_srli_si128(_mm_srli_epi64(prod_hi, 63), 8);

    let fold = _mm_xor_si128(
        _mm_xor_si128(sh7, sh2),
        _mm_xor_si128(sh1, prod_hi),
    );

    let overflow = _mm_xor_si128(_mm_xor_si128(ov7, ov2), ov1);

    let ov_val = _mm_extract_epi64(overflow, 0) as u64;
    let ov_reduced = (ov_val << 7) ^ (ov_val << 2) ^ (ov_val << 1) ^ ov_val;
    let ov_xmm = _mm_set_epi64x(0, ov_reduced as i64);

    let result = _mm_xor_si128(_mm_xor_si128(prod_lo, fold), ov_xmm);

    Lane::new(
        _mm_extract_epi64(result, 1) as u64,
        _mm_extract_epi64(result, 0) as u64,
    )
}

// ---------------------------------------------------------------------------
// Public API: runtime-dispatched mul()
// ---------------------------------------------------------------------------

/// Multiply two elements in GF(2^128).
///
/// On x86_64 with `std` feature (default): uses runtime CPUID detection to
/// dispatch to PCLMULQDQ-accelerated code if available, falling back to
/// the portable bit-serial implementation otherwise.
///
/// On x86_64 without `std` (no_std): uses compile-time `target_feature`
/// detection only.
///
/// On non-x86_64: always uses the portable implementation.
#[inline(always)]
pub fn mul(a: Lane, b: Lane) -> Lane {
    // Runtime detection via std (default path)
    #[cfg(all(target_arch = "x86_64", feature = "std"))]
    {
        if std::is_x86_feature_detected!("pclmulqdq") {
            return unsafe { mul_clmul(a, b) };
        }
        return mul_portable(a, b);
    }

    // Compile-time detection for no_std builds on x86_64
    #[cfg(all(target_arch = "x86_64", not(feature = "std"), target_feature = "pclmulqdq"))]
    {
        return unsafe { mul_clmul(a, b) };
    }

    // Portable fallback (non-x86_64 or no_std without pclmul)
    #[cfg(not(target_arch = "x86_64"))]
    {
        mul_portable(a, b)
    }
}

#[inline(always)]
pub fn sq(a: Lane) -> Lane {
    // Runtime detection via std
    #[cfg(all(target_arch = "x86_64", feature = "std"))]
    {
        if std::is_x86_feature_detected!("pclmulqdq") {
            return unsafe { sq_clmul(a) };
        }
        return mul_portable(a, a);
    }

    // Compile-time detection
    #[cfg(all(target_arch = "x86_64", not(feature = "std"), target_feature = "pclmulqdq"))]
    {
        return unsafe { sq_clmul(a) };
    }

    // Portable fallback
    #[cfg(not(target_arch = "x86_64"))]
    {
        mul_portable(a, a)
    }
}

pub fn inv(a: Lane) -> Lane {
    if a.hi == 0 && a.lo == 0 {
        return Lane::new(0, 0);
    }
    let a2 = sq(a);
    let t_2 = mul(a2, a);
    
    let mut t_4 = t_2;
    for _ in 0..2 { t_4 = sq(t_4); }
    t_4 = mul(t_4, t_2);

    let mut t_8 = t_4;
    for _ in 0..4 { t_8 = sq(t_8); }
    t_8 = mul(t_8, t_4);

    let mut t_16 = t_8;
    for _ in 0..8 { t_16 = sq(t_16); }
    t_16 = mul(t_16, t_8);

    let mut t_32 = t_16;
    for _ in 0..16 { t_32 = sq(t_32); }
    t_32 = mul(t_32, t_16);

    let mut t_64 = t_32;
    for _ in 0..32 { t_64 = sq(t_64); }
    t_64 = mul(t_64, t_32);

    let mut inv_base = t_64;
    for _ in 0..32 { inv_base = sq(inv_base); }
    inv_base = mul(inv_base, t_32);

    for _ in 0..16 { inv_base = sq(inv_base); }
    inv_base = mul(inv_base, t_16);

    for _ in 0..8 { inv_base = sq(inv_base); }
    inv_base = mul(inv_base, t_8);

    for _ in 0..4 { inv_base = sq(inv_base); }
    inv_base = mul(inv_base, t_4);

    for _ in 0..2 { inv_base = sq(inv_base); }
    inv_base = mul(inv_base, t_2);

    inv_base = sq(inv_base);
    inv_base = mul(inv_base, a);
    
    sq(inv_base)
}

use crate::field_b_avx512;

/// Montgomery Batch Inversion for 128 elements in GF(2^128).
/// Inverts the slice in-place. Uses branchless masking for zero-handling.
#[inline]
pub fn batch_inv(lanes: &mut [Lane; 128]) {
    #[cfg(all(target_arch = "x86_64", feature = "std"))]
    {
        if std::is_x86_feature_detected!("avx512f") 
            && std::is_x86_feature_detected!("vpclmulqdq") 
            && std::is_x86_feature_detected!("avx512bw") 
            && std::is_x86_feature_detected!("avx512dq") 
        {
            unsafe {
                static mut PRINTED: bool = false;
                if !PRINTED {
                    println!("--- AVX-512 BATCH INV IS ACTIVE ---");
                    PRINTED = true;
                }
                field_b_avx512::batch_inv_avx512(lanes);
                return;
            }
        }
    }

    let mut masks = [0u64; 128];
    let mut c = [Lane::new(0, 0); 128];
    
    // Helper for branchless zero masking: returns !0 if lane is 0, else 0
    let is_zero = |lane: Lane| -> u64 {
        0u64.wrapping_sub(((lane.hi | lane.lo) == 0) as u64)
    };
    
    let mut acc = Lane::new(0, 1); // Multiplicative identity
    
    // Step 1: Branchless zero-substitution and prefix product
    for i in 0..128 {
        let mask = is_zero(lanes[i]);
        masks[i] = mask;
        // Substitute 0 -> 1: a' = a ^ (mask & 1)
        let a_prime = Lane::new(lanes[i].hi, lanes[i].lo ^ (mask & 1));
        acc = mul(acc, a_prime);
        c[i] = acc;
    }
    
    // Step 2: Single inversion of the full product
    let mut inv_prod = inv(acc);
    
    // Step 3: Back-multiplication
    for i in (1..128).rev() {
        let mask = masks[i];
        let a_prime = Lane::new(lanes[i].hi, lanes[i].lo ^ (mask & 1));
        
        let a_inv = mul(inv_prod, c[i - 1]);
        inv_prod = mul(inv_prod, a_prime);
        
        // Final correction: a_inv & !mask
        lanes[i] = Lane::new(a_inv.hi & !mask, a_inv.lo & !mask);
    }
    
    // Handle index 0
    let mask = masks[0];
    lanes[0] = Lane::new(inv_prod.hi & !mask, inv_prod.lo & !mask);
}

// ---------------------------------------------------------------------------
// GF(2^128) addition (XOR — no dispatch needed)
// ---------------------------------------------------------------------------

#[inline(always)]
pub fn add(a: Lane, b: Lane) -> Lane {
    Lane::new(a.hi ^ b.hi, a.lo ^ b.lo)
}
