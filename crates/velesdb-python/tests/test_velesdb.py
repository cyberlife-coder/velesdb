"""
Tests for VelesDB Python bindings.

Run with: pytest tests/test_velesdb.py -v
"""

import pytest
import tempfile
import shutil
import os

# Import will fail until the module is built with maturin
# These tests are designed to run after: maturin develop
try:
    import velesdb
except ImportError:
    pytest.skip("velesdb module not built yet - run 'maturin develop' first", allow_module_level=True)


@pytest.fixture
def temp_db_path():
    """Create a temporary directory for database tests."""
    path = tempfile.mkdtemp(prefix="velesdb_test_")
    yield path
    # Cleanup after test
    shutil.rmtree(path, ignore_errors=True)


class TestDatabase:
    """Tests for Database class."""

    def test_create_database(self, temp_db_path):
        """Test database creation."""
        db = velesdb.Database(temp_db_path)
        assert db is not None

    def test_list_collections_empty(self, temp_db_path):
        """Test listing collections on empty database."""
        db = velesdb.Database(temp_db_path)
        collections = db.list_collections()
        assert collections == []

    def test_create_collection(self, temp_db_path):
        """Test collection creation."""
        db = velesdb.Database(temp_db_path)
        collection = db.create_collection("test", dimension=4, metric="cosine")
        assert collection is not None
        assert collection.name == "test"

    def test_create_collection_metrics(self, temp_db_path):
        """Test collection creation with different metrics."""
        db = velesdb.Database(temp_db_path)
        
        # Cosine
        c1 = db.create_collection("cosine_col", dimension=4, metric="cosine")
        assert c1 is not None
        
        # Euclidean
        c2 = db.create_collection("euclidean_col", dimension=4, metric="euclidean")
        assert c2 is not None
        
        # Dot product
        c3 = db.create_collection("dot_col", dimension=4, metric="dot")
        assert c3 is not None

    def test_get_collection(self, temp_db_path):
        """Test getting an existing collection."""
        db = velesdb.Database(temp_db_path)
        db.create_collection("my_collection", dimension=4)
        
        collection = db.get_collection("my_collection")
        assert collection is not None
        assert collection.name == "my_collection"

    def test_get_collection_not_found(self, temp_db_path):
        """Test getting a non-existent collection."""
        db = velesdb.Database(temp_db_path)
        collection = db.get_collection("nonexistent")
        assert collection is None

    def test_delete_collection(self, temp_db_path):
        """Test deleting a collection."""
        db = velesdb.Database(temp_db_path)
        db.create_collection("to_delete", dimension=4)
        
        assert "to_delete" in db.list_collections()
        db.delete_collection("to_delete")
        assert "to_delete" not in db.list_collections()


