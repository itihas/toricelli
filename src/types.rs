use chrono::{DateTime, Utc};
use std::collections::{HashSet, HashMap};

use crate::config::ToricelliConfig;

#[derive(Eq, Hash, PartialEq, Clone, Debug)]
pub struct ID(pub String);

impl From<String> for ID {
    fn from(s: String) -> Self {
        ID(s)
    }
}

impl Note {
    pub fn fetch_notes(config: ToricelliConfig) -> HashMap<ID, Self> {
	todo!()
    }
}

#[derive(Debug)]
pub struct NoteMap(pub HashMap<ID, Note>);


impl std::ops::Deref for NoteMap {
    type Target = HashMap<ID, Note>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for NoteMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
	&mut self.0
    }
}

impl NoteMap {
    pub fn new() -> Self {
	Self(HashMap::new())
    }
}

#[derive(Clone, Debug)]
pub struct Note {
    pub id: ID,
    pub mtimes: Vec<DateTime<Utc>>,
    pub stability: f64,
    pub links: HashSet<ID>,
    pub backlinks: HashSet<ID>,
    pub outlinks: HashSet<String>,
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
            outlinks: HashSet::new(),
            score: 0.0,
        }
    }
}
