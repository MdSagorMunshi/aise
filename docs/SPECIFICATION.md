# AEGIS-Ω (AISE) Formal Specification
Version: 1.0.0
Classification: Research / Experimental

**MANDATORY DISCLAIMER:** AEGIS-Ω (AISE) is an experimental, research-grade cryptographic design intended solely to explore the limits of wide-block, multi-algebraic permutations. It has NOT undergone formal cryptanalysis. AISE MUST NOT be used to protect any real data.

## 1. Overview
AEGIS-Ω is a cryptographic sponge construction featuring an enormous 16,384-bit state size. The core permutation, $\Pi_\Omega$, is a triple-cascade composition of three distinct algebraic domains designed to thwart differential, linear, and algebraic cryptanalysis through structural heterogeneity.

- **State Size ($b$)**: 16,384 bits (represented as 128 Lanes of 128 bits each)
- **Capacity ($c$)**: 8,192 bits (Lanes 64–127)
- **Rate ($r$)**: 8,192 bits (Lanes 0–63)
- **Output length**: Up to 4,096 bits (for hashes)

## 2. The State Representation
The state is organized into 128 Lanes.
Each Lane $L_i$ ($0 \le i \le 127$) is a 128-bit value composed of two 64-bit halves: $L_i = (L_i^{hi}, L_i^{lo})$.

The permutations operate on the state as a $16 \times 8$ grid (16 rows, 8 columns) or as a flat vector of 128 elements, depending on the layer.

## 3. The Permutation Cascade ($\Pi_\Omega$)
The core permutation is defined as:
$$ \Pi_\Omega = \Pi_C \circ \Pi_B \circ \Pi_A $$

**Important:** $\Pi_\Omega$ is a *surjection*, not a bijection over $\mathbb{B}^{16384}$. Each 128-bit Lane is reduced to 127 bits via Mersenne reduction ($L_i \bmod 2^{127}-1$) at the boundary of $\Pi_C$, causing ~128 bits of information loss per permutation call across the full 128-lane state. This is an explicit, deliberate design decision documented in `permute.rs`.

### 3.1 Domain A: Non-Linear ARX ($\Pi_A$)
$\Pi_A$ employs a two-level ARX construction operating at two distinct word widths:

- **SubWord_A**: Decomposes each 128-bit Lane into four 32-bit words and applies 8 rounds of a 4-branch ARX network over $(\mathbb{Z}_{2^{32}})^4$. The four branches use asymmetric rotation constants (13, 19, 23, 7) to break rotational symmetry. Per-round constants $RC_A[r]$ are XORed into the upper two words.
- **MixPair_A**: Operates on adjacent Lane pairs, performing 4-round asymmetric ARX mixing over the 64-bit halves $(\mathbb{Z}_{2^{64}})^2$, using rotation constants (26, 39, 46, 19, 13, 41, 15, 35). Two interleaved passes (even-indexed pairs, then odd-indexed pairs) ensure full diffusion.
- **$\sigma_A$**: A specialized fixed permutation routing lanes across the 128-element state to ensure diffusion between disparate ARX components. Per-lane round constants are XORed after routing.

### 3.2 Domain B: $GF(2^{128})$ Algebraic Layer ($\Pi_B$)
Operates over the binary extension field $GF(2^{128})$ defined by the irreducible polynomial $P(x) = x^{128} + x^7 + x^2 + x + 1$.
- **SBox_B**: $S_B(x) = x^{2^{128}-2} \pmod{P(x)}$. An inversion mapping providing optimal differential uniformity ($\delta = 4$ for this field).
- **$M_{COL}$**: A $16 \times 16$ MDS matrix over $GF(2^8)$ providing column diffusion across the 16 rows of each column.
- **$M_{ROW}$**: An $8 \times 8$ MDS matrix over $GF(2^{16})$ providing row diffusion across the 8 columns of each row.
- **$\sigma_B$**: A secondary spatial routing layer with per-lane round constant addition.

