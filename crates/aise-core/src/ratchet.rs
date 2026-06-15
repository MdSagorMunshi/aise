use crate::sponge;
extern crate alloc;
use alloc::vec::Vec;

pub fn aise_ratchet_init(initial_secret: &[u8]) -> Vec<u8> {
    sponge::aise_hash_domain(initial_secret, 0x0A, 128)
}

pub fn aise_ratchet_step(mut state_i: Vec<u8>) -> (Vec<u8>, Vec<u8>) {
    let combined = sponge::aise_xof_domain(&state_i, 0x0A, 192);
    
    // Explicitly zero out the input state for forward secrecy
    for byte in state_i.iter_mut() {
        *byte = 0;
    }
    // state_i is dropped here

    let state_next = combined[0..128].to_vec();
    let output_key = combined[128..192].to_vec();

    (state_next, output_key)
}
