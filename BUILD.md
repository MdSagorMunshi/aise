# Building and Testing AEGIS-Ω (AISE)

This repository is built around the standard Rust `cargo` toolchain. Due to the advanced nature of the $\Pi_B$ and $\Pi_C$ mathematical requirements, AISE utilizes Rust Edition 2024.

## Prerequisites
- **Rust Toolchain**: `rustup` configured with a modern stable or nightly toolchain (Edition 2024).

## Building the Toolkit

To compile the `aise-core` library and the CLI tool, use standard cargo build steps:

```bash
cargo build --release
```

To run the unified command-line hashing interface directly:

```bash
cargo run --release --bin aise-cli -- --help
```

## Running the Benchmark Suite

AISE features a highly granular benchmarking suite designed to analyze the throughput across the three heterogeneous permutation layers ($\Pi_A$, $\Pi_B$, and $\Pi_C$).

```bash
cargo run --release --bin aise-bench
```

> [!WARNING]
> Because of the massive 16,384-bit state, unrolled software-based $128 \times 128$ MDS matrices, and raw GF(2^128) arithmetic, the current reference software implementation bottlenecks heavily during $\Pi_B$ and $\Pi_C$ (~0.03 MB/s). An optimized, hardware-accelerated production version utilizing Intel AVX-512 and VPCLMULQDQ intrinsics would vastly improve this as originally designed for the Omega-class architecture.

## Running the Verification Test Suite

The `aise-tests` crate contains the rigorous mathematical proofs validating the internal structures.

```bash
cargo test -p aise-tests
```

### What do the tests do?
- **Level 1 (Field Arithmetic)**: Validates basic properties of $GF(2^{128})$ and the prime field $GF(2^{127}-1)$, including associativity, distributivity, and Frobenius endomorphisms.
- **Level 2 (Permutations)**: Mathematically cross-verifies that $\Pi_A$ ARX blocks are strictly bijective by building exact inverse mappings. Validates mutual inversion for SBox components, and ensures the spatial $\sigma$ routers have zero fixed points.
- **Level 3 & 4 (Modes)**: Validates AISE-HASH, AISE-XOF, AISE-PH tunability, and AISE-RATCHET state-chain security flows using concrete operational tests.