class TestCollection:
    """Tests for Collection class."""

    def test_collection_info(self, temp_db_path):
        """Test getting collection info."""
        db = velesdb.Database(temp_db_path)
        collection = db.create_collection("info_test", dimension=128, metric="cosine")
        
        info = collection.info()
        assert info["name"] == "info_test"
        assert info["dimension"] == 128
        assert info["metric"] == "cosine"
        assert info["point_count"] == 0

    def test_upsert_single_point(self, temp_db_path):
        """Test inserting a single point."""
        db = velesdb.Database(temp_db_path)
        collection = db.create_collection("upsert_test", dimension=4)
        
        count = collection.upsert([
            {"id": 1, "vector": [1.0, 0.0, 0.0, 0.0], "payload": {"title": "Test"}}
        ])
        
        assert count == 1
        assert not collection.is_empty()

    def test_upsert_multiple_points(self, temp_db_path):
        """Test inserting multiple points."""
        db = velesdb.Database(temp_db_path)
        collection = db.create_collection("multi_upsert", dimension=4)
        
        count = collection.upsert([
            {"id": 1, "vector": [1.0, 0.0, 0.0, 0.0], "payload": {"title": "Doc 1"}},
            {"id": 2, "vector": [0.0, 1.0, 0.0, 0.0], "payload": {"title": "Doc 2"}},
            {"id": 3, "vector": [0.0, 0.0, 1.0, 0.0], "payload": {"title": "Doc 3"}},
        ])
        
        assert count == 3

    def test_upsert_without_payload(self, temp_db_path):
        """Test inserting point without payload."""
        db = velesdb.Database(temp_db_path)
        collection = db.create_collection("no_payload", dimension=4)
        
        count = collection.upsert([
            {"id": 1, "vector": [1.0, 0.0, 0.0, 0.0]}
        ])
        
        assert count == 1

    def test_search(self, temp_db_path):
        """Test vector search."""
        db = velesdb.Database(temp_db_path)
        collection = db.create_collection("search_test", dimension=4, metric="cosine")
        
        # Insert test vectors
        collection.upsert([
            {"id": 1, "vector": [1.0, 0.0, 0.0, 0.0], "payload": {"title": "Doc 1"}},
            {"id": 2, "vector": [0.0, 1.0, 0.0, 0.0], "payload": {"title": "Doc 2"}},
            {"id": 3, "vector": [0.9, 0.1, 0.0, 0.0], "payload": {"title": "Doc 3"}},
        ])
        
        # Search for vector similar to [1, 0, 0, 0]
        results = collection.search([1.0, 0.0, 0.0, 0.0], top_k=2)
        
        assert len(results) == 2
        # First result should be exact match (id=1)
        assert results[0]["id"] == 1
        assert results[0]["score"] > 0.9
        assert results[0]["payload"]["title"] == "Doc 1"

    def test_search_top_k(self, temp_db_path):
        """Test search with different top_k values."""
        db = velesdb.Database(temp_db_path)
        collection = db.create_collection("topk_test", dimension=4)
        
        # Insert 5 vectors
        collection.upsert([
            {"id": i, "vector": [float(i), 0.0, 0.0, 0.0]}
            for i in range(1, 6)
        ])
        
        # Search with top_k=3
        results = collection.search([1.0, 0.0, 0.0, 0.0], top_k=3)
        assert len(results) == 3

    def test_get_points(self, temp_db_path):
        """Test getting points by ID."""
        db = velesdb.Database(temp_db_path)
        collection = db.create_collection("get_test", dimension=4)
        
        collection.upsert([
            {"id": 1, "vector": [1.0, 0.0, 0.0, 0.0], "payload": {"title": "Doc 1"}},
            {"id": 2, "vector": [0.0, 1.0, 0.0, 0.0], "payload": {"title": "Doc 2"}},
        ])
        
        points = collection.get([1, 2, 999])
        
        assert len(points) == 3
        assert points[0] is not None
        assert points[0]["id"] == 1
        assert points[1] is not None
        assert points[1]["id"] == 2
        assert points[2] is None  # ID 999 doesn't exist

    def test_delete_points(self, temp_db_path):
        """Test deleting points."""
        db = velesdb.Database(temp_db_path)
        collection = db.create_collection("delete_test", dimension=4)
        
        collection.upsert([
            {"id": 1, "vector": [1.0, 0.0, 0.0, 0.0]},
            {"id": 2, "vector": [0.0, 1.0, 0.0, 0.0]},
        ])
        
        # Delete point 1
        collection.delete([1])
        
        # Verify deletion
        points = collection.get([1, 2])
        assert points[0] is None
        assert points[1] is not None

    def test_is_empty(self, temp_db_path):
        """Test is_empty method."""
        db = velesdb.Database(temp_db_path)
        collection = db.create_collection("empty_test", dimension=4)
        
        assert collection.is_empty()
        
        collection.upsert([{"id": 1, "vector": [1.0, 0.0, 0.0, 0.0]}])
        
        assert not collection.is_empty()

    def test_flush(self, temp_db_path):
        """Test flush method."""
        db = velesdb.Database(temp_db_path)
        collection = db.create_collection("flush_test", dimension=4)
        
        collection.upsert([{"id": 1, "vector": [1.0, 0.0, 0.0, 0.0]}])
        collection.flush()  # Should not raise


