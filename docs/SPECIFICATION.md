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
Each Lane $L_i$ ($0 \le i \le 127$) is a 128-bit value.

The permutations operate on the state as a $16 \times 8$ grid or as a vector of 128 elements, depending on the layer.

## 3. The Permutation Cascade ($\Pi_\Omega$)
The core permutation is defined as:
$$ \Pi_\Omega = \Pi_C \circ \Pi_B \circ \Pi_A $$

### 3.1 Domain A: Non-Linear ARX ($\Pi_A$)
Operates over $\mathbb{Z}_{2^{32}}$ using Addition, Rotation, and XOR (ARX).
- **SubWord_A**: A 4-branch ARX transformation with custom rotation constants.
- **MixPair_A**: A highly asymmetric diffusion layer operating on 256-bit pairs.
- **$\sigma_A$**: A specialized fixed permutation routing lanes across the 128-element state to ensure diffusion between disparate ARX components.

### 3.2 Domain B: $GF(2^{128})$ Algebraic Layer ($\Pi_B$)
Operates over the binary extension field $GF(2^{128})$ defined by the irreducible polynomial $P(x) = x^{128} + x^7 + x^2 + x + 1$.
- **SBox_B**: $S_B(x) = x^{2^{128}-2} \pmod{P(x)}$. An inversion mapping providing optimal differential uniformity.
- **$M_{COL}$ and $M_{ROW}$**: $16 \times 16$ Maximum Distance Separable (MDS) matrices providing rigorous wide-trail diffusion.
- **$\sigma_B$**: A secondary spatial routing layer.

### 3.3 Domain C: $GF(p)$ Prime Field Layer ($\Pi_C$)
Operates over the prime field $GF(2^{127}-1)$, a Mersenne prime.
- **Mapping**: $L_i$ mapped to $f_i = L_i \pmod{2^{127}-1}$.
- **SBox_C**: A power map $S_C(x) = x^5 \pmod{p}$.
- **$M_{COL\_P}$ and $M_{ROW\_P}$**: $16 \times 16$ MDS matrices over $GF(p)$.
- **$\sigma_C$**: The final spatial routing permutation.

## 4. Operational Modes
AEGIS-Ω utilizes the sponge construction to instantiate various cryptographic primitives:
- **AISE-HASH**: A standard collision-resistant hash function.
- **AISE-MAC**: A message authentication code utilizing full-state key absorption to prevent capacity-cancellation attacks.
- **AISE-KDF**: A robust key derivation function.
- **AISE-TREE**: A parallel tree-hashing mode utilizing Sakura-style domain separation.
- **AISE-DUPLEX**: A stateful authenticated encryption mode operating on the duplex sponge principle.
