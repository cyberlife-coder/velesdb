# VelesDB Storage Format Specification

> **Status**: 🔴 Draft - To be completed with EPIC-024
> **Version**: 1.0.0
> **Last Updated**: 2026-01-22

---

## Overview

VelesDB persists data in a binary format optimized for:
- Fast memory-mapped access
- Crash recovery with checksums
- Incremental updates

---

## File Layout

```
collection_directory/
├── metadata.json      # Collection configuration
├── vectors.bin        # Vector data
├── index.bin          # HNSW index
├── properties.bin     # Property index (optional)
└── wal/               # Write-ahead log (if enabled)
    ├── segment_000001.wal
    └── segment_000002.wal
```

---

## Vector Data File (vectors.bin)

### Header (64 bytes)

```
┌─────────────────────────────────────────────────────────────┐
│                      HEADER (64 bytes)                      │
├─────────────┬─────────────┬─────────────┬───────────────────┤
│ Magic (4B)  │ Version (4B)│ Dim (4B)    │ Count (8B)        │
│ "VELS"      │ 0x00010000  │ 128         │ 1000000           │
├─────────────┴─────────────┴─────────────┴───────────────────┤
│ Checksum (4B) │ Flags (4B) │ Reserved (36B)                 │
└─────────────────────────────────────────────────────────────┘
```

### Header Fields

| Offset | Size | Type | Description |
|--------|------|------|-------------|
| 0 | 4 | bytes | Magic: `VELS` (0x56454C53) |
| 4 | 4 | u32 | Format version (major.minor.patch packed) |
| 8 | 4 | u32 | Vector dimension |
| 12 | 8 | u64 | Vector count |
| 20 | 4 | u32 | Header checksum (CRC32) |
| 24 | 4 | u32 | Flags (reserved) |
| 28 | 36 | - | Reserved for future use |

### Vector Entry

| Field | Size | Type | Description |
|-------|------|------|-------------|
| data | dim × 4 | [f32] | Vector components (little-endian) |
| id_len | 2 | u16 | Document ID length |
| id | var | UTF-8 | Document ID |
| meta_len | 4 | u32 | Metadata JSON length (0 if none) |
| meta | var | UTF-8 | Metadata JSON |
| checksum | 4 | u32 | Entry checksum (CRC32) |

---

## Endianness

All multi-byte integers are stored in **little-endian** format.

---

## Checksums

- **Algorithm**: CRC32 (IEEE polynomial)
- **Scope**: Per-entry and header checksum
- **Validation**: On load and periodic integrity checks

---

## Versioning

### Version Format

`MAJOR.MINOR.PATCH` packed as `(major << 16) | (minor << 8) | patch`

- **MAJOR**: Breaking format change (incompatible)
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes (fully compatible)

### Current Version

- **1.0.0** (0x00010000): Initial stable format

### Compatibility Matrix

| Reader Version | File Version | Behavior |
|----------------|--------------|----------|
| 1.x.x | 1.x.x | Full support |
| 1.x.x | 2.x.x | Error: upgrade required |
| 2.x.x | 1.x.x | Read-only or migrate |

---

## Write-Ahead Log (WAL)

### Segment Format

```
┌─────────────────────────────────────────────────────────────┐
│                    SEGMENT HEADER (32 bytes)                │
├─────────────┬─────────────┬─────────────────────────────────┤
│ Magic (4B)  │ Version (4B)│ Segment ID (8B) │ Reserved (16B)│
└─────────────┴─────────────┴─────────────────────────────────┘
│                      WAL ENTRIES                            │
├─────────────────────────────────────────────────────────────┤
│ Entry: Type (1B) + Len (4B) + Data (var) + CRC (4B)        │
│ ...                                                         │
└─────────────────────────────────────────────────────────────┘
```

### WAL Entry Types

| Type | Value | Description |
|------|-------|-------------|
| INSERT | 0x01 | Vector insertion |
| DELETE | 0x02 | Vector deletion |
| UPDATE | 0x03 | Metadata update |
| CHECKPOINT | 0xFF | Checkpoint marker |

---

## Recovery Process

1. Load latest checkpoint
2. Replay WAL entries after checkpoint
3. Verify checksums
4. Rebuild index if inconsistent
5. Truncate corrupt WAL tail

---

## Migration Strategy

When a breaking change is needed:

1. Increment MAJOR version
2. Provide migration tool: `velesdb migrate --from 1 --to 2`
3. Document breaking changes in CHANGELOG
4. Support reading old format for at least 1 major version

---

## Known Limitations

- Maximum vector dimension: 65,535 (u16 limit)
- Maximum document ID length: 65,535 bytes
- Maximum metadata size: 4 GB (u32 limit)
- Maximum file size: Limited by filesystem

---

## References

- [SQLite File Format](https://www.sqlite.org/fileformat.html)
- [LMDB Data Format](http://www.lmdb.tech/doc/)
- [EPIC-024](../.epics/EPIC-024-durability-crash-recovery/)
