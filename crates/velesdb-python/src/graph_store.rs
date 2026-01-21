//! GraphStore bindings for VelesDB Python.
//!
//! Provides a PyO3 wrapper around EdgeStore for graph operations.

use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

use crate::graph::{dict_to_edge, edge_to_dict};
use velesdb_core::collection::graph::{EdgeStore, GraphEdge};

/// In-memory graph store for knowledge graph operations.
///
/// Example:
///     >>> store = GraphStore()
///     >>> store.add_edge({"id": 1, "source": 100, "target": 200, "label": "KNOWS"})
///     >>> edges = store.get_edges_by_label("KNOWS")
#[pyclass]
pub struct GraphStore {
    inner: Arc<std::sync::RwLock<EdgeStore>>,
}

#[pymethods]
impl GraphStore {
    /// Creates a new empty graph store.
    #[new]
    fn new() -> Self {
        Self {
            inner: Arc::new(std::sync::RwLock::new(EdgeStore::new())),
        }
    }

    /// Adds an edge to the graph.
    ///
    /// Args:
    ///     edge: Dict with keys: id (int), source (int), target (int), label (str),
    ///           properties (dict, optional)
    ///
    /// Example:
    ///     >>> store.add_edge({"id": 1, "source": 100, "target": 200, "label": "KNOWS"})
    #[pyo3(signature = (edge))]
    fn add_edge(&self, edge: HashMap<String, PyObject>) -> PyResult<()> {
        Python::with_gil(|py| {
            let graph_edge = dict_to_edge(py, &edge)?;
            let mut store = self
                .inner
                .write()
                .map_err(|e| PyRuntimeError::new_err(format!("Lock error: {e}")))?;
            store
                .add_edge(graph_edge)
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to add edge: {e}")))
        })
    }

    /// Gets all edges with the specified label.
    ///
    /// Args:
    ///     label: The relationship type to filter by (e.g., "KNOWS", "FOLLOWS")
    ///
    /// Returns:
    ///     List of edge dicts with keys: id, source, target, label, properties
    ///
    /// Note:
    ///     This method uses the internal label index for fast O(1) lookup per label.
    ///
    /// Example:
    ///     >>> edges = store.get_edges_by_label("KNOWS")
    ///     >>> for edge in edges:
    ///     ...     print(f"{edge['source']} -> {edge['target']}")
    #[pyo3(signature = (label))]
    fn get_edges_by_label(&self, label: &str) -> PyResult<Vec<HashMap<String, PyObject>>> {
        Python::with_gil(|py| {
            let store = self
                .inner
                .read()
                .map_err(|e| PyRuntimeError::new_err(format!("Lock error: {e}")))?;
            let edges = store.get_edges_by_label(label);
            Ok(edges.into_iter().map(|e| edge_to_dict(py, e)).collect())
        })
    }

    /// Gets outgoing edges from a node.
    ///
    /// Args:
    ///     node_id: The source node ID
    ///
    /// Returns:
    ///     List of edge dicts
    #[pyo3(signature = (node_id))]
    fn get_outgoing(&self, node_id: u64) -> PyResult<Vec<HashMap<String, PyObject>>> {
        Python::with_gil(|py| {
            let store = self
                .inner
                .read()
                .map_err(|e| PyRuntimeError::new_err(format!("Lock error: {e}")))?;
            let edges = store.get_outgoing(node_id);
            Ok(edges.into_iter().map(|e| edge_to_dict(py, e)).collect())
        })
    }

    /// Gets incoming edges to a node.
    ///
    /// Args:
    ///     node_id: The target node ID
    ///
    /// Returns:
    ///     List of edge dicts
    #[pyo3(signature = (node_id))]
    fn get_incoming(&self, node_id: u64) -> PyResult<Vec<HashMap<String, PyObject>>> {
        Python::with_gil(|py| {
            let store = self
                .inner
                .read()
                .map_err(|e| PyRuntimeError::new_err(format!("Lock error: {e}")))?;
            let edges = store.get_incoming(node_id);
            Ok(edges.into_iter().map(|e| edge_to_dict(py, e)).collect())
        })
    }

    /// Gets outgoing edges from a node filtered by label.
    ///
    /// Args:
    ///     node_id: The source node ID
    ///     label: The relationship type to filter by
    ///
    /// Returns:
    ///     List of edge dicts matching the label
    #[pyo3(signature = (node_id, label))]
    fn get_outgoing_by_label(
        &self,
        node_id: u64,
        label: &str,
    ) -> PyResult<Vec<HashMap<String, PyObject>>> {
        Python::with_gil(|py| {
            let store = self
                .inner
                .read()
                .map_err(|e| PyRuntimeError::new_err(format!("Lock error: {e}")))?;
            let edges = store.get_outgoing_by_label(node_id, label);
            Ok(edges.into_iter().map(|e| edge_to_dict(py, e)).collect())
        })
    }

    /// Removes an edge by ID.
    ///
    /// Args:
    ///     edge_id: The edge ID to remove
    #[pyo3(signature = (edge_id))]
    fn remove_edge(&self, edge_id: u64) -> PyResult<()> {
        let mut store = self
            .inner
            .write()
            .map_err(|e| PyRuntimeError::new_err(format!("Lock error: {e}")))?;
        store.remove_edge(edge_id);
        Ok(())
    }

    /// Returns the number of edges in the store.
    fn edge_count(&self) -> PyResult<usize> {
        let store = self
            .inner
            .read()
            .map_err(|e| PyRuntimeError::new_err(format!("Lock error: {e}")))?;
        Ok(store.edge_count())
    }

    /// Clears all edges from the store.
    fn clear(&self) -> PyResult<()> {
        let mut store = self
            .inner
            .write()
            .map_err(|e| PyRuntimeError::new_err(format!("Lock error: {e}")))?;
        store.clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_store_get_edges_by_label() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let store = GraphStore::new();

            // Add edges
            let mut edge1 = HashMap::new();
            edge1.insert("id".to_string(), 1u64.into_pyobject(py).unwrap().into());
            edge1.insert(
                "source".to_string(),
                100u64.into_pyobject(py).unwrap().into(),
            );
            edge1.insert(
                "target".to_string(),
                200u64.into_pyobject(py).unwrap().into(),
            );
            edge1.insert(
                "label".to_string(),
                "KNOWS".into_pyobject(py).unwrap().into(),
            );
            store.add_edge(edge1).unwrap();

            let mut edge2 = HashMap::new();
            edge2.insert("id".to_string(), 2u64.into_pyobject(py).unwrap().into());
            edge2.insert(
                "source".to_string(),
                100u64.into_pyobject(py).unwrap().into(),
            );
            edge2.insert(
                "target".to_string(),
                300u64.into_pyobject(py).unwrap().into(),
            );
            edge2.insert(
                "label".to_string(),
                "WORKS_AT".into_pyobject(py).unwrap().into(),
            );
            store.add_edge(edge2).unwrap();

            // Test get_edges_by_label
            let knows_edges = store.get_edges_by_label("KNOWS").unwrap();
            assert_eq!(knows_edges.len(), 1);

            let works_edges = store.get_edges_by_label("WORKS_AT").unwrap();
            assert_eq!(works_edges.len(), 1);

            let none_edges = store.get_edges_by_label("NONEXISTENT").unwrap();
            assert!(none_edges.is_empty());
        });
    }
}
