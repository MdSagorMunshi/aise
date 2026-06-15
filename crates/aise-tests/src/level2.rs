use aise_core::state::{Lane, State};
use aise_core::constants::*;
use aise_core::{sbox_b, sbox_c, field_p};
use rand::Rng;
use std::collections::HashSet;

fn rand_lane() -> Lane {
    let mut rng = rand::thread_rng();
    Lane::new(rng.r#gen(), rng.r#gen())
}

#[test]
fn test_sigma_properties() {
    // Check bijectivity and no fixed points for SIGMA_A, SIGMA_B, SIGMA_C
    for sigma in &[SIGMA_A, SIGMA_B, SIGMA_C] {
        let mut seen = HashSet::new();
        for i in 0..128 {
            let mapped = sigma[i];
            assert!(mapped < 128);
            assert_ne!(mapped, i, "Fixed point found at {}", i);
            seen.insert(mapped);
        }
        assert_eq!(seen.len(), 128, "Not bijective");
    }
}

#[test]
fn test_subfield_c_mutual_inverse() {
    let mut rng = rand::thread_rng();
    for _ in 0..10000 {
        let mut val: u128 = rng.r#gen();
        val %= field_p::P;

        // Even rounds use InverseSBox_C, Odd rounds use ForwardSBox_C
        let forward = sbox_c::apply(val, 1); // r=1 is odd -> Forward
        let inverse = sbox_c::apply(forward, 0); // r=0 is even -> Inverse
        assert_eq!(inverse, val);

        let inverse2 = sbox_c::apply(val, 0); // Inverse
        let forward2 = sbox_c::apply(inverse2, 1); // Forward
        assert_eq!(forward2, val);
    }
}

#[test]
fn test_sbox_b_no_fixed_points() {
    for _ in 0..10000 {
        let val = rand_lane();
        let mapped = sbox_b::apply(val);
        assert_ne!(mapped, val, "Fixed point in SBox B");
    }
}

#[test]
fn test_subword_a_invertible() {
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

        Lane::new(((a as u64) << 32) | (b as u64), ((c as u64) << 32) | (d as u64))
    }

    fn inv_sub_word_a(lane: Lane, rc0: Lane) -> Lane {
        let mut a = (lane.hi >> 32) as u32;
        let mut b = (lane.hi & 0xFFFFFFFF) as u32;
        let mut c = (lane.lo >> 32) as u32;
        let mut d = (lane.lo & 0xFFFFFFFF) as u32;

        a ^= (rc0.hi >> 32) as u32;
        b ^= (rc0.hi & 0xFFFFFFFF) as u32;

        for _ in 0..8 {
            d = d.wrapping_sub(b).rotate_right(7) ^ a;
            c = (c ^ a).rotate_right(23).wrapping_sub(d);
            b = b.wrapping_sub(d).rotate_right(19) ^ c;
            a = (a ^ c).rotate_right(13).wrapping_sub(b);
        }

        Lane::new(((a as u64) << 32) | (b as u64), ((c as u64) << 32) | (d as u64))
    }

    for _ in 0..10000 {
        let val = rand_lane();
        let rc0 = rand_lane();
        let mapped = sub_word_a(val, rc0);
        let inverted = inv_sub_word_a(mapped, rc0);
        assert_eq!(inverted, val);
    }
}

#[test]
fn test_mix_pair_a_invertible() {
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

    fn inv_mix_pair_a(mut a: Lane, mut b: Lane) -> (Lane, Lane) {
        b.hi = b.hi.rotate_right(35) ^ a.hi;
        b.lo = b.lo.rotate_right(35) ^ a.lo;
        a.hi = a.hi.rotate_right(15).wrapping_sub(b.hi);
        a.lo = a.lo.rotate_right(15).wrapping_sub(b.lo);

        b.hi = b.hi.rotate_right(41) ^ a.hi;
        b.lo = b.lo.rotate_right(41) ^ a.lo;
        a.hi = a.hi.rotate_right(13).wrapping_sub(b.hi);
        a.lo = a.lo.rotate_right(13).wrapping_sub(b.lo);

        b.hi = b.hi.rotate_right(19) ^ a.hi;
        b.lo = b.lo.rotate_right(19) ^ a.lo;
        a.hi = a.hi.rotate_right(46).wrapping_sub(b.hi);
        a.lo = a.lo.rotate_right(46).wrapping_sub(b.lo);

        b.hi = b.hi.rotate_right(39) ^ a.hi;
        b.lo = b.lo.rotate_right(39) ^ a.lo;
        a.hi = a.hi.rotate_right(26).wrapping_sub(b.hi);
        a.lo = a.lo.rotate_right(26).wrapping_sub(b.lo);

        (a, b)
    }

    let mut rng = rand::thread_rng();
    for _ in 0..10000 {
        let val_a = Lane::new(rng.r#gen(), rng.r#gen());
        let val_b = Lane::new(rng.r#gen(), rng.r#gen());
        let (ma, mb) = mix_pair_a(val_a, val_b);
        let (ia, ib) = inv_mix_pair_a(ma, mb);
        assert_eq!(ia, val_a);
        assert_eq!(ib, val_b);
    }
}
