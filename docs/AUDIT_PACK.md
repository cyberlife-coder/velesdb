# VelesDB Audit Pack

> **Reading Time**: ~10 minutes  
> **Status**: 🔴 Draft - To be completed with EPIC-027  
> **Last Updated**: 2026-01-22  
> **Version**: 1.x.x

---

## Executive Summary

VelesDB is a high-performance vector database written in Rust, designed for:
- **Fast kNN search** with HNSW indexing
- **Knowledge graphs** with MATCH queries
- **Memory safety** via Rust's ownership model

### Key Guarantees

| Guarantee | Proof | Status |
|-----------|-------|--------|
| No undefined behavior | Miri tests | 🔴 TODO |
| No data races | Loom tests | 🔴 TODO |
| Crash recovery | Kill -9 tests | 🔴 TODO |
| Query correctness | Recall benchmarks | ✅ Available |

---

## 1. Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      VelesDB Core                           │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
│  │  VelesQL    │  │    HNSW     │  │   Graph     │         │
│  │  Parser     │  │   Index     │  │   Engine    │         │
│  └─────────────┘  └─────────────┘  └─────────────┘         │
│         │                │                │                 │
│  ┌──────┴────────────────┴────────────────┴───────┐        │
│  │              Collection API                     │        │
│  └─────────────────────────────────────────────────┘        │
│         │                                                   │
│  ┌──────┴───────────────────────────────────────────┐      │
│  │              Storage Layer                        │      │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐       │      │
│  │  │  Mmap    │  │  WAL     │  │ Snapshot │       │      │
│  │  └──────────┘  └──────────┘  └──────────┘       │      │
│  └───────────────────────────────────────────────────┘      │
└─────────────────────────────────────────────────────────────┘
```

### Components

| Component | Description | Lines of Code |
|-----------|-------------|---------------|
| velesdb-core | Core engine | ~15,000 |
| velesdb-server | REST API | ~3,000 |
| velesdb-cli | CLI/REPL | ~1,500 |

---

## 2. Safety Proofs

### 2.1 Miri (Undefined Behavior Detection)

**Status**: 🔴 TODO (EPIC-025)

```bash
# Reproduce
cargo +nightly miri test --no-default-features --features miri-compat
```

**Documentation**: [docs/SOUNDNESS.md](SOUNDNESS.md)

---

### 2.2 Loom (Concurrency Testing)

**Status**: 🔴 TODO (EPIC-023)

```bash
# Reproduce
cargo +nightly test --test loom_tests --features loom
```

**Documentation**: [docs/CONCURRENCY_MODEL.md](CONCURRENCY_MODEL.md)

---

### 2.3 Crash Recovery

**Status**: 🔴 TODO (EPIC-024)

```powershell
# Reproduce (Windows)
.\scripts\crash_test.ps1 -Seed 42 -Count 10000
```

**Documentation**: [docs/STORAGE_FORMAT.md](STORAGE_FORMAT.md)

---

### 2.4 Fuzzing

**Status**: 🟡 Partial (existing targets, EPIC-025 adds more)

```bash
# Run parser fuzzer
cargo +nightly fuzz run fuzz_velesql_parser
```

**Documentation**: [docs/FUZZING.md](FUZZING.md)

---

## 3. Performance Benchmarks

### Reproduction

```powershell
# Full benchmark suite
.\benchmarks\bench_run.ps1 -Dataset sift1m -Runs 5

# Quick smoke test
cargo bench --bench smoke_test
```

### Reference Results

> **TODO**: Fill after EPIC-026 completion

| Metric | Value | Conditions |
|--------|-------|------------|
| Insert latency (p99) | TBD | 1M vectors, 128d |
| Search latency (p99) | TBD | k=10, ef=200 |
| Recall@10 | TBD | SIFT1M |

---

## 4. Code Quality Metrics

| Metric | Value | Tool |
|--------|-------|------|
| Clippy warnings | 0 | `cargo clippy -- -D warnings` |
| Unsafe blocks | Documented | [SOUNDNESS.md](SOUNDNESS.md) |
| Test coverage | TBD | `cargo tarpaulin` |
| Dependencies audited | ✅ | `cargo deny check` |

---

## 5. Known Limitations & Future Work

### Current Limitations

1. **Single-node only**: No distributed mode in Core
2. **HNSW rebuild**: Full rebuild required for major updates
3. **No transactions**: Atomic ops only

### Planned Improvements

- [ ] Loom tests (EPIC-023)
- [ ] Miri CI (EPIC-025)
- [ ] Crash recovery tests (EPIC-024)
- [ ] Reproducible benchmarks (EPIC-026)

---

## 6. Reproduction Checklist

```bash
# 1. Clone and build
git clone https://github.com/cyberlife-coder/VelesDB
cd VelesDB
cargo build --release

# 2. Run all tests
cargo test --workspace

# 3. Check code quality
cargo clippy -- -D warnings
cargo deny check

# 4. Run benchmarks (when available)
cargo bench --bench smoke_test
```

---

## Contact

- **Website**: https://velesdb.com
- **Email**: contact@wiscale.fr
- **GitHub**: https://github.com/cyberlife-coder/VelesDB

---

*This document will be updated as EPIC-022 through EPIC-027 are completed.*
