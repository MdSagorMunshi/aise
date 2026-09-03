## Summary of Changes

<!-- Provide a concise overview of what this pull request introduces or fixes. -->

### Related Issue(s)
<!-- If applicable, link to the relevant issue(s): e.g. Fixes #12 or Closes #5 -->

---

## Type of Change
<!-- Check all that apply -->
- [ ] 🐛 Bug fix (non-breaking change fixing an incorrect output, crash, or panic)
- [ ] ⚡ Performance / SIMD optimization (vectorization, memory layout, instruction scheduling)
- [ ] 🔬 Cryptographic primitive or sponge mode addition
- [ ] 📚 Documentation, specifications, or mathematical write-ups
- [ ] 🧪 Verification tests, fuzzers, or benchmarks

---

## Technical Details & Hardware Context

- **Domains/Crates Touched:** `aise-core` | `aise-cli` | `aise-gui` | `aise-bench` | `aise-tests`
- **Target Architecture(s) Tested:** `x86_64` (AVX-512) | `x86_64` (Scalar) | `aarch64` | Other: _______
- **Hardware Verified On:** <!-- e.g., AMD Ryzen 7000 / Intel Core 13th Gen / Apple Silicon -->

---

## Verification & Rigor Checklist

Before requesting review, please confirm the following checks:

- [ ] **Compilation:** Code builds cleanly with no new warnings:
  ```bash
  cargo check --workspace --all-targets
  ```
- [ ] **Test Suite:** All existing and new tests pass:
  ```bash
  cargo test --workspace
  ```
- [ ] **Frozen Vector Invariance:** Existing deterministic test vectors match without unintended regression (or changes are justified by an explicit specification update).
- [ ] **Equivalence Verification:** If modifying SIMD/hardware-accelerated paths (`AVX-512`, `VPCLMULQDQ`, `IFMA`), output is proven mathematically identical to the scalar reference implementation across fuzz/random inputs.
- [ ] **Formatting & Linter:** Code passes standard formatting rules:
  ```bash
  cargo fmt --all -- --check
  cargo clippy --workspace --all-targets
  ```
- [ ] **Licensing:** I confirm that all contributions are licensed under the dual MIT / Apache-2.0 licenses as specified in the project root.
