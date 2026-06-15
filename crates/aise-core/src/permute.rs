//! The AISE Full Cascade Permutation (Π_Ω)

use crate::state::State;
use crate::pi_a;
use crate::pi_b;
use crate::pi_c;
use crate::field_p;

/// The full Π_Ω cascade: Π_C ∘ Π_B ∘ Π_A
///
/// NOTE: The cascade is a surjection, not a bijection over 𝔹^16384,
/// due to the lossy Mersenne reduction at the boundary of Π_C.
pub fn permute(state: &mut State) {
    // 1. Pi_A (ℤ_{2^64})
    pi_a::pi_a(&mut state.lanes);

    // 2. Pi_B (GF(2^128))
    pi_b::pi_b(&mut state.lanes);

    // 3. Domain translation: Lane -> GF(p)
    // Lossy explicit Mersenne reduction step before Pi_C
    let mut f = [0u128; 128];
    for i in 0..128 {
        let v = ((state.lanes[i].hi as u128) << 64) | (state.lanes[i].lo as u128);
        f[i] = field_p::reduce(v);
    }

    // 4. Pi_C (GF(p))
    pi_c::pi_c(&mut f);

    // 5. Domain translation: GF(p) -> Lane
    for i in 0..128 {
        state.lanes[i].hi = (f[i] >> 64) as u64;
        state.lanes[i].lo = (f[i] & 0xFFFFFFFFFFFFFFFF) as u64;
    }
}
