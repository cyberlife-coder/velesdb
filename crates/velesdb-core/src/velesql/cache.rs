//! Query cache for `VelesQL` parsed queries.
//!
//! Provides an LRU cache for parsed AST to avoid re-parsing identical queries.
//! Typical cache hit rates exceed 90% on repetitive workloads.

use parking_lot::RwLock;
use rustc_hash::FxHashMap;
use std::collections::VecDeque;
use std::hash::{Hash, Hasher};

use super::ast::Query;
use super::error::ParseError;
use super::Parser;

/// Statistics for the query cache.
#[derive(Debug, Clone, Copy, Default)]
pub struct CacheStats {
    /// Number of cache hits.
    pub hits: u64,
    /// Number of cache misses.
    pub misses: u64,
    /// Number of evictions.
    pub evictions: u64,
}

impl CacheStats {
    /// Returns the cache hit rate as a percentage (0.0 - 100.0).
    #[must_use]
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            #[allow(clippy::cast_precision_loss)]
            let rate = (self.hits as f64 / total as f64) * 100.0;
            rate
        }
    }
}

/// LRU cache for parsed `VelesQL` queries.
///
/// Thread-safe implementation using `parking_lot::RwLock`.
///
/// # Example
///
/// ```ignore
/// use velesdb_core::velesql::QueryCache;
///
/// let cache = QueryCache::new(1000);
/// let query = cache.parse("SELECT * FROM documents LIMIT 10")?;
/// // Second call returns cached AST
/// let query2 = cache.parse("SELECT * FROM documents LIMIT 10")?;
/// assert!(cache.stats().hits >= 1);
/// ```
pub struct QueryCache {
    /// Cache storage: hash -> Query
    cache: RwLock<FxHashMap<u64, Query>>,
    /// LRU order: front = oldest, back = newest
    order: RwLock<VecDeque<u64>>,
    /// Maximum cache size
    max_size: usize,
    /// Cache statistics
    stats: RwLock<CacheStats>,
}

impl QueryCache {
    /// Creates a new query cache with the specified maximum size.
    ///
    /// # Arguments
    ///
    /// * `max_size` - Maximum number of queries to cache (minimum 1)
    #[must_use]
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: RwLock::new(FxHashMap::default()),
            order: RwLock::new(VecDeque::with_capacity(max_size)),
            max_size: max_size.max(1),
            stats: RwLock::new(CacheStats::default()),
        }
    }

    /// Parses a query, returning cached AST if available.
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if the query is invalid.
    pub fn parse(&self, query: &str) -> Result<Query, ParseError> {
        let hash = Self::hash_query(query);

        // Try cache read first
        {
            let cache = self.cache.read();
            if let Some(cached) = cache.get(&hash) {
                let mut stats = self.stats.write();
                stats.hits += 1;
                return Ok(cached.clone());
            }
        }

        // Cache miss - parse the query
        let parsed = Parser::parse(query)?;

        // Insert into cache
        {
            let mut cache = self.cache.write();
            let mut order = self.order.write();
            let mut stats = self.stats.write();

            stats.misses += 1;

            // Evict oldest if at capacity
            while cache.len() >= self.max_size {
                if let Some(oldest) = order.pop_front() {
                    cache.remove(&oldest);
                    stats.evictions += 1;
                }
            }

            cache.insert(hash, parsed.clone());
            order.push_back(hash);
        }

        Ok(parsed)
    }

    /// Returns current cache statistics.
    #[must_use]
    pub fn stats(&self) -> CacheStats {
        *self.stats.read()
    }

    /// Returns the current number of cached queries.
    #[must_use]
    pub fn len(&self) -> usize {
        self.cache.read().len()
    }

    /// Returns true if the cache is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.cache.read().is_empty()
    }

    /// Clears all cached queries and resets statistics.
    pub fn clear(&self) {
        let mut cache = self.cache.write();
        let mut order = self.order.write();
        let mut stats = self.stats.write();

        cache.clear();
        order.clear();
        *stats = CacheStats::default();
    }

    /// Computes a hash for the query string.
    fn hash_query(query: &str) -> u64 {
        let mut hasher = rustc_hash::FxHasher::default();
        query.hash(&mut hasher);
        hasher.finish()
    }
}

