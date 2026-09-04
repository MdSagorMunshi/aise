# Security Policy

## Cryptanalysis Status

**AEGIS-Ω (AISE) has not undergone independent cryptanalysis.**

The security claims in our documentation assume ideal permutation behavior and are theoretical upper limits, not proven security levels. AISE is an experimental research construction — "unproven" is the only honest status for any new cryptographic design before independent analysis.

We actively invite and welcome cryptanalytic contributions.

## Responsible Disclosure

If you discover a cryptographic vulnerability, structural weakness, or exploitable property in AEGIS-Ω, please disclose it responsibly:

### For theoretical attacks, distinguishers, and reduced-round analysis:
**Open a public issue** using the [🔬 Cryptanalysis / Mathematical Observation](https://github.com/MdSagorMunshi/aise/issues/new?template=cryptanalysis.yml) template. We consider transparency essential for an experimental research design — public analysis benefits the entire cryptographic community.

### For implementation-level vulnerabilities (side-channel leaks, memory safety issues):
**Email the maintainer directly** at the address listed in the GitHub profile before public disclosure. Allow 90 days for a fix before publishing.

## Priority Analysis Targets

The following areas represent the most impactful directions for independent cryptanalysis:

### The 128→127-bit Lossy Domain Transition
Each 128-bit Lane is reduced to 127 bits via Mersenne reduction ($L_i \bmod 2^{127}-1$) before Π_C. This makes Π_Ω a surjection, not a bijection. The standard sponge security proof assumes bijectivity — can the information loss at this boundary be exploited?

### Differential Trail Analysis
- What is the minimum number of active S-boxes across the full 96-round cascade?
- Do differential trails propagate predictably across the Π_A → Π_B → Π_C domain transitions?
- What are the best differential characteristics for reduced-round variants?

### Linear Approximations
- What are the maximum linear biases across each individual permutation layer?
- Does the heterogeneous cascade amplify or suppress linear correlations?

### Algebraic Attacks
- **Interpolation attacks** on the alternating power maps ($x^5$ / $x^d$) in Π_C: at what round count does the algebraic degree saturate?
- **Gröbner basis attacks**: what is the practical complexity of solving the polynomial system defined by reduced-round Π_C?
- **Cross-domain algebraic relations**: do the domain transitions (Lane → GF(2^128) → GF(p)) introduce algebraic shortcuts?

### Rotational & Invariant Subspace Attacks
- Does the ARX layer (Π_A) have rotational symmetry that survives into Π_B?
- Are there invariant subspaces that persist across the heterogeneous cascade?

### Structural Distinguishers
- Can a reduced-round Π_Ω be distinguished from a random permutation?
- What is the minimum number of rounds needed for security in each layer?

### Rebound & Internal Differential Attacks
- Targeting the Π_B inversion layer ($x^{2^{128}-2}$) as a rebound point
- Exploiting the MDS structure for efficient inbound/outbound phases

## What We Consider a Valid Finding

- **Reduced-round attacks**: Any attack on Π_Ω with fewer rounds than the full 32+32+32 = 96 that achieves better-than-generic complexity.
- **Structural distinguishers**: Any test that distinguishes Π_Ω output from a random permutation with advantage > $2^{-128}$.
- **Practical exploits**: Any demonstration that AISE-HASH collisions can be found faster than $2^{256}$ work.
- **Theoretical observations**: Bounds, proofs, or counterexamples regarding the security of the surjective sponge construction.

All valid cryptanalytic contributions will be credited in the repository's documentation.
