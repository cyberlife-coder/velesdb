//! FT-1: HNSW Backend Trait Abstraction
//!
//! This module defines the `HnswBackend` trait that abstracts HNSW graph operations,
//! enabling:
//! - Decoupling from the specific `hnsw_rs` implementation
//! - Mock backends for testing
//! - Future alternative backend implementations
//!
//! # Design Rationale
//!
//! The trait mirrors the existing `HnswInner` impl methods, ensuring backward
//! compatibility while providing abstraction benefits.

use hnsw_rs::prelude::Neighbour;
use std::path::Path;

/// Trait for HNSW backend operations.
///
/// This trait abstracts the core HNSW graph operations, allowing `HnswIndex`
/// to work with different backend implementations.
///
/// # Thread Safety
///
/// All implementations must be `Send + Sync` to support concurrent access
/// patterns used by `HnswIndex`.
///
/// # Example
///
/// ```rust,ignore
/// use velesdb_core::index::hnsw::HnswBackend;
///
/// fn search_with_backend<B: HnswBackend>(backend: &B, query: &[f32]) -> Vec<Neighbour> {
///     backend.search(query, 10, 100)
/// }
/// ```
// FT-1: Trait prepared for RF-2 (index.rs split). Will be used in production after RF-2.
#[allow(dead_code)]
pub trait HnswBackend: Send + Sync {
    /// Searches the HNSW graph and returns raw neighbors with distances.
    ///
    /// # Arguments
    ///
    /// * `query` - The query vector
    /// * `k` - Number of nearest neighbors to return
    /// * `ef_search` - Search expansion factor (higher = more accurate, slower)
    ///
    /// # Returns
    ///
    /// Vector of `Neighbour` structs containing (distance, index) pairs.
    fn search(&self, query: &[f32], k: usize, ef_search: usize) -> Vec<Neighbour>;

    /// Inserts a single vector into the HNSW graph.
    ///
    /// # Arguments
    ///
    /// * `data` - Tuple of (vector slice, internal index)
    fn insert(&self, data: (&[f32], usize));

    /// Batch parallel insert into the HNSW graph.
    ///
    /// Uses rayon internally for parallel insertion.
    ///
    /// # Arguments
    ///
    /// * `data` - Slice of (vector reference, internal index) pairs
    fn parallel_insert(&self, data: &[(&Vec<f32>, usize)]);

    /// Sets the index to searching mode after bulk insertions.
    ///
    /// This optimizes the graph structure for search queries.
    ///
    /// # Arguments
    ///
    /// * `mode` - `true` to enable searching mode, `false` to disable
    fn set_searching_mode(&mut self, mode: bool);

    /// Dumps the HNSW graph to files for persistence.
    ///
    /// # Arguments
    ///
    /// * `path` - Directory path for output files
    /// * `basename` - Base name for output files
    ///
    /// # Errors
    ///
    /// Returns `io::Error` if file operations fail.
    fn file_dump(&self, path: &Path, basename: &str) -> std::io::Result<()>;

    /// Transforms raw HNSW distance to the appropriate score based on metric type.
    ///
    /// Different distance metrics require different score transformations:
    /// - **Cosine**: `(1.0 - distance).clamp(0.0, 1.0)` (similarity in `[0,1]`)
    /// - **Euclidean**/**Hamming**/**Jaccard**: raw distance (lower is better)
    /// - **`DotProduct`**: `-distance` (`hnsw_rs` stores negated dot product)
    ///
    /// # Arguments
    ///
    /// * `raw_distance` - The raw distance value from HNSW search
    ///
    /// # Returns
    ///
    /// Transformed score appropriate for the metric type.
    fn transform_score(&self, raw_distance: f32) -> f32;
}

