//! Search quality metrics for evaluating retrieval performance.
//!
//! This module provides standard information retrieval metrics:
//! - **Recall@k**: Proportion of true neighbors found in top-k results
//! - **Precision@k**: Proportion of relevant results among top-k returned
//! - **MRR (Mean Reciprocal Rank)**: Quality of ranking based on first relevant result
//!
//! # Example
//!
//! ```rust
//! use velesdb_core::metrics::{recall_at_k, precision_at_k, mrr};
//!
//! let ground_truth = vec![1, 2, 3, 4, 5];  // True top-5 neighbors
//! let results = vec![1, 3, 6, 2, 7];       // Retrieved results
//!
//! let recall = recall_at_k(&ground_truth, &results);      // 3/5 = 0.6
//! let precision = precision_at_k(&ground_truth, &results); // 3/5 = 0.6
//! let rank_quality = mrr(&ground_truth, &results);         // 1/1 = 1.0 (first result is relevant)
//! ```

use std::collections::HashSet;
use std::hash::Hash;

/// Calculates Recall@k: the proportion of true neighbors found in the results.
///
/// Recall measures how many of the true relevant items were retrieved.
/// A recall of 1.0 means all true neighbors were found.
///
/// # Formula
///
/// `recall@k = |ground_truth ∩ results| / |ground_truth|`
///
/// # Arguments
///
/// * `ground_truth` - The true k-nearest neighbors (expected results)
/// * `results` - The retrieved results from the search
///
/// # Returns
///
/// A value between 0.0 and 1.0, where 1.0 means perfect recall.
///
/// # Panics
///
/// Returns 0.0 if `ground_truth` is empty (to avoid division by zero).
#[must_use]
pub fn recall_at_k<T: Eq + Hash + Copy>(ground_truth: &[T], results: &[T]) -> f64 {
    if ground_truth.is_empty() {
        return 0.0;
    }

    let truth_set: HashSet<T> = ground_truth.iter().copied().collect();
    let found = results.iter().filter(|id| truth_set.contains(id)).count();

    #[allow(clippy::cast_precision_loss)]
    let recall = found as f64 / ground_truth.len() as f64;
    recall
}

/// Calculates Precision@k: the proportion of relevant results among those returned.
///
/// Precision measures how many of the retrieved items are actually relevant.
/// A precision of 1.0 means all returned results are relevant.
///
/// # Formula
///
/// `precision@k = |ground_truth ∩ results| / |results|`
///
/// # Arguments
///
/// * `ground_truth` - The true k-nearest neighbors (relevant items)
/// * `results` - The retrieved results from the search
///
/// # Returns
///
/// A value between 0.0 and 1.0, where 1.0 means perfect precision.
///
/// # Panics
///
/// Returns 0.0 if results is empty (to avoid division by zero).
#[must_use]
pub fn precision_at_k<T: Eq + Hash + Copy>(ground_truth: &[T], results: &[T]) -> f64 {
    if results.is_empty() {
        return 0.0;
    }

    let truth_set: HashSet<T> = ground_truth.iter().copied().collect();
    let relevant = results.iter().filter(|id| truth_set.contains(id)).count();

    #[allow(clippy::cast_precision_loss)]
    let precision = relevant as f64 / results.len() as f64;
    precision
}

/// Calculates Mean Reciprocal Rank (MRR): quality based on the rank of the first relevant result.
///
/// MRR rewards systems that place a relevant result at the top of the list.
/// An MRR of 1.0 means the first result is always relevant.
///
/// # Formula
///
/// `MRR = 1 / rank_of_first_relevant_result`
///
/// # Arguments
///
/// * `ground_truth` - The set of relevant items
/// * `results` - The ranked list of retrieved results
///
/// # Returns
///
/// A value between 0.0 and 1.0, where 1.0 means the first result is relevant.
/// Returns 0.0 if no relevant result is found.
#[must_use]
pub fn mrr<T: Eq + Hash + Copy>(ground_truth: &[T], results: &[T]) -> f64 {
    let truth_set: HashSet<T> = ground_truth.iter().copied().collect();

    for (rank, id) in results.iter().enumerate() {
        if truth_set.contains(id) {
            #[allow(clippy::cast_precision_loss)]
            return 1.0 / (rank + 1) as f64;
        }
    }

    0.0
}