class TestNumpySupport:
    """Tests for NumPy array support (WIS-23)."""

    def test_upsert_with_numpy_vector(self, temp_db_path):
        """Test upserting points with numpy array vectors."""
        import numpy as np
        
        db = velesdb.Database(temp_db_path)
        collection = db.create_collection("numpy_test", dimension=4, metric="cosine")
        
        # Upsert with numpy array
        vector = np.array([1.0, 0.0, 0.0, 0.0], dtype=np.float32)
        count = collection.upsert([
            {"id": 1, "vector": vector, "payload": {"title": "NumPy Doc"}}
        ])
        
        assert count == 1
        assert not collection.is_empty()

    def test_upsert_with_numpy_float64(self, temp_db_path):
        """Test upserting with float64 numpy arrays (should auto-convert)."""
        import numpy as np
        
        db = velesdb.Database(temp_db_path)
        collection = db.create_collection("numpy_f64", dimension=4)
        
        # float64 should be converted to float32
        vector = np.array([0.5, 0.5, 0.0, 0.0], dtype=np.float64)
        count = collection.upsert([{"id": 1, "vector": vector}])
        
        assert count == 1

    def test_search_with_numpy_vector(self, temp_db_path):
        """Test searching with numpy array query vector."""
        import numpy as np
        
        db = velesdb.Database(temp_db_path)
        collection = db.create_collection("numpy_search", dimension=4, metric="cosine")
        
        # Insert with regular list
        collection.upsert([
            {"id": 1, "vector": [1.0, 0.0, 0.0, 0.0], "payload": {"title": "Doc 1"}},
            {"id": 2, "vector": [0.0, 1.0, 0.0, 0.0], "payload": {"title": "Doc 2"}},
        ])
        
        # Search with numpy array
        query = np.array([1.0, 0.0, 0.0, 0.0], dtype=np.float32)
        results = collection.search(query, top_k=2)
        
        assert len(results) == 2
        assert results[0]["id"] == 1  # Exact match should be first

    def test_mixed_numpy_and_list_upsert(self, temp_db_path):
        """Test upserting with mix of numpy arrays and Python lists."""
        import numpy as np
        
        db = velesdb.Database(temp_db_path)
        collection = db.create_collection("mixed_vectors", dimension=4)
        
        count = collection.upsert([
            {"id": 1, "vector": [1.0, 0.0, 0.0, 0.0]},  # Python list
            {"id": 2, "vector": np.array([0.0, 1.0, 0.0, 0.0], dtype=np.float32)},  # NumPy
        ])
        
        assert count == 2


class TestTextSearch:
    """Tests for BM25 text search (WIS-42)."""

    def test_text_search_basic(self, temp_db_path):
        """Test basic text search functionality."""
        db = velesdb.Database(temp_db_path)
        collection = db.create_collection("text_search_test", dimension=4)
        
        # Insert documents with text payloads
        collection.upsert([
            {"id": 1, "vector": [1.0, 0.0, 0.0, 0.0], "payload": {"text": "machine learning algorithms"}},
            {"id": 2, "vector": [0.0, 1.0, 0.0, 0.0], "payload": {"text": "deep learning neural networks"}},
            {"id": 3, "vector": [0.0, 0.0, 1.0, 0.0], "payload": {"text": "natural language processing"}},
        ])
        
        results = collection.text_search("learning", top_k=2)
        
        assert len(results) <= 2
        # Results should have id, score, payload
        if len(results) > 0:
            assert "id" in results[0]
            assert "score" in results[0]

    def test_text_search_no_results(self, temp_db_path):
        """Test text search with no matching results."""
        db = velesdb.Database(temp_db_path)
        collection = db.create_collection("text_no_match", dimension=4)
        
        collection.upsert([
            {"id": 1, "vector": [1.0, 0.0, 0.0, 0.0], "payload": {"text": "hello world"}},
        ])
        
        results = collection.text_search("xyznonexistent", top_k=10)
        assert isinstance(results, list)


