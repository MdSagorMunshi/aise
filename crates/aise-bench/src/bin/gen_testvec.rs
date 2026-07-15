//! Generates frozen test vector for pi_b by running the current implementation
//! on a deterministic non-zero input and outputting a complete Rust test module.

use aise_core::state::Lane;
use aise_core::pi_b;

fn main() {
    // Deterministic non-zero input: each lane is distinct and exercises various bit patterns
    let mut lanes = [Lane::new(0, 0); 128];
    for i in 0..128 {
        lanes[i] = Lane::new(
            (i as u64 + 1).wrapping_mul(0xDEADBEEFCAFEBABE),
            (i as u64 + 1).wrapping_mul(0x0123456789ABCDEF),
        );
    }

    eprintln!("Running pi_b (32 rounds × 128 GF(2^128) lanes)...");
    pi_b::pi_b(&mut lanes);
    eprintln!("Done.");

    // Output a complete Rust source file
    print!("//! Frozen test vector for pi_b.\n");
    print!("//! Generated from unmodified codebase — DO NOT EDIT.\n");
    print!("//!\n");
    print!("//! Input: Lane::new((i+1).wrapping_mul(0xDEADBEEFCAFEBABE), (i+1).wrapping_mul(0x0123456789ABCDEF))\n");
    print!("//! for i in 0..128\n");
    print!("\n");
    print!("use aise_core::state::Lane;\n");
    print!("use aise_core::pi_b;\n");
    print!("\n");
    print!("#[allow(clippy::unreadable_literal)]\n");
    print!("const PI_B_FROZEN_OUTPUT: [(u64, u64); 128] = [\n");
    for lane in lanes.iter() {
        print!("    ({:#018x}, {:#018x}),\n", lane.hi, lane.lo);
    }
    print!("];\n");
    print!("\n");
    print!("#[test]\n");
    print!("fn test_pi_b_frozen_vector() {{\n");
    print!("    let mut lanes = [Lane::new(0, 0); 128];\n");
    print!("    for i in 0..128 {{\n");
    print!("        lanes[i] = Lane::new(\n");
    print!("            (i as u64 + 1).wrapping_mul(0xDEADBEEFCAFEBABE),\n");
    print!("            (i as u64 + 1).wrapping_mul(0x0123456789ABCDEF),\n");
    print!("        );\n");
    print!("    }}\n");
    print!("    pi_b::pi_b(&mut lanes);\n");
    print!("    for i in 0..128 {{\n");
    print!("        assert_eq!(\n");
    print!("            (lanes[i].hi, lanes[i].lo),\n");
    print!("            PI_B_FROZEN_OUTPUT[i],\n");
    print!("            \"Pi_B frozen vector mismatch at lane {{}}\", i\n");
    print!("        );\n");
    print!("    }}\n");
    print!("}}\n");
}
