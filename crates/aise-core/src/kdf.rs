//! AISE-KDF

extern crate alloc;
use alloc::vec::Vec;
use crate::mac::aise_mac_internal;
use crate::sponge;
use crate::permute;

pub fn aise_kdf_extract(salt: &[u8], ikm: &[u8]) -> Vec<u8> {
    aise_mac_internal(salt, ikm, 64, 0x06)
}

pub fn aise_kdf_expand(prk: &[u8], info: &[u8], output_len: usize) -> Vec<u8> {
    let mut s = sponge::init(0x07, output_len as u64);
    
    let padded_prk = crate::padding::pad(prk, 0x07);
    for chunk in padded_prk.chunks_exact(1024) {
        let mut kblk = [0u64; 128];
        for j in 0..64 {
            let start = j * 16;
            let mut hi = [0u8; 8];
            let mut lo = [0u8; 8];
            hi.copy_from_slice(&chunk[start..start+8]);
            lo.copy_from_slice(&chunk[start+8..start+16]);
            kblk[j*2] = u64::from_be_bytes(hi);
            kblk[j*2+1] = u64::from_be_bytes(lo);
        }
        
        for j in 0..64 {
            s.lanes[j].hi ^= kblk[j*2];
            s.lanes[j].lo ^= kblk[j*2+1];
            s.lanes[64+j].hi ^= kblk[j*2];
            s.lanes[64+j].lo ^= kblk[j*2+1];
        }
        permute::permute(&mut s);
    }
    
    sponge::absorb(&mut s, info, 0x07);
    sponge::squeeze(&mut s, output_len)
}

pub fn aise_kdf(salt: &[u8], ikm: &[u8], info: &[u8], output_len: usize) -> Vec<u8> {
    let prk = aise_kdf_extract(salt, ikm);
    aise_kdf_expand(&prk, info, output_len)
}
