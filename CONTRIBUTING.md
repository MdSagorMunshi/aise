# Contributing to AEGIS-Ω (AISE)

Thank you for your interest in contributing to the AISE project! 

As an experimental, research-grade cryptographic design, maintaining the strict mathematical rigidity of the core structures is our top priority.

## Design Philosophy
AISE is built on the philosophy of "no single point of mathematical failure". The three permutation blocks ($\Pi_A$, $\Pi_B$, $\Pi_C$) must operate across mathematically incompatible domains to structurally eliminate cryptanalytic paths spanning the full $\Pi_\Omega$ cascade.

Any pull request that attempts to "unify" the permutations or simplify the underlying fields will be strictly rejected. 

## How to Contribute

### 1. Hardware Optimization
The current reference software implementation relies on basic bit-shifting and software matrices. The most valuable contributions would be implementations of the fields using:
- **Intel AVX-512** intrinsics.
- **AES-NI** and **VPCLMULQDQ** support for the $GF(2^{128})$ operations.
- Parallelization via SIMD instruction sets.

### 2. Cryptanalysis & Auditing
We welcome issues and pull requests documenting algebraic or cryptanalytic reviews of the internal constants, provided they include rigorous mathematical proofs. 

> [!CAUTION]
> Do NOT change the generation parameters for the round constants ($\pi$, $e$, $\zeta(3)$) or the MDS matrices without citing explicit theoretical breaks. 

## Pull Request Guidelines
1. **Pass the Test Suite**: Your code must pass all mathematical verifications. Run `cargo test -p aise-tests` before submitting.
2. **Document Performance**: If your PR aims to optimize performance, provide the before/after results from `cargo run --release --bin aise-bench`.
3. **Format Code**: Ensure your rust code is correctly formatted using `cargo fmt`.
4. **Sign-off**: Include a statement acknowledging the experimental nature of the code.