class TestHybridSearch:
    """Tests for hybrid search combining vector and text (WIS-43)."""

    def test_hybrid_search_basic(self, temp_db_path):
        """Test basic hybrid search functionality."""
        db = velesdb.Database(temp_db_path)
        collection = db.create_collection("hybrid_test", dimension=4, metric="cosine")
        
        collection.upsert([
            {"id": 1, "vector": [1.0, 0.0, 0.0, 0.0], "payload": {"text": "machine learning"}},
            {"id": 2, "vector": [0.0, 1.0, 0.0, 0.0], "payload": {"text": "deep learning"}},
            {"id": 3, "vector": [0.9, 0.1, 0.0, 0.0], "payload": {"text": "supervised learning"}},
        ])
        
        results = collection.hybrid_search(
            vector=[1.0, 0.0, 0.0, 0.0],
            query="learning",
            top_k=3,
            vector_weight=0.5
        )
        
        assert len(results) <= 3
        if len(results) > 0:
            assert "id" in results[0]
            assert "score" in results[0]

    def test_hybrid_search_vector_weight(self, temp_db_path):
        """Test hybrid search with different vector weights."""
        db = velesdb.Database(temp_db_path)
        collection = db.create_collection("hybrid_weight", dimension=4)
        
        collection.upsert([
            {"id": 1, "vector": [1.0, 0.0, 0.0, 0.0], "payload": {"text": "alpha"}},
            {"id": 2, "vector": [0.0, 1.0, 0.0, 0.0], "payload": {"text": "beta"}},
        ])
        
        # High vector weight
        results_vec = collection.hybrid_search([1.0, 0.0, 0.0, 0.0], "beta", top_k=2, vector_weight=0.9)
        # Low vector weight
        results_text = collection.hybrid_search([1.0, 0.0, 0.0, 0.0], "beta", top_k=2, vector_weight=0.1)
        
        assert isinstance(results_vec, list)
        assert isinstance(results_text, list)


class TestBatchSearch:
    """Tests for batch search functionality (WIS-44)."""

    def test_batch_search_basic(self, temp_db_path):
        """Test basic batch search with multiple queries."""
        db = velesdb.Database(temp_db_path)
        collection = db.create_collection("batch_test", dimension=4, metric="cosine")
        
        collection.upsert([
            {"id": 1, "vector": [1.0, 0.0, 0.0, 0.0]},
            {"id": 2, "vector": [0.0, 1.0, 0.0, 0.0]},
            {"id": 3, "vector": [0.0, 0.0, 1.0, 0.0]},
            {"id": 4, "vector": [0.0, 0.0, 0.0, 1.0]},
        ])
        
        queries = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
        ]
        
        batch_results = collection.batch_search(queries, top_k=2)
        
        assert len(batch_results) == 2  # One result list per query
        assert len(batch_results[0]) <= 2
        assert len(batch_results[1]) <= 2
        
        # First query should match id=1 best
        assert batch_results[0][0]["id"] == 1
        # Second query should match id=2 best
        assert batch_results[1][0]["id"] == 2

    def test_batch_search_empty_queries(self, temp_db_path):
        """Test batch search with empty query list."""
        db = velesdb.Database(temp_db_path)
        collection = db.create_collection("batch_empty", dimension=4)
        
        collection.upsert([{"id": 1, "vector": [1.0, 0.0, 0.0, 0.0]}])
        
        batch_results = collection.batch_search([], top_k=5)
        assert batch_results == []

    def test_batch_search_single_query(self, temp_db_path):
        """Test batch search with single query."""
        db = velesdb.Database(temp_db_path)
        collection = db.create_collection("batch_single", dimension=4)
        
        collection.upsert([
            {"id": 1, "vector": [1.0, 0.0, 0.0, 0.0]},
            {"id": 2, "vector": [0.0, 1.0, 0.0, 0.0]},
        ])
        
        batch_results = collection.batch_search([[1.0, 0.0, 0.0, 0.0]], top_k=2)
        
        assert len(batch_results) == 1
        assert batch_results[0][0]["id"] == 1


