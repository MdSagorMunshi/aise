//! Multi-Rate Padding

extern crate alloc;
use alloc::vec::Vec;

pub fn pad(m: &[u8], domain: u8) -> Vec<u8> {
    let mut padded = Vec::with_capacity(m.len() + 1024);
    padded.extend_from_slice(m);
    padded.push(0x01); // start
    padded.push(domain);
    padded.push(0x01); // version
    
    let current_len = padded.len();
    let zeros = (1023 - (current_len % 1024)) % 1024;
    
    for _ in 0..zeros {
        padded.push(0x00);
    }
    padded.push(0x80); // end
    
    padded
}
