//! AISE-TREE

extern crate alloc;
use alloc::vec::Vec;
use crate::sponge;

const LEAF_SIZE: usize = 1024 * 1024; // 1 MiB

fn aise_hash_tree_node(m: &[u8], domain: u8, output_len: usize, ext: &[(usize, u64, u64)]) -> Vec<u8> {
    let mut s = sponge::init_with_ext(domain, output_len as u64, ext);
    sponge::absorb(&mut s, m, domain);
    sponge::squeeze(&mut s, output_len)
}

fn aise_xof_tree_node(m: &[u8], domain: u8, output_len: usize, ext: &[(usize, u64, u64)]) -> Vec<u8> {
    let mut s = sponge::init_with_ext(domain, 0, ext); // 0 output_len for XOF in IV
    sponge::absorb(&mut s, m, domain);
    sponge::squeeze(&mut s, output_len)
}

pub fn aise_tree(m: &[u8], output_len: usize) -> Vec<u8> {
    if m.len() <= LEAF_SIZE {
        return aise_hash_tree_node(m, 0x10, output_len, &[(66, 0, 1)]); // leaf_index=0, depth=0, total_leaves=1
    }

    let chunks: Vec<&[u8]> = m.chunks(LEAF_SIZE).collect();
    let l = chunks.len();
    let mut leaf_digests = Vec::with_capacity(l);
    
    for (i, chunk) in chunks.iter().enumerate() {
        let ext = [(66, i as u64, l as u64)]; // leaf_index=i, depth=0, total_leaves=l
        leaf_digests.push(aise_hash_tree_node(chunk, 0x10, 64, &ext));
    }
    
    let mut level = leaf_digests;
    let mut depth = 1u64;
    
    while level.len() > 1 {
        let mut next_level = Vec::new();
        for group in level.chunks(16) {
            let mut concat = Vec::with_capacity(group.len() * 64);
            for d in group {
                concat.extend_from_slice(d);
            }
            let ext = [(66, depth, group.len() as u64)]; // depth, num_children
            next_level.push(aise_hash_tree_node(&concat, 0x11, 64, &ext));
        }
        level = next_level;
        depth += 1;
    }
    
    let root = &level[0];
    if output_len == 64 {
        root.clone()
    } else {
        aise_xof_tree_node(root, 0x11, output_len, &[(66, depth, 1)])
    }
}
