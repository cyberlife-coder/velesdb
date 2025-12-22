//! WASM Performance Benchmarks
//!
//! Run with: `wasm-pack test --node --release -- --ignored`
//!
//! These benchmarks measure actual WASM performance for:
//! - Single insert vs batch insert
//! - Search latency at various scales
//! - Memory pre-allocation impact

#![cfg(target_arch = "wasm32")]

use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;

use velesdb_wasm::VectorStore;

// Helper to convert batch to JsValue
fn to_js_batch(batch: Vec<(u64, Vec<f32>)>) -> JsValue {
    serde_wasm_bindgen::to_value(&batch).unwrap()
}

// =============================================================================
// Benchmark: Single Insert vs Batch Insert
// =============================================================================

#[wasm_bindgen_test]
#[ignore] // Run with --ignored flag
fn bench_single_insert_10k_128d() {
    let mut store = VectorStore::new(128, "cosine").unwrap();

    let start = js_sys::Date::now();

    for i in 0..10_000_u64 {
        let vector: Vec<f32> = (0..128).map(|j| (i + j) as f32 * 0.001).collect();
        store.insert(i, &vector).unwrap();
    }

    let elapsed = js_sys::Date::now() - start;
    let per_insert = elapsed / 10_000.0;

    web_sys::console::log_1(
        &format!(
            "📊 BENCH single_insert_10k_128d: {:.2}ms total, {:.3}µs/insert",
            elapsed,
            per_insert * 1000.0
        )
        .into(),
    );

    assert!(per_insert < 1.0, "Insert should be < 1ms each");
}

#[wasm_bindgen_test]
#[ignore]
fn bench_batch_insert_10k_128d() {
    let mut store = VectorStore::with_capacity(128, "cosine", 10_000).unwrap();

    let batch: Vec<(u64, Vec<f32>)> = (0..10_000_u64)
        .map(|i| {
            let vector: Vec<f32> = (0..128).map(|j| (i + j) as f32 * 0.001).collect();
            (i, vector)
        })
        .collect();

    let start = js_sys::Date::now();
    store.insert_batch(to_js_batch(batch)).unwrap();
    let elapsed = js_sys::Date::now() - start;

    let per_insert = elapsed / 10_000.0;

    web_sys::console::log_1(
        &format!(
            "📊 BENCH batch_insert_10k_128d: {:.2}ms total, {:.3}µs/insert",
            elapsed,
            per_insert * 1000.0
        )
        .into(),
    );

    assert!(per_insert < 0.5, "Batch insert should be < 0.5ms each");
}

// =============================================================================
// Benchmark: Search Performance at Scale
// =============================================================================

#[wasm_bindgen_test]
#[ignore]
fn bench_search_10k_128d() {
    let mut store = VectorStore::with_capacity(128, "cosine", 10_000).unwrap();

    // Setup: batch insert
    let batch: Vec<(u64, Vec<f32>)> = (0..10_000_u64)
        .map(|i| {
            let vector: Vec<f32> = (0..128).map(|j| (i + j) as f32 * 0.001).collect();
            (i, vector)
        })
        .collect();
    store.insert_batch(to_js_batch(batch)).unwrap();

    let query: Vec<f32> = (0..128).map(|i| i as f32 * 0.001).collect();

    // Warmup
    for _ in 0..10 {
        let _ = store.search(&query, 10);
    }

    // Benchmark
    let iterations = 100;
    let start = js_sys::Date::now();

    for _ in 0..iterations {
        let _ = store.search(&query, 10);
    }

    let elapsed = js_sys::Date::now() - start;
    let per_search = elapsed / iterations as f64;

    web_sys::console::log_1(
        &format!(
            "📊 BENCH search_10k_128d: {:.2}ms avg ({} iterations)",
            per_search, iterations
        )
        .into(),
    );

    assert!(per_search < 50.0, "Search should be < 50ms");
}

