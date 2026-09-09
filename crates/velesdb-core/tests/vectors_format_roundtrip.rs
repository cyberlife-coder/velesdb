#![allow(clippy::cast_precision_loss)] // ids into f32 coordinates, deliberately

//! End-to-end coverage of `{basename}.vectors` across a real database lifecycle.
//!
//! # Why these assertions read the FILE and not the collection
//!
//! An earlier version of this file asserted through `Collection` — `len()` and
//! a handful of `search()` probes — and could not fail. Measured: delete
//! `native_hnsw.vectors` outright, reopen, and `len()` still answers 64 while
//! `search()` still returns the right id. The gap recovery rebuilds the graph
//! from `vectors.idx` and the WAL at open, so **no assertion reaching through
//! the collection API observes `.vectors` at all**.
//!
//! Everything below therefore reads the file: its header, its payload bytes,
//! and its size. That is the only surface on which a claim about this format
//! can be made.
//!
//! # What each test is for
//!
//! 1. **Opening never writes.** Checked on both sides of the arena capacity
//!    floor, because the regression that motivated this file came from the
//!    side *below* it: adoption there would size the arena up to the floor and
//!    extend the file, and `velesdb-memory`'s migration resume proves a source
//!    store unchanged by hashing exactly these files.
//! 2. **Growing an adopted arena persists every vector.** The sequence no
//!    other test performs: reopen (which maps the file), insert past the
//!    persisted count so the mapping must grow, save — which rewrites the
//!    header *in place* on a file the arena still maps — then read every f32
//!    back out of the file.
//! 3. **The grown arena was adopted, not copied.** Without this the file above
//!    could be exercising the copy path on both sides and prove nothing about
//!    mapping. The witness is public even though `backing_path` is not: an
//!    adopted arena doubles its capacity when it grows, so the file ends up
//!    larger than the payload it declares, while the copy path writes an exact
//!    fit.
//! 4. **A disposable arena's owner never deletes the durable store.** Run under
//!    `StorageMode::SQ8`, because `Full` never constructs an `ArenaHome` at all
//!    and the assertion would pass without ever reaching the hazard.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use serde_json::json;
use tempfile::TempDir;
use velesdb_core::{Database, DistanceMetric, Point, StorageMode};

/// Payload offset of a v2 `.vectors`, mirrored from `graph_io.rs`.
///
/// Duplicated rather than imported because this is an integration test: it
/// links the crate from outside, where the constant is `pub(crate)`. The header
/// assertion below refuses any version but 2, so a format that moves this
/// offset makes these tests fail loudly rather than read the wrong bytes.
const DATA_OFFSET: u64 = 4096;
const DIM: usize = 8;

/// Comfortably above the arena capacity floor, so adoption applies.
const ADOPTED: u64 = 64;
/// Comfortably below it, so the copy path applies.
const SMALL: u64 = 5;
/// Enough new points to force the mapping to grow past its persisted capacity.
const EXTRA: u64 = 32;

fn make_vector(id: u64) -> Vec<f32> {
    (0..DIM)
        .map(|i| ((id as f32) * 0.37 + (i as f32) * 0.11).sin())
        .collect()
}

fn points(range: std::ops::Range<u64>) -> Vec<Point> {
    range
        .map(|id| Point::new(id, make_vector(id), Some(json!({ "id": id }))))
        .collect()
}

/// Finds `native_hnsw.vectors` under the database directory.
///
/// Located by walking rather than by reconstructing the layout: a test that
/// hard-codes the path keeps passing when the layout moves, silently hashing a
/// file that is no longer the one under test.
fn vectors_file(root: &Path) -> PathBuf {
    let mut found = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).expect("test: read dir").flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.file_name().is_some_and(|n| n == "native_hnsw.vectors") {
                found.push(path);
            }
        }
    }
    assert_eq!(
        found.len(),
        1,
        "test: expected exactly one native_hnsw.vectors under {}, found {found:?}",
        root.display()
    );
    found.pop().expect("test: checked non-empty just above")
}