### 3.3 Domain C: $GF(p)$ Prime Field Layer ($\Pi_C$)
Operates over the prime field $GF(2^{127}-1)$, a Mersenne prime.
- **Mapping**: $L_i$ mapped to $f_i = L_i \pmod{2^{127}-1}$. This is the lossy reduction step (128 bits → 127 bits).
- **SBox_C**: An alternating power map based on the Marvellous Design Strategy (Rescue) to maximize algebraic degree in both directions. $\Pi_C$ strictly alternates its S-Box based on the round parity:
  - **Even Rounds** ($r = 0, 2, 4...$): Applies the high-degree inverse power map $S_{C\_even}(x) = x^d \pmod p$, where $d \equiv 5^{-1} \pmod{2^{127}-2}$.
  - **Odd Rounds** ($r = 1, 3, 5...$): Applies the low-degree forward power map $S_{C\_odd}(x) = x^5 \pmod p$.
  This alternation guarantees that interpolation and Gröbner basis attacks face maximal, exponential degree growth regardless of whether the attacker analyzes the permutation in the forward or backward direction.
- **$M_{COL\_P}$**: A $16 \times 16$ MDS matrix over $GF(p)$ providing column diffusion.
- **$M_{ROW\_P}$**: An $8 \times 8$ MDS matrix over $GF(p)$ providing row diffusion.
- **$\sigma_C$**: The final spatial routing permutation with per-lane round constant addition.

## 4. Operational Modes
AEGIS-Ω utilizes the sponge construction to instantiate various cryptographic primitives:
- **AISE-HASH**: A standard collision-resistant hash function (default 512-bit output).
- **AISE-XOF**: An extendable-output function for arbitrary-length digests.
- **AISE-MAC**: A message authentication code utilizing full-state key absorption to prevent capacity-cancellation attacks.
- **AISE-KDF**: A robust key derivation function.
- **AISE-TREE**: A parallel tree-hashing mode utilizing Sakura-style domain separation.
- **AISE-DUPLEX**: A stateful authenticated encryption mode operating on the duplex sponge principle.

## 5. Security Analysis & Limitations

### 5.1 Generic Security Bounds (Output-Level)
For the default 512-bit AISE-HASH output, generic security bounds are determined by the output length, not the state size:
- **Classical Collision Resistance:** ~$2^{256}$ (birthday bound)
- **Classical Preimage Resistance:** $2^{512}$
- **Quantum Collision Resistance (BHT):** ~$2^{171}$
- **Quantum Preimage Resistance (Grover):** $2^{256}$

These bounds are identical to SHA-512, SHA3-512, and BLAKE2b, all of which produce 512-bit outputs.

### 5.2 Structural Security Margin (State-Level)
The 16,384-bit state with 8,192-bit capacity provides structural margin *beyond* the generic output-level bounds:
- **Inner-collision resistance**: In the sponge model, the capacity $c$ determines resistance to inner-collision and state-recovery attacks. With $c = 8192$, these attacks require ~$2^{4096}$ work — vastly exceeding the output-level birthday bound.
- **Capacity-targeting attacks**: An attacker attempting to control the capacity portion of the state faces a $2^{8192}$-dimensional search space.

However, these structural margins only matter if the permutation behaves sufficiently close to an ideal random permutation — which has not been established through independent cryptanalysis.

### 5.3 The Surjection Issue
$\Pi_\Omega$ is not bijective due to the Mersenne reduction at the $\Pi_B \to \Pi_C$ boundary. Each 128-bit lane loses 1 bit of information when mapped to $GF(2^{127}-1)$. Across 128 lanes, this is ~128 bits of information loss per permutation call.

The standard sponge security proof by Bertoni et al. assumes a bijective (invertible) permutation. Adapting the proof to a surjective permutation requires demonstrating that the information loss does not create exploitable structure. This remains an open theoretical question for AISE.

### 5.4 Cryptanalysis Status
AEGIS-Ω has **not undergone independent cryptanalysis**. The security claims in this document are upper bounds assuming ideal behavior, not proven security levels. Priority analysis targets include:
- Differential and linear trail analysis across the heterogeneous cascade
- Algebraic attacks (interpolation, Gröbner basis) exploiting the alternating power maps
- Attacks targeting the 128→127-bit lossy domain transition
- Invariant subspace attacks across the three algebraic domains
- Reduced-round distinguishers and structural distinguishers
