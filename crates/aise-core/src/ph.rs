use crate::sponge;
use crate::permute;
use crate::padding;
extern crate alloc;
use alloc::vec::Vec;

pub fn aise_ph(password: &[u8], salt: &[u8], cost: u32, output_len: usize) -> Vec<u8> {
    // 1. INIT(0x08, output_len) with `cost` in Lane 66
    let extensions = [(66, 0u64, cost as u64)];
    let mut s = sponge::init_with_ext(0x08, output_len as u64, &extensions);

    // 2. K' <- pad(salt || password, 0x08); full-state-absorb
    let mut ikm = Vec::with_capacity(salt.len() + password.len());
    ikm.extend_from_slice(salt);
    ikm.extend_from_slice(password);

    let padded = padding::pad(&ikm, 0x08);
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
            // Also into capacity!
            s.lanes[64 + j].hi ^= hi_val;
            s.lanes[64 + j].lo ^= lo_val;
        }
        permute::permute(&mut s);
    }

    // 3. repeat `cost` times
    for _ in 0..cost {
        permute::permute(&mut s);
    }

    // 4. SQUEEZE
    sponge::squeeze(&mut s, output_len)
}