/// `(count, dimension)` from the header, refusing any version but 2.
///
/// The version check is not ceremony: a v3 that moves a field would otherwise
/// be decoded as if it were v2, and every assertion below would compare
/// confidently against the wrong bytes.
fn header(path: &Path) -> (u64, u32) {
    let bytes = std::fs::read(path).expect("test: read .vectors");
    assert!(
        bytes.len() >= usize::try_from(DATA_OFFSET).expect("test: offset fits a usize"),
        "test: file shorter than its own header region"
    );
    let version = u32::from_le_bytes(bytes[0..4].try_into().expect("test: 4 bytes"));
    assert_eq!(version, 2, "test: these assertions decode v2 only");
    let count = u64::from_le_bytes(bytes[4..12].try_into().expect("test: 8 bytes"));
    let dimension = u32::from_le_bytes(bytes[12..16].try_into().expect("test: 4 bytes"));
    (count, dimension)
}

/// Every vector the file declares, decoded from its payload.
///
/// This is the assertion surface the collection API cannot provide. Reads
/// exactly `count * dimension` values from `DATA_OFFSET`; anything the arena
/// left beyond them is uncommitted capacity the reader is meant to ignore.
fn payload(path: &Path) -> Vec<Vec<f32>> {
    let (count, dimension) = header(path);
    let bytes = std::fs::read(path).expect("test: read .vectors");
    let dim = dimension as usize;
    let base = usize::try_from(DATA_OFFSET).expect("test: offset fits a usize");
    let stored = usize::try_from(count).expect("test: count fits a usize");
    let needed = base + stored * dim * 4;
    assert!(
        bytes.len() >= needed,
        "test: file holds {} bytes but its header declares {needed}",
        bytes.len()
    );
    (0..stored)
        .map(|v| {
            (0..dim)
                .map(|k| {
                    let at = base + (v * dim + k) * 4;
                    f32::from_le_bytes(bytes[at..at + 4].try_into().expect("test: 4 bytes"))
                })
                .collect()
        })
        .collect()
}

fn digest(path: &Path) -> u64 {
    let mut hasher = DefaultHasher::new();
    std::fs::read(path)
        .expect("test: read .vectors")
        .hash(&mut hasher);
    hasher.finish()
}

/// Creates a collection holding `ids`, then closes the database.
fn seed(dir: &TempDir, ids: std::ops::Range<u64>, mode: StorageMode) {
    let db = Database::open(dir.path()).expect("test: open database");
    db.create_vector_collection_with_options("docs", DIM, DistanceMetric::Euclidean, mode)
        .expect("test: create collection");
    let collection = db
        .get_vector_collection("docs")
        .expect("test: collection exists");
    collection.upsert(points(ids)).expect("test: upsert");
    collection.flush_full().expect("test: flush");
}

/// Opening a collection must never write to its `.vectors`, on either side of
/// the arena capacity floor.
///
/// Below the floor adoption is refused and the copy path runs; above it the
/// file is mapped. Both must leave the bytes untouched. The regression this
/// guards took out twenty-one `velesdb-memory` tests when adoption first
/// shipped: its migration resume proves a source store unchanged by hashing
/// these very files, so a store that grew on open made a correct resume look
/// like a corrupted one.
#[test]
fn opening_a_collection_never_writes_to_its_vectors_file() {
    for count in [SMALL, ADOPTED] {
        let dir = TempDir::new().expect("test: tempdir");
        seed(&dir, 0..count, StorageMode::Full);
        let file = vectors_file(dir.path());
        let before = digest(&file);

        {
            let db = Database::open(dir.path()).expect("test: reopen");
            let collection = db
                .get_vector_collection("docs")
                .expect("test: collection exists");
            assert_eq!(
                collection.len(),
                usize::try_from(count).expect("test: count fits a usize"),
                "test: the fixture did not load at {count} points"
            );
        }

        assert_eq!(
            digest(&file),
            before,
            "test: opening a collection wrote to its .vectors at {count} points"
        );
    }
}

