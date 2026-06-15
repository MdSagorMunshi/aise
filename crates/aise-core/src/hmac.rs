use crate::sponge;
extern crate alloc;
use alloc::vec;
use alloc::vec::Vec;

const B: usize = 1024; // Rate block size

pub fn aise_hmac(key: &[u8], message: &[u8], output_len: usize) -> Vec<u8> {
    let mut k_prime = vec![0u8; B];
    if key.len() > B {
        let xof_k = sponge::aise_xof(key, B);
        k_prime.copy_from_slice(&xof_k);
    } else {
        k_prime[..key.len()].copy_from_slice(key);
    }

    let mut ipad = vec![0x36; B];
    let mut opad = vec![0x5c; B];

    for i in 0..B {
        ipad[i] ^= k_prime[i];
        opad[i] ^= k_prime[i];
    }

    let mut inner_input = Vec::with_capacity(B + message.len());
    inner_input.extend_from_slice(&ipad);
    inner_input.extend_from_slice(message);
    let inner = sponge::aise_hash_domain(&inner_input, 0x03, 64);

    let mut outer_input = Vec::with_capacity(B + 64);
    outer_input.extend_from_slice(&opad);
    outer_input.extend_from_slice(&inner);
    sponge::aise_hash_domain(&outer_input, 0x04, output_len)
}
