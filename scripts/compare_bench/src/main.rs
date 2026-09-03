// AEGIS-Ω (AISE) Comprehensive Benchmark Comparison
// Compares AISE-HASH against SHA-256, SHA-512, SHA3-256, SHA3-512, BLAKE2b, BLAKE3
//
// Outputs structured JSON for chart generation.

use std::hint::black_box;
use std::time::{Duration, Instant};

use rand::Rng;
use serde::Serialize;

// Hash trait imports
use blake2::Digest as _Blake2Digest;

// ──────────────────────────────────────────────────────────────
//  Data structures for JSON output
// ──────────────────────────────────────────────────────────────

#[derive(Serialize)]
struct BenchmarkResults {
    system_info: SystemInfo,
    throughput: Vec<ThroughputResult>,
    latency: Vec<LatencyResult>,
    permutation_breakdown: Vec<PermutationResult>,
    scalability: Vec<ScalabilityPoint>,
}

#[derive(Serialize)]
struct SystemInfo {
    cpu: String,
    rust_version: String,
    date: String,
    avx512: bool,
    target_cpu: String,
}

#[derive(Serialize)]
struct ThroughputResult {
    algorithm: String,
    input_bytes: usize,
    input_label: String,
    throughput_mbps: f64,
    median_ns: u128,
    stddev_ns: f64,
    iterations: usize,
}

#[derive(Serialize)]
struct LatencyResult {
    algorithm: String,
    input_bytes: usize,
    median_ns: u128,
    p99_ns: u128,
    min_ns: u128,
}

#[derive(Serialize)]
struct PermutationResult {
    component: String,
    median_ns: u128,
    throughput_mbps: f64,
    percentage_of_cascade: f64,
}

#[derive(Serialize)]
struct ScalabilityPoint {
    algorithm: String,
    input_bytes: usize,
    throughput_mbps: f64,
}

// ──────────────────────────────────────────────────────────────
//  Benchmark engine
// ──────────────────────────────────────────────────────────────

const WARMUP_ITERS: usize = 10;
const MIN_ITERS: usize = 100;
const MIN_BENCH_TIME_MS: u128 = 500; // run at least 500ms per measurement

/// Runs `f` repeatedly, collecting per-iteration durations.
/// Uses adaptive iteration count: at least MIN_ITERS, or enough to fill MIN_BENCH_TIME_MS.
fn measure<F: FnMut()>(mut f: F) -> Vec<Duration> {
    // Warmup
    for _ in 0..WARMUP_ITERS {
        f();
    }

    // First pass: run MIN_ITERS to estimate per-iteration time
    let mut durations = Vec::with_capacity(MIN_ITERS * 4);
    for _ in 0..MIN_ITERS {
        let start = Instant::now();
        f();
        durations.push(start.elapsed());
    }

    let total_so_far: Duration = durations.iter().sum();
    if total_so_far.as_millis() < MIN_BENCH_TIME_MS {
        // Need more iterations to get stable results
        let per_iter_ns = total_so_far.as_nanos() / MIN_ITERS as u128;
        if per_iter_ns > 0 {
            let needed = ((MIN_BENCH_TIME_MS * 1_000_000) / per_iter_ns) as usize;
            let extra = needed.saturating_sub(MIN_ITERS);
            for _ in 0..extra {
                let start = Instant::now();
                f();
                durations.push(start.elapsed());
            }
        }
    }

    durations.sort();
    durations
}

fn median(durations: &[Duration]) -> Duration {
    durations[durations.len() / 2]
}

fn stddev_ns(durations: &[Duration]) -> f64 {
    let n = durations.len() as f64;
    let mean = durations.iter().map(|d| d.as_nanos() as f64).sum::<f64>() / n;
    let variance = durations
        .iter()
        .map(|d| {
            let diff = d.as_nanos() as f64 - mean;
            diff * diff
        })
        .sum::<f64>()
        / n;
    variance.sqrt()
}

fn p99(durations: &[Duration]) -> Duration {
    let idx = (durations.len() as f64 * 0.99) as usize;
    durations[idx.min(durations.len() - 1)]
}