/// Growing an adopted arena and saving persists every vector to the file.
///
/// This is the sequence `File::create` would have broken: the dump rewrites the
/// header in place on a file the arena still maps, because truncating it would
/// remove the pages the mapping points at. The failure mode there is a SIGBUS
/// rather than an assertion, so what a test can do is prove the bytes are right
/// afterwards — which is why every vector is read back out of the file rather
/// than out of the collection.
#[test]
fn growing_an_adopted_arena_persists_every_vector_to_the_file() {
    let dir = TempDir::new().expect("test: tempdir");
    seed(&dir, 0..ADOPTED, StorageMode::Full);
    let file = vectors_file(dir.path());
    assert_eq!(header(&file).0, ADOPTED, "test: the seed did not persist");

    {
        let db = Database::open(dir.path()).expect("test: reopen");
        let collection = db
            .get_vector_collection("docs")
            .expect("test: collection exists");
        collection
            .upsert(points(ADOPTED..ADOPTED + EXTRA))
            .expect("test: upsert past the persisted count");
        collection.flush_full().expect("test: flush");
    }

    let total = ADOPTED + EXTRA;
    assert_eq!(
        header(&file).0,
        total,
        "test: the in-place header rewrite did not record the grown count"
    );

    let stored = payload(&file);
    assert_eq!(stored.len(), usize::try_from(total).expect("test: fits"));
    for (id, vector) in stored.iter().enumerate() {
        assert_eq!(
            vector.as_slice(),
            make_vector(id as u64).as_slice(),
            "test: vector {id} did not survive the grow-and-save"
        );
    }
}

/// The grown arena was adopted, not copied.
///
/// Without this, the test above could be exercising the copy path on both sides
/// and would prove nothing about mapping. `backing_path` is `pub(crate)` and
/// out of reach here, but the consequence is public: an adopted arena doubles
/// its capacity when it grows, so the file ends up **larger** than the payload
/// it declares. The copy path writes an exact fit.
#[test]
fn a_grown_arena_was_adopted_rather_than_copied() {
    let dir = TempDir::new().expect("test: tempdir");
    seed(&dir, 0..ADOPTED, StorageMode::Full);
    let file = vectors_file(dir.path());

    {
        let db = Database::open(dir.path()).expect("test: reopen");
        let collection = db
            .get_vector_collection("docs")
            .expect("test: collection exists");
        collection
            .upsert(points(ADOPTED..ADOPTED + EXTRA))
            .expect("test: upsert");
        collection.flush_full().expect("test: flush");
    }

    let (count, dimension) = header(&file);
    let exact_fit = DATA_OFFSET + count * u64::from(dimension) * 4;
    let actual = std::fs::metadata(&file).expect("test: stat").len();
    assert!(
        actual > exact_fit,
        "test: the file is an exact fit ({actual} bytes) — the arena was copied, \
         not adopted, so the mapped path this suite targets never ran"
    );
}

/// A collection whose arena is disposable must still keep its durable store.
///
/// `ArenaHome::drop` removes its file unconditionally, and that is correct: the
/// disposable arena is a cache. The hazard is pointing it at `.vectors`.
///
/// Run under `SQ8` on purpose. `Full` never constructs an `ArenaHome` at all,
/// so the same assertion there passes without the hazard ever being reached —
/// a green test about nothing.
#[test]
fn dropping_a_collection_with_a_disposable_arena_keeps_its_vectors() {
    let dir = TempDir::new().expect("test: tempdir");
    seed(&dir, 0..ADOPTED, StorageMode::SQ8);
    let file = vectors_file(dir.path());
    let before = digest(&file);

    {
        let db = Database::open(dir.path()).expect("test: reopen");
        let collection = db
            .get_vector_collection("docs")
            .expect("test: collection exists");
        assert_eq!(
            collection.len(),
            usize::try_from(ADOPTED).expect("test: fits"),
            "test: the fixture did not load"
        );
    }

    assert!(
        file.exists(),
        "test: closing a collection deleted its durable vector store"
    );
    assert_eq!(
        digest(&file),
        before,
        "test: closing a collection rewrote its durable vector store"
    );
}
