use crate::types::{LinkGraph, Note, NoteMap, ID};
use chrono::{TimeDelta, Utc};
use std::collections::HashMap;

fn standalone_score(n: &Note) -> f64 {
    const F: f64 = 19. / 81.;
    const C: f64 = -0.5;
    let t = match n.mtimes.last() {
        Some(t) => Utc::now() - t,
        None => TimeDelta::weeks(300),
    };
    return (1. + F * (t.num_days() as f64 / n.stability)).powf(C);
}

/// Compute pagerank scores using the link graph.
/// Updates note.score in place for all notes in the graph.
fn pagerank(notes: &mut NoteMap, graph: &LinkGraph) {
    const D: f64 = 0.85; // damping factor
    const ITERATIONS: usize = 5;

    let n = graph.node_count();
    if n == 0 {
        return;
    }

    // Get transpose for efficient backlink iteration
    let transpose = graph.transpose();

    // Initialize scores from notes, defaulting to 1/n for nodes not in notes
    let mut scores: Vec<f64> = (0..n)
        .map(|i| {
            let id = &graph.idx_to_id[i];
            notes.get(id).map(|n| n.score).unwrap_or(1.0 / n as f64)
        })
        .collect();

    // Pagerank iterations
    for _ in 0..ITERATIONS {
        let mut new_scores = vec![0.0; n];

        for i in 0..n {
            // Get backlinks (incoming edges) from transpose
            if let Some(row) = transpose.outer_view(i) {
                let backlink_sum: f64 = row
                    .indices()
                    .iter()
                    .zip(row.data().iter())
                    .map(|(&j, &weight)| {
                        // Get out-degree of source node j
                        let out_degree =
                            graph.adjacency.outer_view(j).map(|r| r.nnz()).unwrap_or(1) as f64;
                        scores[j] * weight / out_degree
                    })
                    .sum();

                let id = &graph.idx_to_id[i];
                let base_score = notes.get(id).map(|n| n.score).unwrap_or(1.0 / n as f64);
                new_scores[i] = (1.0 - D) * base_score + D * backlink_sum;
            } else {
                // No backlinks, just use base score
                let id = &graph.idx_to_id[i];
                new_scores[i] = notes.get(id).map(|n| n.score).unwrap_or(1.0 / n as f64);
            }
        }

        scores = new_scores;
    }

    // Write scores back to notes
    for (i, score) in scores.iter().enumerate() {
        let id = &graph.idx_to_id[i];
        if let Some(note) = notes.get_mut(id) {
            note.score = *score;
        }
    }
}

/// Compute backlink counts for each node (useful for analysis)
pub fn backlink_counts(graph: &LinkGraph) -> HashMap<ID, usize> {
    let transpose = graph.transpose();
    graph
        .idx_to_id
        .iter()
        .enumerate()
        .map(|(i, id)| {
            let count = transpose.outer_view(i).map(|r| r.nnz()).unwrap_or(0);
            (id.clone(), count)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeDelta, Utc};

    fn setup() -> (NoteMap, LinkGraph) {
        let template_note = Note {
            id: ID::from(""),
            mtimes: vec![Utc::now() - TimeDelta::days(1)],
            stability: 1.,
            score: 0.,
            ..Default::default()
        };

        let mut notes: NoteMap = HashMap::new();

        let ids = ["A", "B", "C", "D"].map(|id| ID::from(id));

        for id in ids {
            let mut n = template_note.clone();
            n.id = id.clone();
            notes.insert(id.clone(), n);
        }

        // Build edges: A->B, A->C, A->D, B->C, B->D, C->D
        let edges = vec![
            (ID::from("A"), ID::from("B"), 1.0),
            (ID::from("A"), ID::from("C"), 1.0),
            (ID::from("A"), ID::from("D"), 1.0),
            (ID::from("B"), ID::from("C"), 1.0),
            (ID::from("B"), ID::from("D"), 1.0),
            (ID::from("C"), ID::from("D"), 1.0),
        ];

        let graph = LinkGraph::from_edges(edges);

        (notes, graph)
    }

    #[test]
    fn test_standalone_score() {
        let (notes, _) = setup();
        if let Some(n) = notes.get(&ID::from("A")) {
            assert_eq!(0.9, standalone_score(n));
        }
    }

    #[test]
    fn test_backlink_counts() {
        let (_, graph) = setup();
        let counts = backlink_counts(&graph);

        // A has 0 backlinks
        assert_eq!(counts.get(&ID::from("A")), Some(&0));
        // B has 1 backlink (from A)
        assert_eq!(counts.get(&ID::from("B")), Some(&1));
        // C has 2 backlinks (from A, B)
        assert_eq!(counts.get(&ID::from("C")), Some(&2));
        // D has 3 backlinks (from A, B, C)
        assert_eq!(counts.get(&ID::from("D")), Some(&3));
    }

    #[test]
    fn test_pagerank() {
        let (mut notes, graph) = setup();

        // Initialize standalone scores
        for n in notes.values_mut() {
            n.score = standalone_score(n);
        }

        pagerank(&mut notes, &graph);

        let counts = backlink_counts(&graph);

        // Notes with more backlinks should have higher scores
        for n_1 in notes.values() {
            for n_2 in notes.values() {
                let c1 = counts.get(&n_1.id).unwrap_or(&0);
                let c2 = counts.get(&n_2.id).unwrap_or(&0);
                assert_eq!(
                    c1 > c2,
                    n_1.score > n_2.score,
                    "Backlink count ordering should match score ordering"
                );
            }
        }
    }
}
