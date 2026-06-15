//! Core Sponge Operations

use crate::state::State;
use crate::permute;
use crate::padding;
extern crate alloc;
use alloc::vec::Vec;

pub fn init_with_ext(domain: u8, output_len: u64, extensions: &[(usize, u64, u64)]) -> State {
    let mut s = State::new();
    
    // IV
    s.lanes[64].hi = (128u64 << 48) | (128u64 << 32) | ((output_len & 0xFFFF) << 16) | ((domain as u64) << 8) | 0x01;
    s.lanes[64].lo = 0x414953452D4F4D45; // AISE-OME
    
    // L[65]
    s.lanes[65].hi = 0x47412D4F4D454741; // GA-OMEGA
    s.lanes[65].lo = 0x2D4F4D4547410000; // -OMEGA..

    for &(idx, hi, lo) in extensions {
        if idx >= 66 && idx < 128 {
            s.lanes[idx].hi = hi;
            s.lanes[idx].lo = lo;
        }
    }

    permute::permute(&mut s);
    s
}

pub fn init(domain: u8, output_len: u64) -> State {
    init_with_ext(domain, output_len, &[])
}

pub fn absorb(s: &mut State, m: &[u8], domain: u8) {
    let padded = padding::pad(m, domain);
    
    for chunk in padded.chunks_exact(1024) {
        for j in 0..64 {
            let start = j * 16;
            let mut hi = [0u8; 8];
            let mut lo = [0u8; 8];
            hi.copy_from_slice(&chunk[start..start+8]);
            lo.copy_from_slice(&chunk[start+8..start+16]);
            
            let hi_val = u64::from_be_bytes(hi);
            let lo_val = u64::from_be_bytes(lo);
            
            s.lanes[j].hi ^= hi_val;
            s.lanes[j].lo ^= lo_val;
        }
        permute::permute(s);
    }
}

pub fn squeeze(s: &mut State, output_len: usize) -> Vec<u8> {
    let mut out = Vec::with_capacity(output_len);
    
    while out.len() < output_len {
        for j in 0..64 {
            out.extend_from_slice(&s.lanes[j].hi.to_be_bytes());
            out.extend_from_slice(&s.lanes[j].lo.to_be_bytes());
        }
        if out.len() >= output_len {
            break;
        }
        permute::permute(s);
    }
    
    out.truncate(output_len);
    out
}

pub fn aise_hash_domain(m: &[u8], domain: u8, output_len: usize) -> Vec<u8> {
    let mut s = init(domain, output_len as u64);
    absorb(&mut s, m, domain);
    squeeze(&mut s, output_len)
}

pub fn aise_xof_domain(m: &[u8], domain: u8, output_len: usize) -> Vec<u8> {
    let mut s = init(domain, 0);
    absorb(&mut s, m, domain);
    squeeze(&mut s, output_len)
}

pub fn aise_hash(m: &[u8], output_len: usize) -> Vec<u8> {
    aise_hash_domain(m, 0x00, output_len)
}

pub fn aise_xof(m: &[u8], output_len: usize) -> Vec<u8> {
    aise_xof_domain(m, 0x01, output_len)
}
