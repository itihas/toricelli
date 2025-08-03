use crate::types::{Note, ID};
use chrono::{TimeDelta, Utc};
use std::collections::{HashMap, HashSet};


fn standalone_score(n: &mut Note) {
    const F: f64 = 19. / 81.;
    const C: f64 = -0.5;
    let t = match n.mtimes.last() {
        Some(t) => Utc::now() - t,
        none => TimeDelta::weeks(300),
        None => todo!(),
    };
    n.score = (1. + F * (t.num_days() as f64 / n.stability)).powf(C);
}

fn update_backlinks(ns: &mut HashMap<ID, Note>) {
    let mut backlinks: HashMap<ID, HashSet<ID>> = HashMap::new();
    for (i, n) in ns.iter() {
        for j in &n.links {
            backlinks
                .entry(j.clone())
                .or_insert(HashSet::new())
                .insert(i.clone());
        }
    }
    for (j, b) in backlinks.iter() {
        if let Some(n) = ns.get_mut(j) {
            n.backlinks = b.clone();
        } else {
            let mut n = Note::default();
            n.id = j.clone();
            ns.insert(j.clone(), n);
        }
    }
}

fn pagerank(ns: &mut HashMap<ID, Note>) {
    const D: f64 = 0.85; // damping factor

    let mut new_scores: HashMap<ID, f64> = ns.iter().map(|(i, n)| (i.clone(), n.score)).collect();
    for _ in 1..5 {
        for (i, n) in ns.iter() {
            let avg_backlink_score = n
                .backlinks
                .iter()
                .fold(0., |acc, l| acc + (new_scores.get(l).unwrap()))
                / (1. + n.backlinks.len() as f64);
            new_scores.insert(i.clone(), (1. - D) * n.score + D * avg_backlink_score);
        }
    }
    for (i, n) in ns.iter_mut() {
        n.score = new_scores[i];
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use self::{standalone_score, Note, ID};
    use chrono::{TimeDelta, Utc};

    fn setup() -> HashMap<ID, Note> {
        let template_note = Note {
            id: ID::from(""),
            mtimes: vec![Utc::now() - TimeDelta::days(1)],
            stability: 1.,
            links: HashSet::new(),
            backlinks: HashSet::new(),
            score: 0.,
        };

        let mut ns: HashMap<ID, Note> = HashMap::new();

        let ids = ["A", "B", "C", "D"].map(|id| ID::from(id));

        for id in ids {
            let mut n = template_note.clone();
            n.id = id.clone();
            ns.insert(id.clone(), n);
        }

        if let Some(note_a) = ns.get_mut(&ID::from("A")) {
            note_a.links.insert(ID::from("B"));
            note_a.links.insert(ID::from("C"));
            note_a.links.insert(ID::from("D"));
        }
        if let Some(note_b) = ns.get_mut(&ID::from("B")) {
            note_b.links.insert(ID::from("C"));
            note_b.links.insert(ID::from("D"));
        }
        if let Some(note_c) = ns.get_mut(&ID::from("C")) {
            note_c.links.insert(ID::from("D"));
        }
        return ns;
    }

    #[test]
    fn test_standalone_score() {
        if let Some(mut n) = setup().get_mut(&ID::from("A")) {
            standalone_score(&mut n);
            assert_eq!(0.9, n.score);
        }
    }

    #[test]
    fn test_update_backlinks() {
        let mut ns = setup();
        update_backlinks(&mut ns);
        for (i, n) in ns.iter() {
            for link in n.links.iter() {
                if let Some(target) = ns.get(link) {
                    assert!(target.backlinks.contains(i));
                } else {
                    panic!("{link:?} target does not exist.");
                }
            }
        }
    }

    // TODO: Consider adding some edge cases - what about isolated nodes with no links? Or cycles?
    #[test]
    fn test_pagerank() {
        let mut ns = setup();
        update_backlinks(&mut ns);
        for n in ns.values_mut() {
            standalone_score(n);
        }
        pagerank(&mut ns);
        for n_1 in ns.values() {
            for n_2 in ns.values() {
                assert_eq!(
                    (n_1.backlinks.len() > n_2.backlinks.len()),
                    (n_1.score > n_2.score)
                ); // only true when they all have the same standalone_score, as they do in the setup() data currently (all have the same mtime vectors).
            }
        }
    }
}