fn throughput_mbps(median_duration: Duration, input_bytes: usize) -> f64 {
    let seconds = median_duration.as_secs_f64();
    if seconds == 0.0 {
        return 0.0;
    }
    (input_bytes as f64) / (seconds * 1024.0 * 1024.0)
}

fn size_label(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{}B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{}KB", bytes / 1024)
    } else {
        format!("{}MB", bytes / (1024 * 1024))
    }
}

// ──────────────────────────────────────────────────────────────
//  Hash wrappers (each takes &[u8] and returns a boxed digest)
// ──────────────────────────────────────────────────────────────

fn hash_aise(data: &[u8]) -> Vec<u8> {
    aise_core::aise_hash(data, 64) // 512-bit output
}

fn hash_sha256(data: &[u8]) -> Vec<u8> {
    let mut hasher = sha2::Sha256::new();
    sha2::Digest::update(&mut hasher, data);
    sha2::Digest::finalize(hasher).to_vec()
}

fn hash_sha512(data: &[u8]) -> Vec<u8> {
    let mut hasher = sha2::Sha512::new();
    sha2::Digest::update(&mut hasher, data);
    sha2::Digest::finalize(hasher).to_vec()
}

fn hash_sha3_256(data: &[u8]) -> Vec<u8> {
    let mut hasher = sha3::Sha3_256::new();
    sha3::Digest::update(&mut hasher, data);
    sha3::Digest::finalize(hasher).to_vec()
}

fn hash_sha3_512(data: &[u8]) -> Vec<u8> {
    let mut hasher = sha3::Sha3_512::new();
    sha3::Digest::update(&mut hasher, data);
    sha3::Digest::finalize(hasher).to_vec()
}

fn hash_blake2b(data: &[u8]) -> Vec<u8> {
    let mut hasher = blake2::Blake2b512::new();
    blake2::Digest::update(&mut hasher, data);
    blake2::Digest::finalize(hasher).to_vec()
}

fn hash_blake3_fn(data: &[u8]) -> Vec<u8> {
    blake3::hash(data).as_bytes().to_vec()
}

// ──────────────────────────────────────────────────────────────
//  Main benchmark runner
// ──────────────────────────────────────────────────────────────

type HashFn = fn(&[u8]) -> Vec<u8>;

struct Algorithm {
    name: &'static str,
    func: HashFn,
}

