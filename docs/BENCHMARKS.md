# 📊 VelesDB Performance Benchmarks

*Last updated: December 2025*

This document details the performance benchmarks for VelesDB v0.1.1. Tests were conducted on a standard workstation (8-core CPU, AVX2 support).

> 📈 **See also**: [Performance Optimization Roadmap](./PERFORMANCE_ROADMAP.md) for planned improvements.
>
> 🔧 **See also**: [Benchmarking Guide](./BENCHMARKING_GUIDE.md) for reproducible benchmark setup.

## 🚀 Summary

| Operation | Metric | Time (768d) | Throughput | Speedup vs Baseline |
|-----------|--------|-------------|------------|---------------------|
| **Cosine Similarity** | Latency | **~310 ns** | ~3.2M ops/sec | **2.9x** |
| **Euclidean Distance** | Latency | **~138 ns** | ~7.2M ops/sec | **2.2x** |
| **Dot Product** | Latency | **~130 ns** | ~7.7M ops/sec | **2.1x** |
| **Hamming (Binary)** | Latency | **~6 ns** | ~164M ops/sec | **~34x** (vs f32) |
| **Metadata Filter** | Range (10k) | **~416 µs** | ~24M items/sec | - |
| **VelesQL Parser** | Simple | **~528 ns** | ~1.9M qps | - |

> **Note**: Pure SIMD kernel benchmarks show potential for even higher performance (~88ns for Cosine, ~47ns for Euclidean). The current integration adds overhead which is a target for future optimization.

---

## ⚡ SIMD Vector Operations

Comparison between standard Rust iterators (Baseline) and VelesDB's explicit SIMD optimizations (Optimized).
Results are from `search_benchmark` which measures the full public API call overhead.

### Cosine Similarity (768 dimensions)
> Used for semantic search and text embeddings.

| Implementation | Time per op | Throughput |
|----------------|-------------|------------|
| Baseline (Auto-vec) | 909 ns | 1.1M ops/s |
| **VelesDB Optimized** | **310 ns** | **3.2M ops/s** |
| **Improvement** | **-66% latency** | **2.9x throughput** |

### Euclidean Distance (768 dimensions)
> Used for spatial data and image features.

| Implementation | Time per op | Throughput |
|----------------|-------------|------------|
| Baseline (Auto-vec) | 305 ns | 3.3M ops/s |
| **VelesDB Optimized** | **138 ns** | **7.2M ops/s** |
| **Improvement** | **-55% latency** | **2.2x throughput** |

### Dot Product (768 dimensions)
> Used for raw similarity and inner product.

| Implementation | Time per op | Throughput |
|----------------|-------------|------------|
| Baseline (Auto-vec) | 267 ns | 3.7M ops/s |
| **VelesDB Optimized** | **130 ns** | **7.7M ops/s** |
| **Improvement** | **-51% latency** | **2.1x throughput** |

### Binary Hamming Distance (768 bits / 12 u64)
> Used for binary fingerprints and image hashing.

| Implementation | Time per op | Throughput |
|----------------|-------------|------------|
| Float32 Baseline | ~206 ns | 4.8M ops/s |
| **VelesDB Optimized** | **~6.1 ns** | **164M ops/s** |
| **Improvement** | **-97% latency** | **~34x throughput** |

---

## 🔍 Metadata Filtering

Benchmarks for filtering operations.

| Filter Type | Condition | Time (10k items) | Throughput |
|-------------|-----------|------------------|------------|
| **Equality** | `category = 'tech'` | ~194 µs | 51M items/s |
| **In List** | `status IN ('a', 'b')` | ~242 µs | 41M items/s |
| **Range** | `price > 100` | ~416 µs | 24M items/s |
| **Nested** | `meta.tags CONTAINS 'a'` | ~355 µs | 28M items/s |
| **Complex** | `(A AND B) OR C` | ~500 µs | 20M items/s |

### Throughput vs Scale

| Dataset Size | Time per Batch | Effective Throughput |
|--------------|----------------|----------------------|
| 1,000 items | 14.6 µs | **68.5 M/s** |
| 10,000 items | 151 µs | **66.2 M/s** |
| 100,000 items | 5.15 ms | **19.4 M/s** |

> **Note**: For high-performance filtering at scale, use the new `ColumnStore` module which provides 44-122x faster filtering than JSON. See `column_store.rs` for bitmap-based filtering that supports efficient AND/OR combinations.

---

## 📝 VelesQL Parsing

Performance of the SQL-like query parser.

| Query Type | Complexity | Time | Throughput |
|------------|------------|------|------------|
| **Simple** | `SELECT * FROM table` | 528 ns | 1.9M qps |
| **Vector** | `... WHERE vector NEAR $v` | 835 ns | 1.2M qps |
| **Complex** | Multiple conditions | 3.6 µs | 277k qps |

---

## 🧪 Methodology

- **Hardware**: Windows Workstation, 8-core CPU
- **Environment**: Rust 1.83, Release build (`--release`)
- **Framework**: Criterion.rs
- **Optimizations**: AVX2 enabled, `target-cpu=native`
