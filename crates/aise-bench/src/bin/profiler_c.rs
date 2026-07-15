use aise_core::field_p;
use aise_core::pi_c;
use aise_core::mds_c;
use aise_core::sbox_c;
use std::arch::x86_64::_rdtsc;

fn main() {
    println!("--- Pi_C Coarse Profiling ---");

    let mut a: u128 = 0xDEADBEEFCAFEBABE0123456789ABCDEF;
    let mut b: u128 = 0xCAFEBABEDEADBEEFFEDCBA9876543210;
    a &= field_p::P;
    b &= field_p::P;

    // Warmup
    for _ in 0..1000 {
        a = field_p::mul(a, b);
        b = field_p::sq(b);
    }

    let iters = 1_000_000;
    
    // 1. Measure mul() and sq() cycles
    let start_mul = unsafe { _rdtsc() };
    for _ in 0..iters {
        a = field_p::mul(a, b);
    }
    let end_mul = unsafe { _rdtsc() };
    
    let start_sq = unsafe { _rdtsc() };
    for _ in 0..iters {
        b = field_p::sq(b);
    }
    let end_sq = unsafe { _rdtsc() };
    
    std::hint::black_box(a);
    std::hint::black_box(b);

    let mul_cyc = (end_mul - start_mul) as f64 / iters as f64;
    let sq_cyc = (end_sq - start_sq) as f64 / iters as f64;
    println!("mul: {:.2} cycles/call", mul_cyc);
    println!("sq: {:.2} cycles/call", sq_cyc);

    // 2. Measure pow5() and powd()
    let start_pow5 = unsafe { _rdtsc() };
    for _ in 0..iters {
        a = field_p::pow5(a);
    }
    let end_pow5 = unsafe { _rdtsc() };

    let powd_iters = 100_000;
    let start_powd = unsafe { _rdtsc() };
    for _ in 0..powd_iters {
        a = field_p::powd(a);
    }
    let end_powd = unsafe { _rdtsc() };

    std::hint::black_box(a);

    let pow5_cyc = (end_pow5 - start_pow5) as f64 / iters as f64;
    let powd_cyc = (end_powd - start_powd) as f64 / powd_iters as f64;
    println!("pow5 (odd rounds): {:.2} cycles/call", pow5_cyc);
    println!("powd (even rounds): {:.2} cycles/call", powd_cyc);

    // 3. Measure mix_lanes (MDS)
    let mut lanes = [0u128; 128];
    for i in 0..128 { lanes[i] = (i as u128 + 1) & field_p::P; }
    
    let mds_iters = 10_000;
    let start_mds = unsafe { _rdtsc() };
    for _ in 0..mds_iters {
        mds_c::mix_lanes(&mut lanes);
    }
    let end_mds = unsafe { _rdtsc() };
    std::hint::black_box(lanes);

    let mds_cyc = (end_mds - start_mds) as f64 / mds_iters as f64;
    println!("mix_lanes: {:.2} cycles/call", mds_cyc);
    
    // 4. Breakdown of Pi_C
    let start_pic = unsafe { _rdtsc() };
    for _ in 0..mds_iters {
        pi_c::pi_c(&mut lanes);
    }
    let end_pic = unsafe { _rdtsc() };
    std::hint::black_box(lanes);
    
    let pic_cycles = (end_pic - start_pic) as f64 / mds_iters as f64;
    println!("pi_c (32 rounds): {:.2} cycles/call", pic_cycles);
    
    let avg_round_cycles = pic_cycles / 32.0;
    let avg_sbox_budget = 128.0 * ((pow5_cyc + powd_cyc) / 2.0);
    
    println!("  -- Per Round Breakdown (Average) --");
    println!("  Average Total Round: {:.2} cycles", avg_round_cycles);
    println!("  S-Box (128x pow5 or powd): {:.2} cycles ({:.1}%)", avg_sbox_budget, 100.0 * avg_sbox_budget / avg_round_cycles);
    println!("  MDS Mixing: {:.2} cycles ({:.1}%)", mds_cyc, 100.0 * mds_cyc / avg_round_cycles);
}
