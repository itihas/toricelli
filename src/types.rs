use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sprs::{CsMat, TriMat};
use sqlite::Value;
use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::path::PathBuf;

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
    pub dirty_ids: HashSet<ID>,
}

impl LinkGraph {
    /// Create a new empty LinkGraph
    pub fn new() -> Self {
        Self {
            id_to_idx: HashMap::new(),
            idx_to_id: Vec::new(),
            adjacency: CsMat::empty(sprs::CompressedStorage::CSR, 0),
            dirty_ids: HashSet::new(),
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
            dirty_ids: HashSet::new(),
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

    /// Upsert edge into the graph, marking it as dirty.
    pub fn upsert_link(self: &mut Self, src: ID, dest: ID, weight: f64) {
        let i = self.register_node(src.clone());
        let j = self.register_node(dest);
        self.adjacency.insert(i, j, weight);
        self.dirty_ids.insert(src);
    }

    /// Get all dirty IDs
    pub fn dirty_ids(&self) -> impl Iterator<Item = &ID> {
        self.dirty_ids.iter()
    }

    pub(crate) fn clear_dirty(&mut self) {
        self.dirty_ids = HashSet::new()
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

/// Note storage with dirty tracking for efficient flushing.
/// Tracks which notes have been modified since the last flush.
#[derive(Clone)]
pub struct NoteStore {
    pub notes: NoteMap,
    dirty: HashSet<ID>,
}

impl NoteStore {
    /// Create a new empty NoteStore
    pub fn new() -> Self {
        Self {
            notes: HashMap::new(),
            dirty: HashSet::new(),
        }
    }

    /// Add notes from a NoteMap into an existing NoteStore. Mark all additions as dirty.
    pub fn upsert_from_notes(&mut self, notes: NoteMap) {
        self.notes.extend(notes.clone());
        self.dirty.extend(notes.into_keys());
    }

    /// Create a NoteStore from an existing NoteMap (all notes start clean)
    pub fn from_notes(notes: NoteMap) -> Self {
        Self {
            notes,
            dirty: HashSet::new(),
        }
    }

    /// Get a reference to a note
    pub fn get(&self, id: &ID) -> Option<&Note> {
        self.notes.get(id)
    }

    /// Get a mutable reference to a note, marking it as dirty
    pub fn get_mut(&mut self, id: &ID) -> Option<&mut Note> {
        if self.notes.contains_key(id) {
            self.dirty.insert(id.clone());
            self.notes.get_mut(id)
        } else {
            None
        }
    }

    /// Insert or update a note, marking it as dirty
    pub fn insert(&mut self, note: Note) {
        self.dirty.insert(note.id.clone());
        self.notes.insert(note.id.clone(), note);
    }

    /// Update a note using a closure, marking it as dirty
    pub fn update(&mut self, id: &ID, f: impl FnOnce(&mut Note)) {
        if let Some(note) = self.notes.get_mut(id) {
            f(note);
            self.dirty.insert(id.clone());
        }
    }

    /// Get all dirty note IDs
    pub fn dirty_ids(&self) -> impl Iterator<Item = &ID> {
        self.dirty.iter()
    }

    /// Get all dirty notes
    pub fn dirty_notes(&self) -> impl Iterator<Item = &Note> {
        self.dirty.iter().filter_map(|id| self.notes.get(id))
    }

    /// Number of dirty notes
    pub fn dirty_count(&self) -> usize {
        self.dirty.len()
    }

    /// Clear dirty flags (call after successful flush)
    pub fn clear_dirty(&mut self) {
        self.dirty.clear();
    }

    /// Check if a specific note is dirty
    pub fn is_dirty(&self, id: &ID) -> bool {
        self.dirty.contains(id)
    }

    /// Number of notes
    pub fn len(&self) -> usize {
        self.notes.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.notes.is_empty()
    }
}

// ── Feed types ──────────────────────────────────────────────

/// How to sort a feed.
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "cli", derive(clap::ValueEnum))]
pub enum SortOrder {
    /// Ascending by score (most due for review first)
    #[default]
    ByScore,
    /// Descending by most recent mtime (recently edited first)
    ByRecent,
}

impl Display for SortOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SortOrder::ByScore => write!(f, "by-score"),
            SortOrder::ByRecent => write!(f, "by-recent"),
        }
    }
}

/// Everything needed to describe a feed request,
/// regardless of whether it comes from CLI or HTTP.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "cli", derive(clap::Args))]
pub struct FeedRequest {
    /// How to sort the feed
    #[cfg_attr(feature = "cli", arg(long, value_enum, default_value_t = SortOrder::ByScore))]
    pub sort: SortOrder,

    /// Number of notes to display
    #[cfg_attr(feature = "cli", arg(long, short = 'n', default_value_t = 20))]
    pub count: usize,

    /// Number of notes to skip
    #[cfg_attr(feature = "cli", arg(long, default_value_t = 0))]
    pub offset: usize,
}

impl Default for FeedRequest {
    fn default() -> Self {
        Self {
            sort: SortOrder::ByScore,
            count: 20,
            offset: 0,
        }
    }
}

/// An ordered view into a NoteStore. Holds just IDs — look up
/// note data from the store at display time.
pub struct Feed(Vec<ID>);

impl Feed {
    /// Build a feed by sorting all notes in the store.
    pub fn build(store: &NoteStore, order: &SortOrder) -> Self {
        let mut ids: Vec<ID> = store.notes.keys().cloned().collect();
        match order {
            SortOrder::ByScore => {
                ids.sort_by(|a, b| {
                    let sa = store.get(a).map(|n| n.score).unwrap_or(f64::MAX);
                    let sb = store.get(b).map(|n| n.score).unwrap_or(f64::MAX);
                    sa.partial_cmp(&sb).unwrap_or(std::cmp::Ordering::Equal)
                });
            }
            SortOrder::ByRecent => {
                ids.sort_by(|a, b| {
                    let ma = store.get(a).and_then(|n| n.mtimes.last());
                    let mb = store.get(b).and_then(|n| n.mtimes.last());
                    mb.cmp(&ma) // descending — most recent first
                });
            }
        }
        Feed(ids)
    }

    /// Return a page of IDs, clamped to bounds.
    pub fn page(&self, offset: usize, count: usize) -> &[ID] {
        let start = offset.min(self.0.len());
        let end = (offset + count).min(self.0.len());
        &self.0[start..end]
    }

    /// Total number of items in the feed.
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Note {
    #[serde(skip)]
    pub id: ID,
    #[serde(skip)]
    pub file: Option<PathBuf>,
    pub mtimes: Vec<DateTime<Utc>>,
    pub stability: f64,
    pub score: f64,
}

impl Default for Note {
    fn default() -> Self {
        Note {
            id: ID("".to_string()),
            file: None,
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
