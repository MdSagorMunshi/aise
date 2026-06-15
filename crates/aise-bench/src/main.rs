use std::time::Instant;
use aise_core::state::{Lane, State};
use aise_core::{permute, pi_a, pi_b, pi_c};

const ITERS: usize = 1_000;

fn benchmark_pi_a() {
    let mut lanes = [Lane::new(0, 0); 128];
    let start = Instant::now();
    for _ in 0..ITERS {
        pi_a::pi_a(&mut lanes);
    }
    let elapsed = start.elapsed();
    let throughput = (ITERS as f64 * 16384.0) / (elapsed.as_secs_f64() * 1024.0 * 1024.0 * 8.0);
    println!("Pi_A: {:.2} MB/s", throughput);
}

fn benchmark_pi_b() {
    let mut lanes = [Lane::new(0, 0); 128];
    let start = Instant::now();
    for _ in 0..ITERS {
        pi_b::pi_b(&mut lanes);
    }
    let elapsed = start.elapsed();
    let throughput = (ITERS as f64 * 16384.0) / (elapsed.as_secs_f64() * 1024.0 * 1024.0 * 8.0);
    println!("Pi_B: {:.2} MB/s", throughput);
}

fn benchmark_pi_c() {
    let mut f = [0u128; 128];
    let start = Instant::now();
    for _ in 0..ITERS {
        pi_c::pi_c(&mut f);
    }
    let elapsed = start.elapsed();
    let throughput = (ITERS as f64 * 16384.0) / (elapsed.as_secs_f64() * 1024.0 * 1024.0 * 8.0);
    println!("Pi_C: {:.2} MB/s", throughput);
}

fn benchmark_cascade() {
    let mut s = State::new();
    let start = Instant::now();
    for _ in 0..ITERS {
        permute::permute(&mut s);
    }
    let elapsed = start.elapsed();
    let throughput = (ITERS as f64 * 16384.0) / (elapsed.as_secs_f64() * 1024.0 * 1024.0 * 8.0);
    println!("Cascade Pi_Omega: {:.2} MB/s", throughput);
}

fn main() {
    println!("--- AISE Benchmarks ({} iterations) ---", ITERS);
    benchmark_pi_a();
    benchmark_pi_b();
    benchmark_pi_c();
    benchmark_cascade();
}
