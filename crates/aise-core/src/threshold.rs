use crate::sponge;
use crate::permute;
use crate::padding;
extern crate alloc;
use alloc::vec::Vec;

pub fn aise_threshold_mac(keys: &[&[u8]], message: &[u8], output_len: usize) -> Vec<u8> {
    let n = keys.len() as u32;
    let extensions = [(66, 0u64, n as u64)];
    let mut s = sponge::init_with_ext(0x0D, output_len as u64, &extensions);

    for key in keys {
        let padded_key = padding::pad(key, 0x0D);
        for chunk in padded_key.chunks_exact(1024) {
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
    }

    sponge::absorb(&mut s, message, 0x0D);
    sponge::squeeze(&mut s, output_len)
}
