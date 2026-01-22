# VelesDB Concurrency Model

> **Status**: 🔴 Draft - To be completed with EPIC-023

This document describes VelesDB's concurrency model for users and contributors.

---

## Overview

VelesDB uses a concurrency model optimized for:
- **High read throughput**: Multiple concurrent readers
- **Safe writes**: Serialized writes with minimal blocking
- **Scalability**: Sharding to reduce lock contention

---

## Architecture

### Sharding Strategy

```
┌─────────────────────────────────────────────────────────────┐
│              ConcurrentEdgeStore (256 shards)               │
├─────────┬─────────┬─────────┬─────────┬─────────┬───────────┤
│ Shard 0 │ Shard 1 │ Shard 2 │ Shard 3 │   ...   │ Shard 255 │
│ RwLock  │ RwLock  │ RwLock  │ RwLock  │         │ RwLock    │
└─────────┴─────────┴─────────┴─────────┴─────────┴───────────┘

Shard assignment: hash(node_id) % num_shards
```

### Lock Types

| Component | Lock Type | Granularity | Notes |
|-----------|-----------|-------------|-------|
| EdgeStore shards | `RwLock` | Per-shard | Low contention |
| NodeStore shards | `RwLock` | Per-shard | Low contention |
| HNSW layers | `RwLock` | Global | Read-heavy |
| HNSW neighbors | `RwLock` | Per-node | High contention during insert |
| Metrics | `Atomic` | None | Lock-free |
| Config | `RwLock` | Global | Rarely modified |

---

## Thread Safety Guarantees

### Send + Sync Types

| Type | Send | Sync | Notes |
|------|------|------|-------|
| `Collection` | ✅ | ✅ | Safe to share across threads |
| `HnswIndex` | ✅ | ✅ | Concurrent search/insert |
| `ConcurrentEdgeStore` | ✅ | ✅ | Sharded for scalability |
| `ConcurrentNodeStore` | ✅ | ✅ | Sharded for scalability |

### !Send or !Sync Types

| Type | Reason | Workaround |
|------|--------|------------|
| `GraphTraversal` | Contains references | Use on creation thread |
| `QueryCursor` | Iterator state | Single-thread only |
| `MmapStorage` (internal) | Platform-specific | Wrapped in Arc |

---

## Lock Ordering (Deadlock Prevention)

**CRITICAL**: Always acquire locks in this order:

```
1. edges (EdgeStore shards)
2. outgoing (index)
3. incoming (index)
4. nodes (NodeStore shards)
```

### Anti-pattern ❌

```rust
let nodes = store.nodes.write(); // WRONG: nodes before edges
let edges = store.edges.write(); // DEADLOCK POSSIBLE
```

### Correct Pattern ✅

```rust
let edges = store.edges.write(); // edges FIRST
let nodes = store.nodes.write(); // nodes AFTER
```

---

## Memory Ordering for Atomics

| Use Case | Ordering | Justification |
|----------|----------|---------------|
| Metrics counters | `Relaxed` | No synchronization needed |
| Entry point (HNSW) | `SeqCst` | Must be visible immediately |
| Edge count | `Relaxed` | Approximate is acceptable |

---

## Performance vs Safety Tradeoffs

### Read-Heavy Workloads

- `RwLock` allows unlimited concurrent readers
- Sharding distributes load across locks
- Batch reads minimize lock acquisitions

### Write-Heavy Workloads

- Batch inserts recommended (reduce lock overhead)
- Consider increasing shard count for high write load
- Writes block other writes to same shard only

### Tuning Parameters

```rust
// Increase shards for write-heavy workloads
ConcurrentEdgeStore::with_estimated_edges(1_000_000) // Auto-calculates optimal shards
```

---

## Known Limitations

1. **Cross-shard operations**: Queries spanning multiple shards hold multiple locks
2. **Large traversals**: Can block writers for extended periods
3. **HNSW rebuild**: Currently single-threaded
4. **No distributed locks**: Single-node only in Core

---

## Best Practices

1. **Use batch operations** when inserting many vectors/edges
2. **Limit traversal depth** to avoid long-held locks
3. **Size shards appropriately** using `with_estimated_edges()`
4. **Avoid mixing reads/writes** in tight loops

---

## Loom Testing

VelesDB uses [Loom](https://github.com/tokio-rs/loom) to verify concurrency correctness.

```bash
# Run Loom tests
cargo +nightly test --test loom_tests --features loom
```

See [EPIC-023](../.epics/EPIC-023-loom-concurrency/) for details.

---

## References

- [Rust Atomics and Locks](https://marabos.nl/atomics/)
- [The Rustonomicon - Concurrency](https://doc.rust-lang.org/nomicon/concurrency.html)
- [parking_lot documentation](https://docs.rs/parking_lot/)
