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
pub fn mul(a: u128, b: u128) -> u128 {
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

#[inline(always)]
pub fn sq(a: u128) -> u128 {
    mul(a, a)
}

#[inline(always)]
pub fn pow5(a: u128) -> u128 {
    let a2 = sq(a);
    let a4 = sq(a2);
    mul(a4, a)
}

pub fn powd(a: u128) -> u128 {
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
