<div align="center">

<pre>
   █████╗ ██╗███████╗███████╗      ██████╗ 
  ██╔══██╗██║██╔════╝██╔════╝     ██╔═══██╗
  ███████║██║███████╗█████╗   ██████║   ██║
  ██╔══██║██║╚════██║██╔══╝   ╚════██╗  ██║
  ██║  ██║██║███████║███████╗      ╚██████╔╝
  ╚═╝  ╚═╝╚═╝╚══════╝╚══════╝       ╚═════╝ 
                                           
   A N  O M E G A - L E V E L  H A S H     
</pre>

[![Rust](https://img.shields.io/badge/rust-2021-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](#)
[![Status](https://img.shields.io/badge/status-experimental-red.svg)](#)

**Version:** 1.0.0 | **Classification:** Research / Experimental

**MANDATORY DISCLAIMER:** AEGIS-Ω (AISE) is an experimental, research-grade cryptographic design intended solely to explore the limits of wide-block, multi-algebraic permutations. It has NOT undergone formal cryptanalysis. **AISE MUST NOT be used to protect any real data.**

</div>

## Overview
AEGIS-Ω is a cryptographic sponge construction featuring an enormous **16,384-bit state size**. To eliminate single points of mathematical failure, the core permutation ($\Pi_\Omega$) is a triple-cascade composition of three distinctly heterogeneous algebraic domains.

### The Toolkit
AISE provides a complete suite of domain-separated cryptographic primitives built around the same $\Pi_\Omega$ sponge core:
- **AISE-HASH**: Fixed-length collision-resistant hash
- **AISE-XOF**: Extendable-output hash
- **AISE-MAC**: Full-state key absorption authentication
- **AISE-KDF**: Extract-and-expand key derivation
- **AISE-PH**: Tunable cost password hashing
- **AISE-TREE**: Parallel tree hashing
- **AISE-DUPLEX**: Authenticated encryption
- **AISE-COMMIT**: Binding and hiding commitments
- **AISE-RATCHET**: Forward-secure state chains
- **AISE-THRESHOLD**: N-of-N multi-key threshold MAC

## Visual Architecture

```mermaid
flowchart TD
    subgraph S["AISE State (16,384 bits)"]
        R["Rate (8,192 bits)"]
        C["Capacity (8,192 bits)"]
    end

    Message -->|XOR| R

    subgraph Pi_Omega["Triple-Cascade Permutation (Π_Ω)"]
        direction TB
        PiA["Π_A: Non-Linear ARX Domain<br/>(ℤ_{2^64})"]
        PiB["Π_B: Binary Extension Field<br/>(GF(2^128))"]
        PiC["Π_C: Prime Field Operations<br/>(GF(2^127 - 1))"]
        PiA --> PiB --> PiC
    end

    S --> Pi_Omega
    Pi_Omega --> S

    S -.->|Squeeze| Output["Cryptographic Output"]
```

## Mathematical Core

AEGIS-Ω's massive 128-lane state is transformed by three independent 32-round permutations:
1. **$\Pi_A$ (ARX over $\mathbb{Z}_{2^{64}}$)**: Employs carry-propagation and asymmetric bit-rotations to shatter linear and differential trails.
2. **$\Pi_B$ ($GF(2^{128})$ Binary Field)**: Applies optimal inversion-based S-boxes paired with $16 \times 16$ MDS matrices for rigorous wide-trail diffusion.
3. **$\Pi_C$ ($GF(p)$ Prime Field)**: Treats lanes as elements modulo the Mersenne prime $2^{127}-1$, utilizing an alternating power map (Rescue construction) of $x \mapsto x^5$ and $x \mapsto x^d$ paired with GF(p) MDS mixing, forcing algebraic attackers to grapple with fundamentally incompatible number systems and exponential degree growth.

## Performance & Hardware Acceleration
To achieve production-grade throughput over the massive 16,384-bit state, AISE heavily parallelizes its mathematical domains using modern CPU vector instructions:

- **AVX-512 (avx512f, avx512bw, avx512dq)**: Processes the 128 lanes simultaneously using 512-bit vector registers.
- **VPCLMULQDQ**: Hardware-accelerates the $GF(2^{128})$ multiplications and inversions in $\Pi_B$.
- **AVX-512 IFMA**: Accelerates the large-integer prime modulus arithmetic in $\Pi_C$.

**Throughput (AVX-512 Optimized, AMD Ryzen 5 7600):**
- $\Pi_A$ (ARX): ~145 MB/s
- $\Pi_B$ (Binary Field): ~19.74 MB/s
- $\Pi_C$ (Prime Field): ~4.06 MB/s
- **Full $\Pi_\Omega$ Cascade**: **~3.30 MB/s** (a ~100x speedup from the scalar baseline)

> 📊 For a full comparison against SHA-256, SHA-512, SHA3, BLAKE2b, and BLAKE3, see the **[Benchmark Report](COMPARE.md)**.

*Note: For non-x86_64 architectures, CPUs without AVX-512, or `#![no_std]` embedded targets, the implementation automatically and safely falls back to a purely scalar path.*

## Testing & Verification Methodology
AISE includes a rigorous verification suite to mathematically prove the correctness of its implementations and hardware intrinsics:
- **Frozen Vectors:** Hardcoded deterministic outputs test the full cascade for regressions across architectures.
- **Avalanche Checks:** Validates that flipping a single input bit strictly results in a ~50% bit flip rate across the 16,384-bit state after the permutation, confirming diffusion.
- **Equivalence Fuzzing:** The AVX-512 optimized routines are fuzzed against the scalar reference implementations for tens of thousands of iterations, verifying mathematical equivalence.

## Security Claims

> **⚠️ AISE has not undergone independent cryptanalysis. The bounds below assume ideal permutation behavior and are theoretical upper limits, not proven security levels.**

AISE-HASH produces a **512-bit digest**. Generic security bounds are determined by the output length:
- **Classical Collision Resistance:** ~$2^{256}$ (birthday bound on 512-bit output)
- **Classical Preimage Resistance:** $2^{512}$
- **Quantum Collision Resistance (BHT):** ~$2^{171}$
- **Quantum Preimage Resistance (Grover):** $2^{256}$

### What the 16,384-bit State Provides

The large internal state does **not** increase the output-level collision bound — that is fixed by the 512-bit digest. Instead, it provides *structural* security margin within the sponge construction:

- **8,192-bit capacity**: Resistance to inner-collision attacks, state-recovery attacks, and capacity-targeting attacks. This is 8× larger than SHA3-512's capacity (1,024 bits).
- **Algebraic heterogeneity**: Three independent mathematical domains (ARX over $\mathbb{Z}_{2^{64}}$, inversion in $GF(2^{128})$, alternating power maps in $GF(2^{127}-1)$) that an attacker must simultaneously defeat.
- **96 total rounds** across 3 algebraically incompatible domains.

### Design Note: Surjective Permutation

$\Pi_\Omega$ is a *surjection*, not a bijection: each 128-bit lane is reduced to 127 bits via Mersenne reduction before $\Pi_C$, causing ~128 bits of information loss per permutation call. This is noted explicitly in `permute.rs`. The standard sponge security proof (which assumes a bijective permutation) requires careful adaptation for this design.

### Cryptanalysis Status

AEGIS-Ω has **not been subjected to independent cryptanalysis**. The next step for validating AISE is not adding more rounds — it is attempting to *break* the existing design. Priority analysis targets include: differential trails, linear approximations, rotational attacks, integral attacks, algebraic attacks, rebound attacks, and especially attacks exploiting the 128→127-bit lossy domain transition. See [SECURITY.md](SECURITY.md) for responsible disclosure and how to contribute.

## Usage & Examples

### Prerequisites
Make sure you have [Rust](https://www.rust-lang.org/tools/install) installed.

```bash
# Hash a simple string
cargo run --release --bin aise-cli -- -s "Hello, world!"

# Hash a file
cargo run --release --bin aise-cli -- -f /path/to/my/file.txt

# Extract 1024 arbitrary XOF bytes
cargo run --release --bin aise-cli -- -s "Seed" -l 1024
```

## Documentation
- [Formal Specification](docs/SPECIFICATION.md): Mathematical definitions and bounds.
- [Benchmark Comparison](COMPARE.md): Live performance comparison against SHA-2, SHA-3, BLAKE2b, and BLAKE3.
- [Build Instructions](BUILD.md): Instructions for compiling and testing.
- [Wiki](docs/WIKI.md): Deep-dive documentation index.
- [Contributing Guidelines](CONTRIBUTING.md): How to contribute safely to the core matrices and logic.
