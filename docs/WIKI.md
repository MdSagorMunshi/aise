# AISE Deep-Dive Wiki

Welcome to the comprehensive Wiki for the **AEGIS-Ω (AISE)** cryptographic primitive. This document serves as the central hub for understanding the mathematical philosophy, structural architecture, and security properties of this experimental, "Omega-Level" hash function.

---

## Table of Contents
1. [The AEGIS-Ω Philosophy](#1-the-aegis-ω-philosophy)
2. [The Triple-Cascade Architecture (Π_Ω)](#2-the-triple-cascade-architecture-π_ω)
3. [The 16,384-bit State & Spatial Routing](#3-the-16384-bit-state--spatial-routing)
4. [Security Margins and Cryptanalysis](#4-security-margins-and-cryptanalysis)
5. [The Extended Cryptographic Toolkit](#5-the-extended-cryptographic-toolkit)
6. [Hardware Acceleration and Future Work](#6-hardware-acceleration-and-future-work)

---

## 1. The AEGIS-Ω Philosophy

The fundamental hypothesis driving the creation of AEGIS-Ω is that modern symmetric cryptographic primitives are overwhelmingly monolithic in their algebraic structure. For instance:
- **AES / SHA-3 (Keccak)**: Rely almost entirely on operations over binary extension fields ($GF(2^n)$) or simple bitwise logic ($\mathbb{F}_2$).
- **ChaCha20 / BLAKE3**: Rely entirely on ARX structures (Addition-Rotation-XOR) over integer rings ($\mathbb{Z}_{2^{32}}$ or $\mathbb{Z}_{2^{64}}$).

If a revolutionary mathematical breakthrough occurs in the cryptanalysis of one of these specific domains (e.g., a highly efficient algorithm for solving algebraic systems over binary fields, or a generalized differential attack against ARX structures), the entire primitive fails catastrophically.

**AEGIS-Ω eliminates this single point of mathematical failure.** By forcing the data to continuously transition through three *mutually incompatible* algebraic domains, an attacker cannot utilize a unified mathematical framework to trace a cryptanalytic trail from the input to the output.

---

## 2. The Triple-Cascade Architecture (Π_Ω)

The core permutation of AISE, denoted as **$\Pi_\Omega$**, is defined as the sequential composition of three distinct 32-round permutations:

$$ \Pi_\Omega = \Pi_C \circ \Pi_B \circ \Pi_A $$

Each permutation operates on the massive 16,384-bit state, but treats the underlying bits using entirely different mathematical systems:

### Phase A: $\Pi_A$ — The ARX Domain ($\mathbb{Z}_{2^{64}}$)
The first layer treats the state as an array of 256 64-bit integers. It uses **Addition modulo $2^{64}$**, **Bitwise Rotations**, and **XOR** (ARX). 
- **Purpose**: Addition provides excellent diffusion and non-linearity against linear cryptanalysis due to carry propagation, while rotations provide intra-word diffusion.
- **Strength**: Highly efficient on modern CPUs; deeply resistant to algebraic attacks because mixing addition (integer ring) with XOR (binary field) creates high-degree polynomials.

### Phase B: $\Pi_B$ — The Binary Field Domain ($GF(2^{128})$)
The second layer treats the state as 128 elements of the Galois Field $GF(2^{128})$ defined by the irreducible polynomial $x^{128} + x^7 + x^2 + x + 1$.
- **S-Box**: It applies an optimal $x \mapsto x^{-1}$ inversion S-Box. This is the theoretical maximum for non-linearity (used in AES, but scaled massively).
- **MDS Matrix**: It diffuses the data using a $16 \times 16$ Maximum Distance Separable (MDS) matrix over $GF(2^{128})$.
- **Purpose**: Provides mathematically provable bounds against differential and linear cryptanalysis.

### Phase C: $\Pi_C$ — The Prime Field Domain ($GF(2^{127}-1)$)
The final layer treats the state as 128 elements modulo the 12th Mersenne Prime, $p = 2^{127}-1$.
- **S-Box (Bidirectional Algebraic Resistance)**: It applies an alternating power map based on the Rescue construction. On **even rounds**, it applies the high-degree inverse mapping $x \mapsto x^d \pmod p$ (where $d \equiv 5^{-1} \pmod{2^{127}-2}$). On **odd rounds**, it applies the rapid, low-degree forward mapping $x \mapsto x^5 \pmod p$. By alternating these maps, AISE ensures the algebraic degree of the system equations grows maximally in both the forward and backward directions, neutralizing interpolation and algebraic cryptanalysis.
- **MDS Matrix**: It diffuses the data using another $16 \times 16$ MDS matrix, but this time all arithmetic is performed modulo $p$.
- **Purpose**: By shifting from binary polynomials to modular arithmetic over a large prime, any algebraic system equations built by an attacker in Phase B are completely shattered. The prime field forces polynomials to wrap at a value ($2^{127}-1$) that is fundamentally misaligned with binary memory bounds ($2^{128}$).

---

## 3. The 16,384-bit State & Spatial Routing

AISE utilizes a massive 16,384-bit state, conceptually divided into a $16 \times 8$ grid of 128-bit "Lanes". 
- **Rate ($r$)**: 8,192 bits (Lanes 0–63). This is where message data is XORed.
- **Capacity ($c$)**: 8,192 bits (Lanes 64–127). This remains hidden and provides the security margin.

Between the ARX, Binary, and Prime phases, the state undergoes a strict spatial dispersion routing algorithm known as **$\sigma$**. 
- $\sigma$ acts as a deterministic scatter-gather operation that guarantees that adjacent lanes in Phase A are dispersed as far apart as possible in Phase B, and again in Phase C. 
- This ensures that localized differentials (where an attacker changes a few bits in one lane) are rapidly scattered across the entire 16-kilobit state, achieving full diffusion within just a few rounds of the cascade.

---

## 4. Security Margins and Cryptanalysis

Due to the $c = 8,192$ bit capacity, AISE offers security margins that vastly exceed standard primitives like SHA-3-512 (which has a 1024-bit capacity).

### Classical Security Bounds
- **Collision Resistance**: $2^{c/2} = 2^{4096}$ operations.
- **Preimage Resistance**: $\min(2^n, 2^c)$ operations.
- **State Recovery**: $2^{8192}$ operations.

### Quantum Security Bounds
Assuming a post-quantum environment where an attacker possesses a large-scale, fault-tolerant quantum computer:
- **Quantum Collision (Brassard-Høyer-Tapp / BHT)**: $2^{c/3} \approx 2^{2730}$ operations.
- **Quantum Preimage (Grover's Algorithm)**: $2^{c/2} = 2^{4096}$ operations.

*Context: $2^{2730}$ operations is so incomprehensibly large that it exceeds the thermodynamic limits of computation within the observable universe. Even if an attacker could convert every atom in the universe into a quantum gate operating at the Planck time limit for the entire age of the universe, they would fall short by thousands of orders of magnitude.*

---

## 5. The Extended Cryptographic Toolkit

AISE is not just a hash function; it is a full cryptographic sponge toolkit. By simply altering the "Domain Tag" injected during initialization, the exact same $\Pi_\Omega$ permutation safely provides multiple distinct primitives:

| Mode | Domain Tag | Description |
|---|---|---|
| **AISE-HASH** | `0x00` | Standard fixed-length hashing. |
| **AISE-XOF** | `0x01` | Extendable-Output Function (arbitrary length stream). |
| **AISE-MAC** | `0x02` | Keyed Message Authentication. Features full-state key absorption to prevent rate-cancellation attacks. |
| **AISE-HMAC** | `0x03` & `0x04` | Legacy-compatible nested HMAC wrapper (RFC 2104). |
| **AISE-PERSONALIZED** | `0x05` | Pre-absorbs a context string before hashing the main message. |
| **AISE-KDF** | `0x06` & `0x07` | Extract-and-Expand Key Derivation Function. |
| **AISE-PH** | `0x08` | Tunable Password Hashing. Encodes a time-hard iteration cost directly into Lane 66. |
| **AISE-PRF** | `0x09` | Pseudorandom Function, structurally identical to MAC but strictly domain-separated. |
| **AISE-RATCHET** | `0x0A` | Forward-secure hash-chain ratchet for session state evolution. |
| **AISE-COMMIT** | `0x0B` & `0x0C` | Cryptographic binding and hiding commitments. |
| **AISE-THRESHOLD** | `0x0D` | N-of-N multi-key threshold MAC requiring sequential full-state absorption of all key shares. |
| **AISE-TREE** | `0x10` & `0x11` | Massively parallelizable tree hashing for Exabyte-scale inputs. |
| **AISE-DUPLEX** | `0x20` | Authenticated Encryption with Associated Data (AEAD) duplex session. |

---

## 6. Hardware Acceleration and Future Work

### Current Implementation Performance
The reference implementation provided in `aise-core` is written in Rust. Originally, the wide-block mathematical operations in $\Pi_B$ and $\Pi_C$ acted as severe software bottlenecks (~0.03 MB/s) because performing $128 \times 128$ MDS matrix multiplications and calculating finite field inversions required looping over thousands of individual 128-bit polynomials.

### Hardware Acceleration (AVX-512)
To achieve production-grade throughput, the implementation utilizes modern CPU vector instructions, heavily parallelizing the mathematical domains:
1. **AVX-512 (avx512f, avx512bw, avx512dq)**: Processes the 128 lanes simultaneously using 512-bit vector registers.
2. **VPCLMULQDQ**: Utilizes carry-less multiplication intrinsics to hardware-accelerate the $GF(2^{128})$ multiplications and inversions in $\Pi_B$.
3. **AVX-512 IFMA (Integer Fused Multiply-Add)**: Accelerates the large-integer prime modulus arithmetic in $\Pi_C$.

With these hardware optimizations, the components achieve the following throughput:
- $\Pi_A$ (ARX): ~50 MB/s
- $\Pi_B$ (Binary Field): ~19.17 MB/s
- $\Pi_C$ (Prime Field): ~3.74 MB/s
- **Full $\Pi_\Omega$ Cascade**: ~3.04 MB/s (a ~100x speedup from the scalar baseline)

*Note: For non-x86_64 architectures, CPUs without AVX-512, or `#![no_std]` embedded targets, the implementation automatically falls back to a purely scalar path.*
