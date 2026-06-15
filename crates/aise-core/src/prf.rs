use crate::mac::aise_mac_internal;
extern crate alloc;
use alloc::vec::Vec;

pub fn aise_prf(key: &[u8], input: &[u8], output_len: usize) -> Vec<u8> {
    aise_mac_internal(key, input, output_len, 0x09)
}
