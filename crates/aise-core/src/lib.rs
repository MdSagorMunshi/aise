#![no_std]

extern crate alloc;

pub mod state;
pub mod constants;
pub mod field8;
pub mod field16;
pub mod field_b;
pub mod field_p;
pub mod sbox_b;
pub mod sbox_c;
pub mod mds_b;
pub mod mds_c;
pub mod pi_a;
pub mod pi_b;
pub mod pi_c;
pub mod permute;
pub mod padding;
pub mod sponge;
pub mod mac;
pub mod kdf;
pub mod tree;
pub mod duplex;
pub mod hmac;
pub mod ph;
pub mod commit;
pub mod prf;
pub mod ratchet;
pub mod threshold;
pub mod personalized;


pub use state::{Lane, State};
pub use permute::permute;
pub use sponge::{aise_hash, aise_xof};
pub use mac::aise_mac;
pub use kdf::aise_kdf;
pub use tree::aise_tree;
pub use duplex::DuplexState;
