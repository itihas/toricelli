use chrono::{DateTime, Utc};
use std::collections::HashSet;

#[derive(Eq, Hash, PartialEq, Clone, Debug)]
pub struct ID(String);

impl From<&str> for ID {
    fn from(s: &str) -> Self {
        ID(s.to_string())
    }
}

#[derive(Clone, Debug)]
pub struct Note {
    pub id: ID,
    pub mtimes: Vec<DateTime<Utc>>,
    pub stability: f64,
    pub links: HashSet<ID>,
    pub backlinks: HashSet<ID>,
    pub score: f64,
}

impl Default for Note {
    fn default() -> Self {
        Note {
            id: ID("".to_string()),
            mtimes: vec![],
            stability: 1.0,
            links: HashSet::new(),
            backlinks: HashSet::new(),
            score: 0.0,
        }
    }
}
