//! A real database survives close and reopen with its `.vectors` untouched.
//!
//! # Why this exists as an integration test
//!
//! #2213 moved the `.vectors` payload to a page boundary and #2215 made the
//! durable file *be* the graph's arena. Both shipped with unit tests written
//! against `NativeHnsw` — by the same hand that wrote the change. What none of
//! them exercised is the path a user takes: open a database, insert, close,
//! reopen, and find the data where it was left.
//!
//! Two properties are asserted here that the unit tests structurally cannot,
//! because they need a whole `Database` and a real directory:
//!
//! 1. **Opening a collection does not write to its `.vectors`.** The file is
//!    hashed before and after a reopen. This is not tidiness: `velesdb-memory`'s
//!    migration resume proves a source store unchanged by hashing exactly these
//!    bytes, so a store that grew on open would make a correct resume look like
//!    a corrupted one. The unit test for this drives `NativeHnsw` directly; a
//!    collection open does considerably more.
//!
//! 2. **An adopted arena can grow and be saved without losing anything.** The
//!    riskiest sequence #2215 introduced, and the one with no coverage at all:
//!    reopen (which maps the durable file), insert past the persisted count so
//!    the mapping must grow, save (which rewrites the header *in place* on a
//!    file the arena still maps), then reopen again and read every vector back.
//!    `File::create` on that path would have truncated the bytes the live
//!    mapping still pointed at — a SIGBUS rather than a failed assertion, which
//!    is why the dump branches instead of relying on a test to notice.
//!
//! # The count is not arbitrary
//!
//! Adoption is refused below `ContiguousVectors::MIN_ARENA_CAPACITY`, because
//! an arena sized up to that floor would extend the file merely by opening it.
//! A test running under the floor would take the copy path on both sides while
//! looking like it exercised adoption, so the counts here clear it by a wide
//! margin.
//!
//! **What this file cannot do is observe that adoption happened.**
//! `ContiguousVectors::backing_path` is `pub(crate)`, and reaching for it from
//! outside would mean widening a crate-internal accessor to satisfy a test —
//! the wrong trade. The unit suite in `vectors_format_tests.rs` asserts the
//! mapping directly and is where that belongs.
//!
//! These tests assert the properties a *user* can observe, which must hold on
//! either path: the file survives, it is not written by an open, the header
//! records what was persisted, and every vector reads back. If adoption
//! silently stopped happening they would still pass — and they should, because
//! nothing a user relies on would have broken. What they catch is the opposite
//! and worse case: adoption happening and losing or corrupting data.

#![allow(clippy::cast_precision_loss)] // ids into f32 coordinates, deliberately

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use serde_json::json;
use tempfile::TempDir;
use velesdb_core::{Database, DistanceMetric, Point};

/// Comfortably above the arena capacity floor, so a reopen adopts the file.
const POINTS: u64 = 64;
/// Added after the first reopen, to force the mapped arena to grow.
const EXTRA: u64 = 32;
const DIM: usize = 8;

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
    fn walk(dir: &Path, found: &mut Vec<PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, found);
            } else if path.file_name().is_some_and(|n| n == "native_hnsw.vectors") {
                found.push(path);
            }
        }
    }
    let mut found = Vec::new();
    walk(root, &mut found);
    assert_eq!(
        found.len(),
        1,
        "expected exactly one native_hnsw.vectors under {}, found {found:?}",
        root.display()
    );
    found.pop().expect("checked non-empty")
}

fn digest(path: &Path) -> u64 {
    let bytes = std::fs::read(path).expect("read .vectors");
    let mut hasher = DefaultHasher::new();
    bytes.hash(&mut hasher);
    hasher.finish()
}

/// The declared vector count in the `.vectors` header: version(4), count(8).
fn declared_count(path: &Path) -> u64 {
    let bytes = std::fs::read(path).expect("read .vectors");
    u64::from_le_bytes(bytes[4..12].try_into().expect("count field"))
}

fn seed(dir: &TempDir, ids: std::ops::Range<u64>) {
    let db = Database::open(dir.path()).expect("open database");
    db.create_vector_collection("docs", DIM, DistanceMetric::Cosine)
        .expect("create collection");
    let collection = db.get_vector_collection("docs").expect("collection exists");
    collection.upsert(points(ids)).expect("upsert");
    collection.flush_full().expect("flush");
}

/// Ranked ids for a fixed query — the comparable the round trip preserves.
fn ranking(db: &Database) -> Vec<u64> {
    let collection = db.get_vector_collection("docs").expect("collection exists");
    collection
        .search(&make_vector(7), 10)
        .expect("search")
        .into_iter()
        .map(|hit| hit.point.id)
        .collect()
}

#[test]
fn reopening_a_collection_does_not_touch_its_vectors_file() {
    // Arrange
    let dir = TempDir::new().expect("tempdir");
    seed(&dir, 0..POINTS);
    let file = vectors_file(dir.path());
    let before = digest(&file);
    assert_eq!(
        declared_count(&file),
        POINTS,
        "the dump must declare its rows"
    );

    let db = Database::open(dir.path()).expect("reopen");
    let expected = ranking(&db);
    assert!(!expected.is_empty(), "the query must match something");

    // Act
    drop(db);

    // Assert: the file is still there, and byte-identical.
    assert!(
        file.exists(),
        "closing a collection must never delete its durable vector store"
    );
    assert_eq!(
        digest(&file),
        before,
        "opening a collection wrote to its .vectors"
    );

    // And it still reads back the same ranking.
    let db = Database::open(dir.path()).expect("second reopen");
    assert_eq!(
        ranking(&db),
        expected,
        "the ranking must survive a round trip"
    );
}

#[test]
fn growing_an_adopted_arena_and_saving_keeps_every_vector() {
    // Arrange: a persisted collection, then a reopen that maps the file.
    let dir = TempDir::new().expect("tempdir");
    seed(&dir, 0..POINTS);
    let file = vectors_file(dir.path());
    assert_eq!(declared_count(&file), POINTS);

    // Act: insert past the persisted count, so the mapping has to grow, then
    // save — which rewrites the header in place on the file it maps.
    {
        let db = Database::open(dir.path()).expect("reopen");
        let collection = db.get_vector_collection("docs").expect("collection exists");
        collection
            .upsert(points(POINTS..POINTS + EXTRA))
            .expect("upsert past the persisted count");
        collection.flush_full().expect("flush");
    }

    // Assert: the header caught up, and every vector reads back.
    assert_eq!(
        declared_count(&file),
        POINTS + EXTRA,
        "the header must record the grown count"
    );

    let db = Database::open(dir.path()).expect("final reopen");
    let collection = db.get_vector_collection("docs").expect("collection exists");
    let expected_len = usize::try_from(POINTS + EXTRA).expect("test: the point count fits a usize");
    assert_eq!(collection.len(), expected_len);

    for id in [0, 7, POINTS - 1, POINTS, POINTS + EXTRA - 1] {
        let hits = collection.search(&make_vector(id), 1).expect("search");
        assert_eq!(
            hits.first().map(|hit| hit.point.id),
            Some(id),
            "vector {id} did not survive growth through its own mapping"
        );
    }
}
