//! Permutation A (Pi_A)

use crate::state::Lane;
use crate::constants::{RC_A, SIGMA_A};

#[inline(always)]
fn sub_word_a(lane: Lane, rc0: Lane) -> Lane {
    let mut a = (lane.hi >> 32) as u32;
    let mut b = (lane.hi & 0xFFFFFFFF) as u32;
    let mut c = (lane.lo >> 32) as u32;
    let mut d = (lane.lo & 0xFFFFFFFF) as u32;

    for _ in 0..8 {
        a = a.wrapping_add(b).rotate_left(13) ^ c;
        b = (b ^ c).rotate_left(19).wrapping_add(d);
        c = c.wrapping_add(d).rotate_left(23) ^ a;
        d = (d ^ a).rotate_left(7).wrapping_add(b);
    }

    a ^= (rc0.hi >> 32) as u32;
    b ^= (rc0.hi & 0xFFFFFFFF) as u32;

    Lane::new(
        ((a as u64) << 32) | (b as u64),
        ((c as u64) << 32) | (d as u64)
    )
}

#[inline(always)]
fn mix_pair_a(mut a: Lane, mut b: Lane) -> (Lane, Lane) {
    a.hi = a.hi.wrapping_add(b.hi).rotate_left(26);
    a.lo = a.lo.wrapping_add(b.lo).rotate_left(26);
    b.hi = (b.hi ^ a.hi).rotate_left(39);
    b.lo = (b.lo ^ a.lo).rotate_left(39);
    
    a.hi = a.hi.wrapping_add(b.hi).rotate_left(46);
    a.lo = a.lo.wrapping_add(b.lo).rotate_left(46);
    b.hi = (b.hi ^ a.hi).rotate_left(19);
    b.lo = (b.lo ^ a.lo).rotate_left(19);

    a.hi = a.hi.wrapping_add(b.hi).rotate_left(13);
    a.lo = a.lo.wrapping_add(b.lo).rotate_left(13);
    b.hi = (b.hi ^ a.hi).rotate_left(41);
    b.lo = (b.lo ^ a.lo).rotate_left(41);

    a.hi = a.hi.wrapping_add(b.hi).rotate_left(15);
    a.lo = a.lo.wrapping_add(b.lo).rotate_left(15);
    b.hi = (b.hi ^ a.hi).rotate_left(35);
    b.lo = (b.lo ^ a.lo).rotate_left(35);

    (a, b)
}

pub fn pi_a(lanes: &mut [Lane; 128]) {
    for r in 0..32 {
        let rc0 = Lane::new(RC_A[r][0].0, RC_A[r][0].1);
        for i in 0..128 {
            lanes[i] = sub_word_a(lanes[i], rc0);
        }

        for k in (0..128).step_by(2) {
            let (x, y) = mix_pair_a(lanes[k], lanes[k+1]);
            lanes[k] = x;
            lanes[k+1] = y;
        }

        for k in (1..127).step_by(2) {
            let (x, y) = mix_pair_a(lanes[k], lanes[k+1]);
            lanes[k] = x;
            lanes[k+1] = y;
        }
        let (x, y) = mix_pair_a(lanes[127], lanes[0]);
        lanes[127] = x;
        lanes[0] = y;

        let mut next = [Lane::new(0, 0); 128];
        for i in 0..128 {
            next[i] = lanes[SIGMA_A[i]];
        }
        
        for i in 0..128 {
            next[i].hi ^= RC_A[r][i].0;
            next[i].lo ^= RC_A[r][i].1;
        }
        
        *lanes = next;
    }
}
