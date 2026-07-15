//! Pi_B Round Profiler — Phase 4 Scoping
//! Isolates each component of a Pi_B round:
//!   1. S-box (batch_inv)
//!   2. MDS mixing (mix_lanes)  
//!   3. SIGMA_B permutation
//!   4. Affine (RC_B XOR)
//!   5. Full pi_b() for cross-validation

use aise_core::state::Lane;
use aise_core::constants::{RC_B, SIGMA_B};
use aise_core::{field_b, mds_b, sbox_b};
use std::arch::x86_64::_rdtsc;

fn main() {
    println!("--- Pi_B Per-Round Component Profiler ---");
    
    let iters = 10_000u64;
    let mut lanes = [Lane::new(0, 0); 128];
    for i in 0..128 { lanes[i] = Lane::new(i as u64 * 17 + 3, i as u64 * 31 + 7); }
    
    // Warmup
    for _ in 0..100 { aise_core::pi_b::pi_b(&mut lanes.clone()); }

    // 1. Full pi_b (32 rounds)
    let mut lanes_full = lanes;
    let start = unsafe { _rdtsc() };
    for _ in 0..iters {
        aise_core::pi_b::pi_b(&mut lanes_full);
    }
    let end = unsafe { _rdtsc() };
    std::hint::black_box(&lanes_full);
    let full_pi_b = (end - start) as f64 / iters as f64;
    let per_round = full_pi_b / 32.0;
    
    // 2. Isolated batch_inv (S-box)
    let mut lanes_inv = lanes;
    let start = unsafe { _rdtsc() };
    for _ in 0..iters {
        sbox_b::batch_apply(&mut lanes_inv);
    }
    let end = unsafe { _rdtsc() };
    std::hint::black_box(&lanes_inv);
    let sbox_cycles = (end - start) as f64 / iters as f64;

    // 3. Isolated MDS mixing
    let mut lanes_mds = lanes;
    let start = unsafe { _rdtsc() };
    for _ in 0..iters {
        mds_b::mix_lanes(&mut lanes_mds);
    }
    let end = unsafe { _rdtsc() };
    std::hint::black_box(&lanes_mds);
    let mds_cycles = (end - start) as f64 / iters as f64;
    
    // 4. Isolated SIGMA_B permutation
    let mut lanes_sigma = lanes;
    let start = unsafe { _rdtsc() };
    for _ in 0..iters {
        let mut next = [Lane::new(0, 0); 128];
        for i in 0..128 {
            next[i] = lanes_sigma[SIGMA_B[i]];
        }
        lanes_sigma = next;
    }
    let end = unsafe { _rdtsc() };
    std::hint::black_box(&lanes_sigma);
    let sigma_cycles = (end - start) as f64 / iters as f64;

    // 5. Isolated Affine (RC_B XOR)
    let mut lanes_affine = lanes;
    let start = unsafe { _rdtsc() };
    for _ in 0..iters {
        for i in 0..128 {
            lanes_affine[i].hi ^= RC_B[0][i].0;
            lanes_affine[i].lo ^= RC_B[0][i].1;
        }
    }
    let end = unsafe { _rdtsc() };
    std::hint::black_box(&lanes_affine);
    let affine_cycles = (end - start) as f64 / iters as f64;
    
    // 6. MDS Column mixing only
    let mut lanes_col = lanes;
    let start = unsafe { _rdtsc() };
    for _ in 0..iters {
        // Column mixing: 8 columns × 16 bytes × 16×16 GF(2^8) matrix
        for col in 0..8 {
            let mut col_lanes = [Lane::new(0,0); 16];
            for row in 0..16 { col_lanes[row] = lanes_col[row * 8 + col]; }
            
            let mut out_lanes = [Lane::new(0,0); 16];
            for b in 0..16 {
                let mut vec = [0u8; 16];
                for row in 0..16 {
                    let lane = col_lanes[row];
                    let byte = if b < 8 {
                        (lane.hi >> ((7 - b) * 8)) as u8
                    } else {
                        (lane.lo >> ((15 - b) * 8)) as u8
                    };
                    vec[row] = byte;
                }
                let mut out_vec = [0u8; 16];
                for i in 0..16 {
                    let mut sum = 0u8;
                    for j in 0..16 {
                        sum ^= aise_core::field8::mul(aise_core::constants::M_COL[i][j], vec[j]);
                    }
                    out_vec[i] = sum;
                }
                for row in 0..16 {
                    let out_byte = out_vec[row] as u64;
                    if b < 8 {
                        out_lanes[row].hi |= out_byte << ((7 - b) * 8);
                    } else {
                        out_lanes[row].lo |= out_byte << ((15 - b) * 8);
                    }
                }
            }
            for row in 0..16 { lanes_col[row * 8 + col] = out_lanes[row]; }
        }
    }
    let end = unsafe { _rdtsc() };
    std::hint::black_box(&lanes_col);
    let col_cycles = (end - start) as f64 / iters as f64;

    // 7. MDS Row mixing only
    let mut lanes_row = lanes;
    let start = unsafe { _rdtsc() };
    for _ in 0..iters {
        for row in 0..16 {
            let mut row_lanes = [Lane::new(0,0); 8];
            for col_idx in 0..8 { row_lanes[col_idx] = lanes_row[row * 8 + col_idx]; }
            
            let mut out_lanes = [Lane::new(0,0); 8];
            for c in 0..8 {
                let mut vec = [0u16; 8];
                for col_idx in 0..8 {
                    let lane = row_lanes[col_idx];
                    let chunk = if c < 4 {
                        (lane.hi >> ((3 - c) * 16)) as u16
                    } else {
                        (lane.lo >> ((7 - c) * 16)) as u16
                    };
                    vec[col_idx] = chunk;
                }
                let mut out_vec = [0u16; 8];
                for i in 0..8 {
                    let mut sum = 0u16;
                    for j in 0..8 {
                        sum ^= aise_core::field16::mul(aise_core::constants::M_ROW[i][j], vec[j]);
                    }
                    out_vec[i] = sum;
                }
                for col_idx in 0..8 {
                    let out_chunk = out_vec[col_idx] as u64;
                    if c < 4 {
                        out_lanes[col_idx].hi |= out_chunk << ((3 - c) * 16);
                    } else {
                        out_lanes[col_idx].lo |= out_chunk << ((7 - c) * 16);
                    }
                }
            }
            for col_idx in 0..8 { lanes_row[row * 8 + col_idx] = out_lanes[col_idx]; }
        }
    }
    let end = unsafe { _rdtsc() };
    std::hint::black_box(&lanes_row);
    let row_cycles = (end - start) as f64 / iters as f64;

    // Report
    let accounted = sbox_cycles + mds_cycles + sigma_cycles + affine_cycles;
    let unaccounted = per_round - (sbox_cycles + mds_cycles + sigma_cycles + affine_cycles) / 1.0;
    
    println!();
    println!("=== Full Pi_B (32 rounds): {:.0} cycles ===", full_pi_b);
    println!("=== Per-Round Average: {:.0} cycles ===", per_round);
    println!();
    println!("  Component              | Cycles/call  | % of Round");
    println!("  -----------------------|--------------|----------");
    println!("  S-box (batch_inv)      | {:>10.0}   | {:>5.1}%", sbox_cycles, 100.0 * sbox_cycles / per_round);
    println!("  MDS mixing (total)     | {:>10.0}   | {:>5.1}%", mds_cycles, 100.0 * mds_cycles / per_round);
    println!("    Column (GF(2^8))     | {:>10.0}   | {:>5.1}%", col_cycles, 100.0 * col_cycles / per_round);
    println!("    Row (GF(2^16))       | {:>10.0}   | {:>5.1}%", row_cycles, 100.0 * row_cycles / per_round);
    println!("  SIGMA_B permutation    | {:>10.0}   | {:>5.1}%", sigma_cycles, 100.0 * sigma_cycles / per_round);
    println!("  Affine (RC_B XOR)      | {:>10.0}   | {:>5.1}%", affine_cycles, 100.0 * affine_cycles / per_round);
    println!("  -----------------------|--------------|----------");
    println!("  Accounted Total        | {:>10.0}   | {:>5.1}%", accounted, 100.0 * accounted / per_round);
    println!("  Unaccounted (overhead) | {:>10.0}   | {:>5.1}%", per_round - accounted, 100.0 * (per_round - accounted) / per_round);

    // GF(2^8) mul cost
    let a8 = 0xABu8;
    let b8 = 0xCDu8;
    let start = unsafe { _rdtsc() };
    let mut v8 = a8;
    for _ in 0..1_000_000 {
        v8 = aise_core::field8::mul(v8, b8);
    }
    let end = unsafe { _rdtsc() };
    std::hint::black_box(v8);
    println!();
    println!("  GF(2^8) mul: {:.2} cycles/call", (end - start) as f64 / 1_000_000.0);
    
    // GF(2^16) mul cost
    let a16 = 0xABCDu16;
    let b16 = 0x1234u16;
    let start = unsafe { _rdtsc() };
    let mut v16 = a16;
    for _ in 0..1_000_000 {
        v16 = aise_core::field16::mul(v16, b16);
    }
    let end = unsafe { _rdtsc() };
    std::hint::black_box(v16);
    println!("  GF(2^16) mul: {:.2} cycles/call", (end - start) as f64 / 1_000_000.0);
    
    // Operation count per round
    println!();
    println!("  --- Operation Counts Per Round ---");
    println!("  Column Mix: 8 cols × 16 bytes × (16×16 GF(2^8) muls) = {} GF(2^8) muls", 8 * 16 * 16 * 16);
    println!("  Row Mix: 16 rows × 8 chunks × (8×8 GF(2^16) muls) = {} GF(2^16) muls", 16 * 8 * 8 * 8);
}
