# VelesDB Fuzzing Guide

> **Status**: 🔴 Draft - To be completed with EPIC-025

This guide describes how to run and contribute to VelesDB's fuzzing infrastructure.

---

## Overview

VelesDB uses [cargo-fuzz](https://github.com/rust-fuzz/cargo-fuzz) (libFuzzer) for continuous fuzzing of critical components.

---

## Setup

```bash
# Install cargo-fuzz (requires nightly)
cargo install cargo-fuzz

# List available targets
cargo fuzz list
```

---

## Fuzz Targets

| Target | Component | What It Tests |
|--------|-----------|---------------|
| `fuzz_velesql_parser` | VelesQL Parser | Arbitrary SQL-like strings |
| `fuzz_snapshot_parser` | Snapshot Loader | Arbitrary binary snapshots |
| `fuzz_distance_metrics` | Distance Calculations | Edge case vectors |

> **TODO**: Add targets from EPIC-025: `fuzz_wal_replay`, `fuzz_graph_query`

---

## Running Fuzzers

### Basic Usage

```bash
# Run VelesQL parser fuzzer
cargo +nightly fuzz run fuzz_velesql_parser

# Run with timeout (1 hour)
cargo +nightly fuzz run fuzz_velesql_parser -- -max_total_time=3600

# Run with corpus
cargo +nightly fuzz run fuzz_velesql_parser fuzz/corpus/velesql_parser/
```

### Parallel Fuzzing

```bash
# Run 4 parallel instances
cargo +nightly fuzz run fuzz_velesql_parser -- -jobs=4 -workers=4
```

---

## Reproducing Crashes

When a crash is found, it's saved to `fuzz/artifacts/<target>/`.

```bash
# Reproduce crash
cargo +nightly fuzz run fuzz_velesql_parser fuzz/artifacts/fuzz_velesql_parser/crash-abc123

# Minimize crash (find smallest input)
cargo +nightly fuzz tmin fuzz_velesql_parser fuzz/artifacts/fuzz_velesql_parser/crash-abc123
```

---

## Contributing Corpus

Good corpus inputs improve fuzzing effectiveness:

1. Add inputs to `fuzz/corpus/<target>/`
2. Use descriptive names: `valid_select_01`, `edge_case_empty`
3. Keep corpus minimal but representative
4. Include:
   - Valid inputs (happy path)
   - Edge cases (empty, max size, unicode)
   - Previously found crashes (minimized)

---

## Writing New Fuzz Targets

```rust
// fuzz/fuzz_targets/fuzz_my_component.rs
#![no_main]

use libfuzzer_sys::fuzz_target;
use velesdb_core::my_component;

fuzz_target!(|data: &[u8]| {
    // Convert input if needed
    if let Ok(input) = std::str::from_utf8(data) {
        // Call the function under test
        // It should NOT panic with arbitrary input
        let _ = my_component::parse(input);
    }
});
```

### Guidelines

- **No panics**: Function should return `Result` or handle errors gracefully
- **Bounded resources**: Limit memory/time to prevent hangs
- **Deterministic**: Same input → same behavior
- **Focused**: One component per target

---

## CI Integration

```yaml
# .github/workflows/fuzz.yml (optional, expensive)
fuzz:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@nightly
    - run: cargo install cargo-fuzz
    - run: cargo +nightly fuzz run fuzz_velesql_parser -- -max_total_time=300
```

---

## Invariants to Test

| Component | Invariant |
|-----------|-----------|
| VelesQL Parser | No panic, returns valid AST or error |
| WAL Decoder | Roundtrip: decode(encode(x)) == x |
| Snapshot | Corruption detected, no UB |
| Distance | Valid float result, no NaN propagation |
| Graph Query | Bounded execution, no infinite loops |

---

## Reporting Issues

If you find a crash:

1. Minimize the crash file: `cargo +nightly fuzz tmin ...`
2. Check if it's a known issue
3. Open a GitHub issue with:
   - Crash file (base64 encoded if binary)
   - VelesDB version
   - Reproduction command
   - Expected behavior

---

## References

- [cargo-fuzz documentation](https://rust-fuzz.github.io/book/cargo-fuzz.html)
- [libFuzzer documentation](https://llvm.org/docs/LibFuzzer.html)
- [Rust Fuzz Book](https://rust-fuzz.github.io/book/)
- [EPIC-025](../.epics/EPIC-025-miri-fuzzing/)
