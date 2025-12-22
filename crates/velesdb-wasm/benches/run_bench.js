#!/usr/bin/env node
/**
 * VelesDB WASM Performance Benchmarks
 * 
 * Run with: node benches/run_bench.js
 */

const { VectorStore } = require('../pkg/velesdb_wasm.js');

// Utility: measure execution time
function bench(name, fn, iterations = 1) {
    // Warmup
    if (iterations > 1) {
        for (let i = 0; i < Math.min(5, iterations); i++) fn();
    }
    
    const start = performance.now();
    for (let i = 0; i < iterations; i++) {
        fn();
    }
    const elapsed = performance.now() - start;
    const perOp = elapsed / iterations;
    
    console.log(`📊 ${name}: ${elapsed.toFixed(2)}ms total, ${(perOp * 1000).toFixed(2)}µs/op`);
    return { elapsed, perOp };
}

// Generate random vector
function randomVector(dim) {
    const vec = [];
    for (let i = 0; i < dim; i++) {
        vec.push(Math.random() * 2 - 1);
    }
    return vec;
}

console.log('\n🚀 VelesDB WASM Performance Benchmarks\n');
console.log('='.repeat(60));

// =============================================================================
// Benchmark 1: Single Insert vs Batch Insert
// =============================================================================
console.log('\n📦 INSERT BENCHMARKS\n');

// Single insert 10k vectors (128D)
{
    const store = new VectorStore(128, 'cosine');
    const vectors = [];
    for (let i = 0; i < 10000; i++) {
        vectors.push(randomVector(128));
    }
    
    const start = performance.now();
    for (let i = 0; i < 10000; i++) {
        store.insert(BigInt(i), new Float32Array(vectors[i]));
    }
    const elapsed = performance.now() - start;
    console.log(`📊 single_insert_10k_128d: ${elapsed.toFixed(2)}ms total, ${(elapsed / 10000 * 1000).toFixed(2)}µs/insert`);
    store.free();
}

// Batch insert 10k vectors (128D)
{
    const store = VectorStore.with_capacity(128, 'cosine', 10000);
    const batch = [];
    for (let i = 0; i < 10000; i++) {
        batch.push([BigInt(i), randomVector(128)]);
    }
    
    const start = performance.now();
    store.insert_batch(batch);
    const elapsed = performance.now() - start;
    console.log(`📊 batch_insert_10k_128d: ${elapsed.toFixed(2)}ms total, ${(elapsed / 10000 * 1000).toFixed(2)}µs/insert`);
    store.free();
}

// =============================================================================
// Benchmark 2: Search Performance
// =============================================================================
console.log('\n🔍 SEARCH BENCHMARKS\n');

// Search in 10k vectors (128D)
{
    const store = VectorStore.with_capacity(128, 'cosine', 10000);
    const batch = [];
    for (let i = 0; i < 10000; i++) {
        batch.push([BigInt(i), randomVector(128)]);
    }
    store.insert_batch(batch);
    
    const query = new Float32Array(randomVector(128));
    
    // Warmup
    for (let i = 0; i < 10; i++) store.search(query, 10);
    
    const iterations = 100;
    const start = performance.now();
    for (let i = 0; i < iterations; i++) {
        store.search(query, 10);
    }
    const elapsed = performance.now() - start;
    console.log(`📊 search_10k_128d: ${(elapsed / iterations).toFixed(2)}ms avg (${iterations} iterations)`);
    store.free();
}

// Search in 100k vectors (768D) - BERT dimension
{
    const store = VectorStore.with_capacity(768, 'cosine', 100000);
    
    // Insert in chunks
    for (let chunk = 0; chunk < 10; chunk++) {
        const batch = [];
        for (let i = 0; i < 10000; i++) {
            batch.push([BigInt(chunk * 10000 + i), randomVector(768)]);
        }
        store.insert_batch(batch);
    }
    
    const query = new Float32Array(randomVector(768));
    
    // Warmup
    for (let i = 0; i < 3; i++) store.search(query, 10);
    
    const iterations = 20;
    const start = performance.now();
    for (let i = 0; i < iterations; i++) {
        store.search(query, 10);
    }
    const elapsed = performance.now() - start;
    console.log(`📊 search_100k_768d: ${(elapsed / iterations).toFixed(2)}ms avg (${iterations} iterations)`);
    store.free();
}

// =============================================================================
// Benchmark 3: Pre-allocation Impact
// =============================================================================
console.log('\n💾 MEMORY PRE-ALLOCATION IMPACT\n');

{
    const vectors = [];
    for (let i = 0; i < 5000; i++) {
        vectors.push(randomVector(128));
    }
    
    // Without capacity
    const start1 = performance.now();
    const store1 = new VectorStore(128, 'cosine');
    for (let i = 0; i < 5000; i++) {
        store1.insert(BigInt(i), new Float32Array(vectors[i]));
    }
    const without = performance.now() - start1;
    store1.free();
    
    // With capacity
    const start2 = performance.now();
    const store2 = VectorStore.with_capacity(128, 'cosine', 5000);
    for (let i = 0; i < 5000; i++) {
        store2.insert(BigInt(i), new Float32Array(vectors[i]));
    }
    const with_ = performance.now() - start2;
    store2.free();
    
    console.log(`📊 without_capacity: ${without.toFixed(2)}ms`);
    console.log(`📊 with_capacity: ${with_.toFixed(2)}ms`);
    console.log(`📊 speedup: ${(without / with_).toFixed(2)}x`);
}

// =============================================================================
// Benchmark 4: High Dimension (GPT embeddings)
// =============================================================================
console.log('\n🧠 HIGH DIMENSION BENCHMARKS (1536D - GPT)\n');

{
    const store = VectorStore.with_capacity(1536, 'cosine', 1000);
    const batch = [];
    for (let i = 0; i < 1000; i++) {
        batch.push([BigInt(i), randomVector(1536)]);
    }
    
    const startInsert = performance.now();
    store.insert_batch(batch);
    const insertTime = performance.now() - startInsert;
    console.log(`📊 insert_1k_1536d: ${insertTime.toFixed(2)}ms total`);
    
    const query = new Float32Array(randomVector(1536));
    
    // Warmup
    for (let i = 0; i < 5; i++) store.search(query, 10);
    
    const iterations = 50;
    const startSearch = performance.now();
    for (let i = 0; i < iterations; i++) {
        store.search(query, 10);
    }
    const searchTime = performance.now() - startSearch;
    console.log(`📊 search_1k_1536d: ${(searchTime / iterations).toFixed(2)}ms avg`);
    store.free();
}

console.log('\n' + '='.repeat(60));
console.log('✅ Benchmarks complete!\n');
