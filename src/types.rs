use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sprs::{CsMat, TriMat};
use sqlite::Value;
use std::collections::HashMap;
use std::fmt::Display;

use crate::config::ToricelliConfig;

/// Sparse weighted graph representation for link analysis.
/// Supports efficient pagerank, spectral clustering, and other graph algorithms.
pub struct LinkGraph {
    /// Map from note ID to matrix index
    pub id_to_idx: HashMap<ID, usize>,
    /// Map from matrix index to note ID
    pub idx_to_id: Vec<ID>,
    /// Weighted adjacency matrix in CSR format.
    /// A[i,j] = weight of edge from node i to node j.
    pub adjacency: CsMat<f64>,
}

impl LinkGraph {
    /// Create a new empty LinkGraph
    pub fn new() -> Self {
        Self {
            id_to_idx: HashMap::new(),
            idx_to_id: Vec::new(),
            adjacency: CsMat::empty(sprs::CompressedStorage::CSR, 0),
        }
    }

    /// Build a LinkGraph from an edge list with weights.
    /// Nodes are automatically indexed based on order of appearance.
    pub fn from_edges(edges: Vec<(ID, ID, f64)>) -> Self {
        let mut id_to_idx: HashMap<ID, usize> = HashMap::new();
        let mut idx_to_id: Vec<ID> = Vec::new();

        // First pass: collect all unique IDs
        for (src, dest, _) in &edges {
            if !id_to_idx.contains_key(src) {
                id_to_idx.insert(src.clone(), idx_to_id.len());
                idx_to_id.push(src.clone());
            }
            if !id_to_idx.contains_key(dest) {
                id_to_idx.insert(dest.clone(), idx_to_id.len());
                idx_to_id.push(dest.clone());
            }
        }

        let n = idx_to_id.len();
        let mut tri_mat = TriMat::new((n, n));

        // Second pass: add edges to triplet matrix
        for (src, dest, weight) in edges {
            let i = id_to_idx[&src];
            let j = id_to_idx[&dest];
            tri_mat.add_triplet(i, j, weight);
        }

        Self {
            id_to_idx,
            idx_to_id,
            adjacency: tri_mat.to_csr(),
        }
    }

    /// Register a node ID, returning its index. If already registered, returns existing index.
    pub fn register_node(&mut self, id: ID) -> usize {
        if let Some(&idx) = self.id_to_idx.get(&id) {
            idx
        } else {
            let idx = self.idx_to_id.len();
            self.id_to_idx.insert(id.clone(), idx);
            self.idx_to_id.push(id);
            idx
        }
    }

    /// Get forward links (outgoing edges) for a node
    pub fn links(&self, id: &ID) -> Option<Vec<(&ID, f64)>> {
        let idx = self.id_to_idx.get(id)?;
        let row = self.adjacency.outer_view(*idx)?;
        Some(
            row.indices()
                .iter()
                .zip(row.data().iter())
                .map(|(&j, &w)| (&self.idx_to_id[j], w))
                .collect(),
        )
    }

    /// Get backlinks (incoming edges) for a node.
    /// Note: This is O(E) as it scans all edges. For frequent use, consider caching transpose.
    pub fn backlinks(&self, id: &ID) -> Option<Vec<(&ID, f64)>> {
        let target_idx = self.id_to_idx.get(id)?;
        let mut result = Vec::new();
        for (i, row) in self.adjacency.outer_iterator().enumerate() {
            for (&j, &w) in row.indices().iter().zip(row.data().iter()) {
                if j == *target_idx {
                    result.push((&self.idx_to_id[i], w));
                }
            }
        }
        Some(result)
    }

    /// Get the transpose (for efficient backlink queries)
    pub fn transpose(&self) -> CsMat<f64> {
        self.adjacency.transpose_view().to_csr()
    }

    /// Number of nodes in the graph
    pub fn node_count(&self) -> usize {
        self.idx_to_id.len()
    }

    /// Number of edges in the graph
    pub fn edge_count(&self) -> usize {
        self.adjacency.nnz()
    }
}

#[derive(Eq, Hash, PartialEq, Clone, Debug, Default, Serialize, Deserialize)]
pub struct ID(pub String);

impl From<String> for ID {
    fn from(s: String) -> Self {
        ID(s)
    }
}

impl From<&str> for ID {
    fn from(s: &str) -> Self {
        ID(s.to_string())
    }
}

impl Display for ID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
	write!(f, "{}", self.0.to_string())
    }
}

pub type NoteMap = HashMap<ID, Note>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Note {
    #[serde(skip)]
    pub id: ID,
    pub mtimes: Vec<DateTime<Utc>>,
    pub stability: f64,
    pub score: f64,
}


impl Note {
    pub fn fetch_notes(_config: ToricelliConfig) -> HashMap<ID, Self> {
	todo!()
    }
}

impl Default for Note {
    fn default() -> Self {
        Note {
            id: ID("".to_string()),
            mtimes: vec![],
            stability: 1.0,
            score: 0.0,
        }
    }
}

impl From<Vec<Value>> for Note {
    fn from(values: Vec<Value>) -> Self {
        // Expects columns: id, data (JSON blob)
        let id = match &values[0] {
            Value::String(s) => ID(s.clone()),
            _ => panic!("Expected string for id"),
        };
        let data = match &values[1] {
            Value::String(s) => s.clone(),
            _ => panic!("Expected string for data"),
        };
        let mut note: Note = serde_json::from_str(&data).expect("Failed to parse JSON data");
        note.id = id;
        note
    }
}