/// Calculates average metrics over multiple queries.
///
/// # Arguments
///
/// * `ground_truths` - List of ground truth results for each query
/// * `results_list` - List of retrieved results for each query
///
/// # Returns
///
/// A tuple of (`avg_recall`, `avg_precision`, `avg_mrr`).
#[must_use]
pub fn average_metrics<T: Eq + Hash + Copy>(
    ground_truths: &[Vec<T>],
    results_list: &[Vec<T>],
) -> (f64, f64, f64) {
    if ground_truths.is_empty() || results_list.is_empty() {
        return (0.0, 0.0, 0.0);
    }

    let n = ground_truths.len().min(results_list.len());
    let mut total_recall = 0.0;
    let mut total_precision = 0.0;
    let mut total_mrr = 0.0;

    for (gt, res) in ground_truths.iter().zip(results_list.iter()).take(n) {
        total_recall += recall_at_k(gt, res);
        total_precision += precision_at_k(gt, res);
        total_mrr += mrr(gt, res);
    }

    #[allow(clippy::cast_precision_loss)]
    let n_f64 = n as f64;
    (
        total_recall / n_f64,
        total_precision / n_f64,
        total_mrr / n_f64,
    )
}

// =============================================================================
// TDD Tests - Written BEFORE implementation (following WIS-77 requirements)
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // Recall@k Tests
    // =========================================================================

    #[test]
    fn test_recall_at_k_perfect() {
        // Arrange: all ground truth items are in results
        let ground_truth = vec![1u64, 2, 3, 4, 5];
        let results = vec![1u64, 2, 3, 4, 5];

        // Act
        let recall = recall_at_k(&ground_truth, &results);

        // Assert: 100% recall
        assert!(
            (recall - 1.0).abs() < f64::EPSILON,
            "Expected 1.0, got {recall}"
        );
    }

    #[test]
    fn test_recall_at_k_partial() {
        // Arrange: 3 out of 5 ground truth items found
        let ground_truth = vec![1u64, 2, 3, 4, 5];
        let results = vec![1u64, 3, 6, 2, 7];

        // Act
        let recall = recall_at_k(&ground_truth, &results);

        // Assert: 3/5 = 0.6
        assert!(
            (recall - 0.6).abs() < f64::EPSILON,
            "Expected 0.6, got {recall}"
        );
    }

    #[test]
    fn test_recall_at_k_zero() {
        // Arrange: no ground truth items in results
        let ground_truth = vec![1u64, 2, 3];
        let results = vec![10u64, 20, 30];

        // Act
        let recall = recall_at_k(&ground_truth, &results);

        // Assert: 0% recall
        assert!(
            (recall - 0.0).abs() < f64::EPSILON,
            "Expected 0.0, got {recall}"
        );
    }

    #[test]
    fn test_recall_at_k_empty_ground_truth() {
        // Arrange: empty ground truth
        let ground_truth: Vec<u64> = vec![];
        let results = vec![1u64, 2, 3];

        // Act
        let recall = recall_at_k(&ground_truth, &results);

        // Assert: 0.0 (edge case)
        assert!(
            (recall - 0.0).abs() < f64::EPSILON,
            "Expected 0.0, got {recall}"
        );
    }

    #[test]
    fn test_recall_at_k_empty_results() {
        // Arrange: empty results
        let ground_truth = vec![1u64, 2, 3];
        let results: Vec<u64> = vec![];

        // Act
        let recall = recall_at_k(&ground_truth, &results);

        // Assert: 0% recall
        assert!(
            (recall - 0.0).abs() < f64::EPSILON,
            "Expected 0.0, got {recall}"
        );
    }

    // =========================================================================
    // Precision@k Tests
    // =========================================================================

    #[test]
    fn test_precision_at_k_perfect() {
        // Arrange: all results are relevant
        let ground_truth = vec![1u64, 2, 3, 4, 5];
        let results = vec![1u64, 2, 3, 4, 5];

        // Act
        let precision = precision_at_k(&ground_truth, &results);

        // Assert: 100% precision
        assert!(
            (precision - 1.0).abs() < f64::EPSILON,
            "Expected 1.0, got {precision}"
        );
    }

    #[test]
    fn test_precision_at_k_partial() {
        // Arrange: 3 out of 5 results are relevant
        let ground_truth = vec![1u64, 2, 3, 4, 5];
        let results = vec![1u64, 3, 6, 2, 7];

        // Act
        let precision = precision_at_k(&ground_truth, &results);

        // Assert: 3/5 = 0.6
        assert!(
            (precision - 0.6).abs() < f64::EPSILON,
            "Expected 0.6, got {precision}"
        );
    }

    #[test]
    fn test_precision_at_k_zero() {
        // Arrange: no results are relevant
        let ground_truth = vec![1u64, 2, 3];
        let results = vec![10u64, 20, 30];

        // Act
        let precision = precision_at_k(&ground_truth, &results);

        // Assert: 0% precision
        assert!(
            (precision - 0.0).abs() < f64::EPSILON,
            "Expected 0.0, got {precision}"
        );
    }

    #[test]
    fn test_precision_at_k_empty_results() {
        // Arrange: empty results
        let ground_truth = vec![1u64, 2, 3];
        let results: Vec<u64> = vec![];

        // Act
        let precision = precision_at_k(&ground_truth, &results);

        // Assert: 0.0 (edge case)
        assert!(
            (precision - 0.0).abs() < f64::EPSILON,
            "Expected 0.0, got {precision}"
        );
    }

    #[test]
    fn test_precision_different_k() {
        // Arrange: more results than ground truth
        let ground_truth = vec![1u64, 2, 3];
        let results = vec![1u64, 2, 3, 10, 20, 30, 40, 50, 60, 70]; // 10 results, 3 relevant

        // Act
        let precision = precision_at_k(&ground_truth, &results);

        // Assert: 3/10 = 0.3
        assert!(
            (precision - 0.3).abs() < f64::EPSILON,
            "Expected 0.3, got {precision}"
        );
    }

    // =========================================================================
    // MRR Tests
    // =========================================================================

    #[test]
    fn test_mrr_first_relevant() {
        // Arrange: first result is relevant
        let ground_truth = vec![1u64, 2, 3];
        let results = vec![1u64, 10, 20, 30];

        // Act
        let mrr_val = mrr(&ground_truth, &results);

        // Assert: 1/1 = 1.0
        assert!(
            (mrr_val - 1.0).abs() < f64::EPSILON,
            "Expected 1.0, got {mrr_val}"
        );
    }

    #[test]
    fn test_mrr_second_relevant() {
        // Arrange: second result is relevant
        let ground_truth = vec![1u64, 2, 3];
        let results = vec![10u64, 1, 20, 30];

        // Act
        let mrr_val = mrr(&ground_truth, &results);

        // Assert: 1/2 = 0.5
        assert!(
            (mrr_val - 0.5).abs() < f64::EPSILON,
            "Expected 0.5, got {mrr_val}"
        );
    }

    #[test]
    fn test_mrr_third_relevant() {
        // Arrange: third result is relevant
        let ground_truth = vec![1u64, 2, 3];
        let results = vec![10u64, 20, 2, 30];

        // Act
        let mrr_val = mrr(&ground_truth, &results);

        // Assert: 1/3 ≈ 0.333...
        let expected = 1.0 / 3.0;
        assert!(
            (mrr_val - expected).abs() < f64::EPSILON,
            "Expected {expected}, got {mrr_val}"
        );
    }

    #[test]
    fn test_mrr_no_relevant() {
        // Arrange: no relevant results
        let ground_truth = vec![1u64, 2, 3];
        let results = vec![10u64, 20, 30, 40];

        // Act
        let mrr_val = mrr(&ground_truth, &results);

        // Assert: 0.0
        assert!(
            (mrr_val - 0.0).abs() < f64::EPSILON,
            "Expected 0.0, got {mrr_val}"
        );
    }

    #[test]
    fn test_mrr_empty_results() {
        // Arrange: empty results
        let ground_truth = vec![1u64, 2, 3];
        let results: Vec<u64> = vec![];

        // Act
        let mrr_val = mrr(&ground_truth, &results);

        // Assert: 0.0
        assert!(
            (mrr_val - 0.0).abs() < f64::EPSILON,
            "Expected 0.0, got {mrr_val}"
        );
    }

    // =========================================================================
    // Average Metrics Tests
    // =========================================================================

    #[test]
    fn test_average_metrics_perfect() {
        // Arrange: perfect retrieval for all queries
        let ground_truths = vec![vec![1u64, 2, 3], vec![4u64, 5, 6]];
        let results_list = vec![vec![1u64, 2, 3], vec![4u64, 5, 6]];

        // Act
        let (avg_recall, avg_precision, avg_mrr) = average_metrics(&ground_truths, &results_list);

        // Assert: all metrics = 1.0
        assert!(
            (avg_recall - 1.0).abs() < f64::EPSILON,
            "Expected recall 1.0, got {avg_recall}"
        );
        assert!(
            (avg_precision - 1.0).abs() < f64::EPSILON,
            "Expected precision 1.0, got {avg_precision}"
        );
        assert!(
            (avg_mrr - 1.0).abs() < f64::EPSILON,
            "Expected MRR 1.0, got {avg_mrr}"
        );
    }

    #[test]
    fn test_average_metrics_mixed() {
        // Arrange: mixed retrieval quality
        let ground_truths = vec![
            vec![1u64, 2, 3, 4, 5],      // Query 1: need 5 items
            vec![10u64, 20, 30, 40, 50], // Query 2: need 5 items
        ];
        let results_list = vec![
            vec![1u64, 2, 3, 10, 20],    // Query 1: 3/5 relevant, first is relevant
            vec![10u64, 11, 12, 13, 14], // Query 2: 1/5 relevant, first is relevant
        ];

        // Act
        let (avg_recall, avg_precision, avg_mrr) = average_metrics(&ground_truths, &results_list);

        // Assert
        // Query 1: recall=3/5=0.6, precision=3/5=0.6, mrr=1.0
        // Query 2: recall=1/5=0.2, precision=1/5=0.2, mrr=1.0
        // Avg: recall=(0.6+0.2)/2=0.4, precision=(0.6+0.2)/2=0.4, mrr=(1.0+1.0)/2=1.0
        assert!(
            (avg_recall - 0.4).abs() < f64::EPSILON,
            "Expected recall 0.4, got {avg_recall}"
        );
        assert!(
            (avg_precision - 0.4).abs() < f64::EPSILON,
            "Expected precision 0.4, got {avg_precision}"
        );
        assert!(
            (avg_mrr - 1.0).abs() < f64::EPSILON,
            "Expected MRR 1.0, got {avg_mrr}"
        );
    }

    #[test]
    fn test_average_metrics_empty() {
        // Arrange: empty inputs
        let ground_truths: Vec<Vec<u64>> = vec![];
        let results_list: Vec<Vec<u64>> = vec![];

        // Act
        let (avg_recall, avg_precision, avg_mrr) = average_metrics(&ground_truths, &results_list);

        // Assert: all 0.0
        assert!((avg_recall - 0.0).abs() < f64::EPSILON);
        assert!((avg_precision - 0.0).abs() < f64::EPSILON);
        assert!((avg_mrr - 0.0).abs() < f64::EPSILON);
    }

    // =========================================================================
    // Exact Search Recall Validation (WIS-77 requirement)
    // =========================================================================

    #[test]
    fn test_exact_search_has_100_percent_recall() {
        // Arrange: simulate exact brute-force search
        // Ground truth and results should be identical for exact search
        let ground_truth: Vec<u64> = (0..100).collect();
        let exact_results: Vec<u64> = (0..100).collect();

        // Act
        let recall = recall_at_k(&ground_truth, &exact_results);
        let precision = precision_at_k(&ground_truth, &exact_results);

        // Assert: exact search = 100% recall and precision
        assert!(
            (recall - 1.0).abs() < f64::EPSILON,
            "Exact search must have 100% recall"
        );
        assert!(
            (precision - 1.0).abs() < f64::EPSILON,
            "Exact search must have 100% precision"
        );
    }

    #[test]
    fn test_recall_at_10() {
        // Arrange: k=10 scenario
        let ground_truth: Vec<u64> = (0..10).collect();
        let results = vec![0u64, 1, 2, 3, 4, 5, 6, 7, 100, 101]; // 8 correct, 2 wrong

        // Act
        let recall = recall_at_k(&ground_truth, &results);

        // Assert: 8/10 = 0.8
        assert!(
            (recall - 0.8).abs() < f64::EPSILON,
            "Expected 0.8, got {recall}"
        );
    }

    #[test]
    fn test_recall_at_100() {
        // Arrange: k=100 scenario
        let ground_truth: Vec<u64> = (0..100).collect();
        let mut results: Vec<u64> = (0..90).collect(); // 90 correct
        results.extend(200..210); // 10 wrong

        // Act
        let recall = recall_at_k(&ground_truth, &results);

        // Assert: 90/100 = 0.9
        assert!(
            (recall - 0.9).abs() < f64::EPSILON,
            "Expected 0.9, got {recall}"
        );
    }
}