// ============================================================================
// TDD: Tests written BEFORE implementation (Phase 1.1)
// ============================================================================
// Note: impl HnswBackend for HnswInner is in index.rs to avoid method name
// conflicts with the inherent impl methods.

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // Trait Definition Tests
    // -------------------------------------------------------------------------

    /// Verify trait is object-safe (can be used as `dyn HnswBackend`)
    #[test]
    fn test_trait_is_object_safe() {
        fn accepts_dyn_backend(_backend: &dyn HnswBackend) {}
        // If this compiles, the trait is object-safe
        let mock = MockBackend::default();
        accepts_dyn_backend(&mock);
    }

    /// Verify trait requires Send + Sync
    #[test]
    fn test_trait_is_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}

        // These will fail to compile if trait doesn't require Send + Sync
        fn check_bounds<T: HnswBackend>() {
            assert_send::<T>();
            assert_sync::<T>();
        }

        // Actually call to avoid unused warnings
        check_bounds::<MockBackend>();
    }

    // -------------------------------------------------------------------------
    // Mock Backend for Testing
    // -------------------------------------------------------------------------

    /// Mock backend that records method calls for testing
    #[derive(Default)]
    struct MockBackend {
        search_calls: std::cell::RefCell<Vec<(usize, usize)>>, // (k, ef)
        insert_calls: std::cell::RefCell<Vec<usize>>,          // indices
        searching_mode: std::cell::RefCell<bool>,
    }

    // MockBackend is Send + Sync because RefCell contents are only accessed
    // in single-threaded test contexts
    unsafe impl Send for MockBackend {}
    unsafe impl Sync for MockBackend {}

    impl HnswBackend for MockBackend {
        fn search(&self, _query: &[f32], k: usize, ef_search: usize) -> Vec<Neighbour> {
            self.search_calls.borrow_mut().push((k, ef_search));
            // Return mock neighbors
            #[allow(clippy::cast_precision_loss)]
            (0..k.min(3))
                .map(|i| Neighbour {
                    d_id: i,
                    p_id: hnsw_rs::prelude::PointId::default(),
                    distance: i as f32 * 0.1,
                })
                .collect()
        }

        fn insert(&self, data: (&[f32], usize)) {
            self.insert_calls.borrow_mut().push(data.1);
        }

        fn parallel_insert(&self, data: &[(&Vec<f32>, usize)]) {
            for (_, idx) in data {
                self.insert_calls.borrow_mut().push(*idx);
            }
        }

        fn set_searching_mode(&mut self, mode: bool) {
            *self.searching_mode.borrow_mut() = mode;
        }

        fn file_dump(&self, _path: &Path, _basename: &str) -> std::io::Result<()> {
            Ok(())
        }

        fn transform_score(&self, raw_distance: f32) -> f32 {
            raw_distance // Simple passthrough for mock
        }
    }

    #[test]
    fn test_mock_backend_search() {
        // Arrange
        let backend = MockBackend::default();
        let query = vec![1.0, 2.0, 3.0];

        // Act
        let results = backend.search(&query, 5, 100);

        // Assert
        assert_eq!(results.len(), 3); // Mock returns min(k, 3)
        assert_eq!(backend.search_calls.borrow().len(), 1);
        assert_eq!(backend.search_calls.borrow()[0], (5, 100));
    }

    #[test]
    fn test_mock_backend_insert() {
        // Arrange
        let backend = MockBackend::default();
        let vector = vec![1.0, 2.0, 3.0];

        // Act
        backend.insert((&vector, 42));

        // Assert
        assert_eq!(backend.insert_calls.borrow().len(), 1);
        assert_eq!(backend.insert_calls.borrow()[0], 42);
    }

    #[test]
    fn test_mock_backend_parallel_insert() {
        // Arrange
        let backend = MockBackend::default();
        let v1 = vec![1.0, 2.0];
        let v2 = vec![3.0, 4.0];
        let data: Vec<(&Vec<f32>, usize)> = vec![(&v1, 0), (&v2, 1)];

        // Act
        backend.parallel_insert(&data);

        // Assert
        assert_eq!(backend.insert_calls.borrow().len(), 2);
    }

    #[test]
    fn test_mock_backend_searching_mode() {
        // Arrange
        let mut backend = MockBackend::default();

        // Act
        backend.set_searching_mode(true);

        // Assert
        assert!(*backend.searching_mode.borrow());
    }

    #[test]
    fn test_mock_backend_file_dump() {
        // Arrange
        let backend = MockBackend::default();
        let path = std::path::Path::new("/tmp");

        // Act
        let result = backend.file_dump(path, "test");

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn test_mock_backend_transform_score() {
        // Arrange
        let backend = MockBackend::default();

        // Act
        let score = backend.transform_score(0.5);

        // Assert
        assert!((score - 0.5).abs() < f32::EPSILON);
    }

    // -------------------------------------------------------------------------
    // Generic Function Tests (proves trait is usable)
    // -------------------------------------------------------------------------

    fn generic_search<B: HnswBackend>(backend: &B, query: &[f32], k: usize) -> Vec<Neighbour> {
        backend.search(query, k, 100)
    }

    #[test]
    fn test_generic_function_with_mock() {
        // Arrange
        let backend = MockBackend::default();
        let query = vec![0.0; 8];

        // Act
        let results = generic_search(&backend, &query, 5);

        // Assert
        assert!(!results.is_empty());
    }
}
