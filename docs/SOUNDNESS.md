# VelesDB Soundness Documentation

> **Status**: 🔴 Draft - To be completed with EPIC-022

This document describes the invariants, assumptions, and soundness proofs for all `unsafe` code in VelesDB.

---

## Overview

VelesDB uses `unsafe` code in the following categories:

| Category | Modules | Reason |
|----------|---------|--------|
| **SIMD** | `distance/simd.rs`, `simd_dispatch.rs` | AVX2/AVX-512 intrinsics for vectorized operations |
| **Memory** | `storage/mmap.rs` | Memory-mapped files for zero-copy access |
| **Concurrency** | `graph/concurrent_*.rs` | Lock-free atomics for metrics |
| **FFI** | `velesdb-python`, `velesdb-wasm`, `velesdb-mobile` | Foreign function interface boundaries |

---

## SIMD Operations

### Module: `crates/velesdb-core/src/distance/simd.rs`

**Invariants**:
1. Vector slices passed to SIMD functions have length >= 8 (AVX2) or >= 16 (AVX-512)
2. Slices are aligned to 32-byte (AVX2) or 64-byte (AVX-512) boundaries when using aligned loads
3. Dimension is checked at collection creation time

**Assumptions**:
- CPU supports the target feature (checked at runtime via `is_x86_feature_detected!`)
- Caller has verified slice lengths match

**Why It's Sound**:
```rust
// Runtime feature detection ensures intrinsics are only called on supported CPUs
#[target_feature(enable = "avx2")]
unsafe fn cosine_avx2(a: &[f32], b: &[f32]) -> f32 {
    // SAFETY: Caller ensures a.len() == b.len() and len >= 8
    // ...
}
```

**Forbidden Scenarios**:
- ❌ Calling SIMD functions with mismatched slice lengths
- ❌ Using AVX-512 on CPUs without AVX-512 support
- ❌ Passing unaligned pointers to aligned load intrinsics

---

## Memory-Mapped Storage

### Module: `crates/velesdb-core/src/storage/mmap.rs`

**Invariants**:
1. Memory map is valid for the lifetime of the `MmapStorage` struct
2. File is not modified by external processes during access
3. Reads stay within mapped region bounds

**Assumptions**:
- Operating system enforces memory protection
- File system provides consistent reads

**Why It's Sound**:
```rust
// SAFETY: mmap region is valid for 'self lifetime
// Bounds are checked before any access
unsafe { slice::from_raw_parts(self.ptr.add(offset), len) }
```

**Forbidden Scenarios**:
- ❌ Accessing memory after unmap
- ❌ Concurrent file modification from external process
- ❌ Out-of-bounds access

---

## Concurrency Primitives

### Module: `crates/velesdb-core/src/graph/concurrent_edge_store.rs`

**Invariants**:
1. Lock ordering: `edges` → `outgoing` → `incoming` → `nodes`
2. Atomic counters use appropriate memory ordering
3. No data races on shared mutable state

**Assumptions**:
- `parking_lot::RwLock` provides correct synchronization
- Atomic operations have correct ordering semantics

**Why It's Sound**:
- All shared mutable access goes through locks
- Lock ordering prevents deadlocks
- Metrics use `Relaxed` ordering (acceptable for counters)

**Forbidden Scenarios**:
- ❌ Acquiring locks out of order
- ❌ Mutating shared state without holding lock
- ❌ Using `Relaxed` ordering for synchronization (only metrics)

---

## FFI Boundaries

### Module: `crates/velesdb-python/src/lib.rs`

**Invariants**:
1. PyO3 handles GIL correctly
2. Rust objects outlive Python references
3. No panic across FFI boundary

**Assumptions**:
- PyO3 correctly manages Python object lifetimes
- Python GIL provides thread safety for Python objects

**Why It's Sound**:
- PyO3 `#[pyclass]` enforces lifetime rules
- `catch_unwind` at FFI boundary (if needed)

---

## Soundness Checklist

Before adding new `unsafe` code:

- [ ] Is `unsafe` truly necessary? Can safe abstractions be used instead?
- [ ] All `unsafe fn` have `# Safety` documentation
- [ ] All `unsafe {}` blocks have `// SAFETY:` comments
- [ ] Invariants are documented and enforced
- [ ] No undefined behavior with valid inputs
- [ ] Edge cases tested (null, overflow, alignment)
- [ ] Miri tests added (if compatible)

---

## Unsafe Inventory

> **TODO**: Run `rg "unsafe" --type rust` and document each occurrence

| File | Line | Type | Justification |
|------|------|------|---------------|
| `distance/simd.rs` | TBD | `#[target_feature]` | SIMD intrinsics |
| `storage/mmap.rs` | TBD | `slice::from_raw_parts` | Memory mapping |
| ... | ... | ... | ... |

---

## References

- [Rustonomicon](https://doc.rust-lang.org/nomicon/)
- [Unsafe Code Guidelines](https://rust-lang.github.io/unsafe-code-guidelines/)
- [EPIC-022: Unsafe Auditability](../.epics/EPIC-022-unsafe-auditability/)
