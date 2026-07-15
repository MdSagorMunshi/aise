//! GF(2^127 - 1) arithmetic.

pub const P: u128 = (1 << 127) - 1;

#[inline(always)]
pub fn reduce(v: u128) -> u128 {
    let v_top = v >> 127;
    let v_low = v & P;
    let mut r = v_low + v_top;
    let mask = ((r >= P) as u128).wrapping_neg();
    r -= P & mask;
    r
}

#[inline(always)]
pub fn add(a: u128, b: u128) -> u128 {
    let mut r = a + b;
    let mask = ((r >= P) as u128).wrapping_neg();
    r -= P & mask;
    r
}

#[inline(always)]
pub fn sub(a: u128, b: u128) -> u128 {
    let mut r = a.wrapping_sub(b);
    let mask = ((r > P) as u128).wrapping_neg();
    r = r.wrapping_add(P & mask);
    r
}

#[inline(always)]
pub fn mul_portable(a: u128, b: u128) -> u128 {
    let a_lo = (a & 0xFFFFFFFFFFFFFFFF) as u64 as u128;
    let a_hi = (a >> 64) as u64 as u128;
    let b_lo = (b & 0xFFFFFFFFFFFFFFFF) as u64 as u128;
    let b_hi = (b >> 64) as u64 as u128;

    let p0 = a_lo * b_lo;
    let p1 = a_lo * b_hi;
    let p2 = a_hi * b_lo;
    let p3 = a_hi * b_hi;

    let cross = p1 + p2;
    let cross_lo = (cross & 0xFFFFFFFFFFFFFFFF) as u64 as u128;
    let cross_hi = cross >> 64;

    let (lo, carry1) = p0.overflowing_add(cross_lo << 64);
    let hi = p3 + cross_hi + (carry1 as u128);

    let lo_top = lo >> 127;
    let lo_low = lo & P;

    let sum_a = (hi << 1) + lo_low;
    let (sum1, carry2) = sum_a.overflowing_add(lo_top);

    let r = reduce(sum1) + (carry2 as u128) * 2;
    reduce(r)
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "bmi2")]
#[inline]
unsafe fn mul_bmi2(a: u128, b: u128) -> u128 {
    use core::arch::x86_64::{_addcarry_u64, _mulx_u64};

    let a_lo = a as u64;
    let a_hi = (a >> 64) as u64;
    let b_lo = b as u64;
    let b_hi = (b >> 64) as u64;

    let mut hi_p0 = 0;
    let lo_p0 = _mulx_u64(a_lo, b_lo, &mut hi_p0);
    
    let mut hi_p1 = 0;
    let lo_p1 = _mulx_u64(a_lo, b_hi, &mut hi_p1);
    
    let mut hi_p2 = 0;
    let lo_p2 = _mulx_u64(a_hi, b_lo, &mut hi_p2);
    
    let mut hi_p3 = 0;
    let lo_p3 = _mulx_u64(a_hi, b_hi, &mut hi_p3);

    let mut c1 = 0;
    let mut r1 = 0;
    let mut r2 = 0;
    let mut r3 = 0;
    
    c1 = _addcarry_u64(c1, hi_p0, lo_p1, &mut r1);
    c1 = _addcarry_u64(c1, hi_p1, lo_p3, &mut r2);
    _addcarry_u64(c1, hi_p3, 0, &mut r3);

    let mut c2 = 0;
    c2 = _addcarry_u64(c2, r1, lo_p2, &mut r1);
    c2 = _addcarry_u64(c2, r2, hi_p2, &mut r2);
    _addcarry_u64(c2, r3, 0, &mut r3);

    let lo = (r1 as u128) << 64 | (lo_p0 as u128);
    let hi = (r3 as u128) << 64 | (r2 as u128);

    let lo_top = lo >> 127;
    let lo_low = lo & P;

    let sum_a = (hi << 1) + lo_low;
    let (sum1, carry2) = sum_a.overflowing_add(lo_top);

    let r = reduce(sum1) + (carry2 as u128) * 2;
    reduce(r)
}

#[inline(always)]
pub fn mul(a: u128, b: u128) -> u128 {
    #[cfg(all(target_arch = "x86_64", feature = "std"))]
    {
        if std::is_x86_feature_detected!("bmi2") {
            return unsafe { mul_bmi2(a, b) };
        }
        return mul_portable(a, b);
    }

    #[cfg(all(target_arch = "x86_64", not(feature = "std"), target_feature = "bmi2"))]
    {
        return unsafe { mul_bmi2(a, b) };
    }

    #[cfg(not(any(
        all(target_arch = "x86_64", feature = "std"),
        all(target_arch = "x86_64", not(feature = "std"), target_feature = "bmi2")
    )))]
    {
        return mul_portable(a, b);
    }
}
pub fn sq(a: u128) -> u128 {
    mul(a, a)
}

#[inline(always)]
pub fn pow5(a: u128) -> u128 {
    let a2 = sq(a);
    let a4 = sq(a2);
    mul(a4, a)
}

pub fn powd_binary_fallback(a: u128) -> u128 {
    if a == 0 { return 0; }
    let d: u128 = 136112946768375385385349842972707284581; // 127 bits
    let mut res = 1u128;
    let mut base = a;
    let mut exp = d;
    while exp > 0 {
        if exp & 1 == 1 {
            res = mul(res, base);
        }
        base = sq(base);
        exp >>= 1;
    }
    res
}

pub fn powd(a: u128) -> u128 {
    // We compute a^d where d = 0x66666666666666666666666666666665
    // Zero propagates mathematically perfectly through mul and sq.
    // If a == 0, then a^2 == 0, and all subsequent multiplications return 0.
    
    // t1 = a^6
    let a2 = sq(a);
    let a4 = sq(a2);
    let t1 = mul(a4, a2);
    
    // a5 = a^5
    let a5 = mul(a4, a);

    // t2 = t1^(2^4) * t1
    let mut t2 = t1;
    for _ in 0..4 { t2 = sq(t2); }
    t2 = mul(t2, t1);
    
    // t4 = t2^(2^8) * t2
    let mut t4 = t2;
    for _ in 0..8 { t4 = sq(t4); }
    t4 = mul(t4, t2);

    // t8 = t4^(2^16) * t4
    let mut t8 = t4;
    for _ in 0..16 { t8 = sq(t8); }
    t8 = mul(t8, t4);

    // t16 = t8^(2^32) * t8
    let mut t16 = t8;
    for _ in 0..32 { t16 = sq(t16); }
    t16 = mul(t16, t8);

    // t24 = t16^(2^32) * t8
    let mut t24 = t16;
    for _ in 0..32 { t24 = sq(t24); }
    t24 = mul(t24, t8);
    
    // t28 = t24^(2^16) * t4
    let mut t28 = t24;
    for _ in 0..16 { t28 = sq(t28); }
    t28 = mul(t28, t4);

    // t30 = t28^(2^8) * t2
    let mut t30 = t28;
    for _ in 0..8 { t30 = sq(t30); }
    t30 = mul(t30, t2);

    // t31 = t30^(2^4) * t1
    let mut t31 = t30;
    for _ in 0..4 { t31 = sq(t31); }
    t31 = mul(t31, t1);

    // final: d = t31^(2^4) * a^5
    let mut d_val = t31;
    for _ in 0..4 { d_val = sq(d_val); }
    d_val = mul(d_val, a5);

    d_val
}
