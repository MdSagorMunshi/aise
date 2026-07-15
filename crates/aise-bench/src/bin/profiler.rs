use aise_core::state::{Lane, State};
use aise_core::{field_b, mds_b};
use std::arch::x86_64::_rdtsc;

fn main() {
    println!("--- Coarse Profiling ---");

    // 1. Measure mul() and sq() cycles
    let mut a = Lane::new(0xDEADBEEFCAFEBABE, 0x0123456789ABCDEF);
    let mut b = Lane::new(0xCAFEBABEDEADBEEF, 0xFEDCBA9876543210);
    
    // Warmup
    for _ in 0..1000 {
        a = field_b::mul(a, b);
        b = field_b::sq(b);
    }

    let iters = 1_000_000;
    
    let start_mul = unsafe { _rdtsc() };
    for _ in 0..iters {
        a = field_b::mul(a, b);
    }
    let end_mul = unsafe { _rdtsc() };
    
    let start_sq = unsafe { _rdtsc() };
    for _ in 0..iters {
        b = field_b::sq(b);
    }
    let end_sq = unsafe { _rdtsc() };
    
    // Avoid optimization out
    std::hint::black_box(a);
    std::hint::black_box(b);

    println!("mul_clmul: {:.2} cycles/call", (end_mul - start_mul) as f64 / iters as f64);
    println!("sq_clmul: {:.2} cycles/call", (end_sq - start_sq) as f64 / iters as f64);

    // 3. Measure batch_inv cycles
    let mut batch_lanes = [Lane::new(1, 2); 128];
    let start_inv = unsafe { _rdtsc() };
    for _ in 0..iters {
        field_b::batch_inv(&mut batch_lanes);
    }
    let end_inv = unsafe { _rdtsc() };
    std::hint::black_box(batch_lanes);
    
    let batch_inv_cycles = (end_inv - start_inv) as f64 / iters as f64;
    println!("batch_inv: {:.2} cycles/call", batch_inv_cycles);

    // 3. Measure mix_lanes (MDS) cycles
    let mut lanes = [Lane::new(0,0); 128];
    for i in 0..128 { lanes[i] = Lane::new(i as u64, i as u64); }
    
    let mds_iters = 10_000;
    let start_mds = unsafe { _rdtsc() };
    for _ in 0..mds_iters {
        mds_b::mix_lanes(&mut lanes);
    }
    let end_mds = unsafe { _rdtsc() };
    std::hint::black_box(lanes);

    println!("mix_lanes: {:.2} cycles/call", (end_mds - start_mds) as f64 / mds_iters as f64);
    
    // 4. Breakdown of Pi_B
    let start_pib = unsafe { _rdtsc() };
    for _ in 0..mds_iters {
        aise_core::pi_b::pi_b(&mut lanes);
    }
    let end_pib = unsafe { _rdtsc() };
    std::hint::black_box(lanes);
    
    let pib_cycles = (end_pib - start_pib) as f64 / mds_iters as f64;
    println!("pi_b (32 rounds): {:.2} cycles/call", pib_cycles);
    
    let single_round_cycles = pib_cycles / 32.0;
    let inv_budget = batch_inv_cycles;
    let mds_budget = (end_mds - start_mds) as f64 / mds_iters as f64;
    let sbox_overhead = single_round_cycles - inv_budget - mds_budget;
    
    println!("  -- Per Round Breakdown --");
    println!("  Total Round: {:.2} cycles", single_round_cycles);
    println!("  Batch Inversion: {:.2} cycles ({:.1}%)", inv_budget, 100.0 * inv_budget / single_round_cycles);
    println!("  MDS Mixing: {:.2} cycles ({:.1}%)", mds_budget, 100.0 * mds_budget / single_round_cycles);
    println!("  Other (Overhead/Conversion): {:.2} cycles ({:.1}%)", sbox_overhead, 100.0 * sbox_overhead / single_round_cycles);
}