fn main() {
    eprintln!("╔══════════════════════════════════════════════════════╗");
    eprintln!("║  AEGIS-Ω Comprehensive Benchmark Suite             ║");
    eprintln!("║  Comparing AISE vs SHA-2, SHA-3, BLAKE2b, BLAKE3   ║");
    eprintln!("╚══════════════════════════════════════════════════════╝");
    eprintln!();

    let algorithms = vec![
        Algorithm { name: "AISE-HASH", func: hash_aise },
        Algorithm { name: "SHA-256", func: hash_sha256 },
        Algorithm { name: "SHA-512", func: hash_sha512 },
        Algorithm { name: "SHA3-256", func: hash_sha3_256 },
        Algorithm { name: "SHA3-512", func: hash_sha3_512 },
        Algorithm { name: "BLAKE2b", func: hash_blake2b },
        Algorithm { name: "BLAKE3", func: hash_blake3_fn },
    ];

    let throughput_sizes: Vec<usize> = vec![
        64, 256, 1024, 4096, 16384, 65536, 262144, 1048576,
    ];

    let latency_sizes: Vec<usize> = vec![32, 64, 128];

    let mut rng = rand::thread_rng();

    // ── System info ──
    let system_info = SystemInfo {
        cpu: get_cpu_name(),
        rust_version: env!("CARGO_PKG_VERSION").to_string(),
        date: chrono_lite_date(),
        avx512: detect_avx512(),
        target_cpu: "native".to_string(),
    };

    eprintln!("  CPU: {}", system_info.cpu);
    eprintln!("  AVX-512: {}", if system_info.avx512 { "YES" } else { "NO" });
    eprintln!();

    // ── Sanity check: verify all hashes produce output ──
    eprintln!("  Sanity check...");
    let test_input = b"AEGIS-Omega benchmark sanity check";
    for algo in &algorithms {
        let result = (algo.func)(test_input);
        assert!(!result.is_empty(), "{} produced empty output", algo.name);
        eprintln!("    {} -> {} bytes (ok)", algo.name, result.len());
    }
    eprintln!();

    // ══════════════════════════════════════════════════════════
    //  1. THROUGHPUT BENCHMARK
    // ══════════════════════════════════════════════════════════
    eprintln!("━━━ Phase 1: Throughput Benchmark ━━━");
    let mut throughput_results = Vec::new();

    for &size in &throughput_sizes {
        let data: Vec<u8> = (0..size).map(|_| rng.gen()).collect();
        eprintln!("  Input size: {}", size_label(size));

        for algo in &algorithms {
            let func = algo.func;
            let data_ref = &data;

            let durations = measure(|| {
                black_box(func(black_box(data_ref)));
            });

            let med = median(&durations);
            let sd = stddev_ns(&durations);
            let tp = throughput_mbps(med, size);

            eprintln!(
                "    {:<12} {:>10.2} MB/s  (median: {:>10} ns, n={})",
                algo.name,
                tp,
                med.as_nanos(),
                durations.len()
            );

            throughput_results.push(ThroughputResult {
                algorithm: algo.name.to_string(),
                input_bytes: size,
                input_label: size_label(size),
                throughput_mbps: tp,
                median_ns: med.as_nanos(),
                stddev_ns: sd,
                iterations: durations.len(),
            });
        }
        eprintln!();
    }

    // ══════════════════════════════════════════════════════════
    //  2. LATENCY BENCHMARK (small messages)
    // ══════════════════════════════════════════════════════════
    eprintln!("━━━ Phase 2: Latency Benchmark (small messages) ━━━");
    let mut latency_results = Vec::new();

    for &size in &latency_sizes {
        let data: Vec<u8> = (0..size).map(|_| rng.gen()).collect();
        eprintln!("  Input size: {}", size_label(size));

        for algo in &algorithms {
            let func = algo.func;
            let data_ref = &data;

            let durations = measure(|| {
                black_box(func(black_box(data_ref)));
            });

            let med = median(&durations);
            let p99_val = p99(&durations);
            let min_val = durations[0];

            eprintln!(
                "    {:<12} median: {:>10} ns, p99: {:>10} ns, min: {:>10} ns",
                algo.name,
                med.as_nanos(),
                p99_val.as_nanos(),
                min_val.as_nanos()
            );

            latency_results.push(LatencyResult {
                algorithm: algo.name.to_string(),
                input_bytes: size,
                median_ns: med.as_nanos(),
                p99_ns: p99_val.as_nanos(),
                min_ns: min_val.as_nanos(),
            });
        }
        eprintln!();
    }

    // ══════════════════════════════════════════════════════════
    //  3. AISE PERMUTATION BREAKDOWN
    // ══════════════════════════════════════════════════════════
    eprintln!("━━━ Phase 3: AISE Permutation Breakdown ━━━");
    let mut perm_results = Vec::new();

    let state_bits: usize = 16384;
    let state_bytes: usize = state_bits / 8; // 2048 bytes

    // Pi_A
    {
        use aise_core::state::Lane;
        let mut lanes = [Lane::new(0x1234567890abcdef, 0xfedcba0987654321); 128];
        let durations = measure(|| {
            aise_core::pi_a::pi_a(black_box(&mut lanes));
        });
        let med = median(&durations);
        let tp = throughput_mbps(med, state_bytes);
        eprintln!("    Pi_A (ARX):          {:>10} ns  ({:.2} MB/s)", med.as_nanos(), tp);
        perm_results.push(PermutationResult {
            component: "Pi_A (ARX)".to_string(),
            median_ns: med.as_nanos(),
            throughput_mbps: tp,
            percentage_of_cascade: 0.0, // filled later
        });
    }

    // Pi_B
    {
        use aise_core::state::Lane;
        let mut lanes = [Lane::new(0x1234567890abcdef, 0xfedcba0987654321); 128];
        let durations = measure(|| {
            aise_core::pi_b::pi_b(black_box(&mut lanes));
        });
        let med = median(&durations);
        let tp = throughput_mbps(med, state_bytes);
        eprintln!("    Pi_B (GF(2^128)):    {:>10} ns  ({:.2} MB/s)", med.as_nanos(), tp);
        perm_results.push(PermutationResult {
            component: "Pi_B (GF(2^128))".to_string(),
            median_ns: med.as_nanos(),
            throughput_mbps: tp,
            percentage_of_cascade: 0.0,
        });
    }

    // Pi_C
    {
        let mut f = [0x1234567890abcdefu128; 128];
        let durations = measure(|| {
            aise_core::pi_c::pi_c(black_box(&mut f));
        });
        let med = median(&durations);
        let tp = throughput_mbps(med, state_bytes);
        eprintln!("    Pi_C (GF(p)):        {:>10} ns  ({:.2} MB/s)", med.as_nanos(), tp);
        perm_results.push(PermutationResult {
            component: "Pi_C (GF(p))".to_string(),
            median_ns: med.as_nanos(),
            throughput_mbps: tp,
            percentage_of_cascade: 0.0,
        });
    }

    // Full cascade
    {
        let mut state = aise_core::State::new();
        state.lanes[0].hi = 0xdeadbeef;
        let durations = measure(|| {
            aise_core::permute(black_box(&mut state));
        });
        let med = median(&durations);
        let tp = throughput_mbps(med, state_bytes);
        eprintln!("    Full Cascade:        {:>10} ns  ({:.2} MB/s)", med.as_nanos(), tp);

        let cascade_ns = med.as_nanos() as f64;
        // Compute percentages
        for p in perm_results.iter_mut() {
            p.percentage_of_cascade = (p.median_ns as f64 / cascade_ns) * 100.0;
        }

        perm_results.push(PermutationResult {
            component: "Full Cascade (Pi_Omega)".to_string(),
            median_ns: med.as_nanos(),
            throughput_mbps: tp,
            percentage_of_cascade: 100.0,
        });
    }
    eprintln!();

    // ══════════════════════════════════════════════════════════
    //  4. SCALABILITY (throughput vs input size)
    // ══════════════════════════════════════════════════════════
    eprintln!("━━━ Phase 4: Scalability Profile ━━━");
    let mut scalability_results = Vec::new();

    // Reuse throughput data for scalability
    for tr in &throughput_results {
        scalability_results.push(ScalabilityPoint {
            algorithm: tr.algorithm.clone(),
            input_bytes: tr.input_bytes,
            throughput_mbps: tr.throughput_mbps,
        });
    }
    eprintln!("    (derived from throughput data)");
    eprintln!();

    // ══════════════════════════════════════════════════════════
    //  OUTPUT JSON
    // ══════════════════════════════════════════════════════════
    let results = BenchmarkResults {
        system_info,
        throughput: throughput_results,
        latency: latency_results,
        permutation_breakdown: perm_results,
        scalability: scalability_results,
    };

    let json = serde_json::to_string_pretty(&results).expect("Failed to serialize results");
    println!("{}", json);

    eprintln!("━━━ Benchmark complete. JSON written to stdout. ━━━");
}

// ──────────────────────────────────────────────────────────────
//  Utility functions
// ──────────────────────────────────────────────────────────────

fn get_cpu_name() -> String {
    #[cfg(target_os = "linux")]
    {
        if let Ok(cpuinfo) = std::fs::read_to_string("/proc/cpuinfo") {
            for line in cpuinfo.lines() {
                if line.starts_with("model name") {
                    if let Some(name) = line.split(':').nth(1) {
                        return name.trim().to_string();
                    }
                }
            }
        }
    }
    "Unknown CPU".to_string()
}

fn detect_avx512() -> bool {
    #[cfg(target_arch = "x86_64")]
    {
        return std::is_x86_feature_detected!("avx512f");
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        return false;
    }
}

fn chrono_lite_date() -> String {
    // Simple date from system without pulling in chrono
    #[cfg(target_os = "linux")]
    {
        if let Ok(output) = std::process::Command::new("date")
            .arg("+%Y-%m-%d %H:%M:%S %Z")
            .output()
        {
            return String::from_utf8_lossy(&output.stdout).trim().to_string();
        }
    }
    "unknown".to_string()
}
