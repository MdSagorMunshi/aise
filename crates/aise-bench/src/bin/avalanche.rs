use aise_core::state::Lane;
use aise_core::pi_b;

fn count_set_bits(a: Lane, b: Lane) -> u32 {
    let xor_hi = a.hi ^ b.hi;
    let xor_lo = a.lo ^ b.lo;
    xor_hi.count_ones() + xor_lo.count_ones()
}

fn main() {
    let mut lanes = [Lane::new(0, 0); 128];
    for i in 0..128 {
        lanes[i] = Lane::new(
            (i as u64 + 1).wrapping_mul(0xDEADBEEFCAFEBABE),
            (i as u64 + 1).wrapping_mul(0x0123456789ABCDEF),
        );
    }
    
    // Baseline (same as frozen vector)
    let mut baseline = lanes;
    pi_b::pi_b(&mut baseline);

    // Flip exactly 1 bit in lane 0 (LSB)
    lanes[0].lo ^= 1;
    
    // Test
    let mut test_out = lanes;
    pi_b::pi_b(&mut test_out);

    let mut total_bits_flipped = 0;
    for i in 0..128 {
        total_bits_flipped += count_set_bits(baseline[i], test_out[i]);
    }

    let total_bits = 128 * 128;
    let percentage = (total_bits_flipped as f64 / total_bits as f64) * 100.0;
    
    println!("--- Avalanche Criterion Spot-Check (1-bit diff) ---");
    println!("Total bits: {}", total_bits);
    println!("Bits flipped: {}", total_bits_flipped);
    println!("Percentage: {:.2}%", percentage);
}