#[wasm_bindgen_test]
#[ignore]
fn bench_search_100k_768d() {
    let mut store = VectorStore::with_capacity(768, "cosine", 100_000).unwrap();

    // Setup: batch insert in chunks
    for chunk_start in (0..100_000_u64).step_by(10_000) {
        let batch: Vec<(u64, Vec<f32>)> = (chunk_start..chunk_start + 10_000)
            .map(|i| {
                let vector: Vec<f32> = (0..768).map(|j| (i + j) as f32 * 0.0001).collect();
                (i, vector)
            })
            .collect();
        store.insert_batch(to_js_batch(batch)).unwrap();
    }

    let query: Vec<f32> = (0..768).map(|i| i as f32 * 0.0001).collect();

    // Warmup
    for _ in 0..5 {
        let _ = store.search(&query, 10);
    }

    // Benchmark
    let iterations = 20;
    let start = js_sys::Date::now();

    for _ in 0..iterations {
        let _ = store.search(&query, 10);
    }

    let elapsed = js_sys::Date::now() - start;
    let per_search = elapsed / iterations as f64;

    web_sys::console::log_1(
        &format!(
            "📊 BENCH search_100k_768d: {:.2}ms avg ({} iterations)",
            per_search, iterations
        )
        .into(),
    );

    // 100k vectors at 768D is heavy - allow up to 500ms
    assert!(per_search < 500.0, "Search should be < 500ms");
}

// =============================================================================
// Benchmark: Memory Pre-allocation Impact
// =============================================================================

#[wasm_bindgen_test]
#[ignore]
fn bench_with_vs_without_capacity() {
    // Without capacity
    let start1 = js_sys::Date::now();
    {
        let mut store = VectorStore::new(128, "cosine").unwrap();
        for i in 0..5_000_u64 {
            let vector: Vec<f32> = (0..128).map(|j| (i + j) as f32 * 0.001).collect();
            store.insert(i, &vector).unwrap();
        }
    }
    let without_capacity = js_sys::Date::now() - start1;

    // With capacity
    let start2 = js_sys::Date::now();
    {
        let mut store = VectorStore::with_capacity(128, "cosine", 5_000).unwrap();
        for i in 0..5_000_u64 {
            let vector: Vec<f32> = (0..128).map(|j| (i + j) as f32 * 0.001).collect();
            store.insert(i, &vector).unwrap();
        }
    }
    let with_capacity = js_sys::Date::now() - start2;

    let improvement = without_capacity / with_capacity;

    web_sys::console::log_1(
        &format!(
            "📊 BENCH capacity_impact: without={:.2}ms, with={:.2}ms, speedup={:.2}x",
            without_capacity, with_capacity, improvement
        )
        .into(),
    );

    assert!(
        with_capacity <= without_capacity,
        "Pre-allocation should not be slower"
    );
}

// =============================================================================
// Benchmark: High Dimension Performance
// =============================================================================

#[wasm_bindgen_test]
#[ignore]
fn bench_insert_1k_1536d() {
    // GPT-3/4 embedding dimension
    let mut store = VectorStore::with_capacity(1536, "cosine", 1_000).unwrap();

    let batch: Vec<(u64, Vec<f32>)> = (0..1_000_u64)
        .map(|i| {
            let vector: Vec<f32> = (0..1536).map(|j| (i + j) as f32 * 0.0001).collect();
            (i, vector)
        })
        .collect();

    let start = js_sys::Date::now();
    store.insert_batch(to_js_batch(batch)).unwrap();
    let elapsed = js_sys::Date::now() - start;

    web_sys::console::log_1(
        &format!(
            "📊 BENCH insert_1k_1536d: {:.2}ms total, {:.3}µs/insert",
            elapsed,
            elapsed / 1_000.0 * 1000.0
        )
        .into(),
    );

    assert!(elapsed < 5000.0, "1k inserts at 1536D should be < 5s");
}

#[wasm_bindgen_test]
#[ignore]
fn bench_search_1k_1536d() {
    let mut store = VectorStore::with_capacity(1536, "cosine", 1_000).unwrap();

    let batch: Vec<(u64, Vec<f32>)> = (0..1_000_u64)
        .map(|i| {
            let vector: Vec<f32> = (0..1536).map(|j| (i + j) as f32 * 0.0001).collect();
            (i, vector)
        })
        .collect();
    store.insert_batch(to_js_batch(batch)).unwrap();

    let query: Vec<f32> = (0..1536).map(|i| i as f32 * 0.0001).collect();

    // Benchmark
    let iterations = 50;
    let start = js_sys::Date::now();

    for _ in 0..iterations {
        let _ = store.search(&query, 10);
    }

    let elapsed = js_sys::Date::now() - start;
    let per_search = elapsed / iterations as f64;

    web_sys::console::log_1(
        &format!(
            "📊 BENCH search_1k_1536d: {:.2}ms avg ({} iterations)",
            per_search, iterations
        )
        .into(),
    );

    assert!(per_search < 50.0, "Search 1k at 1536D should be < 50ms");
}