impl Default for QueryCache {
    fn default() -> Self {
        Self::new(1000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_new() {
        // Arrange & Act
        let cache = QueryCache::new(100);

        // Assert
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
        assert_eq!(cache.stats().hits, 0);
        assert_eq!(cache.stats().misses, 0);
    }

    #[test]
    fn test_cache_parse_miss() {
        // Arrange
        let cache = QueryCache::new(100);

        // Act
        let result = cache.parse("SELECT * FROM documents");

        // Assert
        assert!(result.is_ok());
        assert_eq!(cache.len(), 1);
        assert_eq!(cache.stats().misses, 1);
        assert_eq!(cache.stats().hits, 0);
    }

    #[test]
    fn test_cache_parse_hit() {
        // Arrange
        let cache = QueryCache::new(100);
        let query = "SELECT * FROM documents LIMIT 10";

        // Act - first parse (miss)
        let result1 = cache.parse(query);
        // Act - second parse (hit)
        let result2 = cache.parse(query);

        // Assert
        assert!(result1.is_ok());
        assert!(result2.is_ok());
        assert_eq!(result1.unwrap(), result2.unwrap());
        assert_eq!(cache.stats().hits, 1);
        assert_eq!(cache.stats().misses, 1);
    }

    #[test]
    fn test_cache_hit_rate() {
        // Arrange
        let cache = QueryCache::new(100);
        let query = "SELECT * FROM test";

        // Act - 1 miss, 9 hits
        for _ in 0..10 {
            let _ = cache.parse(query);
        }

        // Assert
        let stats = cache.stats();
        assert_eq!(stats.hits, 9);
        assert_eq!(stats.misses, 1);
        assert!((stats.hit_rate() - 90.0).abs() < 0.01);
    }

    #[test]
    fn test_cache_eviction() {
        // Arrange
        let cache = QueryCache::new(3);

        // Act - insert 4 queries into cache of size 3
        let _ = cache.parse("SELECT * FROM a");
        let _ = cache.parse("SELECT * FROM b");
        let _ = cache.parse("SELECT * FROM c");
        let _ = cache.parse("SELECT * FROM d");

        // Assert
        assert_eq!(cache.len(), 3);
        assert_eq!(cache.stats().evictions, 1);
    }

    #[test]
    fn test_cache_clear() {
        // Arrange
        let cache = QueryCache::new(100);
        let _ = cache.parse("SELECT * FROM test");
        let _ = cache.parse("SELECT * FROM test");

        // Act
        cache.clear();

        // Assert
        assert!(cache.is_empty());
        assert_eq!(cache.stats().hits, 0);
        assert_eq!(cache.stats().misses, 0);
    }

    #[test]
    fn test_cache_invalid_query() {
        // Arrange
        let cache = QueryCache::new(100);

        // Act
        let result = cache.parse("INVALID QUERY");

        // Assert
        assert!(result.is_err());
        assert!(cache.is_empty()); // Invalid queries should not be cached
    }

    #[test]
    fn test_cache_different_queries() {
        // Arrange
        let cache = QueryCache::new(100);

        // Act
        let _ = cache.parse("SELECT * FROM a");
        let _ = cache.parse("SELECT * FROM b");
        let _ = cache.parse("SELECT id FROM c WHERE id = 1");

        // Assert
        assert_eq!(cache.len(), 3);
        assert_eq!(cache.stats().misses, 3);
        assert_eq!(cache.stats().hits, 0);
    }

    #[test]
    fn test_cache_min_size() {
        // Arrange - cache size 0 should be clamped to 1
        let cache = QueryCache::new(0);

        // Act
        let _ = cache.parse("SELECT * FROM a");
        let _ = cache.parse("SELECT * FROM b");

        // Assert
        assert_eq!(cache.len(), 1); // Only 1 entry due to min size
        assert_eq!(cache.stats().evictions, 1);
    }

    #[test]
    fn test_cache_thread_safety() {
        use std::sync::Arc;
        use std::thread;

        // Arrange
        let cache = Arc::new(QueryCache::new(100));
        let query = "SELECT * FROM concurrent_test";

        // Act - spawn multiple threads
        let handles: Vec<_> = (0..10)
            .map(|_| {
                let cache = Arc::clone(&cache);
                let q = query.to_string();
                thread::spawn(move || {
                    for _ in 0..100 {
                        let _ = cache.parse(&q);
                    }
                })
            })
            .collect();

        for h in handles {
            h.join().expect("Thread panicked");
        }

        // Assert - should have high hit rate
        let stats = cache.stats();
        assert!(stats.hit_rate() > 90.0);
        assert_eq!(stats.hits + stats.misses, 1000);
    }

    #[test]
    fn test_cache_stats_hit_rate_empty() {
        // Arrange
        let stats = CacheStats::default();

        // Act & Assert
        assert!(stats.hit_rate().abs() < f64::EPSILON);
    }
}