class TestStorageMode:
    """Tests for storage mode (quantization) support (WIS-45)."""

    def test_create_collection_full_mode(self, temp_db_path):
        """Test creating collection with full storage mode (default)."""
        db = velesdb.Database(temp_db_path)
        collection = db.create_collection("full_mode", dimension=4, storage_mode="full")
        
        assert collection is not None
        info = collection.info()
        assert info["storage_mode"] == "full"

    def test_create_collection_sq8_mode(self, temp_db_path):
        """Test creating collection with SQ8 quantization."""
        db = velesdb.Database(temp_db_path)
        collection = db.create_collection("sq8_mode", dimension=4, storage_mode="sq8")
        
        assert collection is not None
        info = collection.info()
        assert info["storage_mode"] == "sq8"

    def test_create_collection_binary_mode(self, temp_db_path):
        """Test creating collection with binary quantization."""
        db = velesdb.Database(temp_db_path)
        collection = db.create_collection("binary_mode", dimension=4, storage_mode="binary")
        
        assert collection is not None
        info = collection.info()
        assert info["storage_mode"] == "binary"

    def test_storage_mode_search_accuracy(self, temp_db_path):
        """Test that search works correctly with different storage modes."""
        db = velesdb.Database(temp_db_path)
        
        for mode in ["full", "sq8", "binary"]:
            collection = db.create_collection(f"mode_{mode}", dimension=4, storage_mode=mode)
            
            collection.upsert([
                {"id": 1, "vector": [1.0, 0.0, 0.0, 0.0]},
                {"id": 2, "vector": [0.0, 1.0, 0.0, 0.0]},
            ])
            
            results = collection.search([1.0, 0.0, 0.0, 0.0], top_k=1)
            assert len(results) == 1
            assert results[0]["id"] == 1

    def test_invalid_storage_mode(self, temp_db_path):
        """Test creating collection with invalid storage mode."""
        db = velesdb.Database(temp_db_path)
        
        with pytest.raises(ValueError):
            db.create_collection("invalid_mode", dimension=4, storage_mode="invalid")


class TestDistanceMetrics:
    """Tests for all distance metrics including Hamming and Jaccard (WIS-46)."""

    def test_hamming_metric(self, temp_db_path):
        """Test Hamming distance metric."""
        db = velesdb.Database(temp_db_path)
        collection = db.create_collection("hamming_test", dimension=4, metric="hamming")
        
        collection.upsert([
            {"id": 1, "vector": [1.0, 0.0, 0.0, 0.0]},
            {"id": 2, "vector": [1.0, 1.0, 0.0, 0.0]},
            {"id": 3, "vector": [1.0, 1.0, 1.0, 0.0]},
        ])
        
        results = collection.search([1.0, 0.0, 0.0, 0.0], top_k=3)
        assert len(results) == 3
        # ID 1 should be closest (identical)
        assert results[0]["id"] == 1

    def test_jaccard_metric(self, temp_db_path):
        """Test Jaccard similarity metric."""
        db = velesdb.Database(temp_db_path)
        collection = db.create_collection("jaccard_test", dimension=4, metric="jaccard")
        
        collection.upsert([
            {"id": 1, "vector": [1.0, 1.0, 0.0, 0.0]},
            {"id": 2, "vector": [1.0, 0.0, 0.0, 0.0]},
            {"id": 3, "vector": [0.0, 0.0, 1.0, 1.0]},
        ])
        
        results = collection.search([1.0, 1.0, 0.0, 0.0], top_k=3)
        assert len(results) == 3
        # ID 1 should be closest (identical)
        assert results[0]["id"] == 1

    def test_all_metrics_create(self, temp_db_path):
        """Test creating collections with all supported metrics."""
        db = velesdb.Database(temp_db_path)
        
        metrics = ["cosine", "euclidean", "dot", "hamming", "jaccard"]
        
        for metric in metrics:
            collection = db.create_collection(f"metric_{metric}", dimension=4, metric=metric)
            assert collection is not None
            info = collection.info()
            assert info["metric"] == metric


class TestEdgeCases:
    """Tests for edge cases and error handling."""

    def test_invalid_metric(self, temp_db_path):
        """Test creating collection with invalid metric."""
        db = velesdb.Database(temp_db_path)
        
        with pytest.raises(ValueError):
            db.create_collection("invalid", dimension=4, metric="invalid_metric")

    def test_upsert_missing_id(self, temp_db_path):
        """Test upserting point without ID."""
        db = velesdb.Database(temp_db_path)
        collection = db.create_collection("missing_id", dimension=4)
        
        with pytest.raises(ValueError):
            collection.upsert([{"vector": [1.0, 0.0, 0.0, 0.0]}])

    def test_upsert_missing_vector(self, temp_db_path):
        """Test upserting point without vector."""
        db = velesdb.Database(temp_db_path)
        collection = db.create_collection("missing_vector", dimension=4)
        
        with pytest.raises(ValueError):
            collection.upsert([{"id": 1}])


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
