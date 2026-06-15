use crate::sponge;
extern crate alloc;
use alloc::vec::Vec;

pub fn aise_personalized_hash(context: &[u8], message: &[u8], output_len: usize) -> Vec<u8> {
    let mut s = sponge::init(0x05, output_len as u64);
    sponge::absorb(&mut s, context, 0x05);
    sponge::absorb(&mut s, message, 0x05);
    sponge::squeeze(&mut s, output_len)
}
