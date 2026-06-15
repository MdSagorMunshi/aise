//! AISE-DUPLEX

extern crate alloc;
use alloc::vec::Vec;
use crate::state::State;
use crate::sponge;
use crate::permute;

pub struct DuplexState {
    pub s: State,
}

impl DuplexState {
    pub fn new(key: &[u8], nonce: &[u8]) -> Self {
        let mut s = sponge::init(0x20, 0);
        
        let mut kn = Vec::with_capacity(key.len() + nonce.len());
        kn.extend_from_slice(key);
        kn.extend_from_slice(nonce);
        
        let padded_kn = crate::padding::pad(&kn, 0x20);
        for chunk in padded_kn.chunks_exact(1024) {
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
        
        Self { s }
    }
    
    pub fn encrypt(&mut self, plaintext: &[u8]) -> Vec<u8> {
        assert!(plaintext.len() <= 1024, "Block must be <= 1024 bytes");
        
        let mut keystream = Vec::with_capacity(plaintext.len());
        for j in 0..64 {
            keystream.extend_from_slice(&self.s.lanes[j].hi.to_be_bytes());
            keystream.extend_from_slice(&self.s.lanes[j].lo.to_be_bytes());
        }
        keystream.truncate(plaintext.len());
        
        let mut ciphertext = Vec::with_capacity(plaintext.len());
        for i in 0..plaintext.len() {
            ciphertext.push(plaintext[i] ^ keystream[i]);
        }
        
        let mut cblk = [0u8; 1024];
        cblk[..ciphertext.len()].copy_from_slice(&ciphertext);
        
        for j in 0..64 {
            let start = j * 16;
            let mut hi = [0u8; 8];
            let mut lo = [0u8; 8];
            hi.copy_from_slice(&cblk[start..start+8]);
            lo.copy_from_slice(&cblk[start+8..start+16]);
            self.s.lanes[j].hi ^= u64::from_be_bytes(hi);
            self.s.lanes[j].lo ^= u64::from_be_bytes(lo);
        }
        
        permute::permute(&mut self.s);
        ciphertext
    }
    
    pub fn decrypt(&mut self, ciphertext: &[u8]) -> Vec<u8> {
        assert!(ciphertext.len() <= 1024, "Block must be <= 1024 bytes");
        
        let mut keystream = Vec::with_capacity(ciphertext.len());
        for j in 0..64 {
            keystream.extend_from_slice(&self.s.lanes[j].hi.to_be_bytes());
            keystream.extend_from_slice(&self.s.lanes[j].lo.to_be_bytes());
        }
        keystream.truncate(ciphertext.len());
        
        let mut plaintext = Vec::with_capacity(ciphertext.len());
        for i in 0..ciphertext.len() {
            plaintext.push(ciphertext[i] ^ keystream[i]);
        }
        
        let mut cblk = [0u8; 1024];
        cblk[..ciphertext.len()].copy_from_slice(&ciphertext);
        
        for j in 0..64 {
            let start = j * 16;
            let mut hi = [0u8; 8];
            let mut lo = [0u8; 8];
            hi.copy_from_slice(&cblk[start..start+8]);
            lo.copy_from_slice(&cblk[start+8..start+16]);
            self.s.lanes[j].hi ^= u64::from_be_bytes(hi);
            self.s.lanes[j].lo ^= u64::from_be_bytes(lo);
        }
        
        permute::permute(&mut self.s);
        plaintext
    }
    
    pub fn finalize(&mut self, tag_len: usize) -> Vec<u8> {
        sponge::squeeze(&mut self.s, tag_len)
    }
}
